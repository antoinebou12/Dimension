//! BVH motion capture parser (gti320-style).
//!
//! Case-insensitive keywords: HIERARCHY, ROOT, JOINT, End Site, OFFSET, CHANNELS,
//! MOTION, Frames, Frame Time.

use crate::error::ParseError;
use crate::parser::Parser;
use mathlib::Vector3f;

/// BVH channel type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BvhChannel {
    /// X position.
    XPosition,
    /// Y position.
    YPosition,
    /// Z position.
    ZPosition,
    /// X rotation.
    XRotation,
    /// Y rotation.
    YRotation,
    /// Z rotation.
    ZRotation,
}

/// BVH joint.
#[derive(Clone, Debug)]
pub struct BvhJoint {
    /// Parent index (-1 for root).
    pub parent: i32,
    /// Joint name.
    pub name: String,
    /// Offset from parent.
    pub offset: Vector3f,
    /// Channels.
    pub channels: Vec<BvhChannel>,
    /// True if End Site.
    pub end_site: bool,
}

/// BVH data: hierarchy and motion.
#[derive(Clone, Debug)]
pub struct BvhData {
    /// Joint hierarchy.
    pub joints: Vec<BvhJoint>,
    /// Frame count.
    pub frame_count: i32,
    /// Frame time (seconds per frame).
    pub frame_time: f32,
    /// Motion data: flat [frame_count * channel_count] floats.
    pub motion_data: Vec<f32>,
}

/// Parses BVH from bytes.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid BVH.
pub fn parse(data: &[u8]) -> Result<BvhData, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    parse_str(s)
}

/// Parses BVH from string.
pub fn parse_str(s: &str) -> Result<BvhData, ParseError> {
    let mut p = Parser::new(s.as_bytes(), None);

    p.require_string_caseless("HIERARCHY")?;
    skip_whitespace_including_newlines(&mut p);
    p.require_string_caseless("ROOT")?;
    p.parse_whitespace();

    let mut joints = Vec::new();
    parse_joint(&mut p, -1, &mut joints)?;

    skip_whitespace_including_newlines(&mut p);
    p.require_string_caseless("MOTION")?;
    skip_whitespace_including_newlines(&mut p);

    p.require_string_caseless("Frames:")?;
    skip_whitespace_including_newlines(&mut p);
    let frame_count = parse_i32(&mut p).ok_or_else(|| p.syntax_err("expected frame count"))?;
    skip_whitespace_including_newlines(&mut p);

    p.require_string_caseless("Frame")?;
    skip_whitespace_including_newlines(&mut p);
    p.require_string_caseless("Time:")?;
    skip_whitespace_including_newlines(&mut p);
    let mut frame_time = parse_f32(&mut p).unwrap_or(1.0 / 60.0);
    if frame_time <= 0.0 {
        frame_time = 1.0 / 60.0;
    }

    let channel_count: usize = joints
        .iter()
        .filter(|j| !j.end_site)
        .map(|j| j.channels.len())
        .sum();

    let mut motion_data = Vec::with_capacity((frame_count as usize) * channel_count);

    for _ in 0..frame_count {
        skip_whitespace_including_newlines(&mut p);
        for _ in 0..channel_count {
            let v = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected motion float"))?;
            motion_data.push(v);
            skip_whitespace_including_newlines(&mut p);
        }
    }

    Ok(BvhData {
        joints,
        frame_count,
        frame_time,
        motion_data,
    })
}

fn skip_whitespace_including_newlines(p: &mut Parser<'_>) {
    while p.one_of(b" \t\r\n\x0b") {
        p.advance();
    }
}

fn parse_end_site(
    p: &mut Parser<'_>,
    parent: i32,
    joints: &mut Vec<BvhJoint>,
) -> Result<(), ParseError> {
    parse_joint_inner(p, parent, joints, "End Site".to_string(), true)
}

