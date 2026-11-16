extern crate alloc;

pub mod simulator;
pub mod slint_backend;

#[cfg(not(feature = "defmt"))]
pub use log::{debug, error, info, trace, warn};

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, trace, warn};
