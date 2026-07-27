//! New-geometry path for the OBJ Reader. Reads OBJ faces into a single
//! `PolygonMesh3D` per feature (Euclidean frame; OBJ is model-space). Appearance
//! is added in a later step. Reuses `super`'s OBJ/MTL parser.

use std::sync::Arc;

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_geometry::{
    coordinate::CoordinateFrame, polygon::Polygon3D, polygon_mesh::PolygonMesh3D,
    Euclidean3DGeometry, Geometry,
};
use reearth_flow_runtime::{
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use tokio::sync::mpsc::Sender;

use crate::errors::SourceError;

use super::{Face, ObjData, ObjReaderCompiledParam};

pub(super) async fn read(
    _ctx: &NodeContext,
    _storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    content: &Bytes,
    params: &ObjReaderCompiledParam,
    sender: &Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    let obj_data = super::parse_obj_content(content)?;

    if params.merge_groups {
        let feature = build_feature(
            &obj_data,
            &obj_data.faces,
            group_attrs_merged(&obj_data),
            params,
        );
        send_feature(sender, feature).await?;
    } else {
        for (name, faces) in group_faces(&obj_data) {
            let feature =
                build_feature(&obj_data, &faces, group_attrs_one(&obj_data, &name), params);
            send_feature(sender, feature).await?;
        }
    }
    Ok(())
}

/// Group faces by object (fallback group, then "default"), preserving the old
/// reader's grouping.
fn group_faces(obj_data: &ObjData) -> Vec<(String, Vec<Face>)> {
    let mut groups: IndexMap<String, Vec<Face>> = IndexMap::new();
    for face in &obj_data.faces {
        let key = face
            .object
            .clone()
            .or_else(|| face.group.clone())
            .unwrap_or_else(|| "default".to_string());
        groups.entry(key).or_default().push(face.clone());
    }
    groups.into_iter().collect()
}

/// Convert a set of OBJ faces into one `PolygonMesh3D` geometry (bare in this task).
fn faces_to_geometry(
    obj_data: &ObjData,
    faces: &[Face],
    _params: &ObjReaderCompiledParam,
) -> Geometry {
    let mut polygons: Vec<Polygon3D> = Vec::new();
    for face in faces {
        let Some(ring) = face_ring(obj_data, face) else {
            continue;
        };
        polygons.push(Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            std::iter::empty::<Vec<[f64; 3]>>(),
        ));
    }
    if polygons.is_empty() {
        return Geometry::None;
    }
    match PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, &polygons) {
        Ok(mesh) => Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(mesh))),
        Err(e) => {
            tracing::warn!("OBJ: failed to build PolygonMesh, dropping feature: {e:?}");
            Geometry::None
        }
    }
}

/// Resolve a face's vertex ring to `[f64;3]` coordinates (OBJ 1-based / negative
/// indices), or `None` if any index is out of bounds or the ring has < 3 vertices.
fn face_ring(obj_data: &ObjData, face: &Face) -> Option<Vec<[f64; 3]>> {
    let mut ring = Vec::with_capacity(face.vertices.len());
    for v in &face.vertices {
        let idx = if v.vertex_index > 0 {
            (v.vertex_index - 1) as usize
        } else {
            (obj_data.vertices.len() as i32 + v.vertex_index) as usize
        };
        ring.push(*obj_data.vertices.get(idx)?);
    }
    (ring.len() >= 3).then_some(ring)
}

fn build_feature(
    obj_data: &ObjData,
    faces: &[Face],
    mut attributes: IndexMap<Attribute, AttributeValue>,
    params: &ObjReaderCompiledParam,
) -> Feature {
    attributes.insert(
        Attribute::new("faceCount"),
        AttributeValue::Number(serde_json::Number::from(faces.len())),
    );
    Feature::new_with_attributes_and_geometry(
        attributes,
        faces_to_geometry(obj_data, faces, params),
    )
}

fn base_attrs() -> IndexMap<Attribute, AttributeValue> {
    let mut a = IndexMap::new();
    a.insert(
        Attribute::new("source"),
        AttributeValue::String("OBJ".to_string()),
    );
    a
}

fn group_attrs_one(obj_data: &ObjData, name: &str) -> IndexMap<Attribute, AttributeValue> {
    let mut a = base_attrs();
    let key = if obj_data.objects.iter().any(|o| o == name) {
        "object"
    } else {
        "group"
    };
    a.insert(
        Attribute::new(key),
        AttributeValue::String(name.to_string()),
    );
    a
}

fn group_attrs_merged(obj_data: &ObjData) -> IndexMap<Attribute, AttributeValue> {
    let mut a = base_attrs();
    let arr = |v: &[String]| {
        AttributeValue::Array(v.iter().cloned().map(AttributeValue::String).collect())
    };
    if !obj_data.objects.is_empty() {
        a.insert(Attribute::new("objects"), arr(&obj_data.objects));
    }
    if !obj_data.groups.is_empty() {
        a.insert(Attribute::new("groups"), arr(&obj_data.groups));
    }
    a
}

async fn send_feature(
    sender: &Sender<(Port, IngestionMessage)>,
    feature: Feature,
) -> Result<(), SourceError> {
    sender
        .send((
            FEATURES_PORT.clone(),
            IngestionMessage::OperationEvent { feature },
        ))
        .await
        .map_err(|e| SourceError::ObjReader(format!("Failed to send feature: {e}")))
}

#[cfg(test)]
fn test_params() -> ObjReaderCompiledParam {
    use crate::file::reader::runner::FileReaderCompiledParam;

    ObjReaderCompiledParam {
        parse_materials: false,
        material_file: None,
        triangulate: false,
        merge_groups: false,
        _include_normals: true,
        _include_texcoords: true,
        common: FileReaderCompiledParam {
            dataset: None,
            inline: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build ObjData directly via the shared parser.
    fn parse(obj: &str) -> super::super::ObjData {
        super::super::parse_obj_content(&bytes::Bytes::copy_from_slice(obj.as_bytes())).unwrap()
    }

    fn mesh_of(geom: Geometry) -> reearth_flow_geometry::polygon_mesh::PolygonMesh3D {
        match geom {
            Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) => *m,
            other => panic!("expected Euclidean3D PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn quad_face_becomes_one_polygon_mesh_face_euclidean() {
        let data = parse("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n");
        let params = test_params();
        let mesh = mesh_of(faces_to_geometry(&data, &data.faces, &params));
        assert_eq!(*mesh.frame(), CoordinateFrame::Euclidean);
        assert_eq!(
            mesh.num_faces(),
            1,
            "one quad face preserved as one n-gon face"
        );
        assert!(mesh.appearance().is_none(), "no materials yet in Task 1");
    }
}
