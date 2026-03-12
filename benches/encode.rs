use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use toon_cli::encode::encode;

fn make_tabular_json(rows: usize) -> String {
    let mut items = Vec::with_capacity(rows);
    for i in 0..rows {
        items.push(format!(
            r#"{{"id":{},"name":"user{}","email":"user{}@example.com","score":{}.{}}}"#,
            i, i, i, i * 10, i % 10
        ));
    }
    format!("[{}]", items.join(","))
}

fn make_nested_json(depth: usize) -> String {
    let mut s = String::new();
    for i in 0..depth {
        s.push_str(&format!(r#"{{"level{}":{}"#, i, ""));
    }
    s.push_str(r#""leaf""#);
    for _ in 0..depth {
        s.push('}');
    }
    s
}

fn make_mixed_json() -> String {
    r#"{
        "status": "ok",
        "metadata": {"version": 2, "region": "us-east-1"},
        "users": [
            {"id": 1, "name": "Alice", "role": "admin", "active": true},
            {"id": 2, "name": "Bob", "role": "user", "active": false},
            {"id": 3, "name": "Carol", "role": "editor", "active": true}
        ],
        "tags": ["production", "v2", "stable"],
        "config": {
            "max_retries": 3,
            "timeout": 30,
            "endpoints": ["api.example.com", "backup.example.com"]
        }
    }"#.to_string()
}

fn parse(json: &str) -> simd_json::OwnedValue {
    let mut data = json.as_bytes().to_vec();
    simd_json::to_owned_value(&mut data).unwrap()
}

fn bench_encode(c: &mut Criterion) {
    // Tabular scaling
    let mut group = c.benchmark_group("tabular");
    for size in [10, 100, 1000, 10000] {
        let json = make_tabular_json(size);
        let value = parse(&json);
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &value, |b, val| {
            b.iter(|| encode(black_box(val)));
        });
    }
    group.finish();

    // Nested object
    let mut group = c.benchmark_group("nested");
    for depth in [5, 20, 50] {
        let json = make_nested_json(depth);
        let value = parse(&json);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &value, |b, val| {
            b.iter(|| encode(black_box(val)));
        });
    }
    group.finish();

    // Mixed real-world shape
    let mut group = c.benchmark_group("mixed");
    let json = make_mixed_json();
    let value = parse(&json);
    group.throughput(Throughput::Bytes(json.len() as u64));
    group.bench_function("api_response", |b| {
        b.iter(|| encode(black_box(&value)));
    });
    group.finish();

    // Parse + encode (full pipeline)
    let mut group = c.benchmark_group("full_pipeline");
    for size in [100, 1000, 10000] {
        let json = make_tabular_json(size);
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json_str| {
            b.iter(|| {
                let mut data = json_str.as_bytes().to_vec();
                let value = simd_json::to_owned_value(&mut data).unwrap();
                encode(black_box(&value))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_encode);
criterion_main!(benches);
