use embassy_executor::Spawner;
use embassy_futures::select::*;
use embassy_nrf::{
    gpio,
    spim::{self},
};
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, RawMutex, ThreadModeRawMutex},
    lazy_lock::LazyLock,
    zerocopy_channel::{self, Channel, Receiver},
};
use embassy_time::{Delay, Timer};
use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};
use embedded_hal_bus::spi::ExclusiveDevice;

use regiface::errors::Error as RegifaceError;
use sx1262::{self, *};

use crate::{debug, error, info, warn};
use crate::{device::hardware::Irqs, device::hardware::RadioResources};

pub const RADIO_BUFFER_SIZE: usize = 127;
pub const RADIO_TX_CHANNEL_LENGTH: usize = 2;
pub type RadioBuffer = heapless::vec::Vec<u8, RADIO_BUFFER_SIZE>;

#[derive(Debug, Copy, Clone)]
pub struct CommonConfig {
    /// RF carrier frequency in Hz.
    pub frequency_hz: u32,
    /// TX output power in dBm, passed straight to SetTxParams.
    pub tx_power_dbm: i8,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 915_000_000,
            tx_power_dbm: 10,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LoraConfig {
    /// Spreading factor (SF5-SF12). Higher = more range, slower + more airtime.
    pub spreading_factor: SpreadingFactor,
    /// Signal bandwidth.
    pub bandwidth: LoRaBandwidth,
    /// Forward error correction coding rate.
    pub coding_rate: CodingRate,
    /// Enable for symbol durations >= 16.38ms (high SF + narrow BW).
    low_data_rate_opt: bool,
    /// Preamble length in symbols.
    pub preamble_length: u16,
    /// Append/check a CRC on the payload.
    pub crc_enabled: bool,
    /// Invert I/Q — set true on one side of a link if you want to avoid
    /// hearing your own retransmissions (common convention: gateways invert,
    /// nodes don't).
    invert_iq: bool,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            spreading_factor: SpreadingFactor::SF7,
            bandwidth: LoRaBandwidth::Bw125,
            coding_rate: CodingRate::Cr45,
            low_data_rate_opt: false,
            preamble_length: 8,
            crc_enabled: true,
            invert_iq: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GfskConfig {
    /// Bit rate in bits/second.
    pub bit_rate: u32,
    /// Frequency deviation in Hz.
    pub freq_deviation: u32,
    /// Gaussian pulse shaping filter.
    pub pulse_shape: GfskPulseShape,
    /// RX channel filter bandwidth.
    pub bandwidth: GfskBandwidth,
    /// Preamble length in bytes.
    pub preamble_length: u16,
    /// Sync word length in bytes.
    pub sync_word_length: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct RadioConfig {
    pub common: CommonConfig,
    pub phy: PhyConfig,
}

#[derive(Debug, Copy, Clone)]
pub enum PhyConfig {
    Lora(LoraConfig),
    Gfsk(GfskConfig),
}

impl Default for RadioConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig::default(),
            phy: PhyConfig::Lora(LoraConfig::default()),
        }
    }
}

pub trait RadioConfigManager<'a> {
    fn set_radio_config(&self, config: &RadioConfig);
    fn try_get_radio_config_watcher(
        &'a self,
    ) -> Option<embassy_sync::watch::DynReceiver<'a, RadioConfig>>;
}

#[embassy_executor::task]
pub async fn runner(
    _spawner: Spawner,
    r: RadioResources,
    state: &'static crate::system::SystemState,
    receiver: Receiver<'static, NoopRawMutex, RadioBuffer>,
) {
    radio(r, state, receiver).await;
}

