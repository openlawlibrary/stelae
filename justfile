#!/usr/bin/env just --justfile

# List all available commands
default:
  @just --list

# Format code and run strict clippy 
lint: format clippy

# Format code
format:
  cargo fmt --all -- --check

# Run all tests
test:
  cargo test
nextest:
  cargo nextest run --all --no-fail-fast && cargo test --doc

# Run clippy maximum strictness. Passes through any flags to clippy.
clippy *FLAGS:
  cargo clippy \
    {{FLAGS}} \
    --all -- \
    -D warnings \

# Continuous integration - test, lint, benchmark
ci: lint nextest bench

# Run all benchmarks
bench:
  cargo bench
