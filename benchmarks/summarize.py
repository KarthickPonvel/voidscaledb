#!/usr/bin/env python3
"""
Parses memtier_benchmark JSON files and prints a clean, detailed metrics
report per scenario (averaged over repeated runs), plus a compact
comparison table at the end.

Usage:
    python3 summarize.py <results_dir>
"""

import sys
import json
import glob
import os
from collections import defaultdict

CATEGORY_ORDER = ["Sets", "Gets", "Waits", "Totals"]

METRIC_SPECS = [
    ("Count",                 "Requests",     "",          ",.0f"),
    ("Ops/sec",               "Throughput",   "ops/sec",   ",.2f"),
    ("Hits/sec",              "Hits",         "ops/sec",   ",.2f"),
    ("Misses/sec",            "Misses",       "ops/sec",   ",.2f"),
    ("Connection Errors/sec", "Conn Errors",  "errors/sec",",.2f"),
    ("Connection Errors",     "Conn Errors",  "(total)",   ",.0f"),
    ("Latency",               "Avg Latency",  "ms",        ",.3f"),
    ("Min Latency",           "Min Latency",  "ms",        ",.3f"),
    ("Max Latency",           "Max Latency",  "ms",        ",.3f"),
    ("KB/sec",                "Bandwidth",    "KB/sec",    ",.2f"),
    ("KB/sec RX",             "  RX",         "KB/sec",    ",.2f"),
    ("KB/sec TX",             "  TX",         "KB/sec",    ",.2f"),
]

SUPPRESSED_KEYS = {"Average Latency", "Accumulated Latency", "KB/sec RX/TX"}

PERCENTILE_LABELS = ["50.00", "95.00", "99.00", "99.90"]


def find_section(obj, name):
    """Recursively find a dict value whose key matches `name` (case-insensitive)."""
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k.lower() == name.lower() and isinstance(v, dict):
                return v
        for v in obj.values():
            result = find_section(v, name)
            if result:
                return result
    return None


def find_all_stats(obj):
    """Find the top-level stats container (usually 'ALL STATS')."""
    section = find_section(obj, "ALL STATS")
    return section if section else obj


def extract_percentiles(category):
    """Pull percentile latencies out of a category dict, in whatever
    shape memtier put them (nested dict, or flat keys like 'p50.00')."""
    result = {}
    perc_dict = None
    for k, v in category.items():
        if "percentile" in k.lower() and isinstance(v, dict):
            perc_dict = v
            break
    if perc_dict is None:
        perc_dict = category  

    for k, v in perc_dict.items():
        key_clean = k.lower().replace("percentile", "").replace("p", "").strip()
        try:
            pct = float(key_clean)
        except ValueError:
            continue
        if isinstance(v, dict):
            continue
        label = f"{pct:.2f}"
        if label in PERCENTILE_LABELS or abs(pct - round(pct)) < 0.001:
            result[f"{pct:.2f}"] = v
    return result


def extract_category(cat_dict):
    """Pull every known scalar metric + percentiles out of one category
    (e.g. Sets, Gets, Totals)."""
    metrics = {}
    for key, value in cat_dict.items():
        if isinstance(value, dict):
            continue
        metrics[key] = value
    percentiles = extract_percentiles(cat_dict)
    return metrics, percentiles


def load_run(json_path):
    with open(json_path) as f:
        data = json.load(f)
    stats = find_all_stats(data)
    if not isinstance(stats, dict):
        return None

    run = {}
    for cat_name in CATEGORY_ORDER:
        cat = None
        for k, v in stats.items():
            if k.lower() == cat_name.lower() and isinstance(v, dict):
                cat = v
                break
        if cat is None:
            continue
        metrics, percentiles = extract_category(cat)
        if metrics or percentiles:
            run[cat_name] = {"metrics": metrics, "percentiles": percentiles}

    if not run:
        metrics, percentiles = extract_category(stats)
        if metrics or percentiles:
            run["Totals"] = {"metrics": metrics, "percentiles": percentiles}

    return run if run else None


