//! `GeometryCollection`: heterogeneous, cross-dimensional. Its payload is
//! `Vec<Geometry>`, which `enum_dispatch` cannot traverse, so the operation
//! impls here are hand-written and recurse over the children. Wrapping the
//! `Vec` in this concrete struct is what lets the collection appear as a clean
//! newtype variant of the top [`Geometry`](crate::new_geom::Geometry) enum.

use crate::new_geom::Geometry;

pub mod bounding_box;
pub mod reproject;
pub mod write_gltf;

#[derive(Clone, Debug)]
pub struct GeometryCollection(pub Vec<Geometry>);
