# JSON-RPC Mock Tests

This project contains TypeScript-based mock tests for the Omni Executor JSON-RPC API endpoints.

## Quick Start

### Local Development Setup

1. **Start the worker services:**
   ```bash
   make build-docker-test
   make start-test
   ```

2. **Run the JSON-RPC tests:**
   ```bash
   pnpm --filter jsonrpc-mock-tests run test jsonrpc.test.ts
   ```