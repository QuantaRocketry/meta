#![cfg_attr(not(test), no_std)]

pub mod config;
pub mod module;

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, warn};
#[cfg(not(feature = "defmt"))]
pub use log::{debug, error, info, warn};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
