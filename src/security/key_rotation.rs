use aes_gcm::Key;
use aes_gcm::Aes256Gcm;
use aes_gcm::aead::Aead;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Rotating,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersion {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub key_material: Vec<u8>,
    pub status: KeyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationEvent {
    pub timestamp: DateTime<Utc>,
    pub old_version: u32,
    pub new_version: u32,
    pub credentials_migrated: usize,
    pub status: String,
}

pub struct KeyStore {
    current_version: u32,
    keys: HashMap<u32, KeyVersion>,
    rotation_history: Vec<KeyRotationEvent>,
}

impl KeyStore {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let key_material: [u8; 32] = rng.gen();

        let mut keys = HashMap::new();

        keys.insert(
            1,
            KeyVersion {
                version: 1,
                created_at: Utc::now(),
                key_material: key_material.to_vec(),
                status: KeyStatus::Active,
            },
        );

        Self {
            current_version: 1,
            keys,
            rotation_history: Vec::new(),
        }
    }

    pub fn rotate_key(&mut self) -> Result<u32, String> {
        let old_version = self.current_version;
        let new_version = old_version + 1;

        // Mark old key as rotating
        if let Some(old_key) = self.keys.get_mut(&old_version) {
            old_key.status = KeyStatus::Rotating;
        }

        // Generate new key
        let mut rng = rand::rng();
        let key_material: [u8; 32] = rng.gen();

        self.keys.insert(
            new_version,
            KeyVersion {
                version: new_version,
                created_at: Utc::now(),
                key_material: key_material.to_vec(),
                status: KeyStatus::Active,
            },
        );

        self.current_version = new_version;

        Ok(new_version)
    }

    pub fn get_current_key(&self) -> Result<Key<Aes256Gcm>, String> {
        let key_version = self
            .keys
            .get(&self.current_version)
            .ok_or("Current key not found")?;

        let key_array: [u8; 32] = key_version
            .key_material
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid key material length")?;

        Ok(Key::<Aes256Gcm>::from(key_array))
    }

    pub fn get_key_by_version(&self, version: u32) -> Result<Key<Aes256Gcm>, String> {
        let key_version = self.keys.get(&version).ok_or("Key version not found")?;

        let key_array: [u8; 32] = key_version
            .key_material
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid key material length")?;

        Ok(Key::<Aes256Gcm>::from(key_array))
    }

    pub fn retire_old_keys(&mut self, keep_versions: u32) {
        let versions_to_keep: Vec<u32> = self
            .keys
            .keys()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(keep_versions as usize)
            .collect();

        self.keys.retain(|version, _| versions_to_keep.contains(version));
    }

    pub fn log_rotation(&mut self, old_version: u32, new_version: u32, migrated: usize) {
        self.rotation_history.push(KeyRotationEvent {
            timestamp: Utc::now(),
            old_version,
            new_version,
            credentials_migrated: migrated,
            status: "completed".to_string(),
        });
    }

    pub fn get_current_version(&self) -> u32 {
        self.current_version
    }

    pub fn get_rotation_history(&self) -> &[KeyRotationEvent] {
        &self.rotation_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_store_creation() {
        let store = KeyStore::new();
        assert_eq!(store.get_current_version(), 1);
        assert!(store.get_current_key().is_ok());
    }

    #[test]
    fn test_key_rotation() {
        let mut store = KeyStore::new();
        let old_version = store.get_current_version();

        let new_version = store.rotate_key().unwrap();
        assert_eq!(new_version, old_version + 1);
        assert_eq!(store.get_current_version(), new_version);
    }

    #[test]
    fn test_get_key_by_version() {
        let mut store = KeyStore::new();
        store.rotate_key().unwrap();

        // Just verify both versions exist
        assert!(store.get_key_by_version(1).is_ok());
        assert!(store.get_key_by_version(2).is_ok());
    }

    #[test]
    fn test_retire_old_keys() {
        let mut store = KeyStore::new();
        store.rotate_key().unwrap();
        store.rotate_key().unwrap();
        store.rotate_key().unwrap();

        assert_eq!(store.keys.len(), 4);
        store.retire_old_keys(2);
        assert_eq!(store.keys.len(), 2);
    }

    #[test]
    fn test_log_rotation() {
        let mut store = KeyStore::new();
        store.log_rotation(1, 2, 100);

        let history = store.get_rotation_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].old_version, 1);
        assert_eq!(history[0].new_version, 2);
        assert_eq!(history[0].credentials_migrated, 100);
    }
}
