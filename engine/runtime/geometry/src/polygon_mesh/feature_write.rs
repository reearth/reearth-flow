//! Lossless intermediate-data encoding for the polygon-mesh leaves.
//!
//! The wire form presents the CSR topology decoded: an explicit list of faces,
//! each a vertex-index exterior ring plus any hole rings, rather than the three
//! stored `face_indices` / `face_offsets` / `interior_offsets` buffers. Decoding
//! flattens the faces back into those buffers, whose widths are re-derived from
//! the vertex and corner counts, so the round trip is exact.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;

use super::{PolygonMesh2D, PolygonMesh3DData};

/// One mesh face: an exterior ring of vertex indices and any hole rings.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct FaceWire {
    exterior: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    holes: Vec<Vec<u32>>,
}

/// Decoded wire form of a [`PolygonMesh2D`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct PolygonMesh2DWire {
    frame: CoordinateFrame,
    vertices: Vec<[f64; 2]>,
    faces: Vec<FaceWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    z: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
}

/// Decoded wire form of a [`PolygonMesh3DData`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct PolygonMesh3DDataWire {
    vertices: Vec<[f64; 3]>,
    faces: Vec<FaceWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<Appearance>,
}

/// Read a scalar CSR buffer as a flat `u32` list.
fn flat(buf: &IndexBuffer<1>) -> Vec<u32> {
    buf.iter_u32().map(|[x]| x).collect()
}

/// Split the flat CSR buffers into explicit faces. `face_offsets` holds the
/// `n_faces - 1` internal boundaries; `interior_offsets` holds each hole ring's
/// start, globally ascending across faces.
fn decode_faces(
    face_indices: &[u32],
    face_offsets: &[u32],
    interior_offsets: &[u32],
) -> Vec<FaceWire> {
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
        // Hole ring starts inside this face, in order.
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
        faces.push(FaceWire {
            exterior,
            holes: hole_rings,
        });
        face_start = face_end;
    }
    faces
}

/// Concatenate the explicit faces back into the flat CSR buffers.
fn flatten_faces(faces: &[FaceWire]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut face_indices = Vec::new();
    let mut face_offsets = Vec::new();
    let mut interior_offsets = Vec::new();
    for (i, face) in faces.iter().enumerate() {
        if i > 0 {
            face_offsets.push(face_indices.len() as u32);
        }
        face_indices.extend_from_slice(&face.exterior);
        for hole in &face.holes {
            interior_offsets.push(face_indices.len() as u32);
            face_indices.extend_from_slice(hole);
        }
    }
    (face_indices, face_offsets, interior_offsets)
}

impl From<&PolygonMesh2D> for PolygonMesh2DWire {
    fn from(m: &PolygonMesh2D) -> Self {
        let (face_indices, face_offsets, interior_offsets) = m.csr_buffers();
        PolygonMesh2DWire {
            frame: m.frame.clone(),
            vertices: m.vertices.clone(),
            faces: decode_faces(
                &flat(face_indices),
                &flat(face_offsets),
                &flat(interior_offsets),
            ),
            z: m.z.as_ref().map(|z| z.to_vec()),
            appearance: m.appearance.clone(),
        }
    }
}

impl TryFrom<PolygonMesh2DWire> for PolygonMesh2D {
    type Error = crate::error::Error;

    fn try_from(w: PolygonMesh2DWire) -> Result<Self, Self::Error> {
        let (face_indices, face_offsets, interior_offsets) = flatten_faces(&w.faces);
        let mut mesh = PolygonMesh2D::from_raw_parts(
            w.frame,
            w.vertices,
            face_indices,
            face_offsets,
            interior_offsets,
        )?;
        mesh.z = w.z.map(Vec::into_boxed_slice);
        mesh.appearance = w.appearance;
        Ok(mesh)
    }
}

impl From<&PolygonMesh3DData> for PolygonMesh3DDataWire {
    fn from(m: &PolygonMesh3DData) -> Self {
        let (face_indices, face_offsets, interior_offsets) = m.csr_buffers();
        PolygonMesh3DDataWire {
            vertices: m.vertices.clone(),
            faces: decode_faces(
                &flat(face_indices),
                &flat(face_offsets),
                &flat(interior_offsets),
            ),
            appearance: m.appearance.clone(),
        }
    }
}

impl TryFrom<PolygonMesh3DDataWire> for PolygonMesh3DData {
    type Error = crate::error::Error;

    fn try_from(w: PolygonMesh3DDataWire) -> Result<Self, Self::Error> {
        let (face_indices, face_offsets, interior_offsets) = flatten_faces(&w.faces);
        let mut mesh = PolygonMesh3DData::from_raw_parts(
            w.vertices,
            face_indices,
            face_offsets,
            interior_offsets,
        )?;
        mesh.appearance = w.appearance;
        Ok(mesh)
    }
}

impl Serialize for PolygonMesh2D {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PolygonMesh2DWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PolygonMesh2D {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        PolygonMesh2D::try_from(PolygonMesh2DWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for PolygonMesh3DData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PolygonMesh3DDataWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PolygonMesh3DData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        PolygonMesh3DData::try_from(PolygonMesh3DDataWire::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

// The intermediate-data schema is the wire form, so each leaf's schema is its
// wire struct's.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for PolygonMesh2D {
    fn schema_name() -> String {
        "PolygonMesh2D".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <PolygonMesh2DWire as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for PolygonMesh3DData {
    fn schema_name() -> String {
        "PolygonMesh3DData".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <PolygonMesh3DDataWire as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh2d_multi_face_round_trips() {
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
        let json = serde_json::to_string(&mesh).unwrap();
        let back: PolygonMesh2D = serde_json::from_str(&json).unwrap();
        assert_eq!(mesh, back);
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
        let mesh = PolygonMesh3DData::from_polygons([&poly]);
        let json = serde_json::to_string(&mesh).unwrap();
        let back: PolygonMesh3DData = serde_json::from_str(&json).unwrap();
        assert_eq!(mesh, back);
    }
}
