//! New-geometry path for the glTF Reader.
//!
//! Declared as a child module of `gltf.rs` so it reuses that module's scene
//! traversal, buffer loading, and old-world triangle extraction via `super::`.
//! The old extraction (`reearth_flow_gltf::create_geometry_from_primitives_with_transform`)
//! still returns the old `Geometry3D`; we convert that into the new
//! `reearth_flow_geometry::Geometry`. glTF vertices are in model space with no
//! CRS, so every leaf uses `CoordinateFrame::Euclidean` (no axis-swap / no
//! reprojection, unlike the GeoPackage/GeoJSON readers).

use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::{
    coordinate::CoordinateFrame,
    triangular_mesh::TriangularMesh3D,
    types::{geometry::Geometry3D as OldGeometry3D, polygon::Polygon3D as OldPolygon3D},
    Euclidean3DGeometry, Geometry,
};
use reearth_flow_runtime::{
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use tokio::sync::mpsc::Sender;

use crate::{errors::SourceError, file::reader::runner::get_input_path};

use super::{load_buffers, merge_geometries, GltfReaderCompiledParam, MeshInfo};

/// New-geometry glTF read: mirrors the old-world `read_gltf` traversal, converting
/// each extracted geometry into the new model and streaming features to `sender` as
/// they are produced (no buffering of the full feature list), matching `read_gltf`.
pub(super) async fn read(
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    content: &Bytes,
    params: &GltfReaderCompiledParam,
    sender: &Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    let gltf_uri = get_input_path(&params.common)
        .map_err(SourceError::GltfReader)?
        .unwrap_or_else(|| Uri::from_str("file://./unknown.gltf").unwrap());

    let gltf = gltf::Gltf::from_slice(content)
        .map_err(|e| SourceError::GltfReader(format!("Failed to parse glTF: {e}")))?;

    let buffer_data = load_buffers(&gltf, ctx, storage_resolver, &gltf_uri, content).await?;

    // Collect lightweight mesh info with transforms (traversal only; heavy geometry
    // processing happens per-mesh below), same as the old-world path.
    let mut mesh_infos = Vec::new();
    for scene in gltf.scenes() {
        reearth_flow_gltf::traverse_scene(
            &scene,
            |node, world_transform| -> Result<(), SourceError> {
                if let Some(mesh) = node.mesh() {
                    let primitives: Vec<_> = mesh.primitives().collect();
                    if !primitives.is_empty() {
                        mesh_infos.push(MeshInfo {
                            primitives,
                            mesh_name: mesh.name().map(|s| s.to_string()),
                            node_name: if params.include_nodes {
                                node.name().map(|s| s.to_string())
                            } else {
                                None
                            },
                            transform: world_transform.clone(),
                        });
                    }
                }
                Ok(())
            },
        )?;
    }

    if !params.merge_meshes {
        // Stream each mesh's feature as it is produced (no full-list buffering).
        for mesh_info in mesh_infos {
            let old = reearth_flow_gltf::create_geometry_from_primitives_with_transform(
                &mesh_info.primitives,
                &buffer_data,
                Some(&mesh_info.transform),
            )
            .map_err(|e| SourceError::GltfReader(format!("Failed to create geometry: {e}")))?;

            let mesh_names = mesh_info.mesh_name.map(|n| vec![n]).unwrap_or_default();
            let node_names = mesh_info.node_name.map(|n| vec![n]).unwrap_or_default();

            let feature = build_feature(
                to_new_geometry(&old),
                &mesh_names,
                &node_names,
                mesh_info.primitives.len(),
                params,
            );
            send_feature(sender, feature).await?;
        }
    } else if !mesh_infos.is_empty() {
        let mut geometries = Vec::new();
        let mut all_mesh_names = std::collections::HashSet::new();
        let mut all_node_names = std::collections::HashSet::new();
        let mut total_primitives = 0;

        for mesh_info in mesh_infos {
            let old = reearth_flow_gltf::create_geometry_from_primitives_with_transform(
                &mesh_info.primitives,
                &buffer_data,
                Some(&mesh_info.transform),
            )
            .map_err(|e| SourceError::GltfReader(format!("Failed to create geometry: {e}")))?;

            geometries.push(old);
            if let Some(name) = mesh_info.mesh_name {
                all_mesh_names.insert(name);
            }
            if let Some(name) = mesh_info.node_name {
                all_node_names.insert(name);
            }
            total_primitives += mesh_info.primitives.len();
        }

        let merged = merge_geometries(geometries.iter().collect());
        let merged_mesh_names: Vec<String> = all_mesh_names.into_iter().collect();
        let merged_node_names: Vec<String> = all_node_names.into_iter().collect();

        let feature = build_feature(
            to_new_geometry(&merged),
            &merged_mesh_names,
            &merged_node_names,
            total_primitives,
            params,
        );
        send_feature(sender, feature).await?;
    }

    Ok(())
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
        .map_err(|e| SourceError::GltfReader(format!("Failed to send feature: {e}")))
}

/// Convert the old-world `Geometry3D` produced by the glTF triangle extraction into
/// the new `reearth_flow_geometry::Geometry`. glTF is a triangle mesh, so the
/// idiomatic target is a single `Euclidean3D::TriangularMesh`: the extraction's
/// per-triangle `Polygon`/`MultiPolygon` faces are flattened back into a triangle
/// soup and fed to `TriangularMesh3D::from_soup`, which deduplicates shared vertices
/// into one pool while preserving each triangle's winding (glTF front faces are CCW).
/// The mesh carries `CoordinateFrame::Euclidean` since glTF is model-space (no CRS).
fn to_new_geometry(old: &OldGeometry3D<f64>) -> Geometry {
    let mut soup: Vec<[f64; 3]> = Vec::new();
    match old {
        OldGeometry3D::Polygon(p) => push_polygon_soup(p, &mut soup),
        OldGeometry3D::MultiPolygon(mp) => {
            for p in &mp.0 {
                push_polygon_soup(p, &mut soup);
            }
        }
        other => {
            // The glTF extraction only produces Polygon / MultiPolygon today
            // (see reearth_flow_gltf::create_geometry_from_primitives_with_transform).
            tracing::warn!(
                "glTF: unsupported geometry variant for new-geometry conversion, dropping: {other:?}"
            );
        }
    }

    if soup.is_empty() {
        return Geometry::None;
    }

    Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
        TriangularMesh3D::from_soup(CoordinateFrame::Euclidean, soup),
    )))
}

