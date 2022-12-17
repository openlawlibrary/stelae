#!/usr/bin/env just --justfile

set shell := ["nu", "-c"]

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

# Run clippy maximum strictness. Passes through any flags to clippy.
clippy *FLAGS:
  cargo clippy \
    {{FLAGS}} \
    --all -- \
    -W missing-docs \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::restriction \
    -W clippy::nursery \
    -D warnings \

# Continuous integration - test, lint, benchmark
ci: lint test bench

# Run all benchmarks
bench:
  cargo bench