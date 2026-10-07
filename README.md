# JSON vs Protobuf

A small benchmark that puts JSON and Protocol Buffers side by side on payload size and request latency. The same API is implemented twice in Python (FastAPI) and twice in Rust (Axum), once per format, so the comparison can be made within one language and across languages.

## Results

Numbers below come from runs on a MacBook over localhost. Your machine will differ, but the ratios have been stable across runs.

### Payload size

Protobuf comes out about 56% smaller every time, regardless of language or how many users are in the response. The reason is structural. JSON writes every field name as a string on every object, while protobuf replaces them with one or two byte integer tags defined in the schema.

```
{"order_id": "xddjgfpvimig", "product": "mllucbrk", "price": 493.31, "quantity": 11}
  vs
0a0c786e74636b...  (binary, no field names)
```

With 1000 users at roughly 3 orders each, the string `"order_id"` goes over the wire 3000 times in JSON and zero times in protobuf.

| Payload (10k users) | JSON    | Protobuf | Saving |
|---------------------|---------|----------|--------|
| Avg size            | 4014 KB | 1767 KB  | 56%    |

### Latency in Python

| Metric      | JSON     | Protobuf | Change |
|-------------|----------|----------|--------|
| Avg latency | 145.95ms | 155.20ms | -6%    |
| p95 latency | 160.94ms | 161.38ms | ~0%    |

Protobuf looks slightly slower here. That is a property of the Python runtime rather than the format. The standard `json` module is a C extension that has had years of tuning, and the Python protobuf bindings carry more overhead per message. On localhost the measurement is dominated by serialisation cost, so the language wins and the format loses.

### Latency in Rust

| Metric      | JSON    | Protobuf | Change |
|-------------|---------|----------|--------|
| Avg latency | 18.18ms | 11.52ms  | +36%   |
| Min latency | 17.14ms | 10.99ms  | +35%   |
| p95 latency | 18.86ms | 12.08ms  | +36%   |

`serde_json` and `prost` are both native Rust with comparable amounts of optimisation, so neither format gets a free runtime advantage. Here protobuf wins by about a third, because it has 56% less data to produce. The latency gain is smaller than the size gain because connection handling, the TCP stack and the HTTP framing cost the same either way.

### Python vs Rust

|                | Python avg | Rust avg |
|----------------|------------|----------|
| JSON latency   | 145ms      | 18ms     |
| Proto latency  | 155ms      | 11ms     |

Rust is around 8x faster overall, which is expected but worth stating since it is the bigger lever if you only care about speed.

### What the size difference means on a real link

Both servers run on localhost so actual transfer time is close to zero. The benchmark also computes the transfer time of each response at a few fixed bandwidths, using the 10k user payload.

| Network | JSON       | Protobuf  | Saved     |
|---------|------------|-----------|-----------|
| 2G      | 200,717ms  | 88,350ms  | 112s      |
| 3G      | 16,057ms   | 7,068ms   | 9s        |
| 4G      | 1,605ms    | 706ms     | 0.9s      |
| WiFi    | 642ms      | 282ms     | 0.36s     |
| 100Mb   | 321ms      | 141ms     | 0.18s     |

### So when is it worth it

| Scenario | JSON | Protobuf |
|---|---|---|
| Localhost or same datacenter | Fine | Fine, a bit faster in Rust |
| Cross region API | Slower | Faster, 56% less to transfer |
| Mobile or 3G clients | Painful | Much better |
| High volume microservices | Bandwidth adds up | Real infra saving |
| Debugging by eye | Easy | Needs tooling |
| Schema evolution | Loose | Strict, breaking changes need care |

The short version: protobuf's advantage is real, but a Python benchmark on localhost will hide it. Test in a compiled language, or test over an actual network, before deciding it does not matter.

## Data model

Each response is a list of users. A user has an address and between one and five orders, so there is enough nesting and repetition for the field name overhead in JSON to show up.

```proto
message User {
  int32 id = 1;
  string name = 2;
  string email = 3;
  int32 age = 4;
  Address address = 5;
  repeated Order orders = 6;
}
```

The full schema is in `schema/user.proto` and is shared by all four servers.

## Layout

```
schema/user.proto        shared schema
generated/user_pb2.py    Python code compiled from the schema
server_json.py           Python JSON API, port 8001
server_proto.py          Python protobuf API, port 8002
benchmark.py             Python client, hits both and prints a comparison
generate_proto.sh        runs protoc for the Python side
network_slow.sh          throttles loopback on macOS with dummynet
network_reset.sh         removes the throttle
rust/
  build.rs               compiles the schema with prost-build
  src/lib.rs
  src/bin/server_json.rs   Rust JSON API, port 8003
  src/bin/server_proto.rs  Rust protobuf API, port 8004
  src/bin/benchmark.rs     Rust client
```

## Running it

### Python

```bash
pip install -r requirements.txt
./generate_proto.sh

# in two terminals
python server_json.py
python server_proto.py

python benchmark.py --count 1000 --requests 200
```

`--count` is how many users each response contains and `--requests` is how many times each endpoint is hit.

### Rust

```bash
cd rust
cargo build --release

# in two terminals
./target/release/server_json
./target/release/server_proto

./target/release/benchmark --count 10000 --requests 200
```

### Simulating a slow network (macOS only)

```bash
./network_slow.sh 5Mbit 50     # bandwidth, delay in ms
python benchmark.py
./network_reset.sh
```

This uses `pfctl` and `dnctl`, so it needs sudo and only applies to the Python ports 8001 and 8002.
