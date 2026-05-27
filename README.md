# voidscaledb

> ⚠️ **Work in progress** — not ready for production use. APIs, architecture, and implementation details may change frequently.

**voidscaledb** is a personal systems programming project focused on exploring shared-nothing database architecture in Rust.

It is a RESP2-compatible in-memory database built around a shard-per-core design, where each worker owns its data and executes requests independently. Communication between workers occurs through asynchronous message passing rather than shared mutable state.

---

## Why voidscaledb?

voidscaledb began as a learning project to explore how a modern shared-nothing database can be built in Rust.

The project is centered around a simple idea: each CPU core owns an independent shard containing its own data and execution engine. Rather than coordinating through shared mutable state, shards communicate through asynchronous message passing when coordination is required.

Building the system from scratch provides an opportunity to study and experiment with topics such as:

* Sharding and request routing
* Network protocol implementation
* Runtime and event-loop design
* Concurrent systems programming
* Memory management and allocation strategies
* Scalability trade-offs in distributed-style architectures

The long-term goal is to evolve the project beyond a simple key-value store while preserving the same shard-based execution model. Planned areas of exploration include additional data structures, persistence, JSON documents, vector indexing, and full-text search.

Today, the primary focus is building a solid foundation: the runtime, networking layer, command execution path, and key-value storage engine.

---

## Architecture Philosophy

The project follows several core design principles:

* **Shared-nothing** — every shard owns its data exclusively.
* **Shard-per-core** — one worker thread per physical CPU core.
* **Message-passing** — cross-shard communication occurs through asynchronous channels.
* **Local execution** — commands execute directly on the owning shard whenever possible.
* **Rust-first implementation** — protocol handling, execution, sharding, and runtime orchestration are implemented within this codebase.

---

## Features (Current)

* RESP2 wire protocol
* Shard-per-core architecture
* CPU affinity pinning per worker thread
* Per-shard Tokio runtime (`LocalSet` + `current_thread`)
* Fast hashing via `ahash`
* Low-latency allocation via `mimalloc`
* Commands:

  * `PING`
  * `GET`
  * `SET`

---

## Current Status

The project is currently focused on validating the core architecture and execution model.

Implemented:

* Basic networking layer
* RESP2 protocol parsing
* Shard-local key-value storage
* Cross-shard request routing
* Multi-worker execution model
* Benchmarking infrastructure

Not yet implemented:

* Persistence
* Replication
* Transactions
* Cluster management
* Advanced data structures
* JSON support
* Vector search
* Full-text search

---

## Benchmarks

> Benchmark results are intended to track project progress and identify bottlenecks. They should not be interpreted as definitive performance comparisons.

### Test Environment

* Apple M4
* macOS
* In-memory only
* 50 concurrent clients
* 1,000,000 requests

### Redis Configuration

```bash
redis-server --save "" --appendonly no
```

Persistence was disabled to ensure a fair comparison against the current in-memory implementation of voidscaledb.

### Non-Pipelined Workload

```bash
redis-benchmark -t get,set -n 1000000 -c 50
```

| System                     | Operation |  Throughput | Avg Latency |      P50 |      P95 |      P99 |
| -------------------------- | --------- | ----------: | ----------: | -------: | -------: | -------: |
| Redis 8                    | SET       | ~251K req/s |    0.105 ms | 0.103 ms | 0.119 ms | 0.135 ms |
| Redis 8                    | GET       | ~250K req/s |    0.105 ms | 0.103 ms | 0.119 ms | 0.159 ms |
| voidscaledb (single shard) | SET       | ~256K req/s |    0.102 ms | 0.103 ms | 0.111 ms | 0.132 ms |
| voidscaledb (single shard) | GET       | ~256K req/s |    0.101 ms | 0.103 ms | 0.111 ms | 0.132 ms |
| voidscaledb (multi shard)  | SET       | ~252K req/s |    0.114 ms | 0.103 ms | 0.119 ms | 0.316 ms |
| voidscaledb (multi shard)  | GET       | ~250K req/s |    0.115 ms | 0.103 ms | 0.119 ms | 0.236 ms |

