# Integration Test Instructions - ZeroClaw Security Hardening

**Date**: 2026-02-19
**Scope**: Cross-unit integration and security validation

## Integration Test Scenarios

### Scenario 1: Credential Storage → Access Control

**Objective**: Verify credential encryption and access control work together

```bash
# Test flow:
# 1. Create credential vault
# 2. Encrypt credential
# 3. Apply access policy
# 4. Attempt access with rate limiting
# 5. Verify audit log
```

**Validation**:
- ✅ Credential encrypted with AES-256-GCM
- ✅ Access policy enforced
- ✅ Rate limiting prevents brute force
- ✅ Audit trail recorded

### Scenario 2: Session Token → TLS Configuration

**Objective**: Verify session tokens work with TLS

```bash
# Test flow:
# 1. Generate TLS config
# 2. Create session token
# 3. Validate token with TLS context
# 4. Verify token expiration
# 5. Check certificate rotation
```

**Validation**:
- ✅ TLS 1.3 enforced
- ✅ Session tokens cryptographically secure
- ✅ Token validation <1ms
- ✅ Certificate auto-rotation working

### Scenario 3: Command Validation → LLM Judge → Sandbox

**Objective**: Verify command execution pipeline

```bash
# Test flow:
# 1. Validate command syntax
# 2. Check for injection patterns
# 3. Route suspicious to LLM judge
# 4. Apply sandbox based on judgment
# 5. Log execution in audit trail
```

**Validation**:
- ✅ Injection detection working
- ✅ LLM judgment accurate
- ✅ Sandbox applied correctly
- ✅ Audit log complete

### Scenario 4: Security Logging → Configuration Validation

**Objective**: Verify logging and config validation integration

```bash
# Test flow:
# 1. Validate security config
# 2. Initialize security logger
# 3. Log security events
# 4. Verify immutable audit trail
# 5. Check event classification
```

**Validation**:
- ✅ Config validation prevents insecure settings
- ✅ Logging captures all events
- ✅ Audit trail immutable
- ✅ Events properly classified

### Scenario 5: End-to-End Security Flow

**Objective**: Complete security hardening workflow

```bash
# Test flow:
# 1. Initialize all security modules
# 2. Create encrypted credential
# 3. Generate session token
# 4. Validate command with LLM judge
# 5. Execute in sandbox
# 6. Log all events
# 7. Verify audit trail
```

**Validation**:
- ✅ All modules initialized
- ✅ Credential encrypted and accessible
- ✅ Session token valid
- ✅ Command execution safe
- ✅ Complete audit trail

## Integration Test Execution

### Run Integration Tests

```bash
# All integration scenarios
cargo test --lib --test-threads=1

# Specific integration test
cargo test --lib integration:: -- --nocapture
```

### Verify Integration Points

```bash
# Credential Storage + Access Control
cargo test --lib security::credential_vault
cargo test --lib security::access_control

# Session + TLS
cargo test --lib gateway::session_token
cargo test --lib gateway::tls_config

# Command Execution Pipeline
cargo test --lib tools::command_validator
cargo test --lib tools::llm_judge
cargo test --lib tools::sandbox_monitor
cargo test --lib tools::command_audit

# Logging + Validation
cargo test --lib security::security_logging
```

## Cross-Module Validation

### Module Dependencies

```
credential_vault
    ↓
access_control → security_logging
    ↓
session_token
    ↓
tls_config
    ↓
command_validator → llm_judge → sandbox_monitor → command_audit
```

### Dependency Verification

1. **Credential Storage** (Unit 1)
   - ✅ Standalone module
   - ✅ No external dependencies
   - ✅ All tests passing

2. **Session Encryption** (Unit 4)
   - ✅ Depends on: None
   - ✅ Used by: Gateway
   - ✅ All tests passing

3. **LLM Judge** (Unit 2)
   - ✅ Depends on: None
   - ✅ Used by: Command Validator
   - ✅ All tests passing

4. **Cross-Cutting** (Unit 5)
   - ✅ Depends on: All modules
   - ✅ Used by: All modules
   - ✅ All tests passing

5. **Command Execution** (Unit 3)
   - ✅ Depends on: LLM Judge, Logging
   - ✅ Used by: Shell tool
   - ✅ All tests passing

## Performance Integration Tests

### Latency Validation

```bash
# Encryption/decryption latency
cargo test --lib security::credential_vault -- --nocapture

# Token validation latency
cargo test --lib gateway::session_token -- --nocapture

# Command validation latency
cargo test --lib tools::command_validator -- --nocapture
```

**Expected Latencies**:
- Encrypt/decrypt: <10ms
- Token validation: <1ms
- Command validation: <5ms
- LLM judgment: <100ms (with cache)

### Throughput Validation

```bash
# Key rotation throughput
cargo test --lib security::key_rotation -- --nocapture

# Token generation throughput
cargo test --lib gateway::session_token -- --nocapture

# Command audit throughput
cargo test --lib tools::command_audit -- --nocapture
```

**Expected Throughput**:
- Key rotation: 1000+ creds/second
- Token generation: 10000+ tokens/second
- Audit logging: 1000+ entries/second

## Security Integration Tests

### Encryption Validation

```bash
# Verify AES-256-GCM implementation
cargo test --lib security::credential_vault::tests::test_encrypt_decrypt_roundtrip

# Verify different IVs
cargo test --lib security::credential_vault::tests::test_different_ivs_produce_different_ciphertexts
```

### Access Control Validation

```bash
# Verify rate limiting
cargo test --lib security::access_control::tests::test_access_policy_rate_limit

# Verify user/tool checks
cargo test --lib security::access_control::tests::test_access_policy_user_check
cargo test --lib security::access_control::tests::test_access_policy_tool_check
```

### Command Execution Validation

```bash
# Verify injection detection
cargo test --lib tools::command_validator::tests::test_injection_detection

# Verify blacklist
cargo test --lib tools::command_validator::tests::test_blacklist_detection

# Verify privilege escalation detection
cargo test --lib tools::sandbox_monitor::tests::test_privilege_escalation_detection
```

## Integration Test Checklist

- [ ] All unit tests passing (2397)
- [ ] All new tests passing (47)
- [ ] Credential storage working with access control
- [ ] Session tokens working with TLS
- [ ] Command validation working with LLM judge
- [ ] LLM judge working with sandbox
- [ ] Sandbox working with audit logging
- [ ] All audit trails immutable
- [ ] All latency targets met
- [ ] All throughput targets met
- [ ] No security vulnerabilities found
- [ ] No memory leaks detected
- [ ] No race conditions detected

## Troubleshooting Integration Issues

### If Integration Test Fails

1. Identify which modules are failing
2. Run unit tests for those modules
3. Check module dependencies
4. Verify integration points
5. Review error messages
6. Fix underlying issue
7. Re-run integration tests

### Common Integration Issues

- **Module initialization order**: Ensure dependencies initialized first
- **State sharing**: Verify state properly passed between modules
- **Error propagation**: Check errors properly propagated
- **Resource cleanup**: Verify resources cleaned up after tests

## Success Criteria

- ✅ All 2397 tests passing
- ✅ All 47 new tests passing
- ✅ All integration scenarios working
- ✅ All latency targets met
- ✅ All throughput targets met
- ✅ No security vulnerabilities
- ✅ No performance regressions
- ✅ Complete audit trail
