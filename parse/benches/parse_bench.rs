//! Parse crate benchmarks.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "json")]
fn bench_json(c: &mut Criterion) {
    let data = br#"{"a":1,"b":[2,3],"c":"hello"}"#;
    c.bench_function("json_parse_small", |b| {
        b.iter(|| {
            let _ = parse::json::parse(black_box(data));
        });
    });
}

#[cfg(not(feature = "json"))]
fn bench_json(_c: &mut Criterion) {}

criterion_group!(benches, bench_json);
criterion_main!(benches);
