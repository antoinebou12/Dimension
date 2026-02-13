//! glTF parser tests. Uses minimal embedded GLB.

#[test]
fn parse_minimal_glb() {
    // Minimal valid GLB: 12-byte header + JSON chunk (no meshes)
    let json = br#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{}]}"#;
    let json_len = json.len();
    let total = 12 + 8 + json_len; // header + chunk header + json
    let mut glb = Vec::with_capacity(total);
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total as u32).to_le_bytes()); // length
    glb.extend_from_slice(&(json_len as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // JSON chunk type
    glb.extend_from_slice(json);
    let data = parse::gltf::parse(&glb).unwrap();
    assert!(data.is_empty()); // no meshes in minimal
}
