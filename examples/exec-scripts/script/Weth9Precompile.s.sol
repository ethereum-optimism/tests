// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import { StdAssertions } from "forge-std/StdAssertions.sol";

interface Weth9 {
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function decimals() external pure returns (uint8);
    function balanceOf(address owner) external view returns (uint256);
    function allowance(address owner, address spender) external view returns (uint256);
    function deposit() external payable;
    function withdraw(uint256 wad) external;
}

/// Calls the WETH9 Precompile
contract Weth9Precompile is Script, StdAssertions {

  Weth9 constant WETH9 = Weth9(address(0x4200000000000000000000000000000000000006));

  function setUp() public {}

  function run() public {
    // Validate Weth9 Precompile Metadata
    assertEq(WETH9.name(), "Wrapped Ether");
    assertEq(WETH9.symbol(), "WETH");
    assertEq(WETH9.decimals(), 18);

    // Deposit 1 Ether
    vm.broadcast();
    WETH9.deposit{value: 1 ether}();
    assertEq(WETH9.balanceOf(address(this)), 1 ether);

    // Withdraw 1 Ether
    vm.broadcast();
    WETH9.withdraw(1 ether);
    assertEq(WETH9.balanceOf(address(this)), 0);
  }
}
