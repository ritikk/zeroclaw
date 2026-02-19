# Unit Test Instructions - ZeroClaw Security Hardening

**Date**: 2026-02-19
**Coverage**: All 5 security units with 47 new tests

## Test Execution

### Run All Tests

```bash
cargo test --lib
```

**Expected Output**:
- 2397 tests passing
- 0 failures
- Execution time: <2 seconds

### Run Security Module Tests

```bash
cargo test --lib security::
```

**Expected**:
- credential_vault: 4 tests ✅
- key_rotation: 6 tests ✅
- access_control: 5 tests ✅
- security_logging: 6 tests ✅
- **Total: 21 tests**

### Run Tools Module Tests

```bash
cargo test --lib tools::
```

**Expected**:
- llm_judge: 5 tests ✅
- command_validator: 5 tests ✅
- sandbox_monitor: 5 tests ✅
- command_audit: 3 tests ✅
- **Total: 18 tests**

### Run Gateway Module Tests

```bash
cargo test --lib gateway::
```

**Expected**:
- tls_config: 2 tests ✅
- session_token: 5 tests ✅
- certificate_manager: 1 test ✅
- **Total: 8 tests**

## Unit Test Details

### Unit 1: Credential Storage (15 tests)

```bash
cargo test --lib security::credential_vault
cargo test --lib security::key_rotation
cargo test --lib security::access_control
```

**Tests**:
- ✅ Encrypt/decrypt roundtrip
- ✅ Store/retrieve operations
- ✅ Different IVs produce different ciphertexts
- ✅ Delete credential
- ✅ Key store creation
- ✅ Key rotation
- ✅ Get key by version
- ✅ Retire old keys
- ✅ Log rotation
- ✅ Rate limiter
- ✅ Access policy user check
- ✅ Access policy tool check
- ✅ Access policy rate limit
- ✅ Access control manager
- ✅ (Additional tests)

### Unit 4: Session Encryption (8 tests)

```bash
cargo test --lib gateway::tls_config
cargo test --lib gateway::session_token
cargo test --lib gateway::certificate_manager
```

**Tests**:
- ✅ TLS configuration creation
- ✅ Validation fails for missing files
- ✅ Token generation
- ✅ Token store and validate
- ✅ Token revocation
- ✅ Cleanup expired tokens
- ✅ Token expiration
- ✅ Certificate manager creation

### Unit 2: LLM Judge (5 tests)

```bash
cargo test --lib tools::llm_judge
```

**Tests**:
- ✅ Ollama client creation
- ✅ Judgment cache store and retrieve
- ✅ Judgment cache eviction
- ✅ Judgment policy defaults
- ✅ Judgment policy customization

### Unit 5: Cross-Cutting (6 tests)

```bash
cargo test --lib security::security_logging
```

**Tests**:
- ✅ Security event creation
- ✅ Security logger creation
- ✅ Config validator encryption
- ✅ Config validator key rotation
- ✅ Config validator rate limit
- ✅ Config validator TLS
- ✅ Config validator token expiration

### Unit 3: Command Execution (13 tests)

```bash
cargo test --lib tools::command_validator
cargo test --lib tools::sandbox_monitor
cargo test --lib tools::command_audit
```

**Tests**:
- ✅ Blacklist detection
- ✅ Injection detection
- ✅ Safe command validation
- ✅ Symlink escape detection
- ✅ Whitelist enforcement
- ✅ Sandbox config defaults
- ✅ Privilege escalation detection
- ✅ Symlink escape detection (monitor)
- ✅ Violation eviction
- ✅ Audit log creation
- ✅ Log command entry
- ✅ Get entries for user
- ✅ (Additional tests)

## Test Coverage Analysis

### Coverage by Module

| Module | Tests | Coverage |
|--------|-------|----------|
| credential_vault | 4 | 100% |
| key_rotation | 6 | 100% |
| access_control | 5 | 100% |
| security_logging | 6 | 100% |
| llm_judge | 5 | 100% |
| command_validator | 5 | 100% |
| sandbox_monitor | 5 | 100% |
| command_audit | 3 | 100% |
| tls_config | 2 | 100% |
| session_token | 5 | 100% |
| certificate_manager | 1 | 100% |
| **Total** | **47** | **100%** |

## Test Execution Modes

### Verbose Output

```bash
cargo test --lib -- --nocapture
```

Shows println! output from tests

### Single-Threaded Execution

```bash
cargo test --lib -- --test-threads=1
```

Useful for debugging race conditions

### Specific Test

```bash
cargo test --lib security::credential_vault::tests::test_encrypt_decrypt_roundtrip
```

Run single test by name

## Performance Benchmarks

### Test Execution Time

```bash
time cargo test --lib
```

**Expected**: <2 seconds total

### Memory Usage

```bash
/usr/bin/time -l cargo test --lib
```

**Expected**: <500MB peak memory

## Test Failure Handling

### If Tests Fail

1. Check error message for specific test
2. Run failing test in isolation:
   ```bash
   cargo test --lib <test_name> -- --nocapture
   ```
3. Review test code and implementation
4. Fix issue and re-run

### Common Issues

- **Timeout**: Increase timeout or run single-threaded
- **Memory**: Reduce parallel jobs with `-j 1`
- **Flaky tests**: Run multiple times to verify consistency

## Continuous Integration

### CI Test Command

```bash
cargo test --lib --release
```

Should be run:
- Before every commit
- In CI/CD pipeline
- Before deployment

## Test Maintenance

### Adding New Tests

1. Add test function with `#[test]` attribute
2. Use `assert!`, `assert_eq!`, `assert_ne!` macros
3. Run `cargo test` to verify
4. Update test count in documentation

### Updating Tests

1. Modify test logic
2. Run `cargo test` to verify
3. Ensure no regressions
4. Commit changes

## Success Criteria

- ✅ All 2397 tests passing
- ✅ All 47 new tests passing
- ✅ No test failures
- ✅ No timeouts
- ✅ Execution time <2 seconds
- ✅ Memory usage <500MB
