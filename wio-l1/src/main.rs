#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

mod task;

#[rtic::app(device = nrf52840_hal::pac, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod app {
    use defmt::info;
    use embedded_graphics::{
        mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
        pixelcolor::BinaryColor,
        prelude::*,
        text::{Baseline, Text},
    };
    use nrf52840_hal::{
        Clocks, Twim, clocks, gpio,
        pac::TWIM0,
        usbd::{UsbPeripheral, Usbd},
    };
    use rtic_monotonics::nrf::timer::prelude::*;
    use static_cell::StaticCell;
    use usb_device::{
        bus::UsbBusAllocator,
        device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
    };
    use usbd_serial::{SerialPort, USB_CLASS_CDC};

    use crate::task;

    nrf_timer0_monotonic!(Systick, 1_000_000);
    nrf_timer1_monotonic!(Mono, 1_000_000);

    defmt::timestamp!("{=u64:us}", { Systick::now().ticks() });

    #[shared]
    struct Shared {
        inputs: heapless::Vec<embedded_touch::Touch, 10>,
    }

    #[local]
    struct Local {
        usb_dev: UsbDevice<'static, Usbd<UsbPeripheral<'static>>>,
        usb_serial: SerialPort<'static, Usbd<UsbPeripheral<'static>>>,
        led_pin: gpio::Pin<gpio::Output<gpio::PushPull>>,
        oled_display: sh1106::mode::GraphicsMode<sh1106::interface::I2cInterface<Twim<TWIM0>>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        info!("Initializing...");
        let p = cx.device;

        let clocks = Clocks::new(p.CLOCK).enable_ext_hfosc();

        // Start systick for defmt
        Systick::start(p.TIMER0);

        // Start timer for delays
        let port1 = gpio::p1::Parts::new(p.P1);
        Mono::start(p.TIMER1);

        let clocks_ref = {
            static CLOCKS_CELL: StaticCell<
                Clocks<clocks::ExternalOscillator, clocks::Internal, clocks::LfOscStopped>,
            > = StaticCell::new();
            CLOCKS_CELL.init(clocks)
        };

        // workaround to fire the start-of-frame interrupt to allow host enumeration
        p.USBD.intenset.write(|w| w.sof().set());
        let usb_bus_ref = {
            static USB_BUS_CELL: StaticCell<UsbBusAllocator<Usbd<UsbPeripheral<'static>>>> =
                StaticCell::new();

            USB_BUS_CELL.init(UsbBusAllocator::new(Usbd::new(UsbPeripheral::new(
                p.USBD, clocks_ref,
            ))))
        };

        let usb_serial = SerialPort::new(usb_bus_ref);
        let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x2886, 0x1667))
            .strings(&[
                StringDescriptors::default()
                    .manufacturer("Seeed")
                    .product("Wio Tracker L1 Pro"), // .serial_number("TEST") // TODO set this to the actual serial number
            ])
            .unwrap()
            .device_class(USB_CLASS_CDC)
            .max_packet_size_0(64)
            .unwrap()
            .build();

        let port0 = gpio::p0::Parts::new(p.P0);
        let mut oled_i2c = {
            let scl = port0.p0_05.into_floating_input().degrade();
            let sda = port0.p0_06.into_floating_input().degrade();
            Twim::new(
                p.TWIM0,
                nrf52840_hal::twim::Pins { scl, sda },
                nrf52840_hal::twim::Frequency::K100,
            )
        };

        let oled_display = {
            // Probe for the OLED address (standard addresses are 0x3C and 0x3D per schematic)
            let addr = if oled_i2c.write(0x3c, &[]).is_ok() {
                0x3c
            } else {
                0x3d
            };

            let mut display: sh1106::mode::GraphicsMode<_> = sh1106::Builder::new()
                .with_i2c_addr(addr)
                .connect_i2c(oled_i2c)
                .into();

            display.init().unwrap();
            display.flush().unwrap();
            display
        };

        blink::spawn().unwrap();
        oled_task::spawn().unwrap();

        let inputs = heapless::Vec::new();

        (
            Shared { inputs },
            Local {
                usb_dev,
                usb_serial,
                led_pin: port1
                    .p1_01
                    .into_push_pull_output(gpio::Level::Low)
                    .degrade(),
                oled_display,
            },
        )
    }

    #[task(binds = USBD, priority = 2, local = [usb_dev, usb_serial])]
    fn usb_interrupt(cx: usb_interrupt::Context) {
        let usb_dev = cx.local.usb_dev;
        let usb_serial = cx.local.usb_serial;

        let mut buf = [0u8; 512];

        while usb_dev.poll(&mut [usb_serial]) {
            // Drain the FIFO
            while let Ok(count) = usb_serial.read(&mut buf) {
                if count == 0 {
                    break;
                }

                buf[..count].make_ascii_uppercase();

                let mut offset = 0;
                while offset < count {
                    match usb_serial.write(&buf[offset..count]) {
                        Ok(len) => offset += len,
                        Err(_) => break,
                    }
                }
            }
        }
    }

    #[task(priority = 2, shared = [inputs], local = [oled_display, ])]
    async fn oled_task(cx: oled_task::Context) {
        let display = cx.local.oled_display;

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline("Hello World!", Point::zero(), text_style, Baseline::Top)
            .draw(display)
            .unwrap();

        display.flush().unwrap();

        loop {
            Mono::delay(1000.millis()).await;
        }
    }

    #[task(priority = 1, local = [led_pin])]
    async fn blink(cx: blink::Context) {
        task::blink_task(cx.local.led_pin, Mono).await;
    }
}
