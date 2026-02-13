//! OBJ parser tests.

#[test]
fn parse_simple() {
    let data = br#"
v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 3
"#;
    let obj = parse::obj::parse(data, None).unwrap();
    assert!(!obj.meshes.is_empty());
    assert!(!obj.meshes[0].vertices.is_empty());
}
