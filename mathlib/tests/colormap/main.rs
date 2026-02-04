//! Integration tests for colormap: Rgb, Hsv, conversions, height_to_rgb, hex.

use mathlib::{
    Rgb, Rgba, height_to_rgb, height_to_rgba, hex_to_rgb, hsv_to_rgb, rgb_to_hex, rgb_to_hsv,
    rgb_to_rgba, rgba_to_rgb,
};

#[test]
fn colormap_rgb_to_hsv_roundtrip() {
    let rgb: Rgb = [100, 150, 200];
    let hsv = rgb_to_hsv(rgb[0], rgb[1], rgb[2]);
    let back = hsv_to_rgb(hsv.h, hsv.s, hsv.v);
    assert_eq!(back, rgb);
}

#[test]
fn colormap_hsv_type() {
    let hsv = rgb_to_hsv(255, 0, 0);
    assert!(hsv.h >= 0.0 && hsv.h <= 360.0);
    assert!(hsv.s >= 0.0 && hsv.s <= 1.0);
    assert!(hsv.v >= 0.0 && hsv.v <= 1.0);
}

#[test]
fn colormap_height_to_rgb_bounds() {
    let low = height_to_rgb(0.0);
    assert_eq!(low, [0, 0, 255]);
    let high = height_to_rgb(1.0);
    assert_eq!(high, [255, 255, 0]);
    let mid = height_to_rgb(0.5);
    assert_eq!(mid[1], 255);
}

#[test]
fn colormap_height_to_rgba_has_alpha() {
    let rgba = height_to_rgba(0.5);
    assert_eq!(rgba.len(), 4);
    assert_eq!(rgba[3], 255);
}

#[test]
fn colormap_rgb_to_hex_roundtrip() {
    let rgb: Rgb = [10, 20, 30];
    let hex = rgb_to_hex(rgb);
    assert_eq!(hex_to_rgb(&hex), Some(rgb));
}

#[test]
fn colormap_rgba_rgb_convert() {
    let rgba: Rgba = [1, 2, 3, 255];
    let rgb = rgba_to_rgb(rgba);
    assert_eq!(rgb, [1, 2, 3]);
    let back = rgb_to_rgba(rgb, 128);
    assert_eq!(back[0], 1);
    assert_eq!(back[1], 2);
    assert_eq!(back[2], 3);
    assert_eq!(back[3], 128);
}

#[test]
fn colormap_black_white_hsv() {
    let hsv_black = rgb_to_hsv(0, 0, 0);
    assert!(hsv_black.v < 1e-6);
    let hsv_white = rgb_to_hsv(255, 255, 255);
    assert!((hsv_white.v - 1.0).abs() < 1e-6);
    assert!(hsv_white.s < 1e-6);
}
