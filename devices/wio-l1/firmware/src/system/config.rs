use common::config::{ConfigError, ConfigItem};
use embassy_sync::mutex::Mutex;
use embassy_sync::watch::{DynReceiver, DynSender, Watch};
use qcp::RadioSettings;
use sequential_storage::cache::{Cache, Uncached};
use sequential_storage::map::{MapConfig, MapStorage};
use serde::{Serialize, de::DeserializeOwned};

use crate::device::{CONFIG_FLASH_RANGE, ConfigFlash};
use crate::{error, warn};

/// Flash map keys. Never reuse a key for a different type.
const RADIO_SETTINGS_KEY: u16 = 0x0001;

/// Scratch space for a single map item: key + postcard-encoded value,
/// rounded up to the flash word size.
const BUFFER_SIZE: usize = 64;

/// The nRF QSPI peripheral requires word-aligned RAM buffers.
#[repr(align(4))]
struct AlignedBuffer([u8; BUFFER_SIZE]);

pub struct ConfigStorage {
    map: MapStorage<u16, ConfigFlash, Cache<Uncached, Uncached, Uncached, u16>>,
    buffer: AlignedBuffer,
}

impl ConfigStorage {
    pub fn new(flash: ConfigFlash) -> Self {
        Self {
            map: MapStorage::new(
                flash,
                MapConfig::new(CONFIG_FLASH_RANGE),
                Cache::new_uncached(),
            ),
            buffer: AlignedBuffer([0; BUFFER_SIZE]),
        }
    }

    /// Read `key` from flash, returning `None` if it is missing or undecodable.
    async fn load<T: DeserializeOwned>(&mut self, key: u16) -> Option<T> {
        match self.map.fetch_item::<&[u8]>(&mut self.buffer.0, &key).await {
            Ok(Some(bytes)) => postcard::from_bytes(bytes)
                .inspect_err(|_| warn!("config: discarding undecodable key {}", key))
                .ok(),
            Ok(None) => None,
            Err(e) => {
                warn!("config: failed to fetch key {}: {:?}", key, e);
                None
            }
        }
    }

    async fn store<T: Serialize>(&mut self, key: u16, value: &T) -> Result<(), ConfigError> {
        let mut encoded = [0u8; BUFFER_SIZE];
        let bytes =
            postcard::to_slice(value, &mut encoded).map_err(|_| ConfigError::Serialization)?;
        self.map
            .store_item(&mut self.buffer.0, &key, &&*bytes)
            .await
            .map_err(|e| {
                error!("config: failed to store key {}: {:?}", key, e);
                ConfigError::Storage
            })
    }
}

pub struct SystemConfig {
    radio_settings: Watch<super::Mutex, RadioSettings, 1>,
    storage: Mutex<super::Mutex, Option<ConfigStorage>>,
}

impl SystemConfig {
    pub fn new() -> Self {
        Self {
            radio_settings: Watch::new(),
            storage: Mutex::new(None),
        }
    }

    /// Attach flash storage and publish the stored values. Must run before
    /// any task reads config.
    pub async fn init_storage(&self, mut storage: ConfigStorage) {
        let radio_settings = storage
            .load::<RadioSettings>(RADIO_SETTINGS_KEY)
            .await
            .unwrap_or_default();
        self.radio_settings.sender().send(radio_settings);

        *self.storage.lock().await = Some(storage);
    }

    pub fn try_get_radio_config_manager(&self) -> Option<RadioSettingsManager<'_>> {
        let receiver = self.radio_settings.dyn_receiver()?;
        let sender = self.radio_settings.dyn_sender();
        Some(RadioSettingsManager {
            receiver,
            sender,
            storage: &self.storage,
        })
    }
}

pub struct RadioSettingsManager<'a> {
    receiver: DynReceiver<'a, RadioSettings>,
    sender: DynSender<'a, RadioSettings>,
    storage: &'a Mutex<super::Mutex, Option<ConfigStorage>>,
}

impl ConfigItem<'_, RadioSettings> for RadioSettingsManager<'_> {
    async fn get(&mut self) -> RadioSettings {
        self.receiver.get().await
    }

    async fn changed(&mut self) -> RadioSettings {
        self.receiver.changed().await
    }

    fn try_get(&mut self) -> Option<RadioSettings> {
        self.receiver.try_get()
    }

    async fn update(&mut self, state: RadioSettings) -> Result<(), ConfigError> {
        self.sender.send(state);

        // mark the new value as seen
        _ = self.receiver.try_get();

        match self.storage.lock().await.as_mut() {
            Some(storage) => storage.store(RADIO_SETTINGS_KEY, &state).await,
            None => Err(ConfigError::Storage),
        }
    }
}
