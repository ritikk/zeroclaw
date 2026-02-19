# Unit 1: Credential Storage Hardening - NFR Requirements

**Date**: 2026-02-19
**Unit**: Credential Storage Hardening
**Phase**: CONSTRUCTION - NFR Requirements

## 1. Performance Requirements

| Requirement | Target | Rationale |
|-------------|--------|-----------|
| Credential decrypt latency | <10ms | User-facing operations must be responsive |
| Key rotation time (1000 creds) | <5 seconds | Background operation, minimal impact |
| Rate limit check latency | <1ms | Per-access overhead must be minimal |
| Access log write latency | <5ms | Async, non-blocking |

## 2. Security Requirements

| Requirement | Specification |
|-------------|---------------|
| Encryption algorithm | AES-256-GCM |
| Key size | 256 bits |
| IV size | 96 bits (random) |
| Authentication tag | 128 bits |
| Key derivation | PBKDF2 (100k iterations) or Argon2 |
| Random number generator | OsRng (cryptographically secure) |
| Memory protection | Zeroize on drop |
| Key rotation | Every 90 days (configurable) |
| Credential TTL | Per-type (API: 365d, OAuth: 30d, DB: 90d) |

## 3. Scalability Requirements

| Requirement | Target |
|-------------|--------|
| Max credentials per store | 100,000 |
| Max concurrent accesses | 1,000 |
| Key rotation throughput | 1,000 creds/second |
| Access log retention | 90 days |
| Memory overhead per credential | <1KB |

## 4. Reliability Requirements

| Requirement | Specification |
|-------------|---------------|
| Key backup | Automatic encrypted backup |
| Rotation failure recovery | Rollback to previous key |
| Partial migration recovery | Resume from checkpoint |
| Data integrity | HMAC verification on decrypt |
| Audit trail immutability | Append-only log |

## 5. Compliance Requirements

| Requirement | Standard |
|-------------|----------|
| Encryption | NIST SP 800-38D (GCM) |
| Key management | NIST SP 800-57 |
| Random generation | NIST SP 800-90A |
| Credential handling | OWASP Secrets Management |

## 6. Operational Requirements

| Requirement | Specification |
|-------------|---------------|
| Configuration | TOML-based, validated on startup |
| Monitoring | Metrics for key rotation, access patterns |
| Alerting | Failed decryption, rate limit breaches |
| Logging | Structured JSON logs, no plaintext secrets |
| Backup/restore | Encrypted backup with versioning |

## 7. Tech Stack Selection

### Encryption Library
- **Choice**: `aes-gcm` crate (NIST-approved)
- **Rationale**: Hardware-accelerated on modern CPUs, well-audited
- **Alternative**: `chacha20poly1305` (if AES unavailable)

### Key Derivation
- **Choice**: `pbkdf2` crate (NIST SP 800-132)
- **Rationale**: Widely supported, configurable iterations
- **Alternative**: `argon2` (memory-hard, better for password-based)

### Memory Protection
- **Choice**: `zeroize` crate
- **Rationale**: Prevents compiler optimizations that skip zeroing
- **Alternative**: Manual volatile writes (less reliable)

### Random Generation
- **Choice**: `rand::rngs::OsRng`
- **Rationale**: OS-provided cryptographic randomness
- **Alternative**: `getrandom` (lower-level, more control)

### Serialization
- **Choice**: `serde` + `bincode` (binary) or `serde_json` (text)
- **Rationale**: Flexible, widely supported
- **Alternative**: `protobuf` (more compact)

## 8. Success Criteria

- [ ] All encryption operations meet <10ms latency
- [ ] Key rotation completes in <5 seconds for 1000 credentials
- [ ] Zero plaintext secrets in logs or memory dumps
- [ ] 100% test coverage for encryption paths
- [ ] Compliance with NIST standards verified
- [ ] Performance benchmarks documented