/// Fan-triangulate one extracted face into `soup` (three vertices per triangle).
/// The glTF extraction only ever emits triangles, so for the common case this just
/// forwards the three corners; the fan keeps it correct if a face ever has more.
/// The extraction stores rings closed (last vertex repeats the first), so the
/// trailing closing vertex is dropped before fanning.
fn push_polygon_soup(p: &OldPolygon3D<f64>, soup: &mut Vec<[f64; 3]>) {
    let ring: Vec<[f64; 3]> = p.exterior().0.iter().map(|c| [c.x, c.y, c.z]).collect();
    let corners: &[[f64; 3]] = match ring.split_last() {
        Some((last, head)) if ring.first() == Some(last) => head,
        _ => &ring,
    };
    if corners.len() < 3 {
        return;
    }
    for i in 1..corners.len() - 1 {
        soup.push(corners[0]);
        soup.push(corners[i]);
        soup.push(corners[i + 1]);
    }
}

fn build_feature(
    geometry: Geometry,
    mesh_names: &[String],
    node_names: &[String],
    primitive_count: usize,
    params: &GltfReaderCompiledParam,
) -> Feature {
    let mut attributes = IndexMap::new();

    attributes.insert(
        Attribute::new("source"),
        AttributeValue::String("glTF".to_string()),
    );

    if !mesh_names.is_empty() {
        let key = if mesh_names.len() == 1 {
            "mesh"
        } else {
            "meshes"
        };
        attributes.insert(Attribute::new(key), string_or_array(mesh_names));
    }

    if params.include_nodes && !node_names.is_empty() {
        let key = if node_names.len() == 1 {
            "node"
        } else {
            "nodes"
        };
        attributes.insert(Attribute::new(key), string_or_array(node_names));
    }

    attributes.insert(
        Attribute::new("primitiveCount"),
        AttributeValue::Number(serde_json::Number::from(primitive_count)),
    );

    let mut feature = Feature::new_with_attributes(attributes);
    feature.geometry = Arc::new(geometry);
    feature
}

