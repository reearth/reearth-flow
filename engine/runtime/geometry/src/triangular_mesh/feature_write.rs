//! Intermediate-data encoding for the triangular-mesh leaves: each triangle as
//! its three corner coordinates, in place of the stored vertex pool and index
//! buffer. Per-corner UV is nested to match, one three-corner entry per triangle.
//!
//! Triangles carry coordinates rather than indices into a pool, so a consumer
//! reads a triangle without resolving anything. Decoding rebuilds the pool by
//! welding corners that match on exact `f64` bits, which is what makes the mesh
//! vertex-sharing again; the pool comes out in first-use order, which need not be
//! the order the mesh was originally built with.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::feature_write::{
    decode_appearance, encode_appearance, Appearance, FaceRings,
};
use crate::coordinate::CoordinateFrame;

/// One face per triangle, each a single three-corner ring.
fn triangle_layout(triangles: usize) -> Vec<FaceRings> {
    (0..triangles).map(|_| FaceRings::simple(3)).collect()
}

/// Resolve index triples against the vertex pool into corner coordinates.
fn explode<const N: usize>(
    vertices: &[[f64; N]],
    triangles: impl Iterator<Item = [u32; 3]>,
) -> Vec<[[f64; N]; 3]> {
    triangles.map(|t| t.map(|i| vertices[i as usize])).collect()
}

/// Rebuild the shared vertex pool and the flat index list from corner
/// coordinates, welding corners that match on exact `f64` bits.
fn weld<const N: usize>(triangles: Vec<[[f64; N]; 3]>) -> (Vec<[f64; N]>, Vec<u32>) {
    let mut vertices: Vec<[f64; N]> = Vec::new();
    let mut seen: HashMap<[u64; N], u32> = HashMap::new();
    let mut indices: Vec<u32> = Vec::with_capacity(triangles.len() * 3);
    for triangle in triangles {
        for coord in triangle {
            let index = *seen.entry(coord.map(f64::to_bits)).or_insert_with(|| {
                let next = vertices.len() as u32;
                vertices.push(coord);
                next
            });
            indices.push(index);
        }
    }
    (vertices, indices)
}

/// A triangle mesh in 2D space, lying at a single optional elevation. Each
/// triangle carries its three corners as coordinates.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Triangle mesh (2D)"))]
pub(crate) struct TriangularMesh2D {
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Triangles"))]
    triangles: Vec<[[f64; 2]; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Elevation"))]
    z: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Appearance"))]
    appearance: Option<Appearance>,
}

/// The triangles of a 3D triangle mesh, with no coordinate frame of their own:
/// a solid's shell stores this form and takes the frame from the solid.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Triangle mesh data (3D)"))]
pub(crate) struct TriangularMesh3DData {
    #[cfg_attr(feature = "schema", schemars(title = "Triangles"))]
    triangles: Vec<[[f64; 3]; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Appearance"))]
    appearance: Option<Appearance>,
}

/// A triangle mesh in 3D space. Each triangle carries its three corners as
/// coordinates, written alongside the frame rather than nested under the
/// frameless mesh data a solid's shell shares.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Triangle mesh (3D)"))]
pub(crate) struct TriangularMesh3D {
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Triangles"))]
    triangles: Vec<[[f64; 3]; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Appearance"))]
    appearance: Option<Appearance>,
}

impl TryFrom<&super::TriangularMesh2D> for TriangularMesh2D {
    type Error = crate::error::Error;

    fn try_from(m: &super::TriangularMesh2D) -> Result<Self, Self::Error> {
        let triangles = explode(&m.vertices, m.triangles());
        let layout = triangle_layout(triangles.len());
        Ok(TriangularMesh2D {
            frame: m.frame.clone(),
            appearance: encode_appearance(&m.appearance, &layout)?,
            triangles,
            z: m.elevation(),
        })
    }
}

impl TryFrom<TriangularMesh2D> for super::TriangularMesh2D {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh2D) -> Result<Self, Self::Error> {
        let appearance = decode_appearance(w.appearance, &triangle_layout(w.triangles.len()))?;
        let (vertices, indices) = weld(w.triangles);
        let mut mesh = super::TriangularMesh2D::from_parts(w.frame, vertices, indices)?;
        mesh.z = w.z;
        mesh.appearance = appearance;
        Ok(mesh)
    }
}

impl TryFrom<&super::TriangularMesh3DData> for TriangularMesh3DData {
    type Error = crate::error::Error;