### Pipelined Workload (`-P 16`)

```bash
redis-benchmark -t get,set -n 1000000 -c 50 -P 16
```

| System                     | Operation |   Throughput | Avg Latency |      P50 |      P95 |      P99 |
| -------------------------- | --------- | -----------: | ----------: | -------: | -------: | -------: |
| Redis 8                    | SET       | ~1.95M req/s |    0.354 ms | 0.347 ms | 0.437 ms | 0.641 ms |
| Redis 8                    | GET       | ~2.60M req/s |    0.254 ms | 0.253 ms | 0.345 ms | 0.379 ms |
| voidscaledb (single shard) | SET       | ~3.00M req/s |    0.200 ms | 0.191 ms | 0.279 ms | 0.581 ms |
| voidscaledb (single shard) | GET       | ~3.17M req/s |    0.135 ms | 0.135 ms | 0.147 ms | 0.179 ms |
| voidscaledb (multi shard)  | SET       | ~1.87M req/s |    0.415 ms | 0.319 ms | 1.663 ms | 1.733 ms |
| voidscaledb (multi shard)  | GET       | ~1.87M req/s |    0.415 ms | 0.319 ms | 1.661 ms | 1.731 ms |

---

## Roadmap

### Runtime & Scalability

* Optimize cross-shard communication
* Reduce message-passing overhead
* Improve multi-shard throughput scaling
* Add profiling and benchmark automation

### Commands & Protocol

* DEL
* MGET
* MSET
* INCR
* APPEND
* TTL and expiry support
* Additional RESP compatibility improvements

### Data Structures

* Lists
* Sets
* Sorted sets
* Hashes
* Streams
* Probabilistic data structures

### Storage

* Write-ahead log (WAL)
* Crash recovery
* LSM-based persistence
* Multi-tier storage

### Multi-Model Exploration

* JSON documents
* Vector indexing
* Approximate nearest-neighbor search
* Full-text search

---

## Architecture

```text
                        ┌──────────────────────────────────┐
                        │             Server               │
                        │  spawns N workers (one per core) │
                        └────────────────┬─────────────────┘
                                         │
          ┌──────────────────────────────┼──────────────────────────────┐
          │                              │                              │
   ┌──────▼──────┐               ┌───────▼──────┐               ┌───────▼──────┐
   │  Worker 0   │               │  Worker 1    │               │  Worker N    │
   │  (core 0)   │               │  (core 1)    │               │  (core N)    │
   │ Listener    │               │ Listener     │               │ Listener     │
   │ ShardEngine │               │ ShardEngine  │               │ ShardEngine  │
   └──────┬──────┘               └──────┬───────┘               └──────┬───────┘
          │                             │                              │
          └──────────────┬──────────────┴──────────────┬───────────────┘
                         │                             │
                 ┌───────▼─────────────────────────────▼─────────┐
                 │         Async Message-Passing Layer           │
                 │     cross-shard routing + worker channels     │
                 └───────────────────────────────────────────────┘
```

Each worker:

1. Binds to the same TCP address using `SO_REUSEPORT`
2. Runs a single-threaded Tokio runtime pinned to a physical CPU core
3. Handles connections and executes commands against its local shard
4. Routes cross-shard requests to the owning worker via async channels

---

## Getting Started

### Requirements

* Rust 1.85+
* Edition 2024

### Build

```bash
git clone https://github.com/karthickponvel/voidscaledb
cd voidscaledb
cargo run --release
```

The server starts on `127.0.0.1:9379` by default.

### Example

```bash
redis-cli -p 9379 ping
# PONG

redis-cli -p 9379 set hello world
# OK

redis-cli -p 9379 get hello
# "world"
```

---

## License

Apache-2.0. See `LICENSE` for details.
