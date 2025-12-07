# Fulcrum Project Architecture

The Fulcrum project is a sophisticated high-performance, low-latency trading engine built in Rust, designed to interact with decentralized exchanges (DEXs) on the Arbitrum blockchain. Its architecture is modular, separating concerns into distinct crates for optimized performance and maintainability.

## High-Level Component Interaction

The `fulcrum` application acts as a command-line dispatcher, allowing users to run one of its three primary components: `engine`, `sequencer-feed`, or `ws-cli`.

1.  **`fulcrum-sequencer-feed`**:
    *   **Role:** Data Ingress.
    *   **Functionality:** Connects via WebSocket to the Arbitrum One sequencer to provide a raw, real-time stream of transaction data. It decodes base64-encoded JSON messages containing RLP-encoded Arbitrum transactions and efficiently streams them to the `fulcrum-engine` in a `TxBuffer`.

2.  **`fulcrum-ws-cli`**:
    *   **Role:** Blockchain Interaction Layer.
    *   **Functionality:** A fast, optimized Ethereum JSON-RPC client. It implements the `ethers-providers::JsonRpcClient` trait, enabling the `fulcrum-engine` to perform high-performance RPC calls (e.g., `eth_call` for reading contract state, `eth_blockNumber` for current block number) to query blockchain state and potentially send transactions. It uses an internal `RequestManager` for asynchronous communication.

3.  **`fulcrum-engine`**:
    *   **Role:** Core Trading Logic and Orchestrator.
    *   **Functionality:** This is the "brain" of the operation, coordinating data from the sequencer feed and blockchain state to identify and execute arbitrage opportunities.
        *   **Data Consumption:** It continuously consumes the real-time transaction stream from `fulcrum-sequencer-feed`.
        *   **State Management:** Utilizes `fulcrum-ws-cli` to fetch current blockchain state and maintain an internal `PriceGraph`. This `PriceGraph` represents the current state of various DEX liquidity pools across different protocols (Uniswap V2/V3, 0x).
        *   **Trade Simulation:** Employs a `TradeSimulator` to predict the impact of incoming transactions on the `PriceGraph`. This allows the engine to react to potential state changes even before they are finalized on-chain, crucial for MEV protection and competitive arbitrage.
        *   **Arbitrage Identification:** Continuously searches for profitable arbitrage opportunities within the simulated or current `PriceGraph`.
        *   **Trade Execution:** If a profitable arbitrage is identified, it leverages an `OrderService`. The `OrderService`, in turn, uses `fulcrum-ws-cli` and likely interacts with a `TradeExecutor.sol` smart contract (located in the `contracts` directory) to construct and execute the trade on-chain.

## Supporting Components (Smart Contracts)

The `contracts` directory contains Solidity smart contracts that facilitate the `fulcrum-engine`'s operations:

*   **`TradeExecutor.sol`**: (Inferred) This contract is likely used by the `OrderService` to execute identified trades efficiently and atomically across various decentralized exchanges.
*   **`V3PoolViewer.sol`**: (Inferred) This contract probably provides optimized read-only access to Uniswap V3 pool data, which is critical for the `PriceService` and `PriceGraph` to accurately reflect and update liquidity information.

## Dependencies

The project heavily relies on the `ethers-rs` ecosystem for robust blockchain interaction. The `ethabi-static` dependency in `fulcrum-engine` indicates ABI generation for seamless and type-safe interaction with smart contracts. Other key dependencies include `ws-tool` for WebSocket communication in `sequencer-feed` and `ws-cli`.

This entire system is meticulously designed for high-performance and low-latency, which are paramount for successful arbitrage and Maximal Extractable Value (MEV) strategies in the highly competitive decentralized finance landscape.