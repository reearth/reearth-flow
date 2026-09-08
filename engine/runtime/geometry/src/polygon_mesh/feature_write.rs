//! Intermediate-data encoding for the polygon-mesh leaves: an explicit list of
//! faces, each an exterior ring of coordinates plus any hole rings, in place of
//! the stored vertex pool and its `face_indices` / `face_offsets` /
//! `interior_offsets` buffers. Per-corner UV is nested to mirror those faces and
//! rings.
//!
//! Rings carry coordinates rather than indices into a pool, so a consumer reads
//! a face without resolving anything. Decoding rebuilds the pool by welding
//! coordinates that match on exact `f64` bits, which is what makes the mesh
//! vertex-sharing again; see [`weld`].

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::feature_write::{
    decode_appearance, encode_appearance, Appearance, FaceRings,
};
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;

/// One mesh face in 2D: an exterior ring of coordinates and any hole rings.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Face (2D)"))]
pub(crate) struct Face2D {
    #[cfg_attr(feature = "schema", schemars(title = "Exterior ring"))]
    exterior: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(title = "Hole rings"))]
    holes: Vec<Vec<[f64; 2]>>,
}

/// One mesh face in 3D: an exterior ring of coordinates and any hole rings.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Face (3D)"))]
pub(crate) struct Face3D {
    #[cfg_attr(feature = "schema", schemars(title = "Exterior ring"))]
    exterior: Vec<[f64; 3]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(title = "Hole rings"))]
    holes: Vec<Vec<[f64; 3]>>,
}

/// A face's rings, exterior first, in the arity-independent form the encode and
/// decode helpers work in.
type Rings<const N: usize> = (Vec<[f64; N]>, Vec<Vec<[f64; N]>>);

macro_rules! face_conversions {
    ($ty:ident, $n:literal) => {
        impl From<Rings<$n>> for $ty {
            fn from((exterior, holes): Rings<$n>) -> Self {
                $ty { exterior, holes }
            }
        }
        impl $ty {
            fn into_rings(self) -> Rings<$n> {
                (self.exterior, self.holes)
            }
            fn ring_lengths(&self) -> FaceRings {
                FaceRings {
                    exterior: self.exterior.len(),
                    holes: self.holes.iter().map(Vec::len).collect(),
                }
            }
        }
    };
}
face_conversions!(Face2D, 2);
face_conversions!(Face3D, 3);

/// The UV ring layout of a face list.
fn layout_2d(faces: &[Face2D]) -> Vec<FaceRings> {
    faces.iter().map(Face2D::ring_lengths).collect()
}

/// The UV ring layout of a face list.
fn layout_3d(faces: &[Face3D]) -> Vec<FaceRings> {
    faces.iter().map(Face3D::ring_lengths).collect()
}

/// Read a scalar CSR buffer as a flat `u32` list.
fn flat(buf: &IndexBuffer<1>) -> Vec<u32> {
    buf.iter_u32().map(|[x]| x).collect()
}

/// Split the flat CSR buffers into per-face index rings. `face_offsets` holds the
/// `n_faces - 1` internal boundaries; `interior_offsets` holds each hole ring's
/// start, ascending across all faces.
fn index_rings(
    face_indices: &[u32],
    face_offsets: &[u32],
    interior_offsets: &[u32],
) -> Vec<(Vec<u32>, Vec<Vec<u32>>)> {
    if face_indices.is_empty() {
        return Vec::new();
    }
    let corner_count = face_indices.len();
    let mut holes = interior_offsets.iter().copied().peekable();
    let mut faces = Vec::with_capacity(face_offsets.len() + 1);
    let mut face_start = 0usize;
    let face_ends = face_offsets
        .iter()
        .map(|&o| o as usize)
        .chain(std::iter::once(corner_count));
    for face_end in face_ends {
        let mut ring_starts = Vec::new();
        while let Some(&h) = holes.peek() {
            if (h as usize) < face_end {
                ring_starts.push(h as usize);
                holes.next();
            } else {
                break;
            }
        }
        let exterior_end = ring_starts.first().copied().unwrap_or(face_end);
        let exterior = face_indices[face_start..exterior_end].to_vec();
        let hole_rings = ring_starts
            .iter()
            .enumerate()
            .map(|(j, &start)| {
                let end = ring_starts.get(j + 1).copied().unwrap_or(face_end);
                face_indices[start..end].to_vec()
            })
            .collect();
        faces.push((exterior, hole_rings));
        face_start = face_end;
    }
    faces
}

