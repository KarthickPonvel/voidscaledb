#!/usr/bin/env bash
#
# Usage: ./benchmarks/run.sh [repeats]

REPEATS="${1:-2}"

# Helper to run the test and summary
run_and_summarize() {
    local host=$1
    local port=$2
    local label=$3

    echo "############################################################"
    echo "# Benchmarking: $label @ $host:$port"
    echo "############################################################"

    ./benchmarks/benchmark.sh "$host" "$port" "$label" "$REPEATS"
    ./benchmarks/summarize.sh "$label"
}

# Run Benchmark for VoidScaleDB
run_and_summarize 127.0.0.1 9379 voidscaledb

echo -e "\n------------------------------------------------------------\n"

# Run Benchmark for Redis 8
run_and_summarize 127.0.0.1 6379 redis8