//! Build script: compiles dimension.proto to Rust via tonic-prost-build.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc_path =
        protoc_bin_vendored::protoc_bin_path().expect("protoc binary not found for this platform");
    std::env::set_var("PROTOC", protoc_path);

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/dimension.proto"], &["proto/"])?;
    Ok(())
}
