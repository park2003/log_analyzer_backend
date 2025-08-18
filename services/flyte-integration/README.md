# Flyte Integration Service

A gRPC service that acts as an Anti-Corruption Layer between the Savassan system and Flyte workflow orchestration platform.

## Architecture

The service follows Domain-Driven Design (DDD) with a layered architecture:

- **Domain Layer**: Core business logic and models (`WorkflowExecution`, `ExecutionStatus`)
- **Application Layer**: Use cases and orchestration logic
- **Infrastructure Layer**: External integrations (Flyte API client, PostgreSQL repository)
- **Presentation Layer**: gRPC service implementation

## Configuration

Copy `.env.example` to `.env` and adjust the values for your local Flyte instance:

```bash
cp .env.example .env
```

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string
- `GRPC_ADDR`: Address for the gRPC server (default: `[::1]:50051`)
- `FLYTE_ADMIN_URL`: Flyte Admin API URL (default: `http://localhost:30080` for local)
- `FLYTE_PROJECT`: Flyte project name (default: `flytesnacks`)
- `FLYTE_DOMAIN`: Flyte domain (default: `development`)
- `FLYTE_INSECURE`: Use insecure connection for local development (default: `true`)

## Prerequisites

1. Local Flyte instance running (typically on port 30080)
2. PostgreSQL database
3. Rust toolchain

## Running the Service

1. Ensure PostgreSQL is running and create the database:
```bash
createdb flyte_integration
```

2. Build and run the service:
```bash
cargo build --release
cargo run
```

The service will:
- Run database migrations automatically
- Start gRPC server on port 50051
- Monitor active workflow executions every 30 seconds

## API

The service exposes two gRPC methods:

### ExecuteWorkflow
Submit a workflow definition to Flyte for execution.

### GetExecutionStatus
Query the status of a workflow execution.

## Testing

To test the service with a gRPC client:

```bash
# Install grpcurl if not already installed
brew install grpcurl

# List available services
grpcurl -plaintext -proto protos/savassan/v1/execution.proto localhost:50051 list

# Execute a workflow (example)
grpcurl -plaintext -proto protos/savassan/v1/execution.proto \
  -d '{"workflow_id": "test-workflow", "flyte_workflow_definition_json": "{}"}' \
  localhost:50051 savassan.v1.ExecutionService/ExecuteWorkflow

# Get execution status
grpcurl -plaintext -proto protos/savassan/v1/execution.proto \
  -d '{"execution_id": "UUID-HERE"}' \
  localhost:50051 savassan.v1.ExecutionService/GetExecutionStatus
```

## Development

The service uses:
- **Axum**: Web framework (though primarily using Tonic for gRPC)
- **Tonic**: gRPC implementation
- **SQLx**: Database access with compile-time query verification
- **Tokio**: Async runtime
- **Tracing**: Structured logging