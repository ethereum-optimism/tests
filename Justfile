set positional-arguments
alias t := test
alias f := fmt
alias l := lint
alias b := build

# default recipe to display help information
default:
  @just --list

# Run all tests
tests: test test-docs

# Test for the native target with all features
test *args='':
  cargo nextest run --workspace --all --all-features $@

# Test the Rust documentation
test-docs:
  cargo test --doc --all --locked

# Fixes and checks all workspace formatting
fmt: fmt-fix fmt-check

# Fixes the formatting of the workspace
fmt-fix:
  cargo +nightly fmt --all

# Check the formatting of the workspace
fmt-check:
  cargo +nightly fmt --all -- --check

# Lint workspace and docs
lint: lint-docs clippy

# Lint the Rust documentation
lint-docs:
  RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items

# Run clippy lints on the workspace
clippy:
  cargo +nightly clippy --workspace --all --all-features --all-targets -- -D warnings

# Build for the native target
build *args='':
  cargo build --workspace --all $@
