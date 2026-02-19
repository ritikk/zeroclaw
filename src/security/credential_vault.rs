use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedValue {
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct SensitiveData {
    plaintext: Vec<u8>,
}

impl Drop for SensitiveData {
    fn drop(&mut self) {
        self.plaintext.zeroize();
    }
}

impl SensitiveData {
    pub fn new(data: Vec<u8>) -> Self {
        Self { plaintext: data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.plaintext
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.plaintext)
    }
}

pub struct CredentialVault {
    key: Key<Aes256Gcm>,
    version: u32,
    credentials: HashMap<String, EncryptedValue>,
}

impl CredentialVault {
    pub fn new(key_material: [u8; 32]) -> Self {
        Self {
            key: Key::<Aes256Gcm>::from(key_material),
            version: 1,
            credentials: HashMap::new(),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedValue, Box<dyn std::error::Error>> {
        let mut rng = rand::rng();
        let iv: [u8; 12] = rng.gen();

        let nonce = Nonce::from_slice(&iv);
        let cipher = Aes256Gcm::new(&self.key);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok(EncryptedValue {
            ciphertext,
            iv: iv.to_vec(),
            version: self.version,
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedValue) -> Result<SensitiveData, Box<dyn std::error::Error>> {
        let nonce = Nonce::from_slice(&encrypted.iv);
        let cipher = Aes256Gcm::new(&self.key);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        Ok(SensitiveData::new(plaintext))
    }

    pub fn store(&mut self, id: String, plaintext: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted = self.encrypt(plaintext)?;
        self.credentials.insert(id, encrypted);
        Ok(())
    }

    pub fn retrieve(&self, id: &str) -> Result<SensitiveData, Box<dyn std::error::Error>> {
        let encrypted = self
            .credentials
            .get(id)
            .ok_or("Credential not found")?;
        self.decrypt(encrypted)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.credentials.remove(id).ok_or("Credential not found")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let vault = CredentialVault::new(key);

        let plaintext = b"secret_api_key_12345";
        let encrypted = vault.encrypt(plaintext).unwrap();
        let decrypted = vault.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted.as_bytes(), plaintext);
    }

    #[test]
    fn test_store_retrieve() {
        let key = [42u8; 32];
        let mut vault = CredentialVault::new(key);

        let plaintext = b"my_secret";
        vault.store("api_key_1".to_string(), plaintext).unwrap();

        let retrieved = vault.retrieve("api_key_1").unwrap();
        assert_eq!(retrieved.as_bytes(), plaintext);
    }

    #[test]
    fn test_different_ivs_produce_different_ciphertexts() {
        let key = [42u8; 32];
        let vault = CredentialVault::new(key);

        let plaintext = b"same_plaintext";
        let encrypted1 = vault.encrypt(plaintext).unwrap();
        let encrypted2 = vault.encrypt(plaintext).unwrap();

        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_ne!(encrypted1.iv, encrypted2.iv);
    }

    #[test]
    fn test_delete_credential() {
        let key = [42u8; 32];
        let mut vault = CredentialVault::new(key);

        vault.store("key1".to_string(), b"secret").unwrap();
        vault.delete("key1").unwrap();

        assert!(vault.retrieve("key1").is_err());
    }
}
