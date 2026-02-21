# Docker Build Issue Summary

## Status
Docker build fails during `cargo build --release` inside the container.

## Root Cause Chain

1. **Rust version mismatch** — Original Dockerfile used `rust:1.70`, but `matrix-sdk@0.16.0` requires rustc ≥ 1.88.

2. **Cargo.lock format** — Cargo.lock is version 4 format (generated locally with Rust 1.82+), which `rust:1.70` cannot parse.

3. **Missing workspace member** — `Cargo.toml` referenced `crates/robot-kit` which doesn't exist in the repo. Fixed by removing it.

4. **Missing bench file** — `Cargo.toml` had `[[bench]] name = "agent_benchmarks"` with no corresponding file. Fixed by removing it.

5. **Compile error in container** — After fixing the above, `rust:1.88` starts compiling zeroclaw but fails with an unknown error (interrupted before capturing).

## Fixes Applied So Far
- `Cargo.toml`: removed `crates/robot-kit` workspace member
- `Cargo.toml`: removed `[[bench]]` agent_benchmarks entry
- `Cargo.lock`: regenerated with `cargo generate-lockfile`
- `Dockerfile`: updated from `rust:1.70` → `rust:1.88`

## Current State
- Local `cargo check` and `cargo build --release` both pass ✅
- Docker build reaches zeroclaw compilation but fails ❌
- Ollama service pulls and starts successfully ✅

## Next Steps to Investigate
1. Run `docker build --progress=plain` and capture the full compile error from zeroclaw
2. Likely a missing system dependency (e.g., `libssl-dev`, `pkg-config`) in the builder stage
3. Fix: add `RUN apt-get install -y pkg-config libssl-dev` before `cargo build` in Dockerfile

## Suggested Dockerfile Fix
```dockerfile
FROM rust:1.88 as builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
```
