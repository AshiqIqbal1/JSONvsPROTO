# JSON vs Protobuf Benchmark

Comparing JSON and Protocol Buffers across payload size and latency — in both Python and Rust.

---

## Findings

### 1. Payload Size — Protobuf wins 56% every time

Protobuf is consistently ~56% smaller regardless of language, payload size, or number of requests. This is structural: JSON repeats field names as strings on every object. Protobuf replaces them with tiny integer tags defined in the `.proto` schema.

```
{"order_id": "xddjgfpvimig", "product": "mllucbrk", "price": 493.31, "quantity": 11}
  vs
0a0c786e74636b...  (binary, no field names)
```

With 1000 users each having ~3 orders, `"order_id"` alone is sent 3000 times on the wire in JSON. In Protobuf: never.

| Payload (10k users) | JSON    | Protobuf | Saving |
|---------------------|---------|----------|--------|
| Avg size            | 4014 KB | 1767 KB  | 56%    |

---

### 2. Latency — Python lied, Rust told the truth

#### Python (FastAPI, ports 8001/8002)

| Metric      | JSON     | Protobuf | Saving  |
|-------------|----------|----------|---------|
| Avg latency | 145.95ms | 155.20ms | **-6%** |
| p95 latency | 160.94ms | 161.38ms | ~0%     |

Protobuf appears *slower* in Python. This is an artifact: Python's `json` module is a C extension, heavily optimised. Python's protobuf bindings carry more overhead. The benchmark was measuring the language runtime, not the format.

#### Rust (Axum, ports 8003/8004)

| Metric      | JSON    | Protobuf | Saving   |
|-------------|---------|----------|----------|
| Avg latency | 18.18ms | 11.52ms  | **+36%** |
| Min latency | 17.14ms | 10.99ms  | **+35%** |
| p95 latency | 18.86ms | 12.08ms  | **+36%** |

Rust removes the bias. `serde_json` and `prost` are both native Rust — equal footing. With no runtime advantage for either side, protobuf wins because it has 56% less data to encode. Less work = faster. The latency saving (~36%) is less than the payload saving (~56%) because server overhead, TCP stack, and connection handling are constant regardless of format.

#### Rust is also 8x faster overall

| | Python avg | Rust avg |
|---|---|---|
| JSON latency | 145ms | 18ms |
| Proto latency | 155ms | 11ms |

---

### 3. Simulated Real-World Transfer (payload / bandwidth)

Since both servers run on localhost, network transfer time is ~0ms. This table shows what the payload size difference means over real networks:

| Network | JSON       | Protobuf  | Saved     |
|---------|------------|-----------|-----------|
| 2G      | 200,717ms  | 88,350ms  | **112s**  |
| 3G      | 16,057ms   | 7,068ms   | **9s**    |
| 4G      | 1,605ms    | 706ms     | **0.9s**  |
| WiFi    | 642ms      | 282ms     | **0.36s** |
| 100Mb   | 321ms      | 141ms     | **0.18s** |

Per request, 10,000 users payload. On 3G, every single API call saves 9 seconds.

---

### 4. When does Protobuf actually matter?

| Scenario | JSON | Protobuf |
|---|---|---|
| Localhost / same datacenter | Fine | Fine (slightly faster in Rust) |
| Cross-region API | Slower | Faster (56% less transfer) |
| Mobile / 3G clients | Very slow | Much faster |
| High-frequency microservices (millions of req/day) | High bandwidth cost | Significant infra saving |
| Human-readable debugging | Easy | Needs tooling |
| Schema changes / versioning | Flexible | Strict (breaking changes need care) |

---

## Project Structure

```
.
├── schema/
│   └── user.proto          # Shared proto schema
├── generated/
│   └── user_pb2.py         # Generated Python protobuf code
├── server_json.py          # Python JSON API (port 8001)
├── server_proto.py         # Python Protobuf API (port 8002)
├── benchmark.py            # Python benchmark client
├── requirements.txt
├── generate_proto.sh       # Compiles user.proto -> user_pb2.py
├── network_slow.sh         # macOS network throttle (pfctl/dummynet)
├── network_reset.sh        # Remove network throttle
└── rust/
    ├── Cargo.toml
    ├── build.rs             # Compiles proto via prost-build
    └── src/
        ├── lib.rs
        └── bin/
            ├── server_json.rs   # Rust JSON API (port 8003)
            ├── server_proto.rs  # Rust Protobuf API (port 8004)
            └── benchmark.rs     # Rust benchmark client
```

---

## Running It

### Python

```bash
# Install dependencies
pip install -r requirements.txt

# Generate protobuf Python code
./generate_proto.sh

# Start servers (two terminals)
python server_json.py    # port 8001
python server_proto.py   # port 8002

# Run benchmark
python benchmark.py --count 1000 --requests 200
```

### Rust

```bash
cd rust

# Build all binaries
cargo build --release

# Start servers (two terminals)
./target/release/server_json    # port 8003
./target/release/server_proto   # port 8004

# Run benchmark
./target/release/benchmark --count 10000 --requests 200
```

---

## Data Model

A realistic nested structure — not a toy example:

```proto
message User {
  int32 id = 1;
  string name = 2;
  string email = 3;
  int32 age = 4;
  Address address = 5;
  repeated Order orders = 6;  // 1-5 orders per user
}
```

Each user has a nested `Address` and a list of `Order` objects with product, price, and quantity. This is representative of a real API response where field name repetition in JSON adds up fast.

---

## Key Takeaway

> Protobuf's advantage is real but the language you benchmark in can hide it.
> Python's optimised JSON parser makes JSON look artificially fast.
> Rust benchmarks both formats fairly — and protobuf wins on payload size AND latency.
> The payload saving always translates directly to bandwidth and transfer time savings on real networks.
