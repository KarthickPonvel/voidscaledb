# voidscaledb

> ⚠️ **Work in progress** — this isn't ready for production. Things like the API, architecture, and internals are still changing.

**voidscaledb** is a personal project by an undergraduate student who's using it to learn how "shared-nothing" databases work under the hood, built in Rust.

It's a **single-node database** — all shards run as worker threads inside one process on one machine. "Shared-nothing" here describes how the shards inside that single node avoid sharing memory with each other, not a distributed, multi-machine cluster. There's no cluster management, node discovery, or cross-machine replication (yet — see "What's not built yet").

It speaks the RESP2 protocol (so you can talk to it with `redis-cli`), and it's built around a "shard-per-core" design — each CPU core runs its own independent worker that owns a slice of the data and handles requests on its own. Workers don't share memory; when they need to talk to each other, they pass messages asynchronously.

---

## Why build this?

voidscaledb started as a way to actually understand, hands-on, how a modern shared-nothing database gets built in Rust.

The core idea is simple: give each CPU core its own shard — its own chunk of data and its own execution engine — instead of having everything fight over shared, mutable state. When shards do need to coordinate, they send messages to each other instead.

Working on this touches a lot of interesting ground:

* Sharding and routing requests to the right place
* Building a network protocol from scratch
* Designing a runtime and event loop
* Concurrent programming in general
* Memory management and allocation
* The trade-offs that come with distributed-style architectures

Longer term, the plan is to grow this past a basic key-value store — while keeping the shard-based design — into something that supports more data structures, persistence, JSON documents, vector search, and full-text search.

Right now, though, the focus is on the fundamentals: the runtime, the networking layer, how commands get executed, and the storage engine itself.

---

## Design principles

* **Shared-nothing** — each shard owns its own data, full stop.
* **Shard-per-core** — one worker thread per physical CPU core.
* **Single-node** — all shards live within one process on one machine; there's no multi-node clustering yet.
* **Message-passing** — shards talk to each other over async channels, not shared memory.
* **Local execution** — a command runs on the shard that owns the data whenever possible, to avoid unnecessary hops.
* **Everything in Rust** — the protocol, execution engine, sharding logic, and runtime are all part of this codebase.

---

## What's implemented so far

* RESP2 protocol support
* Shard-per-core architecture, with each worker thread pinned to a CPU core
* A dedicated Tokio runtime per shard (`LocalSet` + `current_thread`)
* A per-worker `Coordinator` that handles the command execution path
* Cross-shard messaging via `crossfire` channels, with reply routing
* Command routing based on an `ExecutionPolicy` (`Local`, `SingleKey`, `MultiKey`)
* A fast, allocation-free command lookup table using `phf`
* Fast hashing (`ahash`) and a low-latency allocator (`mimalloc`)
* Commands: `PING`, `GET`, `SET` (with `NX`, `XX`, `GET`, `EX`, `PX`, `EXAT`, `PXAT`, `KEEPTTL`), `DEL`, and TTL/expiry

## What's not built yet

* Persistence
* Replication
* Transactions
* Cluster management
* More advanced data structures
* JSON support
* Vector search
* Full-text search

---

## Benchmarks

voidscaledb has been benchmarked against Redis 8 across 8 different workloads, both with sharding on and off, on two different machines. Short version: it's roughly on par with Redis 8 on most single-request workloads, pulls ahead under pipelining and high concurrency, and falls behind at very low concurrency where there's no parallelism for sharding to exploit. Redis also tends to hold onto a tail-latency (P99) edge in most scenarios.

### macOS (Apple M4)

Test environment: Apple M4, macOS, in-memory only, 50 concurrent clients, 1,000,000 requests. Redis 8 run with `redis-server --save "" --appendonly no` to keep the comparison fair.

**Non-pipelined workload** (`redis-benchmark -t get,set -n 1000000 -c 50`)

