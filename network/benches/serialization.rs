//! Serialization benchmarks: protobuf encode/decode throughput.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use network::protocol::proto::{EntityState, Transform, Vec3, WorldSnapshot};
use network::serialize::binary;

fn bench_vec3_encode(c: &mut Criterion) {
    let v = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    c.bench_function("encode_vec3", |b| {
        b.iter(|| black_box(binary::encode_vec3(&v).unwrap()));
    });
}

fn bench_vec3_decode(c: &mut Criterion) {
    let v = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let bytes = binary::encode_vec3(&v).unwrap();
    c.bench_function("decode_vec3", |b| {
        b.iter(|| black_box(binary::decode_vec3(&bytes).unwrap()));
    });
}

fn bench_world_snapshot_encode(c: &mut Criterion) {
    let entity = EntityState {
        id: 1,
        transform: Some(Transform {
            position: Some(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            rotation: Some(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            scale: Some(Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
        }),
        primitive_type: 0,
        color: None,
    };
    let snap = WorldSnapshot {
        entities: vec![entity; 100],
        tick: 42,
        timestamp: 123.456,
    };
    c.bench_function("encode_world_snapshot_100_entities", |b| {
        b.iter(|| black_box(binary::encode_world_snapshot(&snap).unwrap()));
    });
}

criterion_group!(
    benches,
    bench_vec3_encode,
    bench_vec3_decode,
    bench_world_snapshot_encode
);
criterion_main!(benches);
