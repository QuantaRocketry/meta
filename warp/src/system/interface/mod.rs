use embassy_executor::{SpawnError, Spawner};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::{self, CdcAcmClass};
use embassy_usb::class::cdc_ncm::{self, CdcNcmClass};
use embassy_usb::UsbDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use crate::resources::{InterfaceResources, Irqs};

pub mod acm;
pub mod ncm;

pub struct USBInterface {
    pub opts: USBOpts,
    pub usb_config: embassy_usb::Config<'static>,
}

#[derive(Default)]
pub struct USBOpts {
    pub acm: bool,
    pub ncm: bool,
}

impl USBInterface {
    pub fn new(manufacturer: &'static str, product: &'static str, opts: USBOpts) -> Self {
        let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
        usb_config.manufacturer = Some(manufacturer);
        usb_config.product = Some(product);
        usb_config.serial_number = None;
        usb_config.max_power = 100;
        usb_config.max_packet_size_0 = 64;

        USBInterface { opts, usb_config }
    }

    pub fn start(&mut self, spawner: &Spawner, r: InterfaceResources) -> Result<(), SpawnError> {
        // Create the driver, from the HAL.
        let driver = Driver::new(r.usb, Irqs);

        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let mut builder = embassy_usb::Builder::new(
            driver,
            self.usb_config.clone(),
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );

        let mut acm_res = Ok(());
        let mut ncm_res = Ok(());

        // Create classes on the builder.
        if self.opts.acm {
            let acm_class = {
                static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
                let state = STATE.init(cdc_acm::State::new());
                CdcAcmClass::new(&mut builder, state, 64)
            };
            acm_res = spawner.spawn(acm::task(acm_class));
        }

        // Our MAC addr.
        // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
        let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

        // if self.opts.ncm {
        //     let ncm_class = {
        //         static STATE: StaticCell<cdc_ncm::State> = StaticCell::new();
        //         let state = STATE.init(cdc_ncm::State::new());
        //         CdcNcmClass::new(&mut builder, state, host_mac_addr, 64)
        //     };
        //     ncm_res = ncm::start(spawner, ncm_class);
        // }

        let usb = builder.build();

        if let Err(err) = spawner.spawn(usb_task(usb)) {
            return Err(err);
        };

        acm_res?;
        ncm_res?;
        Ok(())
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}
