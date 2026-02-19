# Build Instructions - ZeroClaw Security Hardening

**Date**: 2026-02-19
**Target**: Production-ready security hardening implementation

## Prerequisites

- Rust 1.70+ (stable)
- Cargo
- System build tools (gcc/clang)
- OpenSSL development headers (for rustls)

## Build Steps

### 1. Clean Build

```bash
cd /Users/ritik/Projects/Claw/zeroclaw
cargo clean
cargo build --release --locked
```

**Expected Output**:
- Compilation time: ~2-3 minutes
- Binary size: ~8-10MB (release)
- No warnings or errors

### 2. Verify Build Artifacts

```bash
ls -lh target/release/zeroclaw
file target/release/zeroclaw
```

**Expected**:
- Binary exists and is executable
- Size: 8-10MB
- Type: ELF 64-bit executable

### 3. Run All Tests

```bash
cargo test --lib --release
```

**Expected**:
- 2397 tests passing
- 0 failures
- Execution time: <2 seconds

### 4. Run Security-Specific Tests

```bash
cargo test --lib security:: --release
cargo test --lib tools::command_validator --release
cargo test --lib tools::sandbox_monitor --release
cargo test --lib tools::command_audit --release
cargo test --lib tools::llm_judge --release
```

**Expected**:
- All security module tests passing
- All command execution tests passing
- All LLM judge tests passing

### 5. Clippy Linting

```bash
cargo clippy --all-targets --release -- -D warnings
```

**Expected**:
- No clippy warnings
- All suggestions addressed

### 6. Format Check

```bash
cargo fmt --all -- --check
```

**Expected**:
- All files properly formatted
- No formatting issues

## Build Profiles

### Development Build (Fast)
```bash
cargo build
```
- Faster compilation
- Larger binary (~20MB)
- Debug symbols included

### Release Build (Optimized)
```bash
cargo build --release --locked
```
- Slower compilation
- Smaller binary (~8MB)
- Optimized for performance
- **Recommended for production**

### Fast Release Build (High-RAM)
```bash
cargo build --profile release-fast
```
- Faster compilation than release
- Requires 16GB+ RAM
- Slightly larger binary

## Verification Checklist

- [ ] `cargo check --lib` passes
- [ ] `cargo test --lib` passes (2397 tests)
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt --check` passes
- [ ] Binary size is 8-10MB
- [ ] No security warnings in build output
- [ ] All dependencies locked in Cargo.lock

## Troubleshooting

### Build Fails with OpenSSL Error
```bash
# Use rustls instead
cargo build --release --locked --no-default-features
```

### Out of Memory During Build
```bash
# Use slower but less memory-intensive build
cargo build --release -j 1
```

### Tests Timeout
```bash
# Increase timeout
cargo test --lib -- --test-threads=1
```

## Build Output Artifacts

After successful build:
- `target/release/zeroclaw` - Main binary
- `target/release/deps/zeroclaw-*.d` - Dependency files
- `Cargo.lock` - Locked dependencies

## Next Steps

After successful build:
1. Proceed to unit testing
2. Run integration tests
3. Execute performance benchmarks
4. Conduct security audit
