use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Only the triangle patches of gml:TriangulatedSurface and gml:Tin are affected for now.
/// # Geometry Interpretation
/// How strictly written coordinates are interpreted when building geometry.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GeometryInterpretation {
    /// # Lenient
    /// Builds each geometry in the form its type expects, using only the positions needed for
    /// that form, so slightly malformed input still yields usable geometry.
    #[default]
    Lenient,
    /// # Strict
    /// Keeps positions exactly as written, so malformed input such as a wrong vertex count or an
    /// unclosed ring stays visible to a later check.
    Strict,
}
