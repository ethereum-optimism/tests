// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import {StdAssertions} from "forge-std/StdAssertions.sol";

contract SelfDestructor {
    constructor() {
        selfdestruct(payable(address(0x0)));
    }
}

contract SelfDestructPrecompile is Script, StdAssertions {
    function setUp() public {}

    function run() public {
        // Selfdestruct the contract
        vm.broadcast();
        new SelfDestructor();
    }
}
