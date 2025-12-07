// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/Vm.sol";
import {IERC20} from "../src/External.sol";

import "../src/TradeExecutor.sol";

contract TradeExecutorTest is Test {
    TradeExecutor public executor;
    address constant payee = address(0x1337);
    address constant gateway = address(0x42);

    // Token Addresses on Arbitrum
    address constant WETH = 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1;
    address constant USDC = 0xaf88d065e77c8cC2239327C5EDb3A432268e5831;
    address constant USDC_E = 0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8; // Bridged USDC
    address constant ARB = 0x912CE59144191C1204E64559FE8253a0e49E6548;

    function setUp() public {
        vm.createSelectFork("https://arb1.arbitrum.io/rpc");
        executor = new TradeExecutor(payee);
        vm.prank(payee);
        executor.setGateway(gateway);
    }

    function testDecode() public view {
        uint128 payload = 0x000001f401f4ff0201000101;
        (uint8[3] memory exchanges, uint8[3] memory tokens, uint16[3] memory fees) = executor.decode(payload);

        assertEq(exchanges[0], 1, "exchange0");
        assertEq(exchanges[1], 1, "exchange1");
        assertEq(exchanges[2], 0, "exchange2");

        assertEq(tokens[0], 1, "token0");
        assertEq(tokens[1], 2, "token1");
        assertEq(tokens[2], 255, "token2");

        assertEq(fees[0], 500, "fee0");
        assertEq(fees[1], 500, "fee1");
        assertEq(fees[2], 0, "fee2");
    }

    function test_RevertOnUnprofitable_Swap2Step() public {
        // Path: USDC.e -> WETH (UniV3, 0.05% fee) -> USDC.e (UniV3, 0.3% fee)
        uint128 payload = 0x00000bb801f4ff0100000000;
        uint128 amountIn = 10000 * 1e6; // 10,000 USDC.e
        deal(USDC_E, address(executor), amountIn);

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.swap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_Swap3Step() public {
        // Path: WETH -> ARB (Sushi) -> USDC (UniV3) -> WETH (Camelot)
        uint128 payload = 0x00000bb801f4000301010002;
        uint128 amountIn = 3 * 1e18; // 3 WETH
        deal(WETH, address(executor), amountIn);

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.swap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwap2StepUniV3() public {
        // Path: USDC.e -> WETH (UniV3, 0.05%) -> USDC.e (UniV3, 0.3%)
        uint128 payload = 0x00000bb801f4ff0100000000;
        uint128 amountIn = 10000 * 1e6; // 10,000 USDC.e

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwap2StepMixedDex() public {
        // Path: USDC.e -> WETH (Camelot, V2) -> USDC.e (UniV3, 0.3%)
        uint128 payload = 0x00000bb801f4ff0100000100;
        uint128 amountIn = 10000 * 1e6; // 10,000 USDC.e

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwap3Step() public {
        // Path: WETH -> USDC (UniV3, 0.05%) -> ARB (Sushi) -> WETH (UniV3, 0.3%)
        uint128 payload = 0x01f4000001f4030001000200;
        uint128 amountIn = 3 * 1e18; // 3 WETH

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwapXTick() public {
        // Path: WETH -> USDC (UniV3, 0.05%) -> ARB (Sushi) -> WETH (UniV3, 0.3%)
        // Larger amount to test crossing ticks.
        uint128 payload = 0x01f4000001f4030001000200;
        uint128 amountIn = 1000 * 1e18; // 1000 WETH

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwapV2Entry() public {
        // Path: WETH -> USDC (Sushi) -> WETH (UniV3, 0.05%)
        uint128 payload = 0x01f401f401f4ff0001000002;
        uint128 amountIn = 5 * 1e18; // 5 WETH

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwapV2EntryOneForZero() public {
        // Path: USDC -> WETH (Sushi) -> USDC (UniV3, 0.05%)
        uint128 payload = 0x01f401f401f4ff0001000201;
        uint128 amountIn = 5000 * 1e6; // 5000 USDC

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwap3StepOneForZero() public {
        // Path: USDC -> WETH (UniV3, 0.3%) -> ARB (Sushi) -> USDC (UniV3, 0.05%)
        uint128 payload = 0x01f4000001f4030100000200;
        uint128 amountIn = 10000 * 1e6; // 10,000 USDC

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function test_RevertOnUnprofitable_FlashSwapChronos() public {
        // Path: ARB -> USDC.e (Chronos, 0.01%) -> ARB (UniV3, 0.05%)
        uint128 payload = 0x000000000000000000000000000000000000000000b401f4ff0401000300;
        uint128 amountIn = 1000 * 1e18; // 1000 ARB

        vm.expectRevert(TradeExecutor.Loss.selector);
        vm.prank(gateway);
        executor.flashSwap(amountIn, payload);
    }

    function testAdmin_Withdraw() public {
        // Test withdrawing ERC20 token
        uint256 dealAmount = 1 ether;
        deal(WETH, address(executor), dealAmount);

        uint256 payeeBalanceBefore = IERC20(WETH).balanceOf(payee);
        uint256 executorBalanceBefore = IERC20(WETH).balanceOf(address(executor));
        assertEq(executorBalanceBefore, dealAmount);

        vm.prank(payee);
        executor.withdrawToken(WETH);

        uint256 payeeBalanceAfter = IERC20(WETH).balanceOf(payee);
        uint256 executorBalanceAfter = IERC20(WETH).balanceOf(address(executor));

        assertEq(executorBalanceAfter, 0);
        assertEq(payeeBalanceAfter, payeeBalanceBefore + dealAmount);

        // Test withdrawing native ETH
        uint256 nativeAmount = 2 ether;
        deal(address(executor), nativeAmount);

        uint256 payeeNativeBalanceBefore = payee.balance;
        
        vm.prank(payee);
        executor.withdrawNative();
        
        assertEq(address(executor).balance, 0);
        assertEq(payee.balance, payeeNativeBalanceBefore + nativeAmount);
    }

    function testAdmin_AccessControl() public {
        address randomAddress = address(0x99);
        vm.prank(randomAddress);
        vm.expectRevert();
        executor.setPayee(address(0x157));

        vm.prank(randomAddress);
        vm.expectRevert();
        executor.setGateway(address(0x157));

        vm.prank(randomAddress);
        vm.expectRevert();
        executor.withdrawNative();

        vm.prank(randomAddress);
        vm.expectRevert();
        executor.withdrawToken(WETH);
    }

    function testAdmin_SetAddresses() public {
        assertEq(executor.payee(), payee);
        assertEq(executor.gateway(), gateway);

        vm.prank(payee);
        executor.setPayee(address(0x157));
        assertEq(executor.payee(), address(0x157));

        vm.prank(address(0x157));
        executor.setGateway(address(0x555));
        assertEq(executor.gateway(), address(0x555));
    }
}
