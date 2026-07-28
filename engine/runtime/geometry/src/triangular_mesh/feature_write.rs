//! Lossless intermediate-data encoding for the triangular-mesh leaves.
//!
//! The wire form lists the triangles as explicit vertex-index triples, widened
//! from the stored index buffer. Decoding packs them back through `from_parts`,
//! whose index width is fixed by the vertex count, so the round trip is exact.
//!
//! Per-corner UV is nested to match, one three-corner entry per triangle, rather
//! than one flat buffer across the whole mesh.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::feature_write::{
    decode_appearance, encode_appearance, AppearanceWire, FaceRings,
};
use crate::coordinate::CoordinateFrame;

use super::{TriangularMesh2D, TriangularMesh3DData};

/// One face per triangle, each a single three-corner ring.
fn triangle_layout(triangles: usize) -> Vec<FaceRings> {
    (0..triangles).map(|_| FaceRings::simple(3)).collect()
}

/// Decoded wire form of a [`TriangularMesh2D`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct TriangularMesh2DWire {
    frame: CoordinateFrame,
    vertices: Vec<[f64; 2]>,
    triangles: Vec<[u32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<AppearanceWire>,
}

/// Decoded wire form of a [`TriangularMesh3DData`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct TriangularMesh3DDataWire {
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[u32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<AppearanceWire>,
}

impl TryFrom<&TriangularMesh2D> for TriangularMesh2DWire {
    type Error = crate::error::Error;

    fn try_from(m: &TriangularMesh2D) -> Result<Self, Self::Error> {
        let triangles: Vec<[u32; 3]> = m.triangles().collect();
        let layout = triangle_layout(triangles.len());
        Ok(TriangularMesh2DWire {
            frame: m.frame.clone(),
            vertices: m.vertices.clone(),
            appearance: encode_appearance(&m.appearance, &layout)?,
            triangles,
            z: m.elevation(),
        })
    }
}

impl TryFrom<TriangularMesh2DWire> for TriangularMesh2D {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh2DWire) -> Result<Self, Self::Error> {
        let appearance = decode_appearance(w.appearance, &triangle_layout(w.triangles.len()))?;
        let mut mesh =
            TriangularMesh2D::from_parts(w.frame, w.vertices, w.triangles.into_iter().flatten())?;
        mesh.z = w.z;
        mesh.appearance = appearance;
        Ok(mesh)
    }
}

impl TryFrom<&TriangularMesh3DData> for TriangularMesh3DDataWire {
    type Error = crate::error::Error;

    fn try_from(m: &TriangularMesh3DData) -> Result<Self, Self::Error> {
        let triangles: Vec<[u32; 3]> = m.triangles().collect();
        let layout = triangle_layout(triangles.len());
        Ok(TriangularMesh3DDataWire {
            vertices: m.vertices.clone(),
            appearance: encode_appearance(&m.appearance, &layout)?,
            triangles,
        })
    }
}

impl TryFrom<TriangularMesh3DDataWire> for TriangularMesh3DData {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh3DDataWire) -> Result<Self, Self::Error> {
        let appearance = decode_appearance(w.appearance, &triangle_layout(w.triangles.len()))?;
        let mut mesh =
            TriangularMesh3DData::from_parts(w.vertices, w.triangles.into_iter().flatten())?;
        mesh.appearance = appearance;
        Ok(mesh)
    }
}

impl Serialize for TriangularMesh2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangularMesh2DWire::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TriangularMesh2D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TriangularMesh2D::try_from(TriangularMesh2DWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for TriangularMesh3DData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangularMesh3DDataWire::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TriangularMesh3DData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TriangularMesh3DData::try_from(TriangularMesh3DDataWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

// The intermediate-data schema is the wire form, so each leaf's schema is its
// wire struct's.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for TriangularMesh2D {
    fn schema_name() -> String {
        "TriangularMesh2D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <TriangularMesh2DWire as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for TriangularMesh3DData {
    fn schema_name() -> String {
        "TriangularMesh3DData".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <TriangularMesh3DDataWire as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangular_mesh_uv_nests_per_triangle() {
        use crate::test_support::{explicit_uv, textured, theme};

        let mut mesh = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        // Distinct values, so the flattening order is observable.
        let corners: Vec<[f64; 2]> = (0..6).map(|i| [i as f64, 0.0]).collect();
        mesh.set_appearance(theme("rgb"), textured(), Some(explicit_uv(&corners)))
            .unwrap();

        let json = serde_json::to_value(&mesh).unwrap();
        let nested = &json["appearance"]["themes"][0]["uv_sets"][0]["uv"]["Explicit"];
        assert_eq!(
            nested.as_array().unwrap().len(),
            2,
            "one entry per triangle"
        );
        assert_eq!(
            nested[0]["exterior"],
            serde_json::json!([[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])
        );
        assert_eq!(
            nested[1]["exterior"],
            serde_json::json!([[3.0, 0.0], [4.0, 0.0], [5.0, 0.0]])
        );
        assert!(
            nested[0].get("holes").is_none(),
            "a triangle carries no hole rings"
        );

        let back: TriangularMesh2D =
            serde_json::from_str(&serde_json::to_string(&mesh).unwrap()).unwrap();
        assert_eq!(mesh, back);
    }

    #[test]
    fn mesh2d_round_trips_with_elevation() {
        let mesh = TriangularMesh2D::from_parts_at_elevation(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 1, 2],
            10.0,
        )
        .unwrap();
        let json = serde_json::to_value(&mesh).unwrap();
        // One elevation for the whole mesh, not one per vertex.
        assert_eq!(json["z"], serde_json::json!(10.0));

        let back: TriangularMesh2D = serde_json::from_value(json).unwrap();
        assert_eq!(mesh, back);
    }

    #[test]
    fn mesh3d_data_round_trips() {
        let mesh = TriangularMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        let json = serde_json::to_string(&mesh).unwrap();
        let back: TriangularMesh3DData = serde_json::from_str(&json).unwrap();
        assert_eq!(mesh, back);
    }
}
