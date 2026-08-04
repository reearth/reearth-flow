//! LineString leaves.
//!
//! A `LineString` is a polyline: an ordered chain of coordinates, a variant in
//! both embeddings. It follows the `Polygon` flat-buffer convention: a single
//! closed/open chain of coordinates in one `Box<[_]>` allocation, with the 2D
//! form carrying one optional elevation for the whole chain. Lines carry no
//! appearance.

use serde::{Deserialize, Serialize};

use crate::coordinate::CoordinateFrame;

mod constructor;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// A polyline in 2D space, lying at a single optional elevation (2.5D).
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Polyline (2D)"))]
pub struct LineString2D {
    /// Coordinate frame these coords are expressed in.
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Coordinates"))]
    coords: Box<[[f64; 2]]>,
    /// The one elevation the whole chain lies at. `None` = pure 2D.
    #[cfg_attr(feature = "schema", schemars(title = "Elevation"))]
    z: Option<f64>,
}

/// A polyline in 3D space.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Polyline (3D)"))]
pub struct LineString3D {
    /// Coordinate frame these coords are expressed in.
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Coordinates"))]
    coords: Box<[[f64; 3]]>,
}

impl LineString2D {
    /// The coordinate frame these coords are expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The chain's vertices in order.
    #[inline]
    pub fn coords(&self) -> &[[f64; 2]] {
        &self.coords
    }

    /// The elevation the chain lies at, or `None` when it is pure 2D.
    #[inline]
    pub fn elevation(&self) -> Option<f64> {
        self.z
    }
}

impl LineString3D {
    /// The coordinate frame these coords are expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The chain's vertices in order.
    #[inline]
    pub fn coords(&self) -> &[[f64; 3]] {
        &self.coords
    }
}

crate::unsupported!(LineString2D: Triangulate);
crate::unsupported!(LineString3D: Triangulate);

crate::unsupported!(LineString2D: Split);
crate::unsupported!(LineString3D: Split);

crate::unsupported!(LineString2D: RemoveAppearance);
crate::unsupported!(LineString3D: RemoveAppearance);

crate::unsupported!(LineString2D: CountHoles);
crate::unsupported!(LineString3D: CountHoles);

// A curve bounds no area, so there is nothing to take apart.
crate::unsupported!(LineString2D: ExtractHoles);
crate::unsupported!(LineString3D: ExtractHoles);