/// Resolve per-face index rings against the vertex pool into coordinate rings.
fn explode<const N: usize>(
    vertices: &[[f64; N]],
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
) -> Vec<Rings<N>> {
    let resolve = |ring: Vec<u32>| -> Vec<[f64; N]> {
        ring.into_iter().map(|i| vertices[i as usize]).collect()
    };
    index_rings(
        &flat(face_indices),
        &flat(face_offsets),
        &flat(interior_offsets),
    )
    .into_iter()
    .map(|(exterior, holes)| {
        (
            resolve(exterior),
            holes.into_iter().map(resolve).collect::<Vec<_>>(),
        )
    })
    .collect()
}

/// Rebuild the shared vertex pool and the flat CSR buffers from coordinate rings,
/// welding corners that match on exact `f64` bits.
///
/// The pool comes out in first-use order, which need not be the order the mesh
/// was originally built with: welding preserves the geometry, not the pool
/// layout. Encoding is therefore stable across a round trip even where the
/// in-memory pool is not.
fn weld<const N: usize>(faces: Vec<Rings<N>>) -> (Vec<[f64; N]>, Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut vertices: Vec<[f64; N]> = Vec::new();
    let mut seen: HashMap<[u64; N], u32> = HashMap::new();
    let mut face_indices: Vec<u32> = Vec::new();
    let mut face_offsets: Vec<u32> = Vec::new();
    let mut interior_offsets: Vec<u32> = Vec::new();
    let mut index_of = |coord: [f64; N], vertices: &mut Vec<[f64; N]>| {
        *seen.entry(coord.map(f64::to_bits)).or_insert_with(|| {
            let next = vertices.len() as u32;
            vertices.push(coord);
            next
        })
    };
    for (i, (exterior, holes)) in faces.into_iter().enumerate() {
        if i > 0 {
            face_offsets.push(face_indices.len() as u32);
        }
        for coord in exterior {
            let index = index_of(coord, &mut vertices);
            face_indices.push(index);
        }
        for hole in holes {
            interior_offsets.push(face_indices.len() as u32);
            for coord in hole {
                let index = index_of(coord, &mut vertices);
                face_indices.push(index);
            }
        }
    }
    (vertices, face_indices, face_offsets, interior_offsets)
}

/// A connected, vertex-sharing polygon mesh in 2D space, lying at a single
/// optional elevation. Each face carries its rings as coordinates.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Polygon mesh (2D)"))]
pub(crate) struct PolygonMesh2D {
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Faces"))]
    faces: Vec<Face2D>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Elevation"))]
    z: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Appearance"))]
    appearance: Option<Appearance>,
}

/// The faces of a 3D polygon mesh, with no coordinate frame of their own: a
/// solid's shell stores this form and takes the frame from the solid.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Shell polygon mesh (3D)"))]
pub(crate) struct PolygonMesh3DData {
    #[cfg_attr(feature = "schema", schemars(title = "Faces"))]
    faces: Vec<Face3D>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Appearance"))]
    appearance: Option<Appearance>,
}

/// A connected, vertex-sharing polygon mesh in 3D space. Each face carries its
/// rings as coordinates, written alongside the frame rather than nested under
/// the frameless mesh data a solid's shell shares.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Polygon mesh (3D)"))]
pub(crate) struct PolygonMesh3D {
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Faces"))]
    faces: Vec<Face3D>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Appearance"))]
    appearance: Option<Appearance>,
}

impl TryFrom<&super::PolygonMesh2D> for PolygonMesh2D {
    type Error = crate::error::Error;

    fn try_from(m: &super::PolygonMesh2D) -> Result<Self, Self::Error> {
        let (face_indices, face_offsets, interior_offsets) = m.csr_buffers();
        let faces: Vec<Face2D> = explode(&m.vertices, face_indices, face_offsets, interior_offsets)
            .into_iter()
            .map(Face2D::from)
            .collect();
        let layout = layout_2d(&faces);
        Ok(PolygonMesh2D {
            frame: m.frame.clone(),
            appearance: encode_appearance(&m.appearance, &layout)?,
            faces,
            z: m.elevation(),
        })
    }
}

impl TryFrom<PolygonMesh2D> for super::PolygonMesh2D {
    type Error = crate::error::Error;

