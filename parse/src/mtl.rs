//! Wavefront MTL material parser.
//!
//! Hand-rolled parser for `newmtl`, `Ka`/`Kd`/`Ks`/`Ke`, `Ns`, `Ni`, `illum`,
//! PBR (`Pr`, `Pm`, `Ps`, `Pc`, `Pcr`), and texture maps (`map_bump`, `map_roughness`).

use crate::error::ParseError;
use crate::parser::Parser;
use crate::Material;
use std::collections::HashMap;

/// Parses MTL from bytes.
///
/// Returns a map from material name to material. Caller resolves texture paths.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid MTL.
pub fn parse(data: &[u8]) -> Result<HashMap<String, Material>, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    parse_str(s)
}

/// Parses MTL from string.
pub fn parse_str(s: &str) -> Result<HashMap<String, Material>, ParseError> {
    let mut p = Parser::new(s.as_bytes(), None);
    let mut materials = HashMap::new();
    let mut current: Option<Material> = None;

    while !p.is_eof() {
        p.parse_whitespace();
        if p.match_char(b'\n') {
            p.advance();
            continue;
        }
        if p.match_char(b'#') {
            skip_comment(&mut p);
            continue;
        }
        if p.is_eof() {
            break;
        }

        let keyword = parse_keyword(&mut p)?;
        p.parse_whitespace();

        match keyword.as_str() {
            "newmtl" => {
                if let Some(m) = current.take() {
                    materials.insert(m.name.clone(), m);
                }
                let name = parse_rest_of_line(&mut p)?;
                current = Some(Material {
                    name: name.trim().to_string(),
                    ..Material::default()
                });
            }
            "Ka" => {
                if let Some(ref mut m) = current {
                    let (r, g, b, a) = parse_rgba(&mut p)?;
                    m.ka = [r, g, b, a];
                }
                skip_rest_of_line(&mut p);
            }
            "Kd" => {
                if let Some(ref mut m) = current {
                    let (r, g, b, a) = parse_rgba(&mut p)?;
                    m.kd = [r, g, b, a];
                }
                skip_rest_of_line(&mut p);
            }
            "Ks" => {
                if let Some(ref mut m) = current {
                    let (r, g, b, a) = parse_rgba(&mut p)?;
                    m.ks = [r, g, b, a];
                }
                skip_rest_of_line(&mut p);
            }
            "Ke" => {
                if let Some(ref mut m) = current {
                    let (r, g, b, a) = parse_rgba(&mut p)?;
                    m.ke = [r, g, b, a];
                }
                skip_rest_of_line(&mut p);
            }
            "Ns" => {
                if let Some(ref mut m) = current {
                    m.ns = parse_f32(&mut p).unwrap_or(128.0);
                }
                skip_rest_of_line(&mut p);
            }
            "Ni" => {
                if let Some(ref mut m) = current {
                    m.ni = parse_f32(&mut p).unwrap_or(1.0);
                }
                skip_rest_of_line(&mut p);
            }
            "illum" => {
                if let Some(ref mut m) = current {
                    m.illum = parse_i32(&mut p).unwrap_or(2);
                }
                skip_rest_of_line(&mut p);
            }
            "Pr" | "map_Pr" => {
                if let Some(ref mut m) = current {
                    m.pr = parse_f32(&mut p).unwrap_or(0.0);
                }
                let rest = parse_rest_of_line(&mut p)?;
                if keyword == "map_Pr" && !rest.trim().is_empty() {
                    if let Some(ref mut m) = current {
                        m.map_roughness = Some(rest.trim().to_string());
                    }
                }
            }
            "Pm" => {
                if let Some(ref mut m) = current {
                    m.pm = parse_f32(&mut p).unwrap_or(0.0);
                }
                skip_rest_of_line(&mut p);
            }
            "Ps" => {
                if let Some(ref mut m) = current {
                    m.ps = parse_f32(&mut p).unwrap_or(0.0);
                }
                skip_rest_of_line(&mut p);
            }
            "Pc" => {
                if let Some(ref mut m) = current {
                    m.pc = parse_f32(&mut p).unwrap_or(0.0);
                }
                skip_rest_of_line(&mut p);
            }
            "Pcr" => {
                if let Some(ref mut m) = current {
                    m.pcr = parse_f32(&mut p).unwrap_or(0.0);
                }
                skip_rest_of_line(&mut p);
            }
            "map_Kd" | "map_Ka" | "map_Ks" | "map_Ke" => {
                skip_rest_of_line(&mut p);
            }
            "map_Bump" | "map_bump" | "bump" => {
                if let Some(ref mut m) = current {
                    let path = parse_rest_of_line(&mut p)?.trim().to_string();
                    if !path.is_empty() {
                        m.map_bump = Some(path);
                    }
                }
            }
            "map_roughness" => {
                if let Some(ref mut m) = current {
                    let path = parse_rest_of_line(&mut p)?.trim().to_string();
                    if !path.is_empty() {
                        m.map_roughness = Some(path);
                    }
                }
            }
            _ => {
                skip_rest_of_line(&mut p);
            }
        }
    }

    if let Some(m) = current {
        materials.insert(m.name.clone(), m);
    }

    Ok(materials)
}