fn string_or_array(values: &[String]) -> AttributeValue {
    if values.len() == 1 {
        AttributeValue::String(values[0].clone())
    } else {
        AttributeValue::Array(
            values
                .iter()
                .map(|v| AttributeValue::String(v.clone()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::{
        coordinate::Coordinate, line_string::LineString, multi_polygon::MultiPolygon3D,
    };

    fn old_ring(coords: &[[f64; 3]]) -> LineString<f64, f64> {
        LineString(
            coords
                .iter()
                .map(|c| Coordinate {
                    x: c[0],
                    y: c[1],
                    z: c[2],
                })
                .collect(),
        )
    }

    fn old_polygon(exterior: &[[f64; 3]]) -> OldPolygon3D<f64> {
        OldPolygon3D::new(old_ring(exterior), vec![])
    }

    #[test]
    fn single_triangle_converts_to_triangular_mesh_preserving_z_and_winding() {
        let ext = [[0.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 1.0, 3.0]];
        let old = OldGeometry3D::Polygon(old_polygon(&ext));

        match to_new_geometry(&old) {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => {
                assert_eq!(*m.frame(), CoordinateFrame::Euclidean);
                assert_eq!(m.num_triangles(), 1);
                // `from_soup` assigns vertex indices in first-seen order, so a single
                // triangle keeps its original winding v0, v1, v2 (glTF front faces are
                // CCW; this must not be reordered).
                let tris: Vec<[u32; 3]> = m.triangles().collect();
                assert_eq!(tris, vec![[0, 1, 2]]);
                // The three distinct vertices survive in order, per-vertex Z included.
                assert_eq!(m.vertices(), ext.as_slice());
            }
            other => panic!("expected Euclidean3D TriangularMesh, got {other:?}"),
        }
    }

    #[test]
    fn multiple_triangles_share_a_single_vertex_pool() {
        // Two triangles sharing the edge (1,0,0)-(0,1,0): 4 distinct vertices, not 6.
        let a = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let b = [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let old = OldGeometry3D::MultiPolygon(MultiPolygon3D::new(vec![
            old_polygon(&a),
            old_polygon(&b),
        ]));

        match to_new_geometry(&old) {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => {
                assert_eq!(m.num_triangles(), 2, "both triangles kept");
                assert_eq!(
                    m.vertices().len(),
                    4,
                    "shared vertices deduplicated into one pool (not 6)"
                );
            }
            other => panic!("expected Euclidean3D TriangularMesh, got {other:?}"),
        }
    }

    // Minimal self-contained glTF 2.0: one triangle with distinct per-vertex Z
    // (1, 2, 3), POSITION (VEC3 f32) + indices (u16) in an embedded data-URI buffer.
    // Buffer layout: 36 bytes positions at offset 0, 6 bytes indices at offset 36.
    const TRIANGLE_GLTF: &str = r#"{
      "asset": {"version": "2.0"},
      "scenes": [{"nodes": [0]}],
      "nodes": [{"mesh": 0}],
      "meshes": [{"name": "tri", "primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "mode": 4}]}],
      "accessors": [
        {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 1.0], "max": [1.0, 1.0, 3.0]},
        {"bufferView": 1, "componentType": 5123, "count": 3, "type": "SCALAR"}
      ],
      "bufferViews": [
        {"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962},
        {"buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963}
      ],
      "buffers": [{"byteLength": 42, "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAIA/AACAPwAAAAAAAABAAAAAAAAAgD8AAEBAAAABAAIA"}]
    }"#;

    /// End-to-end reader-side check on a REAL glTF: parse -> primitives -> the
    /// crate's triangle extraction -> old->new conversion. Unlike the converter
    /// unit tests above (synthetic old geometry), this exercises the actual glTF
    /// parsing/extraction path, closing the gap between the pure converter and the
    /// live reader. No NodeContext needed: the buffer is an embedded data URI.
    #[test]
    fn real_gltf_triangle_reads_as_triangular_mesh_preserving_z() {
        let gltf = gltf::Gltf::from_slice(TRIANGLE_GLTF.as_bytes()).expect("parse glTF");

        // The single buffer is the embedded data URI; build its exact bytes here
        // (positions VEC3 f32 at offset 0, indices u16 at offset 36) so the test is
        // self-contained and independent of the reader's private buffer-loading path.
        let mut buf = Vec::new();
        for xyz in [[0.0f32, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 1.0, 3.0]] {
            for c in xyz {
                buf.extend_from_slice(&c.to_le_bytes());
            }
        }
        for i in [0u16, 1, 2] {
            buf.extend_from_slice(&i.to_le_bytes());
        }
        let buffer_data = vec![buf];

        let primitives: Vec<_> = gltf
            .meshes()
            .next()
            .expect("one mesh")
            .primitives()
            .collect();

        let old = reearth_flow_gltf::create_geometry_from_primitives_with_transform(
            &primitives,
            &buffer_data,
            None,
        )
        .expect("extract geometry from real glTF triangle");

        let geom = to_new_geometry(&old);

        match geom {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => {
                assert_eq!(*m.frame(), CoordinateFrame::Euclidean);
                assert_eq!(m.num_triangles(), 1);
                // All three distinct input Z values must survive the full
                // parse->convert chain (guards against Z being zeroed or dropped
                // somewhere in the pipeline).
                let zs: Vec<f64> = m.vertices().iter().map(|v| v[2]).collect();
                for z in [1.0_f64, 2.0, 3.0] {
                    assert!(zs.contains(&z), "z={z} missing from mesh vertices {zs:?}");
                }
            }
            other => panic!("expected Euclidean3D TriangularMesh, got {other:?}"),
        }
    }
}
