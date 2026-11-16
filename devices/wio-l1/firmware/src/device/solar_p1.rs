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
    gnss: GnssResources {
        uart: UARTE0,
        timer: TIMER0,
        ppi0: PPI_CH0,
        ppi1: PPI_CH1,
        ppi_group: PPI_GROUP0,
        tx_pin: P1_11,
        rx_pin: P1_12,
        rst_pin: P1_03,
        wakeup_pin: P0_02,
    },
    led: LedResources {
        user: P0_19,
        heartbeat: P0_15,
    },
    battery: BatteryResources {
        saadc: SAADC,
        ctrl: P0_14,
        adc_read: P0_31,
    }
    radio: RadioResources {
        spi: SPI3,
        sck: P1_13,
        miso: P1_14,
        mosi: P1_15,
        cs: P0_04,
        reset: P0_28,
        sw: P0_05,
        busy: P0_29,
        dio1: P0_03,
    },
    button: ButtonResources {
        user: P1_07,
    }
    storage: StorageResources {
        QSPI_SCK: P0_21, // D21 P0.21 (QSPI_SCK)
        QSPI_CSN: P0_25, // D22 P0.25 (QSPI_CSN)
        QSPI_SIO_0: P0_20, // D23 P0.20 (QSPI_SIO_0 DI)
        QSPI_SIO_1: P0_24, // D24 P0.24 (QSPI_SIO_1 DO)
        QSPI_SIO_2: P0_22, // D25 P0.22 (QSPI_SIO_2 WP)
        QSPI_SIO_3: P0_23, // D26 P0.23 (QSPI_SIO_3 HOLD)
    }
}
