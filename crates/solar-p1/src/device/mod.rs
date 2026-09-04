#[cfg(feature = "wio-l1")]
pub mod wio_l1;
#[cfg(feature = "wio-l1")]
pub use wio_l1 as hardware;

#[cfg(feature = "solar-p1")]
pub mod solar_p1;
#[cfg(feature = "solar-p1")]
pub use solar_p1 as hardware;
