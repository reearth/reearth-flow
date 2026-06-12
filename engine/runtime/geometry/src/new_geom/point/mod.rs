//! `Point` leaves for both embeddings.

use crate::new_geom::Coordinate;

pub mod bounding_box;
pub mod reproject;
pub mod write_gltf;

/// 2D-embedded point with an optional per-vertex elevation (§3.1.1 convention:
/// `None` costs no storage).
#[derive(Clone, Debug)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub coord: Coordinate,
}

/// 3D-embedded point.
#[derive(Clone, Debug)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub coord: Coordinate,
}

// Point3D supports neither of the edit/export ops in this reference set.
crate::unsupported!(Point3D: Reproject, WriteGltf);
