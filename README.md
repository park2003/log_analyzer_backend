# Log Analyzer Backend

This project is a backend service for collecting, analyzing, and storing logs from various services. It is built with Rust and utilizes the Axum web framework.

## Features

*   **Log Collection:** Collects logs from different services. The current implementation simulates fetching logs from a legacy backend.
*   **Log Ontology:** Defines a structured log format using a `LogEventType` enum, which helps in classifying and analyzing logs effectively.
*   **Log Analysis:** Analyzes logs to identify relationships and patterns. The initial analysis groups logs by a `trace_id` to represent causal transactions.
*   **API Endpoint:** Provides a `/collect-and-analyze` endpoint to trigger the log collection and analysis process.
*   **Structured Logging:** Uses the `tracing` library for structured and configurable logging.

## Getting Started

### Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install)

### Building and Running

1.  Clone the repository:
    ```sh
    git clone <repository-url>
    cd log_analyzer_backend
    ```

2.  Build the project:
    ```sh
    cargo build
    ```

3.  Run the project:
    ```sh
    cargo run
    ```

The server will start on `127.0.0.1:3000`.

## API

### `POST /collect-and-analyze`

This endpoint triggers the log collection and analysis process. It fetches logs from the legacy backend (simulated), stores them, and runs the analysis.

**Request:**
```bash
curl -X POST http://127.0.0.1:3000/collect-and-analyze
```

**Response:**

The endpoint returns a JSON array of `AnalysisResult` objects, each representing a discovered relationship between logs.

```json
[
  {
    "relationship_type": "Causal Transaction",
    "related_log_ids": [
      "...",
      "..."
    ],
    "summary": "Found a transaction with trace_id ... involving 3 steps."
  }
]
```

## Dependencies

*   [axum](https://github.com/tokio-rs/axum): Web framework
*   [tokio](https://tokio.rs/): Asynchronous runtime
*   [serde](https://serde.rs/): Serialization/deserialization framework
*   [reqwest](https://docs.rs/reqwest/latest/reqwest/): HTTP client
*   [tracing](https://docs.rs/tracing/latest/tracing/): Application-level tracing
*   [uuid](https://docs.rs/uuid/latest/uuid/): For generating unique identifiers
*   [chrono](https://docs.rs/chrono/latest/chrono/): Date and time library
