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
    collection::Collection3D,
    coordinate::CoordinateFrame,
    polygon::Polygon3D,
    types::{
        geometry::Geometry3D as OldGeometry3D, line_string::LineString3D as OldLineString3D,
        polygon::Polygon3D as OldPolygon3D,
    },
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
/// the new `reearth_flow_geometry::Geometry`. Faithful port of the old output shape:
/// a single `Polygon` becomes `Euclidean3D::Polygon`, a `MultiPolygon` becomes a
/// `Euclidean3D::Collection` of polygons (the new model has no `MultiPolygon`). Every
/// leaf carries `CoordinateFrame::Euclidean` since glTF is model-space (no CRS).
fn to_new_geometry(old: &OldGeometry3D<f64>) -> Geometry {
    match old {
        OldGeometry3D::Polygon(p) => Geometry::Euclidean3D(old_polygon_to_new(p)),
        OldGeometry3D::MultiPolygon(mp) => Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            Collection3D::new(mp.0.iter().map(old_polygon_to_new)),
        )),
        other => {
            // The glTF extraction only produces Polygon / MultiPolygon today
            // (see reearth_flow_gltf::create_geometry_from_primitives_with_transform).
            tracing::warn!(
                "glTF: unsupported geometry variant for new-geometry conversion, dropping: {other:?}"
            );
            Geometry::None
        }
    }
}

fn old_polygon_to_new(p: &OldPolygon3D<f64>) -> Euclidean3DGeometry {
    let exterior = ring_coords(p.exterior());
    let interiors: Vec<Vec<[f64; 3]>> = p.interiors().iter().map(ring_coords).collect();
    Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
        CoordinateFrame::Euclidean,
        exterior,
        interiors,
    )))
}

fn ring_coords(ls: &OldLineString3D<f64>) -> Vec<[f64; 3]> {
    ls.0.iter().map(|c| [c.x, c.y, c.z]).collect()
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
    fn polygon_converts_to_euclidean3d_polygon_preserving_z() {
        let ext = [[0.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 1.0, 3.0]];
        let old = OldGeometry3D::Polygon(old_polygon(&ext));

        // The old `Polygon3D::new` auto-closes the ring (repeats the first vertex);
        // the faithful conversion preserves that closed ring and its per-vertex z.
        let expected_closed = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 2.0],
            [0.0, 1.0, 3.0],
            [0.0, 0.0, 1.0],
        ];

        match to_new_geometry(&old) {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => {
                assert_eq!(*p.frame(), CoordinateFrame::Euclidean);
                // Z must survive the conversion (glTF meshes carry real per-vertex z).
                assert_eq!(p.exterior(), expected_closed.as_slice());
            }
            other => panic!("expected Euclidean3D Polygon, got {other:?}"),
        }
    }

    #[test]
    fn multipolygon_converts_to_collection3d() {
        let a = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let b = [[2.0, 2.0, 5.0], [3.0, 2.0, 5.0], [2.0, 3.0, 5.0]];
        let old = OldGeometry3D::MultiPolygon(MultiPolygon3D::new(vec![
            old_polygon(&a),
            old_polygon(&b),
        ]));

        match to_new_geometry(&old) {
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) => {
                assert_eq!(c.len(), 2, "both polygons should become collection members");
            }
            other => panic!("expected Euclidean3D Collection, got {other:?}"),
        }
    }
}
