#!/usr/bin/env just --justfile

release:
  cargo build --release    

lint:
  cargo clippy

bin:
  cargo run --bin bin -- arg1

format:
    cargo fmt

check-format:
    cargo fmt --check

test:
    cargo test
