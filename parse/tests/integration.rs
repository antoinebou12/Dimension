//! Integration tests (no feature requirements).

#[test]
fn error_display() {
    let e = parse::ParseError::Eof;
    assert!(!format!("{}", e).is_empty());
}
