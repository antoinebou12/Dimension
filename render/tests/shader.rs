//! Shader validation tests: parse WGSL, validate, and verify entry points. No GPU required.
//!
//! Uses naga (wgpu's shader frontend; developed in [gfx-rs/wgpu]) to validate shaders in CI
//! without a GPU. Version-aligned with wgpu (naga 28 = wgpu 28).

#![cfg(not(target_arch = "wasm32"))]

const SCENE_WGSL: &str = include_str!("../shaders/scene.wgsl");
const UI_WGSL: &str = include_str!("../shaders/ui.wgsl");

fn parse_and_validate(source: &str) -> Result<naga::Module, String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| format!("parse: {e}"))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| format!("validate: {e}"))?;
    Ok(module)
}

fn assert_vertex_fragment_entry_points(module: &naga::Module, name: &str) {
    assert!(
        module.entry_points.iter().any(|ep| ep.name == "vs_main"),
        "{name} should have vs_main vertex entry point"
    );
    assert!(
        module.entry_points.iter().any(|ep| ep.name == "fs_main"),
        "{name} should have fs_main fragment entry point"
    );
}

#[test]
fn scene_shader_parses_and_has_entry_points() {
    let module = parse_and_validate(SCENE_WGSL).expect("scene shader should parse and validate");
    assert_vertex_fragment_entry_points(&module, "scene shader");
}

#[test]
fn ui_shader_parses_and_has_entry_points() {
    let module = parse_and_validate(UI_WGSL).expect("ui shader should parse and validate");
    assert_vertex_fragment_entry_points(&module, "ui shader");
}
