# Unit 4: Session Encryption and TLS - NFR Design

**Date**: 2026-02-19
**Unit**: Session Encryption and TLS
**Phase**: CONSTRUCTION - NFR Design

## 1. TLS 1.3 Configuration Pattern

### Rustls Configuration
```rust
use rustls::{ServerConfig, RootCertStore};
use rustls_pemfile as pemfile;

pub struct TlsConfigBuilder {
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl TlsConfigBuilder {
    pub fn build(&self) -> Result<ServerConfig> {
        let certs = pemfile::certs(&mut BufReader::new(File::open(&self.cert_path)?))
            .collect::<Result<Vec<_>, _>>()?;
        
        let key = pemfile::private_key(&mut BufReader::new(File::open(&self.key_path)?))
            .map_err(|_| TlsError::InvalidKey)?
            .ok_or(TlsError::NoPrivateKey)?;
        
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        
        Ok(config)
    }
}
```

## 2. Self-Signed Certificate Generation

### Certificate Generation with rcgen
```rust
use rcgen::{generate_simple_self_signed_cert, Certificate};

pub struct CertificateGenerator;

impl CertificateGenerator {
    pub fn generate_self_signed(cn: &str) -> Result<(String, String)> {
        let subject_alt_names = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ];
        
        let cert = generate_simple_self_signed_cert(
            vec![cn.to_string()],
            subject_alt_names,
        )?;
        
        let cert_pem = cert.serialize_pem()?;
        let key_pem = cert.serialize_private_key_pem();
        
        Ok((cert_pem, key_pem))
    }
}
```

## 3. Session Token Generation Pattern

### Cryptographically Secure Token
```rust
use rand::Rng;
use base64::{engine::general_purpose, Engine as _};

pub struct TokenGenerator;

impl TokenGenerator {
    pub fn generate() -> String {
        let mut rng = rand::rngs::OsRng;
        let mut token_bytes = [0u8; 32];
        rng.fill(&mut token_bytes);
        
        general_purpose::URL_SAFE_NO_PAD.encode(&token_bytes)
    }
}
```

## 4. Session Token Validation Pattern

### Token Verification
```rust
pub struct SessionValidator {
    token_store: TokenStore,
}

impl SessionValidator {
    pub fn validate_token(&self, token: &str) -> Result<SessionToken> {
        let session = self.token_store.get(token)
            .ok_or(SessionError::InvalidToken)?;
        
        if session.expires_at < Utc::now() {
            return Err(SessionError::TokenExpired);
        }
        
        Ok(session)
    }

    pub fn validate_with_binding(
        &self,
        token: &str,
        ip: IpAddr,
        user_agent: &str,
    ) -> Result<SessionToken> {
        let session = self.validate_token(token)?;
        
        if let Some(bound_ip) = session.ip_address {
            if bound_ip != ip {
                return Err(SessionError::IpMismatch);
            }
        }
        
        if let Some(bound_ua) = &session.user_agent {
            if bound_ua != user_agent {
                return Err(SessionError::UserAgentMismatch);
            }
        }
        
        Ok(session)
    }
}
```

## 5. HSTS Header Injection Pattern

### Middleware for HSTS
```rust
pub struct HstsMiddleware;

impl HstsMiddleware {
    pub fn inject_hsts_header(response: &mut Response) {
        response.headers_mut().insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains; preload"
                .parse()
                .unwrap(),
        );
    }
}
```

## 6. Perfect Forward Secrecy Pattern

### Ephemeral Key Management
```rust
pub struct EphemeralKeyManager {
    current_key: EphemeralKey,
    rotation_interval: Duration,
}

impl EphemeralKeyManager {
    pub fn get_or_rotate(&mut self) -> Result<&EphemeralKey> {
        if self.current_key.created_at.elapsed() > self.rotation_interval {
            self.rotate_key()?;
        }
        Ok(&self.current_key)
    }

    fn rotate_key(&mut self) -> Result<()> {
        let new_key = EphemeralKey::generate()?;
        self.current_key = new_key;
        Ok(())
    }
}
```

## 7. Replay Attack Prevention Pattern

### Nonce and Timestamp Validation
```rust
pub struct ReplayProtection {
    seen_nonces: HashSet<String>,
    timestamp_tolerance: Duration,
}

impl ReplayProtection {
    pub fn validate(&mut self, nonce: &str, timestamp: DateTime<Utc>) -> Result<()> {
        let now = Utc::now();
        
        // Check timestamp within tolerance
        if (now - timestamp).abs() > self.timestamp_tolerance {
            return Err(ReplayError::TimestampOutOfRange);
        }
        
        // Check nonce not seen before
        if self.seen_nonces.contains(nonce) {
            return Err(ReplayError::NonceReused);
        }
        
        self.seen_nonces.insert(nonce.to_string());
        Ok(())
    }
}
```

## 8. Certificate Rotation Pattern

### Automatic Certificate Renewal
```rust
pub struct CertificateRotationManager {
    cert_path: PathBuf,
    key_path: PathBuf,
    rotation_interval: Duration,
    last_rotation: DateTime<Utc>,
}

impl CertificateRotationManager {
    pub async fn check_and_rotate(&mut self) -> Result<()> {
        if self.last_rotation.elapsed() > self.rotation_interval {
            let (cert_pem, key_pem) = CertificateGenerator::generate_self_signed("zeroclaw")?;
            
            tokio::fs::write(&self.cert_path, cert_pem).await?;
            tokio::fs::write(&self.key_path, key_pem).await?;
            
            self.last_rotation = Utc::now();
        }
        Ok(())
    }
}
```

## 9. Session Cleanup Pattern

### Expired Session Removal
```rust
pub struct SessionCleanup {
    token_store: TokenStore,
    cleanup_interval: Duration,
}

impl SessionCleanup {
    pub async fn cleanup_expired_sessions(&mut self) -> Result<usize> {
        let now = Utc::now();
        let expired_count = self.token_store
            .tokens
            .retain(|_, token| token.expires_at > now);
        
        Ok(expired_count)
    }
}
```

## 10. Configuration Validation

### TLS Configuration Validation
```rust
pub struct TlsConfig {
    certificate_path: PathBuf,
    key_path: PathBuf,
    tls_min_version: TlsVersion,
}

impl TlsConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.certificate_path.exists() {
            return Err(TlsError::CertificateNotFound);
        }
        
        if !self.key_path.exists() {
            return Err(TlsError::KeyNotFound);
        }
        
        // Verify TLS version is 1.3
        if self.tls_min_version != TlsVersion::Tls13 {
            return Err(TlsError::UnsupportedTlsVersion);
        }
        
        Ok(())
    }
}
```

## 11. Success Criteria

- [ ] TLS 1.3 enforced, no downgrade
- [ ] TLS handshake <100ms
- [ ] Session tokens 32+ bytes
- [ ] Token validation <1ms
- [ ] Certificate auto-generation working
- [ ] HSTS headers on all responses
- [ ] Perfect forward secrecy implemented
- [ ] Replay protection working
- [ ] Performance benchmarks met
