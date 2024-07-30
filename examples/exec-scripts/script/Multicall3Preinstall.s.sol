// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import { StdAssertions } from "forge-std/StdAssertions.sol";

interface Multicall3 {
  struct Call3 {
      address target;
      bool allowFailure;
      bytes callData;
  }

  struct Result {
      bool success;
      bytes returnData;
  }

  function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);
}

interface GovernanceToken {
  function name() external view returns (string memory);
  function approve(address spender, uint256 amount) external returns (bool);
  function allowance(address owner, address spender) external view returns (uint256);
}

contract Multicall3Preinstall is Script, StdAssertions {
  Multicall3 constant MULTICALL = Multicall3(address(0xcA11bde05977b3631167028862bE2a173976CA11));
  GovernanceToken constant TOKEN = GovernanceToken(address(0x4200000000000000000000000000000000000042));

  function setUp() public {}

  function run() public {
    Multicall3.Call3[] memory calls = new Multicall3.Call3[](2);
    calls[0] = Multicall3.Call3({
      target: address(TOKEN),
      allowFailure: false,
      callData: abi.encodeWithSelector(TOKEN.name.selector)
    });
    calls[1] = Multicall3.Call3({
      target: address(TOKEN),
      allowFailure: false,
      callData: abi.encodeWithSelector(TOKEN.allowance.selector, msg.sender, address(0xdeadeee))
    });

    vm.broadcast();
    Multicall3.Result[] memory results = MULTICALL.aggregate3(calls);
    assertEq(string(abi.decode(results[0].returnData, (string))), "Optimism");
    assertEq(abi.decode(results[1].returnData, (uint256)), 0);
  }
}
