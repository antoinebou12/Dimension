//! XML parser using quick-xml (streaming, wasm-compatible).
//!
//! Provides a simple DOM-like representation suitable for config and asset parsing.

use crate::error::ParseError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;

/// XML element (tag with attributes and children).
#[derive(Clone, Debug, PartialEq)]
pub struct XmlElement {
    /// Local name.
    pub name: String,
    /// Attributes.
    pub attributes: BTreeMap<String, String>,
    /// Child elements and text.
    pub children: Vec<XmlNode>,
}

/// XML node: element or text.
#[derive(Clone, Debug, PartialEq)]
pub enum XmlNode {
    /// Element.
    Element(XmlElement),
    /// Text content.
    Text(String),
}

/// Parses XML from bytes into a root element.
///
/// Returns the first root element. Comments and processing instructions are skipped.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid XML.
pub fn parse(data: &[u8]) -> Result<XmlElement, ParseError> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<XmlElement> = Vec::new();
    let mut root: Option<XmlElement> = None;

    loop {
        match reader
            .read_event()
            .map_err(|e: quick_xml::Error| ParseError::Io(e.to_string()))?
        {
            Event::Start(e) => {
                let name = String::from_utf8(e.name().as_ref().to_vec())
                    .map_err(|e| ParseError::Io(e.to_string()))?;
                let mut attributes = BTreeMap::new();
                for attr in e.attributes() {
                    let a = attr.map_err(|e| ParseError::Io(e.to_string()))?;
                    let k = String::from_utf8(a.key.as_ref().to_vec())
                        .map_err(|e| ParseError::Io(e.to_string()))?;
                    let v = String::from_utf8(a.value.to_vec())
                        .map_err(|e| ParseError::Io(e.to_string()))?;
                    attributes.insert(k, v);
                }
                let elem = XmlElement {
                    name,
                    attributes,
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(elem.clone()));
                }
                stack.push(elem);
            }
            Event::End(_) => {
                if let Some(elem) = stack.pop() {
                    if stack.is_empty() {
                        root = Some(elem);
                        break;
                    }
                }
            }
            Event::Text(e) => {
                let text = e
                    .unescape()
                    .map_err(|e| ParseError::Io(e.to_string()))?
                    .trim()
                    .to_string();
                if !text.is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(XmlNode::Text(text));
                    }
                }
            }
            Event::CData(e) => {
                let text = String::from_utf8(e.into_inner().to_vec())
                    .map_err(|e| ParseError::Io(e.to_string()))?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Text(text));
                }
            }
            Event::Eof => break,
            Event::Comment(_) | Event::Decl(_) | Event::PI(_) | Event::DocType(_) => {}
            _ => {}
        }
    }

    root.ok_or_else(|| ParseError::Syntax {
        filename: None,
        row: 0,
        col: 0,
        msg: "no root element found".to_string(),
    })
}
