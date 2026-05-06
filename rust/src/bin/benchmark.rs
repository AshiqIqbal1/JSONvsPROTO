use std::time::Instant;

const JSON_BASE: &str = "http://localhost:8003";
const PROTO_BASE: &str = "http://localhost:8004";

struct BandwidthPreset {
    name: &'static str,
    bps: u64,
}

const BANDWIDTHS: &[BandwidthPreset] = &[
    BandwidthPreset { name: "2G",    bps: 160_000 },
    BandwidthPreset { name: "3G",    bps: 2_000_000 },
    BandwidthPreset { name: "4G",    bps: 20_000_000 },
    BandwidthPreset { name: "WiFi",  bps: 50_000_000 },
    BandwidthPreset { name: "100Mb", bps: 100_000_000 },
];

fn transfer_ms(size_bytes: f64, bps: u64) -> f64 {
    (size_bytes * 8.0 / bps as f64) * 1000.0
}

struct Results {
    _label: String,
    size_avg: f64,
    size_min: usize,
    size_max: usize,
    lat_avg: f64,
    lat_min: f64,
    lat_max: f64,
    lat_p95: f64,
}

async fn run_benchmark(
    client: &reqwest::Client,
    label: &str,
    url: &str,
    n: usize,
) -> Results {
    // warm up
    let _ = client.get(url).send().await.unwrap();

    let mut latencies = Vec::with_capacity(n);
    let mut sizes = Vec::with_capacity(n);

    for _ in 0..n {
        let t0 = Instant::now();
        let resp = client.get(url).send().await.unwrap();
        let body = resp.bytes().await.unwrap();
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        sizes.push(body.len());
        latencies.push(elapsed);
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let size_avg = sizes.iter().sum::<usize>() as f64 / n as f64;
    let lat_avg = latencies.iter().sum::<f64>() / n as f64;
    let lat_p95 = latencies[(n as f64 * 0.95) as usize];

    Results {
        _label: label.to_string(),
        size_avg,
        size_min: *sizes.iter().min().unwrap(),
        size_max: *sizes.iter().max().unwrap(),
        lat_avg,
        lat_min: *latencies.first().unwrap(),
        lat_max: *latencies.last().unwrap(),
        lat_p95,
    }
}

fn print_results(j: &Results, p: &Results) {
    let w = 68;
    let saving = |jv: f64, pv: f64| (1.0 - pv / jv) * 100.0;

    println!("\n{}", "=".repeat(w));
    println!("  {:<28} {:>12}  {:>12}  {:>8}", "Metric", "JSON", "Protobuf", "Saving");
    println!("{}", "-".repeat(w));

    println!(
        "  {:<28} {:>10.0}B  {:>10.0}B  {:>+7.1}%",
        "Avg payload size", j.size_avg, p.size_avg,
        saving(j.size_avg, p.size_avg)
    );
    println!(
        "  {:<28} {:>10}B  {:>10}B",
        "Min payload size", j.size_min, p.size_min
    );
    println!(
        "  {:<28} {:>10}B  {:>10}B",
        "Max payload size", j.size_max, p.size_max
    );

    println!("\n  --- Actual localhost latency (no real network) ---");
    let rows = [
        ("Avg", j.lat_avg, p.lat_avg),
        ("Min", j.lat_min, p.lat_min),
        ("Max", j.lat_max, p.lat_max),
        ("p95", j.lat_p95, p.lat_p95),
    ];
    for (name, jv, pv) in &rows {
        println!(
            "  {:<28} {:>9.2}ms  {:>9.2}ms  {:>+7.1}%",
            name, jv, pv, saving(*jv, *pv)
        );
    }

    println!("\n  --- Simulated transfer time (payload / bandwidth) ---");
    println!("  {:<28} {:>12}  {:>12}  {:>8}", "Network", "JSON", "Protobuf", "Saved");
    for bw in BANDWIDTHS {
        let jt = transfer_ms(j.size_avg, bw.bps);
        let pt = transfer_ms(p.size_avg, bw.bps);
        println!(
            "  {:<28} {:>9.2}ms  {:>9.2}ms  {:>+7.1}%",
            bw.name, jt, pt, saving(jt, pt)
        );
    }

    println!("{}", "-".repeat(w));
    println!("  + saving = protobuf faster/smaller");
    println!("  Transfer time = payload_bytes * 8 / bandwidth_bps");
    println!("{}\n", "=".repeat(w));
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = args.iter()
        .position(|a| a == "--count")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let n: usize = args.iter()
        .position(|a| a == "--requests")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);

    println!("\nRust Benchmark: {n} requests x {count} users each");
    println!("Checking servers...");

    let client = reqwest::Client::new();

    let check_json = client.get(&format!("{JSON_BASE}/users?count=1")).send().await;
    if check_json.is_err() {
        eprintln!("ERROR: Rust JSON server not reachable at {JSON_BASE}. Run: cargo run --bin server_json");
        std::process::exit(1);
    }
    let check_proto = client.get(&format!("{PROTO_BASE}/users?count=1")).send().await;
    if check_proto.is_err() {
        eprintln!("ERROR: Rust Proto server not reachable at {PROTO_BASE}. Run: cargo run --bin server_proto");
        std::process::exit(1);
    }

    println!("Running JSON...");
    let json_r = run_benchmark(&client, "JSON", &format!("{JSON_BASE}/users?count={count}"), n).await;

    println!("Running Protobuf...");
    let proto_r = run_benchmark(&client, "Protobuf", &format!("{PROTO_BASE}/users?count={count}"), n).await;

    print_results(&json_r, &proto_r);
}
