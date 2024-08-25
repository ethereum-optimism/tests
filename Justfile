set positional-arguments
alias t := test
alias f := fmt
alias l := lint
alias b := build
alias opt8n := run

# default recipe to display help information
default:
  @just --list

# Runs opt8n
run:
  cargo run --bin opt8n

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

# Generates all test fixtures for scripts in examples/exec-scripts
gen fork_url:
  @just ./examples/exec-scripts/gen {{fork_url}}

# Install the devnet
install-devnet:
  #!/bin/bash

  if [ -d "./devnet" ]; then
    exit 0
  fi

  git clone https://github.com/ethpandaops/optimism-package && mv optimism-package devnet

  T8N_NETWORK_PARAMS=$(cat <<- "EOM"
  optimism_package:
    participants:
      - el_type: op-geth
        cl_type: op-node
    network_params:
      seconds_per_slot: 2
      network_id: 1337
  ethereum_package:
    participants:
      - el_type: reth
        cl_type: lighthouse
    network_params:
      preset: minimal
  EOM
  )
  printf "%s" "$T8N_NETWORK_PARAMS" > ./devnet/network_params.yaml

# Start the devnet
start-devnet:
  #!/bin/bash

  SCRIPT_DIR=$( pwd )
  KURTOSIS_DIR="$SCRIPT_DIR/devnet"

  # Exit if Kurtosis is already running
  kurtosis enclave inspect devnet && exit 0

  echo "Starting Kurtosis network..."
  cd "$KURTOSIS_DIR" || exit 1
  kurtosis clean -a
  kurtosis run --enclave devnet . --args-file ./network_params.yaml

  echo "Returning to opt8n..."
  cd "$SCRIPT_DIR" || exit 1

# Stop the devnet
stop-devnet:
  #!/bin/bash
  kurtosis clean -a

# Run t8n
t8n *args='': install-devnet start-devnet
  #!/bin/bash

  SCRIPT_DIR=$( pwd )
  T8N_PATH="$SCRIPT_DIR/target/release/opt8n"

  echo "Building opt8n..."
  cargo build --bin opt8n --release

  # Download L2 genesis configs and contract addresses
  echo "Downloading L2 genesis configs and contract addresses from devnet..."
  kurtosis files download devnet op-genesis-configs

  OPTIMISM_PORTAL_PROXY=$(jq -r .OptimismPortalProxy ./op-genesis-configs/kurtosis.json)
  GENESIS="./op-genesis-configs/genesis.json"

  L1_PORT=$(kurtosis enclave inspect devnet | grep 'el-1-reth-lighthouse' -A5 | grep " rpc:" | awk -F ' -> ' '{print $2}' | awk -F ':' '{print $2}' | tr -d ' \n\r')
  L2_PORT=$(kurtosis enclave inspect devnet | grep 'op-el-1-op-geth-op-node' -A5 | grep " rpc:" | awk -F ' -> ' '{print $2}' | awk -F ':' '{print $3}' | tr -d ' \n\r')

  $T8N_PATH \
    --l1-port "$L1_PORT" \
    --l2-port "$L2_PORT" \
    -o "$OPTIMISM_PORTAL_PROXY" \
    --l2-genesis "$GENESIS" \
    $@

  echo "Cleaning up genesis + configs..."
  rm -rf ./op-genesis-configs