def average_runs(runs):
    """Average metrics/percentiles across a list of per-run dicts,
    keyed by category."""
    categories = set()
    for r in runs:
        categories.update(r.keys())

    averaged = {}
    for cat in categories:
        cat_runs = [r[cat] for r in runs if cat in r]
        n = len(cat_runs)

        metric_keys = set()
        for cr in cat_runs:
            metric_keys.update(k for k, v in cr["metrics"].items()
                                if isinstance(v, (int, float)))
        avg_metrics = {}
        for mk in metric_keys:
            vals = [cr["metrics"][mk] for cr in cat_runs if mk in cr["metrics"]]
            if vals:
                avg_metrics[mk] = sum(vals) / len(vals)

        perc_keys = set()
        for cr in cat_runs:
            perc_keys.update(cr["percentiles"].keys())
        avg_perc = {}
        for pk in perc_keys:
            vals = [cr["percentiles"][pk] for cr in cat_runs if pk in cr["percentiles"]]
            if vals:
                avg_perc[pk] = sum(vals) / len(vals)

        averaged[cat] = {"metrics": avg_metrics, "percentiles": avg_perc, "n": n}

    return averaged


def fmt_metric_line(label, value, unit, spec):
    try:
        val_str = format(value, spec)
    except (ValueError, TypeError):
        val_str = str(value)
    return f"    {label:<16} {val_str:>14} {unit}"


def print_scenario_detail(scenario, averaged, n_runs):
    print("=" * 78)
    print(f"Scenario: {scenario}   ({n_runs} run{'s' if n_runs != 1 else ''} averaged)")
    print("=" * 78)

    for cat_name in CATEGORY_ORDER:
        if cat_name not in averaged:
            continue
        cat = averaged[cat_name]
        if not cat["metrics"] and not cat["percentiles"]:
            continue

        print(f"\n  [{cat_name}]")

        seen_keys = set()
        for key, label, unit, spec in METRIC_SPECS:
            if key in cat["metrics"] and key not in seen_keys:
                print(fmt_metric_line(label, cat["metrics"][key], unit, spec))
                seen_keys.add(key)

        for key, value in sorted(cat["metrics"].items()):
            if key in seen_keys or key in SUPPRESSED_KEYS:
                continue
            print(fmt_metric_line(key, value, "", ",.3f"))

        if cat["percentiles"]:
            print(f"\n    {'Percentile':<16} {'Latency (ms)':>14}")
            for label in PERCENTILE_LABELS:
                if label in cat["percentiles"]:
                    print(f"    p{label:<15} {cat['percentiles'][label]:>14,.3f}")
            for label, value in sorted(cat["percentiles"].items(),
                                        key=lambda kv: float(kv[0])):
                if label in PERCENTILE_LABELS:
                    continue
                print(f"    p{label:<15} {value:>14,.3f}")

    print()


def print_comparison_table(scenario_results):
    """Compact one-line-per-scenario table using Totals category."""
    print("=" * 78)
    print("Summary (Totals)")
    print("=" * 78)
    header = f"{'Scenario':<26} {'Ops/sec':>12} {'Avg (ms)':>10} {'P50':>9} {'P95':>9} {'P99':>9} {'Runs':>5}"
    print(header)
    print("-" * len(header))

    for scenario, averaged, n in scenario_results:
        totals = averaged.get("Totals", {"metrics": {}, "percentiles": {}})
        m = totals["metrics"]
        p = totals["percentiles"]
        ops = m.get("Ops/sec", 0)
        lat = m.get("Latency", m.get("Average Latency", 0))
        p50 = p.get("50.00", 0)
        p95 = p.get("95.00", 0)
        p99 = p.get("99.00", 0)
        print(f"{scenario:<26} {ops:>12,.0f} {lat:>10.3f} {p50:>9.3f} {p95:>9.3f} {p99:>9.3f} {n:>5}")
    print()


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <results_dir>")
        sys.exit(1)

    results_dir = sys.argv[1]
    json_files = sorted(glob.glob(os.path.join(results_dir, "*.json")))

    if not json_files:
        print(f"No results found in {results_dir}")
        sys.exit(1)

    groups = defaultdict(list)
    for path in json_files:
        base = os.path.basename(path)
        scenario = base.rsplit("-run", 1)[0]
        groups[scenario].append(path)

    scenario_results = []
    for scenario in sorted(groups.keys()):
        runs = [load_run(p) for p in groups[scenario]]
        runs = [r for r in runs if r]
        if not runs:
            print(f"Scenario: {scenario}  -- no parsable data\n")
            continue
        averaged = average_runs(runs)
        print_scenario_detail(scenario, averaged, len(runs))
        scenario_results.append((scenario, averaged, len(runs)))

    if scenario_results:
        print_comparison_table(scenario_results)


if __name__ == "__main__":
    main()