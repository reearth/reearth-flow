//! New-geometry-era glTF/GLB writing, feature-gated behind `new-geometry`.
//! `glb` is a schema-agnostic gltf-rs wrapper (buffer/accessor bookkeeping,
//! generic per-vertex-attribute and extension attachment); `metadata` builds
//! a per-tile property table and encodes it as Cesium's
//! `EXT_structural_metadata`/`EXT_mesh_features` extensions on top of `glb`;
//! `draco` optionally re-compresses a finished GLB with Draco.
//! Callers compose these directly — see
//! `reearth_flow_action_sink`'s Cesium 3D Tiles writer.

pub mod draco;
pub mod glb;
pub mod metadata;
