#!/usr/bin/env bash
#
# Usage:
#   ./run-all-tests.sh <host> <port> <label> [repeats]
#
# Examples:
#   ./run-all-tests.sh 127.0.0.1 9379 voidscaledb
#   ./run-all-tests.sh 127.0.0.1 6379 redis8 3

set -euo pipefail

HOST="${1:?Usage: $0 <host> <port> <label> [repeats]}"
PORT="${2:?Usage: $0 <host> <port> <label> [repeats]}"
LABEL="${3:?Usage: $0 <host> <port> <label> [repeats]}"
REPEATS="${4:-2}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

STAMP="$(date +%Y-%m-%d_%H%M%S)"
OUT_DIR="$SCRIPT_DIR/results/$LABEL/$STAMP"
mkdir -p "$OUT_DIR"

command -v memtier_benchmark >/dev/null 2>&1 || {
  echo "Error: memtier_benchmark not found on PATH." >&2
  exit 1
}

CORES="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"
# Leave headroom for the server under test — don't let the benchmark
# client claim every core on the machine.
HALF_CORES=$((CORES / 2))
if [[ "$HALF_CORES" -lt 1 ]]; then
  HALF_CORES=1
fi

# format: name | clients | threads | requests-per-client | pipeline | ratio | data-size
TESTS=(
  "01-nopipe-baseline|50|4|150000|1|1:1|100"
  "02-pipeline16|50|4|150000|16|1:1|100"
  "03-read-heavy|50|4|150000|1|1:10|100"
  "04-write-heavy|50|4|150000|1|10:1|100"
  "05-small-values-64b|50|4|150000|1|1:1|64"
  "06-large-values-10kb|50|4|30000|1|1:1|10240"
  "07-high-concurrency|100|$HALF_CORES|150000|1|1:1|100"
  "08-low-concurrency|1|1|30000|1|1:1|100"
)

echo "############################################################"
echo "# Benchmarking: $LABEL @ $HOST:$PORT"
echo "# Repeats: $REPEATS"
echo "# Cores: $CORES  (using $HALF_CORES for high-concurrency scenario)"
echo "# Output: $OUT_DIR"
echo "############################################################"
echo ""

for test in "${TESTS[@]}"; do
  IFS='|' read -r NAME CLIENTS THREADS REQUESTS PIPELINE RATIO DATA_SIZE <<< "$test"

  echo "=== Scenario: $NAME ==="
  echo "    clients=$CLIENTS threads=$THREADS requests/client=$REQUESTS pipeline=$PIPELINE ratio=$RATIO data-size=$DATA_SIZE"

  for run in $(seq 1 "$REPEATS"); do
    JSON_OUT="$OUT_DIR/${NAME}-run${run}.json"
    LOG_OUT="$OUT_DIR/${NAME}-run${run}.log"

    echo "  -> run $run/$REPEATS"
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
      --json-out-file="$JSON_OUT" \
      > "$LOG_OUT" 2>&1

    sleep 4
  done
  echo ""
done

ln -sfn "$OUT_DIR" "$SCRIPT_DIR/results/$LABEL/latest"

echo "############################################################"
echo "# Done. Results: $OUT_DIR"
echo "# Next: ./benchmarks/summarize.sh $LABEL"
echo "############################################################"