async fn radio<'a>(
    r: RadioResources,
    config_manager: &'a impl RadioConfigManager<'a>,
    mut receiver: Receiver<'static, NoopRawMutex, RadioBuffer>,
) {
    Timer::after_secs(3).await;
    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M4; // Set clock speed
    config.mode = spim::MODE_0; // CPOL=0, CPHA=0
    config.orc = 0x00; // Over-read character

    let nss = gpio::Output::new(r.cs, gpio::Level::High, gpio::OutputDrive::Standard);
    let reset = gpio::Output::new(r.reset, gpio::Level::High, gpio::OutputDrive::Standard);
    let mut dio1 = gpio::Input::new(r.dio1, gpio::Pull::Down);
    let busy = gpio::Input::new(r.busy, gpio::Pull::None);
    let rf_switch_rx = gpio::Output::new(r.sw, gpio::Level::Low, gpio::OutputDrive::Standard);

    let spim = spim::Spim::new(r.spi, Irqs, r.sck, r.miso, r.mosi, config);
    let spi = ExclusiveDevice::new(spim, nss, Delay);

    let mut config_watcher = match config_manager.try_get_radio_config_watcher() {
        Some(cw) => cw,
        None => {
            error!("failed to get radio config watcher");
            return;
        }
    };

    let mut radio = sx1262::Device::new(spi);

    if let Err(e) = set_radio_params(&mut radio, config_watcher.get().await).await {
        error!("Failure configuring radio: {e:?}\nAborting radio module.");
        return;
    };

    loop {
        match select(
            config_watcher.get(),
            drive_radio(&mut radio, &mut dio1, &mut receiver),
        )
        .await
        {
            Either::First(c) => {
                // cleanup receiver, drops packet
                receiver.receive_done();

                if let Err(e) = set_radio_params(&mut radio, c).await {
                    error!("Failure configuring radio after config update: {e:?}");
                };
            }
            Either::Second(res) => {
                if let Err(e) = res {
                    // hardware error
                    error!("sx1262 error: {:?}", e);
                } else {
                    // this should never happen
                    error!("radio driver unexpectedly exited.");
                    return;
                }
            }
        };
    }
}

