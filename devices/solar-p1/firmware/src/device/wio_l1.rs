use assign_resources::assign_resources;
use embassy_nrf::{Peri, bind_interrupts, buffered_uarte, peripherals, saadc, spim, twim, usb};

bind_interrupts!(pub struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
    UARTE0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    TWISPI1 => twim::InterruptHandler<peripherals::TWISPI1>;
    SAADC => saadc::InterruptHandler;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
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
    }
}
