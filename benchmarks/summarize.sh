#!/usr/bin/env bash
#
# Generates a summary table from benchmark results.
#
# Usage:
#   ./summarize.sh <label> [timestamp]

set -euo pipefail

LABEL="${1:?Usage: $0 <label> [timestamp]}"
STAMP="${2:-}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [[ -n "$STAMP" ]]; then
  RESULTS_DIR="$SCRIPT_DIR/results/$LABEL/$STAMP"
else
  RESULTS_DIR="$SCRIPT_DIR/results/$LABEL/latest"
fi

if [[ ! -d "$RESULTS_DIR" ]]; then
  echo "No results at $RESULTS_DIR" >&2
  exit 1
fi

echo "Reading results from: $RESULTS_DIR"
echo ""
python3 "$SCRIPT_DIR/summarize.py" "$RESULTS_DIR"