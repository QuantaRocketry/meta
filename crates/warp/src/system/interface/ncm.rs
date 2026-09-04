use defmt::*;
use embassy_executor::{SpawnError, Spawner};
use embassy_net::{tcp::TcpSocket, StackResources, Ipv4Cidr, Ipv4Address};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner, State as NetState};
use embassy_usb::class::cdc_ncm::CdcNcmClass;
use embedded_io_async::Write;
use static_cell::StaticCell;
use heapless::Vec;

use {defmt_rtt as _, panic_probe as _};

const MTU: usize = 1514;

pub fn start(
    spawner: &Spawner,
    class: CdcNcmClass<'static, Driver<'static, USB>>,
) -> Result<(), SpawnError> {
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    static NET_STATE: StaticCell<NetState<MTU, 4, 4>> = StaticCell::new();
    let (runner, device) =
        class.into_embassy_net_device::<MTU, 4, 4>(NET_STATE.init(NetState::new()), our_mac_addr);

    unwrap!(spawner.spawn(usb_ncm_task(runner)));

    // let config = embassy_net::Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    });

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

    // TODO generate a seed
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), 0);

    unwrap!(spawner.spawn(net_task(runner)));

    spawner.spawn(ncm_task(stack))?;
    Ok(())
}

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, Driver<'static, USB>, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static, MTU>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn ncm_task(stack: embassy_net::Stack<'static>) -> ! {
    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("rxd {:02x}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}
