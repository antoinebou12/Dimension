//! PLY parser tests.

#[test]
fn parse_ascii_ply() {
    let data = br#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
0 0 0
1 0 0
0 1 0
3 0 1 2
"#;
    let mesh = parse::ply::parse(data).unwrap();
    assert!(!mesh.vertices.is_empty());
}
