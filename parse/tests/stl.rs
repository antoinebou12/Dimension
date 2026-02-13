//! STL parser tests.

use parse::stl;

#[test]
fn stl_ascii_single_triangle() {
    let data = b"solid foo
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
vertex 0.5 1 0
endloop
endfacet
endsolid";
    let mesh = stl::parse(data).unwrap();
    assert_eq!(mesh.vertices.len(), 3);
}

#[test]
fn stl_binary_minimal() {
    let mut data = vec![0u8; 84 + 50];
    data[0..5].copy_from_slice(b"solid");
    data[80..84].copy_from_slice(&1u32.to_le_bytes());
    let floats: [f32; 12] = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
    for (i, &f) in floats.iter().enumerate() {
        data[84 + i * 4..84 + (i + 1) * 4].copy_from_slice(&f.to_le_bytes());
    }
    let mesh = stl::parse(&data).unwrap();
    assert_eq!(mesh.vertices.len(), 3);
}
