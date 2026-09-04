use core::fmt::Write;
use embassy_time::Instant;
use embassy_usb_logger::Writer;
use log::Record;

pub fn log_style<const N: usize>(record: &Record, writer: &mut Writer<'_, N>) {
    let millis = Instant::now().as_millis();

    // Format: [12345ms] INFO: message
    let _ = write!(
        writer,
        "[{:6}ms] {:5}: {}\r\n",
        millis,
        record.level(),
        record.args()
    );
}
