//! Point leaves.

use serde::{Deserialize, Serialize};

use super::coordinate::CoordinateFrame;

mod constructor;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// A single position in 2D space.
/// Used for CityGML `gml:Point` and 2D point features.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Point (2D)"))]
pub struct Point2D {
    /// Coordinate frame this position is expressed in.
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Position"))]
    position: [f64; 2],
}

/// A single position in 3D space.
/// Used for CityGML `gml:Point`, OBJ vertices, and 3D point features.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Point (3D)"))]
pub struct Point3D {
    /// Coordinate frame this position is expressed in.
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Position"))]
    position: [f64; 3],
}

impl Point2D {
    /// The coordinate frame this position is expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The `[x, y]` position.
    #[inline]
    pub fn position(&self) -> [f64; 2] {
        self.position
    }
}

impl Point3D {
    /// The coordinate frame this position is expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The `[x, y, z]` position.
    #[inline]
    pub fn position(&self) -> [f64; 3] {
        self.position
    }
}

crate::unsupported!(Point2D: Triangulate);
crate::unsupported!(Point3D: Triangulate);

crate::unsupported!(Point2D: Split);
crate::unsupported!(Point3D: Split);

crate::unsupported!(Point2D: RemoveAppearance);
crate::unsupported!(Point3D: RemoveAppearance);

crate::unsupported!(Point2D: CountHoles);
crate::unsupported!(Point3D: CountHoles);

// A point bounds no area, so there is nothing to take apart.
crate::unsupported!(Point2D: ExtractHoles);
crate::unsupported!(Point3D: ExtractHoles);

// A single position is none of the coercion targets, and has no vertices to
// re-arrange into one.
crate::unsupported!(Point2D: Coerce);
crate::unsupported!(Point3D: Coerce);

// A position has no extent, so it reports the empty boundary rather than
// refusing the operation.
impl crate::ops::ExtractBoundary for Point2D {
    fn extract_boundary(&self) -> Result<crate::ops::Boundary, crate::ops::UnsupportedOperation> {
        Ok(crate::ops::Boundary::EMPTY)
    }
}

impl crate::ops::ExtractBoundary for Point3D {
    fn extract_boundary(&self) -> Result<crate::ops::Boundary, crate::ops::UnsupportedOperation> {
        Ok(crate::ops::Boundary::EMPTY)
    }
}
