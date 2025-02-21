# Rust Engineer(AA) Homework (Ethereum Gas Estimator)

A high-performance Ethereum gas estimation service built with Rust and Axum.

## Features

- **Accurate Gas Estimation**: Provides detailed gas estimates for Ethereum transactions
- **High Performance**: Built with Rust and Axum for maximum efficiency
- **Caching**: Implements intelligent caching to reduce RPC calls
- **Modern Architecture**: Uses async/await patterns with Tokio
- **Multiple Transaction Types**: Supports both legacy and EIP-1559 transactions
- **Robust Error Handling**: Provides detailed error messages for troubleshooting
- **Configurable**: Easily configure via environment variables
- **Health Monitoring**: Built-in health check endpoint

## Installation

### Prerequisites

- Rust 1.85+
- Access to an Ethereum JSON-RPC endpoint

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/eth-gas-estimator.git
   cd eth-gas-estimator
   ```

2. Configure environment variables :
   follow the .env.template

3. Build and run:
   ```bash
   cargo build --release
   ./target/release/eth-gas-estimator
   ```

## Configuration

The service can be configured using the following environment variables:

| Variable              | Description                      | Default                 |
| --------------------- | -------------------------------- | ----------------------- |
| `ETHEREUM_RPC_URLS`   | Comma-separated list of RPC URLs | `http://localhost:8545` |
| `CACHE_DURATION_SECS` | Cache TTL in seconds             | `15`                    |
| `HOST`                | Server host address              | `0.0.0.0`               |
| `PORT`                | Server port                      | `8080`                  |
| `LOG_LEVEL`           | Logging level                    | `info`                  |

## API Usage

### Estimate Gas

**Endpoint**: `POST /api/v1/estimate-gas`

**Request Body**:

```json
{
  "from": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
  "to": "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
  "value": "100000000000000000",
  "data": "0x38ed1739000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000003e387000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000742d35cc6634c0532925a3b844bc454e4438f44e00000000000000000000000000000000000000000000000000000000663c81da0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7"
}
```

**Response**:

```json
{
  "gas_limit": "255000",
  "gas_price": "20000000000",
  "estimated_cost_wei": "5100000000000000",
  "estimated_cost_eth": "0.0051",
  "estimated_execution_time": "~15 seconds",
  "type_of_transaction": "legacy"
}
```

### Health Check

**Endpoint**: `GET /health`

**Response**:

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## Architecture

The service follows a clean architecture pattern:

- **Handlers**: Process HTTP requests and responses
- **Services**: Contain business logic and external integrations
- **Models**: Define data structures
- **Utils**: Provide shared utilities
- **Config**: Handle application configuration

## Performance Optimizations

- **Connection Pooling**: Reuses connections to Ethereum nodes
- **Caching**: Implements TTL-based caching for gas prices and other network data
- **Parallel Execution**: Uses `tokio::join!` for concurrent operations
- **Provider Fallback**: Automatically switches providers if one fails
- **Efficient Error Handling**: Provides meaningful errors without unnecessary overhead
- **Type-Driven Design**: Leverages Rust's type system for safety and performance

## Development

### Running Tests

```bash
cargo test
```

### Benchmarking

```bash
cargo bench
```

## License

MIT License
