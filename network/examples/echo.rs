//! Minimal echo server/client example.

use network::protocol::proto::Vec3;
use network::serialize::binary;

fn main() {
    let v = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let bytes = binary::encode_vec3(&v).unwrap();
    let decoded = binary::decode_vec3(&bytes).unwrap();
    println!("Echo: ({}, {}, {})", decoded.x, decoded.y, decoded.z);
}
