# Security Hardening Requirements - ZeroClaw

**Date**: 2026-02-19
**Scope**: Credential storage, command execution, input validation with LLM judge, session encryption
**Risk Level**: High (security-critical infrastructure)

## 1. Credential Storage Security

### Current State
- Encryption at rest via `SecretStore` (AES-256-GCM with random IV)
- Encrypted key file at `~/.zeroclaw/.secret_key`
- Support for multiple auth profiles with encryption
- Nested credential encryption in config

### Requirements

#### 1.1 Encryption Key Management
- **REQ-CS-1.1**: Implement key rotation mechanism with versioning
  - Support multiple active keys during rotation period
  - Automatic re-encryption of old credentials with new key
  - Audit trail for key rotation events
  
- **REQ-CS-1.2**: Secure key derivation
  - Use PBKDF2 or Argon2 for key derivation from master password (optional)
  - Minimum 100,000 iterations for PBKDF2 or equivalent for Argon2
  - Salt stored separately from key

#### 1.2 Credential Storage Isolation
- **REQ-CS-2.1**: Separate storage for different credential types
  - API keys in dedicated encrypted store
  - OAuth tokens in separate encrypted store with TTL tracking
  - Database credentials in isolated encrypted vault
  
- **REQ-CS-2.2**: Memory protection
  - Clear sensitive data from memory after use (zeroize crate)
  - Prevent credential leakage in debug logs
  - Disable core dumps when credentials are in memory

#### 1.3 Credential Access Control
- **REQ-CS-3.1**: Fine-grained access policies
  - Per-credential access logging
  - Rate limiting on credential access (prevent brute force)
  - Automatic credential rotation on suspicious access patterns

- **REQ-CS-3.2**: Credential expiration
  - Support TTL for temporary credentials
  - Automatic refresh before expiration
  - Alert on expired credentials

---

## 2. Command Execution Security

### Current State
- Shell tool with basic input validation
- Docker sandbox support
- Filesystem scoping with workspace_only mode
- Landlock/Bubblewrap/Firejail sandbox options

### Requirements

#### 2.1 Command Validation Pipeline
- **REQ-CE-1.1**: Pre-execution validation
  - Whitelist allowed commands (configurable)
  - Blacklist dangerous commands (rm -rf, dd, etc.)
  - Validate command syntax before execution
  - Detect command injection patterns

- **REQ-CE-1.2**: LLM-based command judgment (Ollama)
  - Route suspicious commands to LLM for safety assessment
  - Configurable Ollama endpoint (local or remote)
  - Fallback to conservative deny on LLM unavailability
  - Cache LLM judgments for identical commands

#### 2.2 Execution Sandboxing
- **REQ-CE-2.1**: Enhanced sandbox isolation
  - Mandatory sandbox for untrusted commands
  - Network isolation option (no external connections)
  - Resource limits (CPU, memory, disk I/O)
  - Timeout enforcement with graceful termination

- **REQ-CE-2.2**: Sandbox escape detection
  - Monitor for privilege escalation attempts
  - Detect symlink escape attempts
  - Validate all file operations against workspace boundaries

#### 2.3 Command Audit Trail
- **REQ-CE-3.1**: Comprehensive logging
  - Log all executed commands with timestamp and user
  - Log command output (with sensitive data redaction)
  - Log sandbox violations and escape attempts
  - Immutable audit log (append-only)

---

## 3. Input Validation with LLM Judge

### Current State
- Basic input validation in tools
- No LLM-based judgment system
- No Ollama integration

### Requirements

