use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Only the triangle patches of gml:TriangulatedSurface and gml:Tin are affected for now.
/// # Coordinate Handling
/// Whether written coordinates are normalized to the form each geometry type expects or
/// preserved as written.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CoordinateHandling {
    /// # Normalize
    /// Builds each geometry in the form its type expects, using only the positions needed for
    /// that form, so slightly malformed input still yields usable geometry.
    #[default]
    Normalize,
    /// # Preserve
    /// Keeps positions exactly as written, so malformed input such as a wrong vertex count or an
    /// unclosed ring stays visible to a later check.
    Preserve,
}