    fn try_from(w: PolygonMesh2D) -> Result<Self, Self::Error> {
        let appearance = decode_appearance(w.appearance, &layout_2d(&w.faces))?;
        let (vertices, face_indices, face_offsets, interior_offsets) =
            weld(w.faces.into_iter().map(Face2D::into_rings).collect());
        let mut mesh = super::PolygonMesh2D::from_raw_parts(
            w.frame,
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
        )?;
        mesh.z = w.z;
        mesh.appearance = appearance;
        Ok(mesh)
    }
}

impl TryFrom<&super::PolygonMesh3DData> for PolygonMesh3DData {
    type Error = crate::error::Error;

    fn try_from(m: &super::PolygonMesh3DData) -> Result<Self, Self::Error> {
        let (face_indices, face_offsets, interior_offsets) = m.csr_buffers();
        let faces: Vec<Face3D> = explode(&m.vertices, face_indices, face_offsets, interior_offsets)
            .into_iter()
            .map(Face3D::from)
            .collect();
        let layout = layout_3d(&faces);
        Ok(PolygonMesh3DData {
            appearance: encode_appearance(&m.appearance, &layout)?,
            faces,
        })
    }
}

impl TryFrom<PolygonMesh3DData> for super::PolygonMesh3DData {
    type Error = crate::error::Error;

    fn try_from(w: PolygonMesh3DData) -> Result<Self, Self::Error> {
        let appearance = decode_appearance(w.appearance, &layout_3d(&w.faces))?;
        let (vertices, face_indices, face_offsets, interior_offsets) =
            weld(w.faces.into_iter().map(Face3D::into_rings).collect());
        let mut mesh = super::PolygonMesh3DData::from_raw_parts(
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
        )?;
        mesh.appearance = appearance;
        Ok(mesh)
    }
}

impl TryFrom<&super::PolygonMesh3D> for PolygonMesh3D {
    type Error = crate::error::Error;

    fn try_from(m: &super::PolygonMesh3D) -> Result<Self, Self::Error> {
        let data = PolygonMesh3DData::try_from(&m.data)?;
        Ok(PolygonMesh3D {
            frame: m.frame.clone(),
            faces: data.faces,
            appearance: data.appearance,
        })
    }
}

impl TryFrom<PolygonMesh3D> for super::PolygonMesh3D {
    type Error = crate::error::Error;

    fn try_from(w: PolygonMesh3D) -> Result<Self, Self::Error> {
        let data = super::PolygonMesh3DData::try_from(PolygonMesh3DData {
            faces: w.faces,
            appearance: w.appearance,
        })?;
        Ok(super::PolygonMesh3D::new(w.frame, data))
    }
}