#### 3.1 LLM Judge Architecture
- **REQ-IV-1.1**: Ollama integration
  - Support local Ollama (default: http://localhost:11434)
  - Support remote Ollama endpoints (configurable URL)
  - Support Docker-hosted Ollama (auto-discovery)
  - Fallback to conservative validation on LLM unavailability

- **REQ-IV-1.2**: Model selection
  - Configurable model for judgment (default: llama2 or mistral)
  - Support for specialized security models
  - Model performance benchmarking

#### 3.2 Judgment Categories
- **REQ-IV-2.1**: Input classification
  - Classify inputs as: safe, suspicious, dangerous, unknown
  - Confidence scoring (0-100%)
  - Reasoning explanation from LLM

- **REQ-IV-2.2**: Judgment caching
  - Cache identical input judgments (24-hour TTL)
  - Cache by input hash to prevent collision attacks
  - Audit cache hits/misses

#### 3.3 Judgment Policies
- **REQ-IV-3.1**: Policy enforcement
  - Safe: execute immediately
  - Suspicious: require explicit user confirmation
  - Dangerous: block by default, allow with override flag
  - Unknown: conservative deny or require confirmation

- **REQ-IV-3.2**: Policy customization
  - Per-tool judgment policies
  - Per-user judgment policies
  - Custom judgment rules (regex, patterns)

---

## 4. Session Encryption

### Current State
- Bearer token authentication for webhooks
- No session encryption for gateway connections
- No TLS enforcement by default

### Requirements

#### 4.1 Transport Security
- **REQ-SE-1.1**: TLS enforcement
  - Mandatory TLS for all gateway connections
  - Support TLS 1.3 minimum
  - Certificate pinning for known endpoints
  - HSTS headers for HTTP responses

- **REQ-SE-1.2**: Certificate management
  - Auto-generate self-signed certs for local development
  - Support custom certificates
  - Certificate rotation mechanism
  - Certificate expiration monitoring

#### 4.2 Session Management
- **REQ-SE-2.1**: Session tokens
  - Cryptographically secure token generation (32+ bytes)
  - Token expiration (configurable, default 24 hours)
  - Token refresh mechanism
  - Revocation support

- **REQ-SE-2.2**: Session state
  - Encrypted session storage
  - Session binding to client IP (optional)
  - Session binding to user agent (optional)
  - Concurrent session limits

#### 4.3 Message Encryption
- **REQ-SE-3.1**: End-to-end encryption
  - Encrypt webhook payloads (optional)
  - Support for message signing
  - Replay attack prevention (nonce/timestamp)

- **REQ-SE-3.2**: Key exchange
  - Diffie-Hellman key exchange for session setup
  - Perfect forward secrecy support
  - Key derivation for message encryption

---

## 5. Cross-Cutting Security Concerns

### 5.1 Logging and Monitoring
- **REQ-CC-1.1**: Security event logging
  - Log all security-relevant events
  - Prevent sensitive data in logs (redaction)
  - Structured logging for analysis
  - Log rotation and retention policies

- **REQ-CC-1.2**: Alerting
  - Real-time alerts for security violations
  - Configurable alert thresholds
  - Integration with monitoring systems

### 5.2 Configuration Security
- **REQ-CC-2.1**: Secure defaults
  - Deny-by-default for all access
  - Explicit opt-in for risky features
  - Validation of security-critical config values

- **REQ-CC-2.2**: Config validation
  - Validate all security-related config on startup
  - Warn on insecure configurations
  - Prevent downgrade attacks

### 5.3 Dependency Security
- **REQ-CC-3.1**: Dependency scanning
  - Regular vulnerability scanning
  - Automated updates for security patches
  - Audit trail for dependency changes

---

## 6. Implementation Priority

### Phase 1 (Critical)
- REQ-CS-1.1: Key rotation mechanism
- REQ-CE-1.2: LLM-based command judgment
- REQ-SE-1.1: TLS enforcement

### Phase 2 (High)
- REQ-CS-2.1: Credential storage isolation
- REQ-CE-2.1: Enhanced sandbox isolation
- REQ-IV-1.1: Ollama integration
- REQ-SE-2.1: Session tokens

### Phase 3 (Medium)
- REQ-CS-3.1: Fine-grained access policies
- REQ-CE-3.1: Comprehensive logging
- REQ-IV-2.1: Input classification
- REQ-SE-3.1: End-to-end encryption

---

## 7. Success Criteria

- [ ] All Phase 1 requirements implemented and tested
- [ ] Security audit passes with no critical findings
- [ ] LLM judge operational with <100ms latency
- [ ] Zero credential leaks in logs/memory
- [ ] All sessions encrypted with TLS 1.3+
- [ ] Audit trail complete and immutable
- [ ] Documentation updated with security guidelines
