# Fulcrum Project Overview

The 'fulcrum' project is a high-performance, low-latency trading engine designed for decentralized exchanges (DEXs), primarily targeting the Arbitrum blockchain. Its core mission is to identify and capitalize on profitable arbitrage opportunities within the decentralized finance (DeFi) ecosystem. Fulcrum is engineered to provide advanced features such as MEV (Maximal Extractable Value) protection and real-time processing of market data.

## Purpose

The main purpose of Fulcrum is to enable automated, efficient, and rapid execution of arbitrage strategies across various DEXs. By leveraging real-time data feeds and optimized blockchain interactions, it aims to extract value from price discrepancies across different liquidity pools.

## Key Features

*   **High-Performance & Low-Latency:** Optimized for speed to react swiftly to market changes and execute trades before opportunities vanish.
*   **Arbitrage Opportunity Identification:** Continuously monitors and analyzes DEX liquidity pools to find profitable price differences.
*   **MEV Protection:** Designed with mechanisms to protect against front-running and other forms of MEV, ensuring trades are executed fairly and profitably.
*   **Real-time Market Data Processing:** Integrates with sequencer feeds and blockchain RPCs to get the most up-to-date market information.
*   **Modular Architecture:** Composed of specialized crates for clear separation of concerns and optimized performance.

## Project Structure

The project is structured as a Rust workspace, comprising three main crates:

1.  **`fulcrum-engine`**: The core trading logic, responsible for orchestrating the entire arbitrage process.
2.  **`fulcrum-sequencer-feed`**: Provides a low-latency stream of raw transaction data from the Arbitrum One sequencer.
3.  **`fulcrum-ws-cli`**: A stripped-down, optimized Ethereum JSON-RPC client used for high-performance blockchain interactions.

Additionally, the `contracts` directory contains Solidity smart contracts (e.g., `TradeExecutor.sol`, `V3PoolViewer.sol`) that the `fulcrum-engine` interacts with for trade execution and data fetching.