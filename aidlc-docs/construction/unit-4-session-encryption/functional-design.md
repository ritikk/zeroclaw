# Unit 4: Session Encryption and TLS - Functional Design

**Date**: 2026-02-19
**Unit**: Session Encryption and TLS
**Phase**: CONSTRUCTION - Functional Design

## 1. TLS Enforcement Architecture

### Certificate Management
```rust
pub struct CertificateManager {
    cert_path: PathBuf,
    key_path: PathBuf,
    cert: X509Certificate,
    key: PrivateKey,
    expiry: DateTime<Utc>,
    rotation_schedule: CertRotationSchedule,
}

pub struct CertRotationSchedule {
    auto_rotate: bool,
    rotate_before_expiry_days: u32, // Default: 30
    rotation_interval_days: u32,    // Default: 365
    last_rotation: DateTime<Utc>,
}

pub struct CertificateInfo {
    subject: String,
    issuer: String,
    valid_from: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    fingerprint: String,
    is_self_signed: bool,
}
```

### Certificate Generation
```rust
pub struct CertificateGenerator {
    common_name: String,
    key_size: u32, // 2048 or 4096
    validity_days: u32,
}

impl CertificateGenerator {
    pub fn generate_self_signed() -> Result<(X509Certificate, PrivateKey)> {
        // Generate RSA key pair
        // Create self-signed certificate
        // Valid for 365 days
        // CN = "zeroclaw-local"
    }
}
```

---

## 2. TLS Configuration

### TLS Settings
```rust
pub struct TlsConfig {
    enabled: bool,
    min_version: TlsVersion, // TLS 1.3
    max_version: TlsVersion,
    cipher_suites: Vec<CipherSuite>,
    certificate_path: PathBuf,
    key_path: PathBuf,
    hsts_enabled: bool,
    hsts_max_age: u32, // seconds
    certificate_pinning: Option<CertificatePinning>,
}

pub enum TlsVersion {
    Tls12,
    Tls13,
}

pub struct CertificatePinning {
    pinned_certs: Vec<String>, // SHA256 fingerprints
    allow_backup_pins: bool,
}
```

### HSTS Headers
```rust
pub struct HstsHeader {
    max_age: u32,
    include_subdomains: bool,
    preload: bool,
}

impl HstsHeader {
    pub fn to_header_value(&self) -> String {
        format!(
            "max-age={}{}{}",
            self.max_age,
            if self.include_subdomains { "; includeSubDomains" } else { "" },
            if self.preload { "; preload" } else { "" }
        )
    }
}
```

---

## 3. Session Token Management

### Token Generation
```rust
pub struct SessionToken {
    token_id: String,           // 32 bytes, base64
    user_id: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_used: DateTime<Utc>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
    refresh_token: Option<String>,
}

pub struct TokenGenerator {
    token_length: usize, // 32 bytes minimum
    expiration_hours: u32,
    refresh_token_length: usize,
}

impl TokenGenerator {
    pub fn generate() -> Result<SessionToken> {
        // Use OsRng for cryptographic randomness
        // Generate 32+ bytes of random data
        // Encode as base64url
        // Set expiration to now + expiration_hours
    }
}
```

### Token Storage
```rust
pub struct TokenStore {
    tokens: HashMap<String, SessionToken>,
    revoked_tokens: HashSet<String>,
    token_index: HashMap<String, String>, // user_id -> token_id
}

pub struct TokenMetadata {
    token_id: String,
    user_id: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_used: DateTime<Utc>,
    access_count: u64,
    ip_binding: Option<IpAddr>,
    user_agent_binding: Option<String>,
}
```

---

## 4. Session Management

### Session State
```rust
pub struct Session {
    session_id: String,
    token: SessionToken,
    user_id: String,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    ip_address: IpAddr,
    user_agent: String,
    concurrent_sessions: u32,
    max_concurrent_sessions: u32,
}

pub struct SessionConfig {
    session_timeout_minutes: u32,
    idle_timeout_minutes: u32,
    max_concurrent_per_user: u32,
    enable_ip_binding: bool,
    enable_user_agent_binding: bool,
}
```

### Session Lifecycle
```rust
pub enum SessionState {
    Active,
    Idle,
    Expired,
    Revoked,
    Refreshing,
}

pub struct SessionEvent {
    timestamp: DateTime<Utc>,
    session_id: String,
    event_type: SessionEventType,
    details: String,
}

pub enum SessionEventType {
    Created,
    Accessed,
    Refreshed,
    Expired,
    Revoked,
    IpMismatch,
    UserAgentMismatch,
}
```

