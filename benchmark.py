"""
Benchmark: JSON API (port 8001) vs Protobuf API (port 8002)

Measures:
  - Response payload size (bytes)
  - Actual localhost latency
  - Simulated transfer time at real-world bandwidths

Usage:
  python benchmark.py [--count 100] [--requests 50]
"""
import sys
import os
import time
import json
import argparse
import statistics

import requests

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "generated"))
import user_pb2

JSON_BASE = "http://localhost:8001"
PROTO_BASE = "http://localhost:8002"

# Bandwidth presets in bits/sec
BANDWIDTHS = {
    "2G":       160_000,
    "3G":     2_000_000,
    "4G":    20_000_000,
    "WiFi":  50_000_000,
    "100Mb": 100_000_000,
}


def transfer_ms(size_bytes: float, bps: int) -> float:
    """Time to transfer size_bytes over a link of bps bits/sec."""
    return (size_bytes * 8 / bps) * 1000


def benchmark_endpoint(label: str, url: str, parse_fn, n: int):
    latencies = []
    sizes = []

    requests.get(url)  # warm up

    for _ in range(n):
        t0 = time.perf_counter()
        resp = requests.get(url)
        elapsed = (time.perf_counter() - t0) * 1000
        parse_fn(resp.content)
        latencies.append(elapsed)
        sizes.append(len(resp.content))

    return {
        "label": label,
        "requests": n,
        "size_bytes_avg": statistics.mean(sizes),
        "size_bytes_min": min(sizes),
        "size_bytes_max": max(sizes),
        "latency_ms_avg": statistics.mean(latencies),
        "latency_ms_min": min(latencies),
        "latency_ms_max": max(latencies),
        "latency_ms_p95": sorted(latencies)[int(0.95 * n)],
    }


def parse_json_users(data: bytes):
    return json.loads(data)["users"]


def parse_proto_users(data: bytes):
    ul = user_pb2.UserList()
    ul.ParseFromString(data)
    return ul.users


def print_results(json_r: dict, proto_r: dict):
    W = 68

    def row(name, j, p, fmt=".2f", unit="ms"):
        saving = (1 - p / j) * 100 if j else 0
        print(f"  {name:<28} {j:>9{fmt}}{unit}  {p:>9{fmt}}{unit}  {saving:>+7.1f}%")

    def section(title):
        print(f"\n  {'--- ' + title + ' ---'}")

    print("\n" + "=" * W)
    print(f"  {'Metric':<28} {'JSON':>12}  {'Protobuf':>12}  {'Saving':>8}")
    print("-" * W)

    # Payload size
    js = json_r["size_bytes_avg"]
    ps = proto_r["size_bytes_avg"]
    size_saving = (1 - ps / js) * 100
    print(f"  {'Avg payload size':<28} {js:>10.0f}B  {ps:>10.0f}B  {size_saving:>+7.1f}%")
    print(f"  {'Min payload size':<28} {json_r['size_bytes_min']:>10.0f}B  {proto_r['size_bytes_min']:>10.0f}B")
    print(f"  {'Max payload size':<28} {json_r['size_bytes_max']:>10.0f}B  {proto_r['size_bytes_max']:>10.0f}B")

    # Actual localhost latency
    section("Actual localhost latency (no real network)")
    row("Avg", json_r["latency_ms_avg"], proto_r["latency_ms_avg"])
    row("Min", json_r["latency_ms_min"], proto_r["latency_ms_min"])
    row("Max", json_r["latency_ms_max"], proto_r["latency_ms_max"])
    row("p95", json_r["latency_ms_p95"], proto_r["latency_ms_p95"])

    # Simulated transfer time by bandwidth
    section("Simulated transfer time  (payload / bandwidth)")
    print(f"  {'Network':<28} {'JSON':>12}  {'Protobuf':>12}  {'Saved':>8}")
    for name, bps in BANDWIDTHS.items():
        jt = transfer_ms(js, bps)
        pt = transfer_ms(ps, bps)
        row(name, jt, pt)

    print("-" * W)
    print("  + saving = protobuf faster/smaller")
    print("  Transfer time = payload_bytes * 8 / bandwidth_bps")
    print("=" * W + "\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=100, help="users per request")
    parser.add_argument("--requests", type=int, default=50, help="number of requests")
    args = parser.parse_args()

    count = args.count
    n = args.requests

    print(f"\nBenchmark: {n} requests x {count} users each")
    print("Checking servers...")

    try:
        requests.get(f"{JSON_BASE}/users?count=1", timeout=3)
    except Exception:
        print(f"ERROR: JSON server not reachable. Run: python server_json.py")
        sys.exit(1)

    try:
        requests.get(f"{PROTO_BASE}/users?count=1", timeout=3)
    except Exception:
        print(f"ERROR: Proto server not reachable. Run: python server_proto.py")
        sys.exit(1)

    print("Running JSON...")
    json_r = benchmark_endpoint("JSON", f"{JSON_BASE}/users?count={count}", parse_json_users, n)

    print("Running Protobuf...")
    proto_r = benchmark_endpoint("Protobuf", f"{PROTO_BASE}/users?count={count}", parse_proto_users, n)

    print_results(json_r, proto_r)


if __name__ == "__main__":
    main()