fn skip_comment(p: &mut Parser<'_>) {
    while !p.is_eof() && !p.match_char(b'\n') {
        p.advance();
    }
}

fn parse_keyword(p: &mut Parser<'_>) -> Result<String, ParseError> {
    let mut s = String::new();
    while !p.is_eof() && (p.peek().is_ascii_alphanumeric() || p.peek() == b'_') {
        s.push(p.peek() as char);
        p.advance();
    }
    if s.is_empty() {
        return Err(p.syntax_err("expected keyword"));
    }
    Ok(s)
}

fn parse_f32(p: &mut Parser<'_>) -> Option<f32> {
    let start = p.offset;
    if p.peek() == b'-' {
        p.advance();
    }
    while p.peek().is_ascii_digit() {
        p.advance();
    }
    if p.peek() == b'.' {
        p.advance();
        while p.peek().is_ascii_digit() {
            p.advance();
        }
    }
    if p.peek() == b'e' || p.peek() == b'E' {
        p.advance();
        if p.peek() == b'+' || p.peek() == b'-' {
            p.advance();
        }
        while p.peek().is_ascii_digit() {
            p.advance();
        }
    }
    let slice = &p.data[start..p.offset];
    std::str::from_utf8(slice).ok()?.parse().ok()
}

fn parse_i32(p: &mut Parser<'_>) -> Option<i32> {
    let start = p.offset;
    if p.peek() == b'-' {
        p.advance();
    }
    while p.peek().is_ascii_digit() {
        p.advance();
    }
    let slice = &p.data[start..p.offset];
    std::str::from_utf8(slice).ok()?.parse().ok()
}

fn parse_rgba(p: &mut Parser<'_>) -> Result<(f32, f32, f32, f32), ParseError> {
    let r = parse_f32(p).ok_or_else(|| p.syntax_err("expected number"))?;
    p.parse_whitespace();
    let g = parse_f32(p).ok_or_else(|| p.syntax_err("expected number"))?;
    p.parse_whitespace();
    let b = parse_f32(p).ok_or_else(|| p.syntax_err("expected number"))?;
    p.parse_whitespace();
    let a = parse_f32(p).unwrap_or(1.0);
    Ok((r, g, b, a))
}

fn parse_rest_of_line(p: &mut Parser<'_>) -> Result<String, ParseError> {
    let start = p.offset;
    while !p.is_eof() && !p.match_char(b'\n') {
        p.advance();
    }
    let s = std::str::from_utf8(&p.data[start..p.offset])
        .map_err(|e| ParseError::Io(e.to_string()))?
        .to_string();
    Ok(s)
}

fn skip_rest_of_line(p: &mut Parser<'_>) {
    while !p.is_eof() && !p.match_char(b'\n') {
        p.advance();
    }
}

trait IsAsciiDigit {
    fn is_ascii_digit(self) -> bool;
}
impl IsAsciiDigit for u8 {
    fn is_ascii_digit(self) -> bool {
        (b'0'..=b'9').contains(&self)
    }
}

trait IsAsciiAlphanumeric {
    fn is_ascii_alphanumeric(self) -> bool;
}
impl IsAsciiAlphanumeric for u8 {
    fn is_ascii_alphanumeric(self) -> bool {
        (b'0'..=b'9').contains(&self)
            || (b'a'..=b'z').contains(&self)
            || (b'A'..=b'Z').contains(&self)
    }
}