fn parse_joint(
    p: &mut Parser<'_>,
    parent: i32,
    joints: &mut Vec<BvhJoint>,
) -> Result<(), ParseError> {
    let (name, end_site) = (parse_joint_name(p)?, false);
    parse_joint_inner(p, parent, joints, name, end_site)
}

fn parse_joint_inner(
    p: &mut Parser<'_>,
    parent: i32,
    joints: &mut Vec<BvhJoint>,
    name: String,
    end_site: bool,
) -> Result<(), ParseError> {
    skip_whitespace_including_newlines(p);
    if p.peek() != b'{' {
        return Err(p.syntax_err("expected '{'"));
    }
    p.advance();

    skip_whitespace_including_newlines(p);
    p.require_string_caseless("OFFSET")?;
    p.parse_whitespace();
    let x = parse_f32(p).ok_or_else(|| p.syntax_err("expected offset x"))?;
    p.parse_whitespace();
    let y = parse_f32(p).ok_or_else(|| p.syntax_err("expected offset y"))?;
    p.parse_whitespace();
    let z = parse_f32(p).ok_or_else(|| p.syntax_err("expected offset z"))?;
    let mut offset = Vector3f::with_capacity(3);
    offset.set(0, x);
    offset.set(1, y);
    offset.set(2, z);

    let channels = if end_site {
        Vec::new()
    } else {
        skip_whitespace_including_newlines(p);
        p.require_string_caseless("CHANNELS")?;
        p.parse_whitespace();
        let n = parse_i32(p).ok_or_else(|| p.syntax_err("expected channel count"))?;
        let mut ch = Vec::with_capacity(n as usize);
        for _ in 0..n {
            p.parse_whitespace();
            let tok = parse_token(p)?;
            let c = match tok.to_lowercase().as_str() {
                "xposition" => BvhChannel::XPosition,
                "yposition" => BvhChannel::YPosition,
                "zposition" => BvhChannel::ZPosition,
                "xrotation" => BvhChannel::XRotation,
                "yrotation" => BvhChannel::YRotation,
                "zrotation" => BvhChannel::ZRotation,
                _ => return Err(p.syntax_err(format!("unknown channel {}", tok))),
            };
            ch.push(c);
        }
        ch
    };

    let idx = joints.len() as i32;
    joints.push(BvhJoint {
        parent,
        name,
        offset,
        channels,
        end_site,
    });

    skip_whitespace_including_newlines(p);
    while p.peek() != b'}' && !p.is_eof() {
        if p.expect_string_caseless("JOINT")? {
            skip_whitespace_including_newlines(p);
            parse_joint(p, idx, joints)?;
        } else if p.expect_string_caseless("End")? {
            skip_whitespace_including_newlines(p);
            p.require_string_caseless("Site")?;
            parse_end_site(p, idx, joints)?;
        } else {
            return Err(p.syntax_err(format!(
                "expected JOINT or End Site at '{}'",
                p.peek() as char
            )));
        }
        skip_whitespace_including_newlines(p);
    }

    if p.peek() != b'}' {
        return Err(p.syntax_err("expected '}'"));
    }
    p.advance();
    Ok(())
}

fn parse_joint_name(p: &mut Parser<'_>) -> Result<String, ParseError> {
    let mut s = String::new();
    while !p.is_eof()
        && (p.peek().is_ascii_alphanumeric()
            || p.peek() == b'_'
            || p.peek() == b':'
            || p.peek() == b'.')
    {
        s.push(p.peek() as char);
        p.advance();
    }
    if s.is_empty() {
        return Err(p.syntax_err("expected joint name"));
    }
    Ok(s)
}

fn parse_token(p: &mut Parser<'_>) -> Result<String, ParseError> {
    let mut s = String::new();
    while !p.is_eof() && !p.one_of(b" \t\n\r{") {
        s.push(p.peek() as char);
        p.advance();
    }
    if s.is_empty() {
        return Err(p.syntax_err("expected token"));
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
