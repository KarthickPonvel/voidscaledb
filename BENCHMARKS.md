# Benchmarks

> These numbers are here to track progress and spot bottlenecks as the project develops — they're not meant to be a definitive, general-purpose comparison between voidscaledb and Redis.

This covers the `memtier_benchmark` test suite used to compare **voidscaledb** against **Redis 8**, and to see how voidscaledb performs with sharding on (multi-shard) versus off (single-shard), across a bunch of different workloads.

---

## Test setup

* HP Victus by HP Gaming Laptop 15-fa0xxx
* Intel Core i5-12450H (12th Gen), 12 logical CPUs (8 cores, hybrid P/E, hyperthreading on P-cores)
* Linux
* Everything in memory, no disk involved
* `memtier_benchmark` used as the load generator
* Redis 8 run with persistence turned off, to keep the comparison fair:

  ```bash
  redis-server --save "" --appendonly no
  ```

> Heads up: this is a regular laptop, not a dedicated benchmark rig, so the load generator and the server being tested are sharing the same 12 threads. To stop the server from getting starved of CPU, Scenario 07 (high concurrency) intentionally limits itself to half the machine's cores (`$CORES / 2` = 6). Treat the numbers below as "what a laptop can do," not as a stand-in for what you'd see on dedicated hardware.

---

## How the tests work

Both servers are run through the same 8 scenarios, which vary things like client count, thread count, number of requests, pipeline depth, read/write mix, and value size. The whole thing is driven by `benchmarks/run.sh`, which calls `benchmarks/benchmark.sh` to actually run the scenarios and `benchmarks/summarize.sh` to turn the JSON output into readable tables.

There's just one test profile now (the old `full`/`quick` split is gone). The only thing you can configure is how many times each scenario repeats:

```bash
# Run the suite against both voidscaledb and Redis 8, 2 repeats each (default)
./benchmarks/run.sh

# Or set the repeat count yourself
./benchmarks/run.sh 3
```

### The underlying `memtier_benchmark` command

Each scenario runs `memtier_benchmark` with its own settings for clients, threads, requests, pipeline depth, ratio, and data size:

```bash
memtier_benchmark \
  -s "$HOST" -p "$PORT" \
  --threads="$THREADS" \
  --clients="$CLIENTS" \
  --requests="$REQUESTS" \
  --pipeline="$PIPELINE" \
  --ratio="$RATIO" \
  --data-size="$DATA_SIZE" \
  --key-pattern=R:R \
  --print-percentiles=50,95,99,99.9 \
  --json-out-file="$JSON_OUT"
```

### The 8 scenarios

| Scenario                  | Clients | Threads |  Requests/client | Pipeline | Ratio (set:get) | Data size |
| ------------------------- | ------: | ------: | ----------------: | -------: | ---------------: | --------: |
| 01 – No pipelining        |      50 |       4 |            150,000 |        1 |               1:1 |     100 B |
| 02 – Pipeline depth 16    |      50 |       4 |            150,000 |       16 |               1:1 |     100 B |
| 03 – Read-heavy           |      50 |       4 |            150,000 |        1 |              1:10 |     100 B |
| 04 – Write-heavy          |      50 |       4 |            150,000 |        1 |              10:1 |     100 B |
| 05 – Small values         |      50 |       4 |            150,000 |        1 |               1:1 |      64 B |
| 06 – Large values         |      50 |       4 |             30,000 |        1 |               1:1 |   10.0 KB |
| 07 – High concurrency     |     100 |  half the cores |         150,000 |        1 |               1:1 |     100 B |
| 08 – Low concurrency      |       1 |       1 |             30,000 |        1 |               1:1 |     100 B |

Every scenario runs multiple times (the `repeats` argument to `run.sh`/`benchmark.sh`), and `summarize.sh` aggregates the results.

---

## Latest results (2026-07-08)

The numbers below come from a **multi-shard (sharding enabled)** voidscaledb build, benchmarked against Redis 8 with **3 repeats per scenario**, averaged. Redis 8 was benchmarked in the same session, so machine conditions line up between the two.

### Multi-shard vs. Redis 8 — throughput (ops/sec)

| Scenario             | voidscaledb | Redis 8 | Δ vs Redis |
| --------------------- | -----------: | -------: | ----------: |
| No Pipeline           |      247,886 |  266,686 |       -7.1% |
| Pipeline x16          |    1,699,998 | 1,339,597 |      +26.9% |
| Read Heavy            |      226,820 |  263,580 |      -13.9% |
| Write Heavy           |      238,228 |  246,261 |       -3.3% |
| Small Values (64B)    |      251,242 |  256,903 |       -2.2% |
| Large Values (10KB)   |      211,597 |  207,413 |       +2.0% |
| High Concurrency      |      279,894 |  215,042 |      +30.2% |
| Low Concurrency       |       21,097 |   81,952 |      -74.3% |

### Multi-shard vs. Redis 8 — latency (ms)

| Scenario         | voidscaledb Avg | Redis 8 Avg | voidscaledb P99 | Redis 8 P99 |
| ---------------- | -------------: | ----------: | -------------: | -----------: |
| No Pipeline      |          0.808 |       0.760 |          3.770 |        1.548 |
| Pipeline x16     |          1.879 |       2.389 |          4.650 |        4.532 |
| Read Heavy       |          0.884 |       0.758 |          4.036 |        1.538 |
| Write Heavy      |          0.838 |       0.814 |          3.818 |        1.727 |
| Small Values     |          0.796 |       0.779 |          3.775 |        1.570 |
| Large Values     |          0.945 |       0.963 |          4.234 |        1.884 |
| High Concurrency |          2.433 |       2.820 |          6.143 |        5.450 |
| Low Concurrency  |          0.048 |       0.012 |          0.775 |        0.042 |

### What this actually tells us

* voidscaledb (multi-shard) pulls ahead of Redis 8 on **Pipeline x16 (+26.9%)**, **Large Values (+2.0%)**, and **High Concurrency (+30.2%)** — exactly the conditions where multiple cores independently handling requests should help most.
* On **No Pipeline (-7.1%)**, **Read Heavy (-13.9%)**, **Write Heavy (-3.3%)**, and **Small Values (-2.2%)**, the two are close, with a slight edge to Redis — call it roughly on par.
* The clearest loss is **Low Concurrency (-74.3%)**: with a single client there's no parallel work for sharding to exploit, so the extra cross-shard coordination overhead just adds cost with no benefit.
* Avg latency roughly tracks throughput, but Redis 8 holds a consistent P99 tail-latency advantage in most scenarios except pipelining and high concurrency, where voidscaledb's P99 is comparable or better.

---

## Running this yourself

```bash
# 1. Start voidscaledb (default port 9379)
cargo run --release

# 2. Start Redis 8 without persistence, on a separate port
redis-server --save "" --appendonly no

# 3. Run the suite against both and print summaries
#    (run.sh handles calling benchmark.sh + summarize.sh for each one)
./benchmarks/run.sh 3
```

The raw JSON output from each run (straight from `memtier_benchmark --json-out-file`) is saved under `benchmarks/results/<label>/<timestamp>/`, so you can dig into it later. To reprint a summary without rerunning everything, use `./benchmarks/summarize.sh <label>`.