//! Lossless intermediate-data encoding for the polygon leaves.
//!
//! The wire form presents the rings decoded: an explicit exterior ring and a list
//! of interior rings, rather than the stored flat `coords` buffer plus the
//! `interior_offsets` that slice it. Elevation stays the one number the whole face
//! lies at. Decoding rebuilds `interior_offsets` from the ring lengths.
//!
//! Per-corner UV is nested the same way, so it mirrors the exterior and interior
//! rings rather than the flat corner buffer they concatenate into.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::feature_write::{
    decode_appearance, encode_appearance, AppearanceWire, FaceRings,
};
use crate::coordinate::CoordinateFrame;

use super::{Polygon2D, Polygon3D};

/// The single face a polygon presents: its exterior ring, then its interiors.
fn polygon_layout(exterior: usize, interiors: impl Iterator<Item = usize>) -> Vec<FaceRings> {
    vec![FaceRings {
        exterior,
        holes: interiors.collect(),
    }]
}

/// Decoded wire form of a [`Polygon2D`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct Polygon2DWire {
    frame: CoordinateFrame,
    exterior: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interiors: Vec<Vec<[f64; 2]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<AppearanceWire>,
}

/// Decoded wire form of a [`Polygon3D`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct Polygon3DWire {
    frame: CoordinateFrame,
    exterior: Vec<[f64; 3]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interiors: Vec<Vec<[f64; 3]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<AppearanceWire>,
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

impl TryFrom<&Polygon2D> for Polygon2DWire {
    type Error = crate::error::Error;

    fn try_from(p: &Polygon2D) -> Result<Self, Self::Error> {
        let exterior = p.exterior().to_vec();
        let interiors: Vec<Vec<[f64; 2]>> = p.interiors().map(|r| r.to_vec()).collect();
        let layout = polygon_layout(exterior.len(), interiors.iter().map(Vec::len));
        Ok(Polygon2DWire {
            frame: p.frame.clone(),
            appearance: encode_appearance(&p.appearance, &layout)?,
            exterior,
            interiors,
            z: p.elevation(),
        })
    }
}

impl TryFrom<Polygon2DWire> for Polygon2D {
    type Error = crate::error::Error;

    fn try_from(w: Polygon2DWire) -> Result<Self, Self::Error> {
        let layout = polygon_layout(w.exterior.len(), w.interiors.iter().map(Vec::len));
        let appearance = decode_appearance(w.appearance, &layout)?;
        let (coords, interior_offsets) = flatten_rings(w.exterior, w.interiors);
        let mut polygon = Polygon2D::from_raw_parts(w.frame, coords, interior_offsets, w.z)?;
        polygon.appearance = appearance;
        Ok(polygon)
    }
}

impl TryFrom<&Polygon3D> for Polygon3DWire {
    type Error = crate::error::Error;

    fn try_from(p: &Polygon3D) -> Result<Self, Self::Error> {
        let exterior = p.exterior().to_vec();
        let interiors: Vec<Vec<[f64; 3]>> = p.interiors().map(|r| r.to_vec()).collect();
        let layout = polygon_layout(exterior.len(), interiors.iter().map(Vec::len));
        Ok(Polygon3DWire {
            frame: p.frame.clone(),
            appearance: encode_appearance(&p.appearance, &layout)?,
            exterior,
            interiors,
        })
    }
}

impl TryFrom<Polygon3DWire> for Polygon3D {
    type Error = crate::error::Error;

    fn try_from(w: Polygon3DWire) -> Result<Self, Self::Error> {
        let layout = polygon_layout(w.exterior.len(), w.interiors.iter().map(Vec::len));
        let appearance = decode_appearance(w.appearance, &layout)?;
        let (coords, interior_offsets) = flatten_rings(w.exterior, w.interiors);
        let mut polygon = Polygon3D::from_raw_parts(w.frame, coords, interior_offsets)?;
        polygon.appearance = appearance;
        Ok(polygon)
    }
}

impl Serialize for Polygon2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Polygon2DWire::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
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
        Polygon3DWire::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Polygon3D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Polygon3D::try_from(Polygon3DWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

// The intermediate-data schema is the wire form, so each leaf's schema is its
// wire struct's.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for Polygon2D {
    fn schema_name() -> String {
        "Polygon2D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <Polygon2DWire as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Polygon3D {
    fn schema_name() -> String {
        "Polygon3D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <Polygon3DWire as schemars::JsonSchema>::json_schema(generator)
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
            vec![hole.clone()],
        ));

        let lifted = Polygon2D::from_rings_at_elevation(
            CoordinateFrame::Euclidean,
            square,
            vec![hole],
            10.0,
        );
        round_trip_2d(&lifted);

        // The face lies at one height, so the wire form carries one number for it,
        // not a value per corner.
        let json = serde_json::to_value(&lifted).unwrap();
        assert_eq!(json["z"], serde_json::json!(10.0));
    }

    #[test]
    fn polygon2d_uv_nests_to_mirror_rings() {
        use crate::test_support::{explicit_uv, textured, theme};

        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];
        let mut p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square, vec![hole]);
        // Distinct values, so the flattening order is observable.
        let corners: Vec<[f64; 2]> = (0..10).map(|i| [i as f64, 0.0]).collect();
        p.set_appearance(theme("rgb"), textured(), Some(explicit_uv(&corners)))
            .unwrap();

        let json = serde_json::to_value(&p).unwrap();
        let nested = &json["appearance"]["themes"][0]["uv_sets"][0]["uv"]["Explicit"];
        assert_eq!(nested.as_array().unwrap().len(), 1, "a polygon is one face");
        assert_eq!(
            nested[0]["exterior"],
            serde_json::json!([[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], [4.0, 0.0]])
        );
        assert_eq!(
            nested[0]["holes"][0],
            serde_json::json!([[5.0, 0.0], [6.0, 0.0], [7.0, 0.0], [8.0, 0.0], [9.0, 0.0]])
        );

        round_trip_2d(&p);
    }

    #[test]
    fn polygon2d_rejects_uv_ring_length_mismatch() {
        // The exterior ring has four corners; its UV ring offers two.
        let json = r#"{
            "frame": "Euclidean",
            "exterior": [[0.0,0.0],[4.0,0.0],[4.0,4.0],[0.0,0.0]],
            "appearance": {
                "materials": [],
                "themes": [{
                    "theme": "rgb",
                    "front": {"Uniform": 0},
                    "uv_sets": [{
                        "side": "Front",
                        "channel": 0,
                        "uv": {"Explicit": [{"exterior": [[0.0,0.0],[1.0,0.0]]}]}
                    }]
                }],
                "default_theme": "rgb"
            }
        }"#;
        let err = serde_json::from_str::<Polygon2D>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("UV exterior ring"), "unexpected error: {err}");
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
