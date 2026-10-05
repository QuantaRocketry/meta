use assign_resources::assign_resources;
use embassy_nrf::saadc::Saadc;
use embassy_nrf::{
    Peri, bind_interrupts, buffered_uarte, gpio, pac, peripherals, qspi, saadc, spim, twim, usb,
};
use embassy_time::Timer;

pub fn get_mac_addr() -> u64 {
    let addr0 = pac::FICR.deviceaddr(0).read() as u64;
    let addr1 = pac::FICR.deviceaddr(1).read() as u64;

    let mac_48 = (addr0 | (addr1 << 32)) | 0xC000_0000_0000;
    return mac_48;
}

bind_interrupts!(pub struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
    UARTE0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    TWISPI1 => twim::InterruptHandler<peripherals::TWISPI1>;
    SAADC => saadc::InterruptHandler;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    QSPI => qspi::InterruptHandler<peripherals::QSPI>;
});

assign_resources! {
    usb: UsbResources {
        usbd: USBD,
    },
    buzzer: BuzzerResources {
        pwm: PWM0,
        pin: P1_00
    },
    gnss: GnssResources {
        uart: UARTE0,
        timer: TIMER0,
        ppi0: PPI_CH0,
        ppi1: PPI_CH1,
        ppi_group: PPI_GROUP0,
        tx_pin: P0_27,
        rx_pin: P0_26,
        rst_pin: P1_06,
        wakeup_pin: P1_09,
    },
    led: LedResources {
        heartbeat: P1_01,
    },
    oled: OledResources {
        i2c: TWISPI0,
        scl: P0_05,
        sda: P0_06,
    },
    joystick: JoystickResources {
        right: P1_03,
        up: P1_04,
        select: P1_05,
        left: P0_11,
        down: P0_12,
        menu: P0_08
    },
    battery: BatteryResources {
        saadc: SAADC,
        ctrl: P0_04,
        adc_read: P0_31
    }
    radio: RadioResources {
        spi: SPI3,
        sck: P0_30,
        mosi: P0_28,
        miso: P0_03,
        cs: P1_14,
        reset: P1_07,
        sw: P1_08,
        busy: P1_10,
        dio1: P0_07,
    },
    compass: CompassResources {
        i2c: TWISPI1,
        scl: P0_09,
        sda:P0_10,
    },
    flash: FlashResources {
        qspi: QSPI,
        sck: P0_21,
        csn: P0_25,
        io0: P0_20,
        io1: P0_24,
        io2: P0_22,
        io3: P0_23,
    }
}

pub struct BatteryHardware {
    _ctrl: gpio::Output<'static>,
    saadc: Saadc<'static, 1>,
}

impl BatteryHardware {
    const BATTERY_VREF_MV: i32 = 3600; // 0.6V ref × gain factor of 6, in millivolts
    const BATTERY_ADC_MAX: i32 = 1 << 12; // 2^12
    const BATTERY_DIVIDER: i32 = 2;

    pub async fn from_resources(r: BatteryResources) -> Self {
        // pull ctrl pin high to enable BMS
        let _ctrl = gpio::Output::new(r.ctrl, gpio::Level::High, gpio::OutputDrive::Standard);
        Timer::after_millis(100).await; // let the divider + filter cap settle (tau ~25ms)

        let mut config = saadc::Config::default();
        config.resolution = saadc::Resolution::_12BIT;
        config.oversample = saadc::Oversample::BYPASS;

        let mut channel_config = saadc::ChannelConfig::single_ended(r.adc_read);
        channel_config.gain = saadc::Gain::GAIN1_6;
        channel_config.reference = saadc::Reference::INTERNAL;
        channel_config.time = saadc::Time::_40US; // long acquisition for RC filter impedance

        let saadc = Saadc::new(r.saadc, Irqs, config, [channel_config]);
        saadc.calibrate().await;

        Self { _ctrl, saadc }
    }
}

impl common::module::battery::Battery for BatteryHardware {
    async fn sample_mv(&mut self) -> u16 {
        let mut buf = [0i16; 1];
        self.saadc.sample(&mut buf).await;

        let sample = (buf[0] as i32).max(0);

        // Convert to millivolts at the ADC pin
        let v_pin_mv = (sample * Self::BATTERY_VREF_MV) / Self::BATTERY_ADC_MAX;

        // Multiply by divider ratio to get actual battery voltage
        (v_pin_mv * Self::BATTERY_DIVIDER) as u16
    }
}