async fn set_radio_params(
    radio: &mut sx1262::device::Device<impl SpiDevice>,
    config: RadioConfig,
) -> Result<(), RegifaceError> {
    // All config commands below require STDBY_RC.
    radio
        .execute_command_async(SetStandby {
            config: StandbyConfig::Rc,
        })
        .await?;

    // DC-DC+LDO is lower power but needs the inductor on your board; switch
    // to RegulatorMode::LdoOnly if yours doesn't have it populated.
    radio
        .execute_command_async(SetRegulatorMode {
            mode: RegulatorMode::DcDcLdo,
        })
        .await?;

    // Split the 256-byte data buffer in half between TX and RX so an
    // incoming packet can't clobber a TX payload you haven't sent yet.
    radio
        .execute_command_async(SetBufferBaseAddress {
            config: BufferBaseAddressConfig {
                tx_base_addr: 0,
                rx_base_addr: 128,
            },
        })
        .await?;

    match config.phy {
        PhyConfig::Lora(c) => {
            radio
                .execute_command_async(SetPacketType {
                    packet_type: PacketType::LoRa,
                })
                .await?;

            radio
                .execute_command_async(SetRfFrequency {
                    config: RfFrequencyConfig {
                        frequency: config.common.frequency_hz,
                    },
                })
                .await?;

            radio
                .execute_command_async(SetModulationParams {
                    params: ModulationParams::LoRa(LoRaModParams {
                        spreading_factor: c.spreading_factor,
                        bandwidth: c.bandwidth,
                        coding_rate: c.coding_rate,
                        low_data_rate_opt: c.low_data_rate_opt,
                    }),
                })
                .await?;

            radio
                .execute_command_async(SetPacketParams {
                    params: PacketParams::LoRa(LoRaPacketParams {
                        preamble_length: c.preamble_length,
                        header_type: LoraPacketHeaderType::Variable,
                        payload_length: 0,
                        crc_enable: c.crc_enabled,
                        iq_inversion_enable: c.invert_iq,
                    }),
                })
                .await?;

            // 0 = validate reception starting from the first detected symbol.
            radio
                .execute_command_async(SetLoRaSymbNumTimeout {
                    config: LoRaSymbNumTimeout { symb_num: 0 },
                })
                .await?;
        }

        PhyConfig::Gfsk(c) => {
            radio
                .execute_command_async(SetPacketType {
                    packet_type: PacketType::Gfsk,
                })
                .await?;

            radio
                .execute_command_async(SetRfFrequency {
                    config: RfFrequencyConfig {
                        frequency: config.common.frequency_hz,
                    },
                })
                .await?;

            radio
                .execute_command_async(SetModulationParams {
                    params: ModulationParams::Gfsk(GfskModParams {
                        bit_rate: c.bit_rate,
                        pulse_shape: c.pulse_shape,
                        bandwidth: c.bandwidth,
                        freq_deviation: c.freq_deviation,
                    }),
                })
                .await?;

            radio
                .execute_command_async(SetPacketParams {
                    params: PacketParams::GFSK(GFSKPacketParams {
                        preamble_length: c.preamble_length,
                        preamble_detector_length: PreambleDetectorLength::Off,
                        sync_word_length: c.sync_word_length,
                        address_filtering: AddressFiltering::Disable,
                        packet_type: GFSKPacketHeaderType::Variable,
                        payload_length: 0,
                        crc_type: CrcType::CrcOff,
                        whitening_enable: true,
                    }),
                })
                .await?;
        }
    }

    // PA config for the SX1262 (high-power variant, up to +22dBm). If you're
    // on an SX1261 board, switch device_sel and drop hp_max/duty_cycle per
    // the datasheet's PA optimal-settings table (13-21).
    radio
        .execute_command_async(SetPaConfig {
            config: PaConfig {
                duty_cycle: 0x04,
                hp_max: 0x07,
                device_sel: DeviceSelect::Sx1262,
                pa_lut: 0x01,
            },
        })
        .await?;

    radio
        .execute_command_async(SetTxParams {
            params: TxParams {
                power: config.common.tx_power_dbm,
                ramp_time: RampTime::Micros200,
            },
        })
        .await?;

    // Route TxDone/RxDone/Timeout up to DIO1 so drive_radio's
    // dio1.wait_for_rising_edge() actually fires.
    radio
        .execute_command_async(SetDioIrqParams {
            config: DioIrqConfig {
                irq_mask: IrqMask::TX_DONE | IrqMask::RX_DONE | IrqMask::TIMEOUT,
                dio1_mask: IrqMask::RX_DONE | IrqMask::TIMEOUT,
                dio2_mask: IrqMask::empty(),
                dio3_mask: IrqMask::empty(),
            },
        })
        .await?;

    Ok(())
}

async fn drive_radio(
    radio: &mut sx1262::device::Device<impl SpiDevice>,
    dio1: &mut impl embedded_hal_async::digital::Wait,
    receiver: &mut Receiver<'static, impl RawMutex, RadioBuffer>,
) -> Result<(), RegifaceError> {
    let mut tx_buffer = [0; 127];
    let mut rx_buf = [0u8; 127];

    radio
        .execute_command_async(SetRx {
            mode: RxMode::Continuous,
        })
        .await?;

    loop {
        match select(receiver.receive(), dio1.wait_for_rising_edge()).await {
            Either::First(buffer) => {
                // copy to intermediary buffer to make this cancel-safe
                let tx_size = core::cmp::min(buffer.len(), tx_buffer.len());
                tx_buffer[..tx_size].copy_from_slice(&buffer.as_slice()[..tx_size]);
                receiver.receive_done();

                radio.write_buffer_async(0, &tx_buffer[..tx_size]).await?;
                radio
                    .execute_command_async(SetTx {
                        timeout: Timeout(0),
                    })
                    .await?;
                radio
                    .execute_command_async(SetRx {
                        mode: RxMode::Continuous,
                    })
                    .await?;
            }
            Either::Second(_) => {
                let status = radio.execute_command_async(GetRxBufferStatus).await?;
                let len =
                    core::cmp::min(rx_buf.len(), status.buffer_status.payload_length as usize);
                radio
                    .read_buffer_async(status.buffer_status.buffer_pointer, &mut rx_buf[..len])
                    .await?;

                info!("Received {} bytes: {:02x?}", len, &rx_buf[..len]);
            }
        };
    }
}
