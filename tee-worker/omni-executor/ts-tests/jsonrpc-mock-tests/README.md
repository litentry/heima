# JSON-RPC Mock Tests

This project contains TypeScript-based integration tests for the Omni Executor JSON-RPC API endpoints, including:

-   **JSON-RPC API Tests** (`jsonrpc_mock_test`) - Basic API endpoint testing
-   **SubmitUserOp Integration Tests** (`submitUserOp.test.ts`) - Complete Account Abstraction flow testing

## Quick Start

### Local Development Setup

1. **Start the complete development environment:**

    ```bash
    cd ../../docker
    docker compose up
    ```

    This will automatically:

    - Deploy all Account Abstraction contracts
    - Start Anvil (Ethereum node) on port 8545
    - Start Omni Executor with mock server on port 2100
    - Start Heima node on port 9944
    - Set up all required environment variables

2. **Run all tests:**

    ```bash
    cd ../ts-tests/jsonrpc-mock-tests
    pnpm install
    pnpm test
    ```

3. **Run specific tests:**

    ```bash
    # JSON-RPC API tests only
    pnpm test jsonrpc_mock_test

    # SubmitUserOp integration tests only
    pnpm test submitUserOp.test.ts
    ```

## Available Services

When running `docker compose up`, the following services are available:

-   **Omni-Executor RPC**: http://localhost:2100
-   **Mock Server**: http://localhost:3456
-   **Ethereum Node (Anvil)**: http://localhost:8545

## Contract Addresses

All contracts are automatically deployed and configured. To view deployed addresses:

```bash
docker compose logs aa-contracts-deploy
```
