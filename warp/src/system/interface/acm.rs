use crate::system;
use defmt::*;
use embassy_executor::{SpawnError, Spawner};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::CdcAcmClass;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
pub async fn task(mut class: CdcAcmClass<'static, Driver<'static, USB>>) {
    let mut buf = [0; 64];
    // let event_publisher = unwrap!(event::EVENT_CHANNEL.publisher());

    loop {
        debug!("USB waiting for CDC-ACM connection...");
        class.wait_connection().await;

        debug!("Connected");
        // event_publisher
        //     .publish(SystemEvent::Interface(InterfaceEvent::Connected))
        //     .await;

        loop {
            let n = match class.read_packet(&mut buf).await {
                Ok(n) => n,
                Err(e) => match e {
                    embassy_usb::driver::EndpointError::BufferOverflow => {
                        error!("USB buffer overflow.");
                        continue;
                    }
                    embassy_usb::driver::EndpointError::Disabled => {
                        debug!("USB disconnected.");
                        break;
                    }
                },
            };
            let data = &buf[..n];
            info!("data: {:x}", data);
            if data == [b'a'] {
                system::send_event(system::Event::StateUpdate(system::State::Okay)).await;
            } else if data == [b's'] {
                system::send_event(system::Event::StateUpdate(system::State::Error(
                    system::Error::Alloc,
                )))
                .await;
            }

            if let Err(e) = class.write_packet(data).await {
                // let e = event::SystemEvent::Interface(InterfaceEvent::from(e));
                // event_publisher.publish(e).await;
                break;
            };
        }
        debug!("Disconnected");
    }
}