impl Serialize for super::PolygonMesh2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PolygonMesh2D::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for super::PolygonMesh2D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::PolygonMesh2D::try_from(PolygonMesh2D::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for super::PolygonMesh3DData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PolygonMesh3DData::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for super::PolygonMesh3DData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::PolygonMesh3DData::try_from(PolygonMesh3DData::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for super::PolygonMesh3D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PolygonMesh3D::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for super::PolygonMesh3D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::PolygonMesh3D::try_from(PolygonMesh3D::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::PolygonMesh2D {
    fn schema_name() -> String {
        "PolygonMesh2D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <PolygonMesh2D as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::PolygonMesh3DData {
    fn schema_name() -> String {
        "PolygonMesh3DData".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <PolygonMesh3DData as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::PolygonMesh3D {
    fn schema_name() -> String {
        "PolygonMesh3D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <PolygonMesh3D as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};

    /// Encoding is the stable form: welding may reorder the vertex pool, so a
    /// round trip is checked on the wire form rather than on the in-memory one.
    fn round_trips_2d(mesh: &PolygonMesh2D) -> serde_json::Value {
        let json = serde_json::to_value(mesh).unwrap();
        let back: PolygonMesh2D = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json, serde_json::to_value(&back).unwrap());
        json
    }

    fn round_trips_3d(mesh: &PolygonMesh3DData) -> serde_json::Value {
        let json = serde_json::to_value(mesh).unwrap();
        let back: PolygonMesh3DData = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json, serde_json::to_value(&back).unwrap());
        json
    }

    #[test]
    fn mesh2d_writes_faces_as_coordinates_not_indices() {
        let mesh = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [4.0, 0.0],
                [4.0, 2.0],
            ],
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();

        let json = round_trips_2d(&mesh);
        assert!(json.get("vertices").is_none(), "no vertex pool on the wire");
        assert_eq!(
            json["faces"],
            serde_json::json!([
                {"exterior": [[0.0,0.0],[2.0,0.0],[2.0,2.0],[0.0,2.0]]},
                {"exterior": [[2.0,0.0],[4.0,0.0],[4.0,2.0],[2.0,2.0]]},
            ])
        );
    }

    #[test]
    fn shared_corners_reweld_into_one_pool_entry() {
        // The two faces share the edge (2,0)-(2,2): four distinct corners each,
        // six distinct coordinates overall.
        let mesh = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [4.0, 0.0],
                [4.0, 2.0],
            ],
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let back: PolygonMesh2D =
            serde_json::from_value(serde_json::to_value(&mesh).unwrap()).unwrap();
        assert_eq!(
            back.vertices.len(),
            6,
            "the shared edge is welded, not split"
        );
        assert_eq!(back, mesh, "this pool is already in first-use order");
    }

    #[test]
    fn mesh2d_splits_holes_across_face_boundaries() {
        // Two faces, the first carrying two holes and the second one, so the
        // hole cursor has to stop at the right face boundary.
        let vertices: Vec<[f64; 2]> = (0..15).map(|i| [i as f64, (i * 2) as f64]).collect();
        let mesh = PolygonMesh2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vertices,
            (0..15).collect(),
            vec![9],
            vec![3, 6, 12],
        )
        .unwrap();

        let json = round_trips_2d(&mesh);
        let faces = json["faces"].as_array().unwrap();
        assert_eq!(faces.len(), 2);
        assert_eq!(faces[0]["holes"].as_array().unwrap().len(), 2);
        assert_eq!(faces[1]["holes"].as_array().unwrap().len(), 1);
        assert_eq!(
            faces[0]["exterior"],
            serde_json::json!([[0.0, 0.0], [1.0, 2.0], [2.0, 4.0]])
        );
    }

    #[test]
    fn mesh2d_writes_one_elevation_for_the_whole_mesh() {
        let mesh = PolygonMesh2D::from_parts_at_elevation(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            vec![vec![0u32, 1, 2, 3]],
            10.0,
        )
        .unwrap();
        let json = round_trips_2d(&mesh);
        assert_eq!(json["z"], serde_json::json!(10.0));
    }

    #[test]
    fn mesh2d_omits_elevation_when_pure_2d() {
        let mesh = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        let json = serde_json::to_value(&mesh).unwrap();
        assert!(json.get("z").is_none(), "pure 2D writes no elevation");
    }

    #[test]
    fn mesh_uv_nests_per_face_and_ring() {
        use crate::polygon::Polygon3D;
        use crate::test_support::{explicit_uv, textured, theme};

        let outer = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
        ];
        let mut poly = Polygon3D::from_rings(CoordinateFrame::Euclidean, outer, vec![hole]);
        let corners: Vec<[f64; 2]> = (0..8).map(|i| [i as f64, 0.0]).collect();
        poly.set_appearance(theme("rgb"), textured(), Some(explicit_uv(&corners)))
            .unwrap();
        let mesh = PolygonMesh3DData::from_polygons([&poly]);

        let json = round_trips_3d(&mesh);
        let nested = &json["appearance"]["themes"][0]["uv_sets"][0]["uv"]["Explicit"];
        assert_eq!(nested.as_array().unwrap().len(), 1, "one entry per face");
        // The nesting mirrors the face's own rings.
        assert_eq!(
            nested[0]["exterior"].as_array().unwrap().len(),
            json["faces"][0]["exterior"].as_array().unwrap().len()
        );
        assert_eq!(
            nested[0]["holes"][0].as_array().unwrap().len(),
            json["faces"][0]["holes"][0].as_array().unwrap().len()
        );
    }

    #[test]
    fn mesh3d_writes_its_data_fields_alongside_the_frame() {
        use crate::polygon::Polygon3D;
        let poly = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let mesh = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&poly]).unwrap();

        let json = serde_json::to_value(&mesh).unwrap();
        assert!(json.get("data").is_none(), "the data split stays in memory");
        assert!(json.get("vertices").is_none(), "no vertex pool on the wire");
        assert!(json.get("frame").is_some());
        assert!(json.get("faces").is_some());

        let back: PolygonMesh3D = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(json, serde_json::to_value(&back).unwrap());
    }

    #[test]
    fn mesh3d_with_hole_round_trips() {
        use crate::polygon::Polygon3D;
        let outer = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
        ];
        let poly = Polygon3D::from_rings(CoordinateFrame::Euclidean, outer, vec![hole]);
        round_trips_3d(&PolygonMesh3DData::from_polygons([&poly]));
    }
}
