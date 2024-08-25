#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
T8N_PATH="$SCRIPT_DIR/../../target/release/opt8n"
T8N_NETWORK_PARAMS=$(cat <<- EOM
optimism_package:
  participants:
    - el_type: op-geth
      cl_type: op-node
  additional_services:
    - blockscout
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

echo "Building opt8n..."
cargo build --bin opt8n --release

echo "Starting Kurtosis network..."
cd "$KURTOSIS_DIR" || exit 1
kurtosis clean -a
echo "$T8N_NETWORK_PARAMS" > ./network_params.yaml
kurtosis run --enclave devnet . --args-file ./network_params.yaml

echo "Returning to opt8n..."
cd "$SCRIPT_DIR" || exit 1

# Download L2 genesis configs and contract addresses
echo "Downloading L2 genesis configs and contract addresses from devnet..."
kurtosis files download devnet op-genesis-configs

OPTIMISM_PORTAL_PROXY=$(jq -r .OptimismPortalProxy ./op-genesis-configs/kurtosis.json)
GENESIS="./op-genesis-configs/genesis.json"

L1_PORT=$(kurtosis enclave inspect devnet | grep 'el-1-reth-lighthouse' -A5 | grep " rpc:" | awk -F ' -> ' '{print $2}' | awk -F ':' '{print $2}' | tr -d ' \n\r')
L2_PORT=$(kurtosis enclave inspect devnet | grep 'op-el-1-op-geth-op-node' -A5 | grep " rpc:" | awk -F ' -> ' '{print $2}' | awk -F ':' '{print $3}' | tr -d ' \n\r')

$T8N_PATH "$L1_PORT" --l2-port "$L2_PORT" -o "$OPTIMISM_PORTAL_PROXY" --l2-genesis "$GENESIS"

echo "Cleaning up genesis + configs..."
rm -rf ./op-genesis-configs
