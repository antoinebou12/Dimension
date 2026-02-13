//! JSON parser tests.

#[test]
fn parse_simple() {
    let data = br#"{"a":1,"b":[2,3],"c":"hello"}"#;
    let v = parse::json::parse(data).unwrap();
    match &v {
        parse::json::JsonValue::Object(m) => {
            assert!(m.contains_key("a"));
            assert!(m.contains_key("b"));
            assert!(m.contains_key("c"));
        }
        _ => panic!("expected object"),
    }
}
