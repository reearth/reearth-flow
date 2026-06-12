//! `Solid`: a composite leaf. It holds coordless [`RawMesh`](crate::new_geom::raw::RawMesh)
//! shells and exactly one [`Coordinate`](crate::new_geom::Coordinate). The shells
//! never carry their own frame, so a solid can never hold disagreeing coords.

use crate::new_geom::raw::RawMesh;
use crate::new_geom::Coordinate;

pub mod bounding_box;
pub mod reproject;
pub mod write_gltf;

#[derive(Clone, Debug)]
pub struct Solid {
    pub exterior: RawMesh,
    pub interiors: Vec<RawMesh>,
    pub coord: Coordinate,
}
