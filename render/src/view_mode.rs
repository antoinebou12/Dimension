//! View mode for scene rendering: solid, wireframe, vertex points, or color-map style.

/// How scene geometry is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ViewMode {
    /// Filled triangles (default).
    #[default]
    Solid,
    /// Polygon edges only (wireframe). Uses same vertex data; pipeline uses `PolygonMode::Line`.
    /// Not all wgpu backends support line mode; may fall back to solid on unsupported.
    Wireframe,
    /// Vertices as points. Uses same vertex data; pipeline uses point list topology.
    VertexPoints,
    /// Vertex colors as-is (current default shading). Reserved for future scalar/attribute visualization.
    ColorMap,
    /// Normals visualization: colors by view-space direction (normalize(view_pos)); RGB = (dir+1)/2.
    Normals,
}
