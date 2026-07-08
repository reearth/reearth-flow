//! New-geometry-era glTF/GLB writing, feature-gated behind `new-geometry`.
//! `glb` is a schema-agnostic gltf-rs wrapper (buffer/accessor bookkeeping,
//! generic per-vertex-attribute and extension attachment); `metadata` builds
//! a per-tile property table and encodes it as Cesium's
//! `EXT_structural_metadata`/`EXT_mesh_features` extensions on top of `glb`.
//! Callers compose the two directly — see
//! `reearth_flow_action_sink`'s Cesium 3D Tiles writer.

pub mod glb;
pub mod metadata;
