use embassy_futures::select::*;
use embassy_sync::{blocking_mutex::raw::RawMutex, zerocopy_channel::Receiver};
use embassy_time::Timer;
use embedded_hal_async::spi::SpiDevice;
use qcp::RadioSettings;
use regiface::errors::Error as RegifaceError;
use sx1262::{self, Error::SerializationError, *};

use crate::config::ConfigItem;
use crate::{error, info};

pub const RADIO_BUFFER_SIZE: usize = 127;
pub const RADIO_TX_CHANNEL_LENGTH: usize = 2;
pub type RadioBuffer = heapless::vec::Vec<u8, RADIO_BUFFER_SIZE>;

#[derive(Debug, Clone)]
struct RadioParameters {
    frequency_hz: u32,
    packet_type: sx1262::PacketType,
    modulation_params: sx1262::ModulationParams,
    packet_params: sx1262::PacketParams,
}

impl From<RadioSettings> for RadioParameters {
    fn from(value: RadioSettings) -> Self {
        // TODO implement these somehow
        // match value.protocol {
        //     qcp::Protocol::Quanta => todo!(),
        //     qcp::Protocol::Eggtimer(_) => todo!(),
        //     qcp::Protocol::Uts => todo!(),
        // }
        Self {
            frequency_hz: 915_000_000,
            packet_type: sx1262::PacketType::LoRa,
            modulation_params: sx1262::ModulationParams::LoRa(LoRaModParams {
                spreading_factor: SpreadingFactor::SF7,
                bandwidth: LoRaBandwidth::Bw125,
                coding_rate: CodingRate::Cr45,
                low_data_rate_opt: false,
            }),
            packet_params: sx1262::PacketParams::LoRa(LoRaPacketParams {
                preamble_length: 8,
                header_type: LoraPacketHeaderType::Fixed,
                payload_length: 0,
                crc_enable: true,
                iq_inversion_enable: false,
            }),
        }
    }
}

pub trait RadioResources<
    Spi: embedded_hal_async::spi::SpiDevice,
    Dio1: embedded_hal_async::digital::Wait,
    Mutex: RawMutex + 'static,
>
{
    fn try_get_resources(&self) -> Result<(Spi, Dio1, Receiver<'static, Mutex, RadioBuffer>), ()>;
}

pub async fn runner<
    'a,
    Spi: embedded_hal_async::spi::SpiDevice,
    Dio1: embedded_hal_async::digital::Wait,
    Mutex: RawMutex + 'static,
>(
    mut settings_manager: impl ConfigItem<'a, RadioSettings>,
    spi: Spi,
    mut dio1: Dio1,
    mut receiver: Receiver<'static, Mutex, RadioBuffer>,
) {
    let settings = match select(settings_manager.get(), Timer::after_secs(5)).await {
        Either::First(s) => s,
        Either::Second(_) => {
            error!("Failed to get initial radio settings within timeout");
            return;
        }
    };

    info!("Radio task started");

    let mut radio = sx1262::Device::new(spi);

    if set_radio_params(&mut radio, settings.into()).await.is_err() {
        error!("Failure configuring radio. Aborting radio module.");
        return;
    };

    loop {
        match select(
            settings_manager.changed(),
            drive_radio(&mut radio, &mut dio1, &mut receiver),
        )
        .await
        {
            Either::First(s) => {
                if set_radio_params(&mut radio, s.into()).await.is_err() {
                    error!("Failure configuring radio after config update");
                };
            }
            Either::Second(res) => {
                if res.is_err() {
                    // hardware error
                    error!("sx1262 error");
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
    config: RadioParameters,
) -> Result<(), RegifaceError> {
    // validate the config
    {
        match (
            &config.packet_type,
            &config.modulation_params,
            &config.packet_params,
        ) {
            (PacketType::Gfsk, ModulationParams::Gfsk(_), PacketParams::GFSK(_)) => {}
            (PacketType::LoRa, ModulationParams::LoRa(_), PacketParams::LoRa(_)) => {}
            _ => {
                error!("Invalid radio params");
                return Err(SerializationError);
            }
        }
    }

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
                tx_base_addr: 128,
                rx_base_addr: 0,
            },
        })
        .await?;

    radio
        .execute_command_async(SetPacketType {
            packet_type: config.packet_type,
        })
        .await?;

    radio
        .execute_command_async(SetRfFrequency {
            config: RfFrequencyConfig {
                frequency: config.frequency_hz,
            },
        })
        .await?;

    radio
        .execute_command_async(SetModulationParams {
            params: config.modulation_params,
        })
        .await?;

    radio
        .execute_command_async(SetPacketParams {
            params: config.packet_params,
        })
        .await?;

    // 0 = validate reception starting from the first detected symbol.
    radio
        .execute_command_async(SetLoRaSymbNumTimeout {
            config: LoRaSymbNumTimeout { symb_num: 0 },
        })
        .await?;

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
                power: 14,
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

async fn drive_radio<'a, Spi: SpiDevice, Dio1: embedded_hal_async::digital::Wait, M: RawMutex>(
    radio: &mut sx1262::device::Device<Spi>,
    dio1: &mut Dio1,
    receiver: &mut Receiver<'a, M, RadioBuffer>,
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

                info!("Received {} bytes", len);
            }
        };
    }
}
