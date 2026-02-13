//! Serialization unit tests.

use network::protocol::proto::{Vec3, WorldSnapshot};
use network::serialize::binary;

#[test]
fn encode_decode_vec3() {
    let v = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let bytes = binary::encode_vec3(&v).unwrap();
    let decoded = binary::decode_vec3(&bytes).unwrap();
    assert!((decoded.x - v.x).abs() < 1e-6);
    assert!((decoded.y - v.y).abs() < 1e-6);
    assert!((decoded.z - v.z).abs() < 1e-6);
}

#[test]
fn encode_decode_world_snapshot() {
    let snap = WorldSnapshot {
        entities: vec![],
        tick: 42,
        timestamp: 123.456,
    };
    let bytes = binary::encode_world_snapshot(&snap).unwrap();
    let decoded = binary::decode_world_snapshot(&bytes).unwrap();
    assert_eq!(decoded.tick, snap.tick);
    assert!((decoded.timestamp - snap.timestamp).abs() < 1e-6);
}
