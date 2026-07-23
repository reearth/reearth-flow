//! Lossless intermediate-data encoding for the polygon leaves.
//!
//! The wire form presents the rings decoded: an explicit exterior ring and a list
//! of interior rings, rather than the stored flat `coords` buffer plus the
//! `interior_offsets` that slice it. Elevation stays a flat buffer parallel to the
//! ring concatenation. Decoding rebuilds `interior_offsets` from the ring lengths.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;

use super::{Polygon2D, Polygon3D};

/// Decoded wire form of a [`Polygon2D`].
#[derive(Serialize, Deserialize)]
struct Polygon2DWire {
    frame: CoordinateFrame,
    exterior: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interiors: Vec<Vec<[f64; 2]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
}

/// Decoded wire form of a [`Polygon3D`].
#[derive(Serialize, Deserialize)]
struct Polygon3DWire {
    frame: CoordinateFrame,
    exterior: Vec<[f64; 3]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interiors: Vec<Vec<[f64; 3]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
}

/// Concatenate the exterior and interior rings and record where each interior ring
/// starts in the combined buffer.
fn flatten_rings<const N: usize>(
    exterior: Vec<[f64; N]>,
    interiors: Vec<Vec<[f64; N]>>,
) -> (Box<[[f64; N]]>, Box<[u32]>) {
    let mut coords = exterior;
    let mut interior_offsets = Vec::with_capacity(interiors.len());
    for ring in interiors {
        interior_offsets.push(coords.len() as u32);
        coords.extend(ring);
    }
    (
        coords.into_boxed_slice(),
        interior_offsets.into_boxed_slice(),
    )
}

impl From<&Polygon2D> for Polygon2DWire {
    fn from(p: &Polygon2D) -> Self {
        Polygon2DWire {
            frame: p.frame.clone(),
            exterior: p.exterior().to_vec(),
            interiors: p.interiors().map(|r| r.to_vec()).collect(),
            z: p.z.as_ref().map(|z| z.to_vec()),
            appearance: p.appearance.clone(),
        }
    }
}

impl TryFrom<Polygon2DWire> for Polygon2D {
    type Error = crate::error::Error;

    fn try_from(w: Polygon2DWire) -> Result<Self, Self::Error> {
        let (coords, interior_offsets) = flatten_rings(w.exterior, w.interiors);
        let z = w.z.map(Vec::into_boxed_slice);
        let mut polygon = Polygon2D::from_raw_parts(w.frame, coords, interior_offsets, z)?;
        polygon.appearance = w.appearance;
        Ok(polygon)
    }
}

impl From<&Polygon3D> for Polygon3DWire {
    fn from(p: &Polygon3D) -> Self {
        Polygon3DWire {
            frame: p.frame.clone(),
            exterior: p.exterior().to_vec(),
            interiors: p.interiors().map(|r| r.to_vec()).collect(),
            appearance: p.appearance.clone(),
        }
    }
}

impl TryFrom<Polygon3DWire> for Polygon3D {
    type Error = crate::error::Error;

    fn try_from(w: Polygon3DWire) -> Result<Self, Self::Error> {
        let (coords, interior_offsets) = flatten_rings(w.exterior, w.interiors);
        let mut polygon = Polygon3D::from_raw_parts(w.frame, coords, interior_offsets)?;
        polygon.appearance = w.appearance;
        Ok(polygon)
    }
}

impl Serialize for Polygon2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Polygon2DWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Polygon2D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Polygon2D::try_from(Polygon2DWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Polygon3D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Polygon3DWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Polygon3D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Polygon3D::try_from(Polygon3DWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip_2d(p: &Polygon2D) {
        let json = serde_json::to_string(p).unwrap();
        let back: Polygon2D = serde_json::from_str(&json).unwrap();
        assert_eq!(p, &back);
    }

    fn round_trip_3d(p: &Polygon3D) {
        let json = serde_json::to_string(p).unwrap();
        let back: Polygon3D = serde_json::from_str(&json).unwrap();
        assert_eq!(p, &back);
    }

    #[test]
    fn polygon2d_with_hole_and_elevation_round_trips() {
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];
        round_trip_2d(&Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            vec![hole],
        ));

        let elev = [
            [0.0, 0.0, 1.0],
            [4.0, 0.0, 2.0],
            [4.0, 4.0, 3.0],
            [0.0, 0.0, 1.0],
        ];
        round_trip_2d(&Polygon2D::from_rings_with_elevation(
            CoordinateFrame::Euclidean,
            elev,
            Vec::<Vec<[f64; 3]>>::new(),
        ));
    }

    #[test]
    fn polygon3d_with_hole_round_trips() {
        let outer = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        round_trip_3d(&Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            outer,
            vec![hole],
        ));
    }
}
