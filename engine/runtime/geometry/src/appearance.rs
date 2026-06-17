//! Surface appearance: materials, textures and per-face bindings.
//!
//! This module is intentionally a placeholder for now: the full materials /
//! textures / UV-set graph is implemented in a later step. Only the empty
//! shells that the geometry leaves reference exist here, so the leaf types can
//! already carry their `appearance` and `uv_sets` fields with the right shape
//! (`Option<Appearance>`, `Vec<UvSet>`).

use serde::{Deserialize, Serialize};

/// Materials, themes and per-face bindings hung off a surface geometry.
///
/// TODO(new-geometry): replace this empty shell with the real palette
/// (materials, per-theme bindings, default theme).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Appearance {}

/// One UV set on a mesh leaf: geometric texture coordinates parallel to the
/// host geometry's corner buffer.
///
/// TODO(new-geometry): replace this empty shell with the real fields (theme,
/// material-local channel, and the UV source).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UvSet {}
