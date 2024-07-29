// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import { StdAssertions } from "forge-std/StdAssertions.sol";

interface GovernanceToken {
  function name() external view returns (string memory);
  function approve(address spender, uint256 amount) external returns (bool);
  function allowance(address owner, address spender) external view returns (uint256);
}

/// Calls the Governance Token Precompile
contract GovernanceTokenPrecompile is Script, StdAssertions {
  GovernanceToken constant TOKEN = GovernanceToken(address(0x4200000000000000000000000000000000000042));

  function setUp() public {}

  function run() public {
    // Validation
    assertEq(TOKEN.name(), "Optimism");

    address spender = address(0xdeadeee);

    // Check the allowance of the spender
    uint256 allowance = TOKEN.allowance(msg.sender, spender);
    assertEq(allowance, 0);

    // Increase allowance of the spender
    vm.broadcast();
    TOKEN.approve(spender, 100);

    // Check the allowance of the spender
    allowance = TOKEN.allowance(msg.sender, spender);
    assertEq(allowance, 100);
  }
}
