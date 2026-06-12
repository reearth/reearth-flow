//! `LineString` leaf (2D-embedded shown here).

use crate::new_geom::Coordinate;

pub mod bounding_box;
pub mod reproject;
pub mod write_gltf;

#[derive(Clone, Debug)]
pub struct LineString2D {
    pub pts: Vec<[f64; 2]>,
    pub coord: Coordinate,
}

// Supports bounding_box + reproject; no direct glTF path in this reference set.
crate::unsupported!(LineString2D: WriteGltf);
