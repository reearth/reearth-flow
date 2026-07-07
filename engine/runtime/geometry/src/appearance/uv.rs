//! UV sets and their sources.
//!
//! UV is geometric: its coordinates run parallel to the geometry's vertex /
//! corner buffer, not the material. One UV set feeds several maps, and 
//! a material map references a UV set.
//!
//! A map's UV is resolved by three coordinates: the theme is the
//! [`ThemeBinding`](super::ThemeBinding) that owns this set, the side comes from
//! the per-face binding (face under theme T, on side S -> material), and the
//! channel from the material map's selector. So `channel` carries no global
//! meaning: it is a material-local index into the UV sets of whatever theme/side
//! the face resolves to.

use serde::{Deserialize, Serialize};

use super::{ChannelId, Side};

/// One UV set, owned by the [`ThemeBinding`](super::ThemeBinding) whose theme it
/// belongs to. The `Explicit` array is parallel to the host geometry's corner
/// buffer; the alignment is fixed by the geometry type, not tagged here.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UvSet {
    /// Which surface side these coordinates parameterise. `Front` for the
    /// common single-sided case.
    pub side: Side,
    /// Material-local UV channel a textured map samples (the glTF `texCoord`
    /// index); `ChannelId(0)` in the common single-map case. A `(theme, side)`
    /// holds one UV set per distinct channel its materials reference.
    pub channel: ChannelId,
    pub uv: UvSource,
}

/// Either explicit per-corner coordinates, or a retained world-to-texture
/// matrix.
///
/// An affine georeferenced map is baked to `Explicit` on read; a projective
/// world-to-texture matrix is retained, and collapsed to `Explicit` at any
/// non-affine operation or per-vertex-only sink.
///
/// There is deliberately **no per-vertex variant**. A surface's UV is assumed
/// *affine* in position (a photographic / orthophoto projection onto a planar
/// face), so it reduces to one `WorldToTexture` matrix — and `Explicit` per-corner
/// UV is an exact sampling of that matrix, recoverable from 3 `(position, UV)`
/// pairs. So triangulating a face (earcut / spade) carries the single matrix
/// instead of a per-vertex array: a Delaunay re-ordering is irrelevant (UV is
/// positional) and inserted (Steiner) vertices get exact UV by evaluating it.
/// `Explicit` ⊕ `WorldToTexture` therefore suffices; a *welded multi-face* mesh,
/// whose faces carry different matrices, bakes them to per-corner `Explicit`.
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
