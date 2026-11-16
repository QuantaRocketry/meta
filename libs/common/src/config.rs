use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfigError {
    /// The underlying storage failed.
    Storage,
    /// The value could not be encoded.
    Serialization,
}

pub trait ConfigItem<'de, T: Clone + Serialize + Deserialize<'de>> {
    /// The current value, waiting only if none has been set yet.
    async fn get(&mut self) -> T;
    /// Wait for a value this handle hasn't seen yet.
    async fn changed(&mut self) -> T;
    fn try_get(&mut self) -> Option<T>;
    async fn update(&mut self, state: T) -> Result<(), ConfigError>;
}