| System                     | Operation |  Throughput | Avg Latency |      P50 |      P95 |      P99 |
| -------------------------- | --------- | ----------: | ----------: | -------: | -------: | -------: |
| Redis 8                    | SET       | ~251K req/s |    0.105 ms | 0.103 ms | 0.119 ms | 0.135 ms |
| Redis 8                    | GET       | ~250K req/s |    0.105 ms | 0.103 ms | 0.119 ms | 0.159 ms |
| voidscaledb (single shard) | SET       | ~256K req/s |    0.102 ms | 0.103 ms | 0.111 ms | 0.132 ms |
| voidscaledb (single shard) | GET       | ~256K req/s |    0.101 ms | 0.103 ms | 0.111 ms | 0.132 ms |
| voidscaledb (multi shard)  | SET       | ~252K req/s |    0.114 ms | 0.103 ms | 0.119 ms | 0.316 ms |
| voidscaledb (multi shard)  | GET       | ~250K req/s |    0.115 ms | 0.103 ms | 0.119 ms | 0.236 ms |

**Pipelined workload** (`redis-benchmark -t get,set -n 1000000 -c 50 -P 16`)

| System                     | Operation |   Throughput | Avg Latency |      P50 |      P95 |      P99 |
| -------------------------- | --------- | -----------: | ----------: | -------: | -------: | -------: |
| Redis 8                    | SET       | ~1.95M req/s |    0.354 ms | 0.347 ms | 0.437 ms | 0.641 ms |
| Redis 8                    | GET       | ~2.60M req/s |    0.254 ms | 0.253 ms | 0.345 ms | 0.379 ms |
| voidscaledb (single shard) | SET       | ~3.00M req/s |    0.200 ms | 0.191 ms | 0.279 ms | 0.581 ms |
| voidscaledb (single shard) | GET       | ~3.17M req/s |    0.135 ms | 0.135 ms | 0.147 ms | 0.179 ms |
| voidscaledb (multi shard)  | SET       | ~1.87M req/s |    0.415 ms | 0.319 ms | 1.663 ms | 1.733 ms |
| voidscaledb (multi shard)  | GET       | ~1.87M req/s |    0.415 ms | 0.319 ms | 1.661 ms | 1.731 ms |

> On the M4, single-shard voidscaledb is roughly at parity with Redis 8 for both workloads (and pulls ahead under pipelining), while multi-shard is a bit behind on this particular machine — the opposite of what shows up on Linux below, which is a good reminder that these numbers depend a lot on the hardware.

### Linux

Full numbers, test setup, and methodology for the Linux runs (HP Victus laptop, Intel i5-12450H) are in **[BENCHMARKS.md](./BENCHMARKS.md)**. On that machine, multi-shard voidscaledb pulls ahead of Redis 8 on pipelined and high-concurrency workloads (up to +30%), runs roughly neck-and-neck on most single-request workloads, and falls behind at low concurrency, where there's no parallelism for sharding to take advantage of.

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
   │ Coordinator │               │ Coordinator  │               │ Coordinator  │
   │ ShardEngine │               │ ShardEngine  │               │ ShardEngine  │
   └──────┬──────┘               └──────┬───────┘               └──────┬───────┘
          │                             │                              │
          └──────────────┬──────────────┴──────────────┬───────────────┘
                         │                             │
                 ┌───────▼─────────────────────────────▼─────────┐
                 │      Async Message-Passing Layer (crossfire)  │
                 │     cross-shard routing + worker channels     │
                 └───────────────────────────────────────────────┘
```

Here's what each worker does:

1. Binds to the same TCP address as every other worker, using `SO_REUSEPORT`
2. Runs its own single-threaded Tokio runtime, pinned to one physical CPU core
3. Owns a `Coordinator` (shared via `Rc`) that reads, decodes, runs, and encodes each client request
4. Runs commands locally on its own shard whenever the `ExecutionPolicy` allows it
5. For anything that touches another shard (`SingleKey` / `MultiKey` policies), routes the request over `crossfire` channels and waits for a reply

---

## Getting started

### You'll need

* Rust 1.85+
* Edition 2024

### Build and run

```bash
git clone https://github.com/karthickponvel/voidscaledb
cd voidscaledb
cargo run --release
```

By default the server listens on `127.0.0.1:9379`.

### Try it out

```bash
redis-cli -p 9379 ping
# PONG

redis-cli -p 9379 set hello world
# OK

redis-cli -p 9379 get hello
# "world"

redis-cli -p 9379 set hello everyone NX
# (nil)

redis-cli -p 9379 set session:1 abc123 EX 60
# OK

redis-cli -p 9379 ttl session:1
# (integer) 60

redis-cli -p 9379 del hello
# (integer) 1
```

---

## License

Apache-2.0. See `LICENSE` for details.