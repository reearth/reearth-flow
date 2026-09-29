use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Geometry Interpretation
/// How strictly written coordinates are interpreted when building geometry.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GeometryInterpretation {
    /// # Lenient
    /// Reads each triangle of a `gml:TriangulatedSurface` or `gml:Tin` from its first three
    /// positions and the surface as one triangle mesh.
    #[default]
    Lenient,
    /// # Strict
    /// Reads each triangle of a `gml:TriangulatedSurface` or `gml:Tin` as a polygon whose ring
    /// keeps its positions exactly as written, so a triangle with other than four positions or an
    /// unclosed ring is kept for checking. Only triangle patches are affected for now.
    Strict,
}
