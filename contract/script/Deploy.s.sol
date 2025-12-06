// SPDX-License-Identifier: MIT
pragma solidity ^0.8.10;

import {Script, console} from "forge-std/Script.sol";

import {TradeExecutor} from "../src/TradeExecutor.sol";

contract DeployTradeExecutor is Script {
    function run() external {
        // Load private key and payee address from .env
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address payee = vm.envAddress("PAYEE_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);

        // Deploy your contract
        TradeExecutor trade_executor = new TradeExecutor(payee);

        console.log(" TradeExecutor deployed at:", address(trade_executor));

        vm.stopBroadcast();
    }
}
