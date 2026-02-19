# Unit 4: Session Encryption and TLS - NFR Requirements

**Date**: 2026-02-19
**Unit**: Session Encryption and TLS
**Phase**: CONSTRUCTION - NFR Requirements

## 1. Performance Requirements

| Requirement | Target | Rationale |
|-------------|--------|-----------|
| TLS handshake latency | <100ms | Connection establishment must be fast |
| Session token validation | <1ms | Per-request overhead |
| Certificate loading | <50ms | Startup operation |
| HSTS header injection | <1ms | Per-response overhead |

## 2. Security Requirements

| Requirement | Specification |
|-------------|---------------|
| TLS version | 1.3 minimum (no fallback to 1.2) |
| Cipher suites | TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256 |
| Certificate key size | 2048 bits minimum (4096 recommended) |
| Session token size | 32 bytes minimum (base64url encoded) |
| Token expiration | 24 hours (configurable) |
| Refresh token | Optional, separate from session token |
| Perfect forward secrecy | Ephemeral keys for each session |
| HSTS max-age | 31536000 seconds (1 year) |
| Certificate pinning | Optional, SHA256 fingerprints |
| Replay protection | Nonce + timestamp validation |

## 3. Scalability Requirements

| Requirement | Target |
|-------------|--------|
| Concurrent TLS connections | 10,000 |
| Session tokens per user | 5 (configurable) |
| Token store memory | <100MB for 100k tokens |
| Certificate rotation throughput | 1 cert/second |
| Session cleanup rate | 1,000 expired sessions/second |

## 4. Reliability Requirements

| Requirement | Specification |
|-------------|---------------|
| Certificate expiry monitoring | Alert 30 days before expiry |
| Certificate auto-renewal | Automatic 30 days before expiry |
| Session recovery | Graceful degradation on token store failure |
| TLS error handling | Clear error messages, no information leakage |
| Fallback behavior | Reject connections if TLS unavailable |

## 5. Compliance Requirements

| Requirement | Standard |
|-------------|----------|
| TLS | RFC 8446 (TLS 1.3) |
| Certificate | X.509 v3 |
| Session management | OWASP Session Management Cheat Sheet |
| Token generation | NIST SP 800-63B (Authentication) |
| HSTS | RFC 6797 |

## 6. Operational Requirements

| Requirement | Specification |
|-------------|---------------|
| Configuration | TOML-based, validated on startup |
| Certificate paths | Configurable, default ~/.zeroclaw/certs/ |
| Self-signed generation | Automatic on first run |
| Monitoring | Metrics for TLS handshakes, token validation |
| Alerting | Certificate expiry, TLS errors, token revocation |
| Logging | Structured logs, no sensitive data |

## 7. Tech Stack Selection

### TLS Library
- **Choice**: `rustls` crate
- **Rationale**: Pure Rust, no OpenSSL dependency, modern TLS 1.3 support
- **Alternative**: `native-tls` (system-provided, less control)

### Certificate Generation
- **Choice**: `rcgen` crate
- **Rationale**: Pure Rust, self-signed cert generation, X.509 support
- **Alternative**: `openssl` (more features, external dependency)

### Random Number Generation
- **Choice**: `rand::rngs::OsRng`
- **Rationale**: Cryptographically secure, OS-provided
- **Alternative**: `getrandom` (lower-level)

### Serialization
- **Choice**: `serde` + `bincode` (binary)
- **Rationale**: Compact, fast, widely supported
- **Alternative**: `serde_json` (human-readable)

### HTTP Server
- **Choice**: `axum` + `tokio-rustls`
- **Rationale**: Async, modern, TLS support
- **Alternative**: `actix-web` (also good)

## 8. Success Criteria

- [ ] TLS 1.3 enforced, no downgrade possible
- [ ] TLS handshake <100ms latency
- [ ] Session tokens cryptographically secure
- [ ] Token validation <1ms per request
- [ ] Certificate auto-generation and rotation working
- [ ] HSTS headers present on all responses
- [ ] Perfect forward secrecy implemented
- [ ] Replay attack prevention working
- [ ] 100% test coverage for TLS/session paths
- [ ] Performance benchmarks documented
