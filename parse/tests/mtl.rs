//! MTL parser tests.

#[test]
fn parse_simple() {
    let data = br#"
newmtl mat1
Ka 1 0 0
Kd 0.8 0.2 0.2
"#;
    let mats = parse::mtl::parse(data).unwrap();
    assert!(mats.contains_key("mat1"));
}
