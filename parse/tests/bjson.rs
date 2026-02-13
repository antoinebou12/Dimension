//! BJSON parser tests.

#[test]
fn parse_null() {
    let data = [0u8];
    let v = parse::bjson::parse(&data).unwrap();
    assert!(matches!(v, parse::bjson::BjsonValue::Null));
}