---

## 5. Message Encryption

### Payload Encryption
```rust
pub struct EncryptedPayload {
    ciphertext: Vec<u8>,
    iv: [u8; 12],
    tag: [u8; 16],
    algorithm: EncryptionAlgorithm,
    timestamp: DateTime<Utc>,
    nonce: String, // Replay attack prevention
}

pub struct PayloadEncryption {
    algorithm: EncryptionAlgorithm, // AES-256-GCM
    key_derivation: KeyDerivation,
}

pub enum KeyDerivation {
    DiffieHellman,
    HKDF,
}
```

### Message Signing
```rust
pub struct SignedMessage {
    payload: Vec<u8>,
    signature: Vec<u8>,
    algorithm: SignatureAlgorithm,
    timestamp: DateTime<Utc>,
}

pub enum SignatureAlgorithm {
    Ed25519,
    Sha256WithRsa,
}
```

### Replay Attack Prevention
```rust
pub struct ReplayProtection {
    nonce_store: HashSet<String>,
    timestamp_tolerance_seconds: u32, // Default: 60
    nonce_expiry_seconds: u32,        // Default: 300
}

impl ReplayProtection {
    pub fn validate_nonce(&mut self, nonce: &str, timestamp: DateTime<Utc>) -> Result<()> {
        // Check nonce not seen before
        // Check timestamp within tolerance
        // Store nonce with expiry
    }
}
```

---

## 6. Key Exchange Protocol

### Diffie-Hellman Setup
```rust
pub struct DiffieHellmanExchange {
    prime: BigUint,
    generator: BigUint,
    private_key: BigUint,
    public_key: BigUint,
}

pub struct KeyExchangeMessage {
    public_key: Vec<u8>,
    algorithm: KeyExchangeAlgorithm,
    timestamp: DateTime<Utc>,
}

pub enum KeyExchangeAlgorithm {
    DiffieHellman,
    EllipticCurve,
}
```

### Perfect Forward Secrecy
```rust
pub struct PerfectForwardSecrecy {
    ephemeral_keys: HashMap<String, EphemeralKey>,
    key_rotation_interval: Duration,
}

pub struct EphemeralKey {
    key_id: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    key_material: Vec<u8>,
}
```

---

## 7. Configuration

### TLS Config Schema
```toml
[gateway]
port = 3000
host = "127.0.0.1"
tls_enabled = true
tls_min_version = "1.3"

[gateway.tls]
certificate_path = "~/.zeroclaw/certs/server.crt"
key_path = "~/.zeroclaw/certs/server.key"
auto_generate_self_signed = true
hsts_enabled = true
hsts_max_age = 31536000

[gateway.session]
token_expiration_hours = 24
refresh_token_enabled = true
max_concurrent_sessions = 5
enable_ip_binding = false
enable_user_agent_binding = true
idle_timeout_minutes = 30

[gateway.encryption]
payload_encryption_enabled = false
message_signing_enabled = true
replay_protection_enabled = true
```

---

## 8. State Machines

### TLS Handshake State Machine
```
Idle
  ↓ (client connects)
ClientHello
  ↓ (server responds)
ServerHello
  ↓ (key exchange)
KeyExchange
  ↓ (finished)
Established
  ↓ (session active)
Active
  ↓ (close)
Closed
```

### Session Token State Machine
```
Generated
  ↓ (issued to client)
Active
  ↓ (used in request)
Accessed
  ↓ (refresh requested)
Refreshing
  ↓ (new token issued)
Active
  ↓ (expiration time reached)
Expired
  ↓ (cleanup)
Deleted
```

---

## 9. Success Criteria

- [ ] TLS 1.3 enforced for all gateway connections
- [ ] Self-signed certificates auto-generated
- [ ] Certificate rotation mechanism working
- [ ] Session tokens cryptographically secure (32+ bytes)
- [ ] Token expiration and refresh working
- [ ] Session binding to IP/user-agent optional
- [ ] Concurrent session limits enforced
- [ ] Message encryption optional but available
- [ ] Replay attack prevention working
- [ ] Perfect forward secrecy implemented
- [ ] 100% test coverage for TLS/session paths
