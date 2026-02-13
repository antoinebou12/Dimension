//! TOON parser tests.

#[test]
fn parse_simple() {
    // Flat key: value (TOON core)
    let data = br#"name: "foo""#;
    let v = parse::toon::parse(data).unwrap();
    assert!(matches!(v, parse::toon::ToonValue::Object(_)));
}
