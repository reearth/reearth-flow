//! Lossless intermediate-data encoding for the triangular-mesh leaves.
//!
//! The wire form lists the triangles as explicit vertex-index triples, widened
//! from the stored index buffer. Decoding packs them back through `from_parts`,
//! whose index width is fixed by the vertex count, so the round trip is exact.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;

use super::{TriangularMesh2D, TriangularMesh3DData};

/// Decoded wire form of a [`TriangularMesh2D`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct TriangularMesh2DWire {
    frame: CoordinateFrame,
    vertices: Vec<[f64; 2]>,
    triangles: Vec<[u32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
}

/// Decoded wire form of a [`TriangularMesh3DData`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct TriangularMesh3DDataWire {
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[u32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
}

impl From<&TriangularMesh2D> for TriangularMesh2DWire {
    fn from(m: &TriangularMesh2D) -> Self {
        TriangularMesh2DWire {
            frame: m.frame.clone(),
            vertices: m.vertices.clone(),
            triangles: m.triangles().collect(),
            z: m.z.as_ref().map(|z| z.to_vec()),
            appearance: m.appearance.clone(),
        }
    }
}

impl TryFrom<TriangularMesh2DWire> for TriangularMesh2D {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh2DWire) -> Result<Self, Self::Error> {
        let mut mesh =
            TriangularMesh2D::from_parts(w.frame, w.vertices, w.triangles.into_iter().flatten())?;
        mesh.z = w.z.map(Vec::into_boxed_slice);
        mesh.appearance = w.appearance;
        Ok(mesh)
    }
}

impl From<&TriangularMesh3DData> for TriangularMesh3DDataWire {
    fn from(m: &TriangularMesh3DData) -> Self {
        TriangularMesh3DDataWire {
            vertices: m.vertices.clone(),
            triangles: m.triangles().collect(),
            appearance: m.appearance.clone(),
        }
    }
}

impl TryFrom<TriangularMesh3DDataWire> for TriangularMesh3DData {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh3DDataWire) -> Result<Self, Self::Error> {
        let mut mesh =
            TriangularMesh3DData::from_parts(w.vertices, w.triangles.into_iter().flatten())?;
        mesh.appearance = w.appearance;
        Ok(mesh)
    }
}

impl Serialize for TriangularMesh2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangularMesh2DWire::from(self).serialize(serializer)
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
        TriangularMesh3DDataWire::from(self).serialize(serializer)
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
    fn mesh2d_round_trips_with_elevation() {
        let mesh = TriangularMesh2D::from_parts_with_elevation(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 10.0], [1.0, 0.0, 11.0], [0.0, 1.0, 12.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        let json = serde_json::to_string(&mesh).unwrap();
        let back: TriangularMesh2D = serde_json::from_str(&json).unwrap();
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
