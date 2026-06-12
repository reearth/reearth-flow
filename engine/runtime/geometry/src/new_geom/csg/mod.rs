//! `Csg`: a composite leaf wrapping a coordless boolean tree
//! ([`CsgNode`](crate::new_geom::raw::CsgNode)) plus one
//! [`Coordinate`](crate::new_geom::Coordinate).

use crate::new_geom::raw::CsgNode;
use crate::new_geom::Coordinate;

pub mod bounding_box;
pub mod reproject;
pub mod write_gltf;

#[derive(Clone, Debug)]
pub struct Csg {
    pub root: CsgNode,
    pub coord: Coordinate,
}

// CSG has no direct edit/export path in this reference set: it must be
// evaluated to a mesh first.
crate::unsupported!(Csg: Reproject, WriteGltf);
