# Unit 1: Credential Storage Hardening - NFR Design

**Date**: 2026-02-19
**Unit**: Credential Storage Hardening
**Phase**: CONSTRUCTION - NFR Design

## 1. Encryption Pattern

### AES-256-GCM Implementation
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::rngs::OsRng;

pub struct EncryptionEngine {
    key: Key<Aes256Gcm>,
}

impl EncryptionEngine {
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let nonce = Nonce::from_slice(&OsRng.gen::<[u8; 12]>());
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher.encrypt(nonce, plaintext)?;
        Ok(EncryptedData {
            ciphertext,
            iv: nonce.to_vec(),
            tag: extract_tag(&ciphertext),
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(&encrypted.iv);
        let cipher = Aes256Gcm::new(&self.key);
        cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
    }
}
```

## 2. Key Derivation Pattern

### PBKDF2 with 100k Iterations
```rust
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

pub struct KeyDerivation;

impl KeyDerivation {
    pub fn derive_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            100_000,
            &mut key,
        );
        key
    }
}
```

## 3. Memory Protection Pattern

### Zeroize on Drop
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct SensitiveCredential {
    #[zeroize(drop)]
    plaintext: String,
}

impl Drop for SensitiveCredential {
    fn drop(&mut self) {
        self.plaintext.zeroize();
    }
}
```

## 4. Rate Limiting Pattern

### Sliding Window Rate Limiter
```rust
pub struct RateLimiter {
    window_size: Duration,
    max_requests: u32,
    requests: VecDeque<Instant>,
}

impl RateLimiter {
    pub fn check_limit(&mut self) -> Result<()> {
        let now = Instant::now();
        
        // Remove old requests outside window
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) > self.window_size {
                self.requests.pop_front();
            } else {
                break;
            }
        }
        
        if self.requests.len() >= self.max_requests as usize {
            return Err(RateLimitError::LimitExceeded);
        }
        
        self.requests.push_back(now);
        Ok(())
    }
}
```

## 5. Key Rotation Pattern

### Versioned Key Store
```rust
pub struct VersionedKeyStore {
    current_version: u32,
    keys: HashMap<u32, Key<Aes256Gcm>>,
}

impl VersionedKeyStore {
    pub fn rotate_key(&mut self) -> Result<()> {
        let new_version = self.current_version + 1;
        let new_key = Key::<Aes256Gcm>::from(OsRng.gen::<[u8; 32]>());
        
        self.keys.insert(new_version, new_key);
        self.current_version = new_version;
        
        // Mark old key for retirement
        Ok(())
    }

    pub fn decrypt_with_version(&self, version: u32, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let key = self.keys.get(&version)
            .ok_or(KeyError::VersionNotFound)?;
        
        let nonce = Nonce::from_slice(&encrypted.iv);
        let cipher = Aes256Gcm::new(key);
        cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
    }
}
```

## 6. TTL Management Pattern

### Credential Expiration Tracking
```rust
pub struct CredentialWithTTL {
    credential: EncryptedValue,
    expires_at: DateTime<Utc>,
    refresh_handler: Option<RefreshHandler>,
}

impl CredentialWithTTL {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn needs_refresh(&self) -> bool {
        let refresh_threshold = self.expires_at - Duration::minutes(5);
        Utc::now() > refresh_threshold
    }

    pub async fn auto_refresh(&mut self) -> Result<()> {
        if let Some(handler) = &self.refresh_handler {
            let new_token = handler.refresh().await?;
            self.credential = encrypt_credential(&new_token)?;
            self.expires_at = Utc::now() + Duration::days(30);
        }
        Ok(())
    }
}
```

## 7. Access Logging Pattern

### Immutable Audit Log
```rust
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    log_file: File,
}

pub struct AuditEntry {
    timestamp: DateTime<Utc>,
    credential_id: String,
    action: AccessAction,
    user: String,
    status: AccessStatus,
}

impl AuditLog {
    pub async fn log_access(&mut self, entry: AuditEntry) -> Result<()> {
        // Append-only: never modify existing entries
        let json = serde_json::to_string(&entry)?;
        self.log_file.write_all(json.as_bytes()).await?;
        self.log_file.write_all(b"\n").await?;
        self.entries.push(entry);
        Ok(())
    }
}
```

## 8. Configuration Validation

### Startup Validation
```rust
pub struct CredentialStorageConfig {
    encrypt: bool,
    key_rotation_enabled: bool,
    key_rotation_interval_days: u32,
}

impl CredentialStorageConfig {
    pub fn validate(&self) -> Result<()> {
        if self.encrypt && self.key_rotation_interval_days < 30 {
            return Err(ConfigError::RotationTooFrequent);
        }
        if self.key_rotation_interval_days > 365 {
            return Err(ConfigError::RotationTooInfrequent);
        }
        Ok(())
    }
}
```

## 9. Performance Optimization

### Caching Decrypted Credentials
```rust
pub struct CredentialCache {
    cache: HashMap<String, (SensitiveCredential, Instant)>,
    ttl: Duration,
}

impl CredentialCache {
    pub fn get_or_decrypt(
        &mut self,
        id: &str,
        encrypted: &EncryptedValue,
        engine: &EncryptionEngine,
    ) -> Result<SensitiveCredential> {
        if let Some((cred, cached_at)) = self.cache.get(id) {
            if cached_at.elapsed() < self.ttl {
                return Ok(cred.clone());
            }
        }
        
        let plaintext = engine.decrypt(encrypted)?;
        let cred = SensitiveCredential::new(plaintext);
        self.cache.insert(id.to_string(), (cred.clone(), Instant::now()));
        Ok(cred)
    }
}
```

## 10. Success Criteria

- [ ] All encryption operations <10ms
- [ ] Key rotation <5 seconds for 1000 credentials
- [ ] Memory zeroization verified
- [ ] Rate limiting prevents brute force
- [ ] TTL tracking accurate
- [ ] Audit log immutable and complete
- [ ] Performance benchmarks meet targets
