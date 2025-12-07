# Fulcrum Smart Contracts

This directory contains the Solidity smart contracts for the Fulcrum project, managed with the [Foundry](https://getfoundry.sh/) framework. The core of the project is the `TradeExecutor` contract, designed to execute atomic arbitrage trades across various decentralized exchanges.

## Requirements

- [Foundry](https://getfoundry.sh/)

## Setup

Before you can build, test, or deploy the contracts, you need to set up your environment variables.

1.  Create a `.env` file by copying the example file:
    ```bash
    cp .env.example .env
    ```

2.  Edit the `.env` file with your specific details:
    -   `PRIVATE_KEY`: The private key of the wallet you want to deploy from. This account will pay for the deployment gas fees.
    -   `PAYEE_ADDRESS`: The address that will receive the profits generated from successful arbitrage trades.

## Build

To install dependencies and compile the contracts, run:

```bash
forge build
```

## Test

To run the test suite for the contracts, execute:

```bash
forge test
```

## Deployment

The `TradeExecutor` contract can be deployed using the provided script. Run the following command, replacing `<your_rpc_url>` with the RPC endpoint of the target network (e.g., Arbitrum One):

```bash
forge script script/Deploy.s.sol:DeployTradeExecutor --rpc-url <your_rpc_url> --broadcast
```

This command will execute the `DeployTradeExecutor` script, which deploys the `TradeExecutor` contract and logs its address to the console.

### Key Contracts

-   `src/TradeExecutor.sol`: The main contract responsible for executing trades. It can perform multi-step swaps and flash swaps across integrated DEXs like Uniswap V3, Camelot, Sushi, and Chronos. It is controlled by a `gateway` address (which initiates trades) and sends profits to a `payee` address.
-   `src/V3PoolViewer.sol`: A helper contract designed for efficiently querying price and liquidity data from multiple Uniswap V2 and V3 style pools in a single call.
-   `script/Deploy.s.sol`: The deployment script for the `TradeExecutor` contract. It reads configuration from the `.env` file to set up the contract owner and profit recipient.
