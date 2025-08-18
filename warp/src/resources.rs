use assign_resources::assign_resources;
use embassy_rp::{
    bind_interrupts,
    peripherals::{self, PIO0, PIO1, USB},
    pio, usb, Peri,
};

assign_resources! {
    lora: LoraResources {
        dio0_pin: PIN_21,
        reset_pin: PIN_17,
        miso_pin: PIN_8,
        mosi_pin: PIN_15,
        clk_pin: PIN_14,
        cs_pin: PIN_16,
        spi: SPI1,
        rx_dma: DMA_CH0,
        tx_dma: DMA_CH1,
    },
    indicators: IndicatorResources {
        led_pin: PIN_25,
        led_pwm_slice: PWM_SLICE4,
        rgb_pin: PIN_4,
        pio: PIO0,
    },
    interface: InterfaceResources {
        usb: USB,
    },
}

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});