    fn try_from(m: &super::TriangularMesh3DData) -> Result<Self, Self::Error> {
        let triangles = explode(&m.vertices, m.triangles());
        let layout = triangle_layout(triangles.len());
        Ok(TriangularMesh3DData {
            appearance: encode_appearance(&m.appearance, &layout)?,
            triangles,
        })
    }
}

impl TryFrom<TriangularMesh3DData> for super::TriangularMesh3DData {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh3DData) -> Result<Self, Self::Error> {
        let appearance = decode_appearance(w.appearance, &triangle_layout(w.triangles.len()))?;
        let (vertices, indices) = weld(w.triangles);
        let mut mesh = super::TriangularMesh3DData::from_parts(vertices, indices)?;
        mesh.appearance = appearance;
        Ok(mesh)
    }
}

impl TryFrom<&super::TriangularMesh3D> for TriangularMesh3D {
    type Error = crate::error::Error;

    fn try_from(m: &super::TriangularMesh3D) -> Result<Self, Self::Error> {
        let data = TriangularMesh3DData::try_from(&m.data)?;
        Ok(TriangularMesh3D {
            frame: m.frame.clone(),
            triangles: data.triangles,
            appearance: data.appearance,
        })
    }
}

impl TryFrom<TriangularMesh3D> for super::TriangularMesh3D {
    type Error = crate::error::Error;

    fn try_from(w: TriangularMesh3D) -> Result<Self, Self::Error> {
        let data = super::TriangularMesh3DData::try_from(TriangularMesh3DData {
            triangles: w.triangles,
            appearance: w.appearance,
        })?;
        Ok(super::TriangularMesh3D::new(w.frame, data))
    }
}

impl Serialize for super::TriangularMesh2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangularMesh2D::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for super::TriangularMesh2D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::TriangularMesh2D::try_from(TriangularMesh2D::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for super::TriangularMesh3DData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangularMesh3DData::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for super::TriangularMesh3DData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::TriangularMesh3DData::try_from(TriangularMesh3DData::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for super::TriangularMesh3D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangularMesh3D::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for super::TriangularMesh3D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::TriangularMesh3D::try_from(TriangularMesh3D::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::TriangularMesh2D {
    fn schema_name() -> String {
        "TriangularMesh2D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <TriangularMesh2D as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::TriangularMesh3DData {
    fn schema_name() -> String {
        "TriangularMesh3DData".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <TriangularMesh3DData as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::TriangularMesh3D {
    fn schema_name() -> String {
        "TriangularMesh3D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <TriangularMesh3D as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};

    /// Encoding is the stable form: welding may reorder the vertex pool, so a
    /// round trip is checked on the wire form rather than on the in-memory one.
    fn round_trips_2d(mesh: &TriangularMesh2D) -> serde_json::Value {
        let json = serde_json::to_value(mesh).unwrap();
        let back: TriangularMesh2D = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json, serde_json::to_value(&back).unwrap());
        json
    }

    #[test]
    fn mesh2d_writes_triangles_as_coordinates_not_indices() {
        let mesh = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();

        let json = round_trips_2d(&mesh);
        assert!(json.get("vertices").is_none(), "no vertex pool on the wire");
        assert_eq!(
            json["triangles"],
            serde_json::json!([
                [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ])
        );
    }

    #[test]
    fn shared_corners_reweld_into_one_pool_entry() {
        // The two triangles share the edge (1,0)-(0,1): six corners on the wire,
        // four distinct coordinates in the rebuilt pool.
        let mesh = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        let back: TriangularMesh2D =
            serde_json::from_value(serde_json::to_value(&mesh).unwrap()).unwrap();
        assert_eq!(
            back.vertices.len(),
            4,
            "the shared edge is welded, not split"
        );
        assert_eq!(back, mesh, "this pool is already in first-use order");
    }

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

        let json = round_trips_2d(&mesh);
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
        let json = round_trips_2d(&mesh);
        assert_eq!(json["z"], serde_json::json!(10.0));
    }

    #[test]
    fn mesh3d_writes_its_data_fields_alongside_the_frame() {
        let mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();

        let json = serde_json::to_value(&mesh).unwrap();
        assert!(json.get("data").is_none(), "the data split stays in memory");
        assert!(json.get("vertices").is_none(), "no vertex pool on the wire");
        assert!(json.get("frame").is_some());
        assert!(json.get("triangles").is_some());

        let back: TriangularMesh3D = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json, serde_json::to_value(&back).unwrap());
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
        let json = serde_json::to_value(&mesh).unwrap();
        let back: TriangularMesh3DData = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json, serde_json::to_value(&back).unwrap());
    }
}
