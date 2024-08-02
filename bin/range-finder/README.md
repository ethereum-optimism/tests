# `range-finder`

Determines the L1 range of blocks needed to derive a given L2 block.

Effectively, `range-finder` runs derivation from the l1 origin of
each L2 block between the range of the specified starting L2 block
number and ending L2 block number. The outputted range of blocks
is the L1 blocks with blobs needed to derive the associated L2 block.

## Usage

- `-v` (`-vv`, `-vvv`, ..): The verbosity to log. 
- `--start-block`: The starting L2 block number.
- `--end-block`: The ending L2 block number.
- `--l1-rpc-url`: An L1 RPC URL used by the derivation pipeline.
- `--l2-rpc-url`: An L2 RPC URL used by the derivation pipeline.
- `--beacon-url`: A beacon client url used by the derivation pipeline.
