//! Error types for the render crate.

use std::fmt;

/// Errors that can occur during rendering or engine setup.
#[derive(Debug)]
pub enum RenderError {
    /// wgpu initialization failed.
    WgpuInit(String),
    /// Surface was lost or outdated (e.g. resize).
    SurfaceLost,
    /// Material loading failed (e.g. invalid image file).
    #[cfg(feature = "material")]
    MaterialLoad(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WgpuInit(msg) => write!(f, "wgpu init failed: {msg}"),
            Self::SurfaceLost => write!(f, "surface lost or outdated"),
            #[cfg(feature = "material")]
            Self::MaterialLoad(msg) => write!(f, "material load failed: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}
