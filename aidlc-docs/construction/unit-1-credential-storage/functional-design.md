# Unit 1: Credential Storage Hardening - Functional Design

**Date**: 2026-02-19
**Unit**: Credential Storage Hardening
**Phase**: CONSTRUCTION - Functional Design

## 1. Key Rotation Mechanism

### Data Model
```rust
pub struct KeyVersion {
    version: u32,
    created_at: DateTime<Utc>,
    key_material: [u8; 32],
    algorithm: EncryptionAlgorithm,
    status: KeyStatus, // Active, Rotating, Retired
}

pub struct KeyStore {
    current_version: u32,
    keys: HashMap<u32, KeyVersion>,
    rotation_history: Vec<KeyRotationEvent>,
}

pub struct KeyRotationEvent {
    timestamp: DateTime<Utc>,
    old_version: u32,
    new_version: u32,
    credentials_migrated: usize,
    status: RotationStatus,
}
```

### Key Rotation Algorithm
1. Generate new key with new version number
2. Mark old key as "Rotating"
3. Re-encrypt all credentials with new key (background task)
4. Mark old key as "Retired" after grace period (24h)
5. Optionally delete retired keys after retention period (30d)

### Rotation Triggers
- Manual: `zeroclaw auth rotate-keys`
- Automatic: Configurable interval (default: 90 days)
- On-demand: After security incident

---

## 2. Credential Storage Isolation

### Storage Architecture
```rust
pub struct CredentialVault {
    api_keys: EncryptedStore<ApiKeyCredential>,
    oauth_tokens: EncryptedStore<OAuthToken>,
    db_credentials: EncryptedStore<DatabaseCredential>,
    service_tokens: EncryptedStore<ServiceToken>,
}

pub struct EncryptedStore<T> {
    data: HashMap<String, EncryptedValue>,
    metadata: StoreMetadata,
    access_log: AccessLog,
}

pub struct EncryptedValue {
    ciphertext: Vec<u8>,
    iv: [u8; 12],
    tag: [u8; 16],
    version: u32,
    created_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
    access_count: u64,
}
```

### Isolation Properties
- Separate encryption keys per store type
- Independent access control per store
- Isolated audit logs per store
- Per-store TTL and rotation policies

---

## 3. Memory Protection

### Zeroize Integration
```rust
pub struct SensitiveData {
    #[zeroize(drop)]
    plaintext: String,
}

impl Drop for SensitiveData {
    fn drop(&mut self) {
        self.plaintext.zeroize();
    }
}
```

### Memory Lifecycle
1. Decrypt credential into `SensitiveData`
2. Use credential (minimal scope)
3. Automatic zeroize on drop
4. Prevent copies via `Copy` trait exclusion

---

## 4. Access Control and Logging

### Access Policy
```rust
pub struct AccessPolicy {
    credential_id: String,
    allowed_tools: Vec<String>,
    allowed_users: Vec<String>,
    rate_limit: RateLimit,
    access_log: Vec<AccessEvent>,
}

pub struct AccessEvent {
    timestamp: DateTime<Utc>,
    user: String,
    tool: String,
    action: AccessAction, // Read, Decrypt, Rotate
    status: AccessStatus, // Allowed, Denied, RateLimited
    reason: Option<String>,
}

pub struct RateLimit {
    max_accesses_per_minute: u32,
    max_accesses_per_hour: u32,
    current_window: RateLimitWindow,
}
```

### Rate Limiting Algorithm
- Sliding window rate limiting
- Per-credential limits
- Automatic lockout on threshold breach
- Exponential backoff for retries

---

## 5. Credential TTL and Auto-Refresh

### TTL Model
```rust
pub struct CredentialTTL {
    credential_id: String,
    expires_at: DateTime<Utc>,
    refresh_before: DateTime<Utc>, // Refresh 5 min before expiry
    auto_refresh: bool,
    refresh_handler: Option<RefreshHandler>,
}

pub struct RefreshHandler {
    provider: String,
    refresh_endpoint: String,
    refresh_token: String,
}
```

### Refresh Lifecycle
1. Monitor credential expiration
2. Trigger refresh 5 minutes before expiry
3. Call provider refresh endpoint
4. Update credential with new token
5. Log refresh event
6. Alert if refresh fails

---

## 6. State Machines

### Key Rotation State Machine
```
Idle
  ↓ (rotate_keys triggered)
Generating New Key
  ↓ (key generated)
Rotating (old key active, new key accepting writes)
  ↓ (all credentials migrated)
Validating (verify all credentials decrypt with new key)
  ↓ (validation passed)
Active (new key is primary)
  ↓ (grace period elapsed)
Retiring (old key no longer accepts new credentials)
  ↓ (retention period elapsed)
Deleted
```

### Credential Access State Machine
```
Stored (encrypted at rest)
  ↓ (access requested)
Decrypting
  ↓ (decryption successful)
InMemory (plaintext in memory)
  ↓ (use complete)
Zeroizing
  ↓ (memory cleared)
Stored
```

---

## 7. Configuration

### Config Schema
```toml
[secrets]
encrypt = true
key_rotation_enabled = true
key_rotation_interval_days = 90
key_retention_days = 30
zeroize_on_drop = true

[secrets.access_control]
rate_limit_per_minute = 100
rate_limit_per_hour = 1000
lockout_duration_minutes = 15

[secrets.stores.api_keys]
separate_key = true
ttl_days = 365

[secrets.stores.oauth_tokens]
separate_key = true
ttl_days = 30
auto_refresh = true

[secrets.stores.db_credentials]
separate_key = true
ttl_days = 90
```

---

## 8. Success Criteria

- [ ] Key rotation mechanism implemented and tested
- [ ] All credentials re-encrypt on key rotation
- [ ] Separate stores for API keys, OAuth, DB credentials
- [ ] Memory zeroization on all sensitive data
- [ ] Per-credential access logging
- [ ] Rate limiting prevents brute force
- [ ] TTL tracking and auto-refresh working
- [ ] Zero credential leaks in logs
- [ ] 100% test coverage for encryption paths
