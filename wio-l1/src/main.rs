#![no_std]
#![no_main]

use panic_semihosting as _;

mod task;

#[rtic::app(device = nrf52840_hal::pac, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod app {
    use nrf52840_hal::{
        Clocks, clocks, gpio,
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

    nrf_timer0_monotonic!(Mono, 1_000_000);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        usb_dev: UsbDevice<'static, Usbd<UsbPeripheral<'static>>>,
        usb_serial: SerialPort<'static, Usbd<UsbPeripheral<'static>>>,
        led: gpio::Pin<gpio::Output<gpio::PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let p = cx.device;

        let clocks = Clocks::new(p.CLOCK).enable_ext_hfosc();

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

        let port1 = gpio::p1::Parts::new(p.P1);
        let led = port1
            .p1_01
            .into_push_pull_output(gpio::Level::Low)
            .degrade();

        Mono::start(p.TIMER0);
        blink::spawn().unwrap();

        (
            Shared {},
            Local {
                usb_dev,
                usb_serial,
                led,
            },
        )
    }

    #[task(binds = USBD, priority = 2, local = [usb_dev, usb_serial])]
    fn usb_interrupt(cx: usb_interrupt::Context) {
        let usb_dev = cx.local.usb_dev;
        let usb_serial = cx.local.usb_serial;

        while usb_dev.poll(&mut [usb_serial]) {
            let mut buf = [0u8; 64];

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

    #[task(priority = 1, local = [led])]
    async fn blink(cx: blink::Context) {
        task::blink_task(cx.local.led, Mono).await;
    }
}
