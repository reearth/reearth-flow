//! UV sets and their sources.
//!
//! UV is geometric: it lives on the mesh leaf, parallel to the vertex / corner
//! buffer, not on the material. One UV set feeds several maps (base-colour,
//! normal, occlusion ... all sample it), and a material map references a UV set,
//! never the reverse.
//!
//! A map's UV is resolved by three coordinates: the theme and side come from
//! the per-face binding (face under theme T, on side S -> material), the channel
//! from the material map's selector. So `channel` carries no global meaning: it
//! is a material-local index into the UV sets of whatever theme/side the face
//! resolves to.

use serde::{Deserialize, Serialize};

use super::{ChannelId, Side, ThemeId};

/// One UV set on a mesh leaf. The `Explicit` array is parallel to the host
/// geometry's corner buffer; the alignment is fixed by the geometry type, not
/// tagged here.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UvSet {
    /// Styling variant this UV belongs to; `None` = a single implicit theme.
    pub theme: Option<ThemeId>,
    /// Which surface side these coordinates parameterise. `Front` for the
    /// common single-sided case.
    pub side: Side,
    /// Material-local UV channel; `None` when a theme makes no channel
    /// distinction (then the theme/side holds exactly one UV set).
    pub channel: Option<ChannelId>,
    pub uv: UvSource,
}

/// Either explicit per-corner coordinates, or a retained world-to-texture
/// matrix.
///
/// An affine georeferenced map is baked to `Explicit` on read; a projective
/// world-to-texture matrix is retained, and collapsed to `Explicit` at any
/// non-affine operation or per-vertex-only sink.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UvSource {
    /// Flat UV array parallel to the host geometry's corner buffer.
    Explicit(Box<[[f64; 2]]>),
    /// A 3x4 world-to-texture matrix; projective (perspective divide).
    WorldToTexture(TexMatrix),
}

/// A 3x4 world-to-texture projective matrix. Maps a homogeneous world position
/// `(x, y, z, 1)` to `(s', t', q')`, with the texture coordinate recovered as
/// `(s, t) = (s'/q', t'/q')`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct TexMatrix(pub [[f64; 4]; 3]);
