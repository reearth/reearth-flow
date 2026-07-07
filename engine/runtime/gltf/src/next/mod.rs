//! New-geometry-era glTF/GLB writing, feature-gated behind `new-geometry`.
//!
//! Deliberately independent of the parent modules' `nusamai-gltf`-typed
//! model (`writer.rs`, `metadata.rs`): this writes glTF's JSON document and
//! GLB container directly, since the new-geometry migration is dropping the
//! `nusamai-*` crates project-wide, and the parent modules' API surface is
//! `nusamai_gltf` types end to end. Mirrors the `next/` convention already
//! used by `action-sink`'s Cesium 3D Tiles sink (`cesium3dtiles/next/`),
//! which is this module's first (and so far only) caller.
//!
//! Intended to grow as more new-geometry sinks need glTF output, and to
//! eventually replace the parent modules outright once the migration
//! finishes — at which point this directory is promoted up and the
//! `nusamai-gltf`-based code above it is deleted.

pub mod glb;
pub mod metadata;
