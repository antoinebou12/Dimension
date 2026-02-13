//! XML parser tests.

#[test]
fn parse_simple() {
    let data = br#"<root><child>text</child></root>"#;
    let el = parse::xml::parse(data).unwrap();
    assert_eq!(el.name, "root");
    assert_eq!(el.children.len(), 1);
    match &el.children[0] {
        parse::xml::XmlNode::Element(e) => assert_eq!(e.name, "child"),
        _ => panic!("expected element"),
    }
}
