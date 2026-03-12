use std::{collections::HashMap, str::FromStr, sync::Arc};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::{geometry::Geometry3D, multi_polygon::MultiPolygon3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::{
    errors::SourceError,
    file::reader::runner::{get_content, FileReaderCommonParam},
};

#[derive(Debug, Clone, Default)]
pub(crate) struct GltfReaderFactory;

impl SourceFactory for GltfReaderFactory {
    fn name(&self) -> &str {
        "GltfReader"
    }

    fn description(&self) -> &str {
        "Reads 3D models from glTF 2.0 files, supporting meshes, nodes, scenes, and geometry primitives"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GltfReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File", "3D"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::GltfReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::GltfReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::GltfReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reader = GltfReader { params };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GltfReader {
    params: GltfReaderParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GltfReaderParam {
    #[serde(flatten)]
    pub(super) common: FileReaderCommonParam,
    /// # Triangulate
    /// If true, converts all primitives to triangles (reserved for future use - currently all primitives are processed as triangles)
    #[serde(default = "default_true")]
    pub(super) triangulate: bool,
    /// # Merge Meshes
    /// If true, combines all meshes from the glTF file into a single output feature
    #[serde(default)]
    pub(super) merge_meshes: bool,
    /// # Include Nodes
    /// If true, includes node hierarchy information from the glTF scene graph in feature attributes
    #[serde(default = "default_true")]
    pub(super) include_nodes: bool,
}

fn default_true() -> bool {
    true
}

struct MeshInfo<'a> {
    primitives: Vec<gltf::Primitive<'a>>,
    mesh_name: Option<String>,
    node_name: Option<String>,
    transform: reearth_flow_gltf::Transform,
}

#[async_trait::async_trait]
impl Source for GltfReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "GltfReader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let content = get_content(&ctx, &self.params.common, storage_resolver.clone()).await?;

        read_gltf(&ctx, storage_resolver, &content, &self.params, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}

async fn read_gltf(
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    content: &Bytes,
    params: &GltfReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    let gltf_uri = if let Some(dataset) = &params.common.dataset {
        Uri::from_str(dataset.to_string().trim_matches('"'))
            .unwrap_or_else(|_| Uri::from_str("file://./unknown.gltf").unwrap())
    } else {
        Uri::from_str("file://./unknown.gltf").unwrap()
    };

    let gltf = gltf::Gltf::from_slice(content)
        .map_err(|e| SourceError::GltfReader(format!("Failed to parse glTF: {e}")))?;

    let buffer_data = load_buffers(&gltf, ctx, storage_resolver, &gltf_uri, content).await?;

    // Collect lightweight mesh info with transforms (just traversal, no heavy geometry processing)
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

    // Process and send features based on merge_meshes setting
    if !params.merge_meshes {
        // Process and send each geometry immediately without collecting
        for mesh_info in mesh_infos {
            let geometry = reearth_flow_gltf::create_geometry_from_primitives_with_transform(
                &mesh_info.primitives,
                &buffer_data,
                Some(&mesh_info.transform),
            )
            .map_err(|e| SourceError::GltfReader(format!("Failed to create geometry: {e}")))?;

            let mesh_names = mesh_info.mesh_name.map(|n| vec![n]).unwrap_or_default();
            let node_names = mesh_info.node_name.map(|n| vec![n]).unwrap_or_default();

            send_feature(
                &sender,
                geometry,
                &mesh_names,
                &node_names,
                mesh_info.primitives.len(),
                params,
            )
            .await?;
        }
    } else if !mesh_infos.is_empty() {
        let mut geometries = Vec::new();
        let mut all_mesh_names = std::collections::HashSet::new();
        let mut all_node_names = std::collections::HashSet::new();
        let mut total_primitives = 0;

        for mesh_info in mesh_infos {
            let geometry = reearth_flow_gltf::create_geometry_from_primitives_with_transform(
                &mesh_info.primitives,
                &buffer_data,
                Some(&mesh_info.transform),
            )
            .map_err(|e| SourceError::GltfReader(format!("Failed to create geometry: {e}")))?;

            geometries.push(geometry);
            if let Some(name) = mesh_info.mesh_name {
                all_mesh_names.insert(name);
            }
            if let Some(name) = mesh_info.node_name {
                all_node_names.insert(name);
            }
            total_primitives += mesh_info.primitives.len();
        }

        let merged_geometry = merge_geometries(geometries.iter().collect());
        let merged_mesh_names: Vec<String> = all_mesh_names.into_iter().collect();
        let merged_node_names: Vec<String> = all_node_names.into_iter().collect();

        send_feature(
            &sender,
            merged_geometry,
            &merged_mesh_names,
            &merged_node_names,
            total_primitives,
            params,
        )
        .await?;
    }

    Ok(())
}

fn merge_geometries(geometries: Vec<&Geometry3D<f64>>) -> Geometry3D<f64> {
    let mut all_polygons = Vec::new();

    for geometry in geometries {
        match geometry {
            Geometry3D::Polygon(polygon) => {
                all_polygons.push(polygon.clone());
            }
            Geometry3D::MultiPolygon(multipolygon) => {
                all_polygons.extend(multipolygon.0.iter().cloned());
            }
            _ => {
                // currently reearth-flow-gltf only creates Polygon and MultiPolygon geometries
                tracing::warn!("Unsupported geometry type for merging: {:?}", geometry);
            }
        }
    }

    if all_polygons.len() == 1 {
        Geometry3D::Polygon(all_polygons.into_iter().next().unwrap())
    } else {
        Geometry3D::MultiPolygon(MultiPolygon3D::new(all_polygons))
    }
}

async fn send_feature(
    sender: &Sender<(Port, IngestionMessage)>,
    flow_geometry: Geometry3D<f64>,
    mesh_names: &[String],
    node_names: &[String],
    primitive_count: usize,
    params: &GltfReaderParam,
) -> Result<(), SourceError> {
    let geometry = Geometry::with_value(GeometryValue::FlowGeometry3D(flow_geometry));
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
        attributes.insert(
            Attribute::new(key),
            if mesh_names.len() == 1 {
                AttributeValue::String(mesh_names[0].clone())
            } else {
                AttributeValue::Array(
                    mesh_names
                        .iter()
                        .map(|m| AttributeValue::String(m.clone()))
                        .collect(),
                )
            },
        );
    }

    if params.include_nodes && !node_names.is_empty() {
        let key = if node_names.len() == 1 {
            "node"
        } else {
            "nodes"
        };
        attributes.insert(
            Attribute::new(key),
            if node_names.len() == 1 {
                AttributeValue::String(node_names[0].clone())
            } else {
                AttributeValue::Array(
                    node_names
                        .iter()
                        .map(|n| AttributeValue::String(n.clone()))
                        .collect(),
                )
            },
        );
    }

    attributes.insert(
        Attribute::new("primitiveCount"),
        AttributeValue::Number(serde_json::Number::from(primitive_count)),
    );

    let feature =
        Feature::new_with_attributes_and_geometry(attributes, geometry, Default::default());

    sender
        .send((
            DEFAULT_PORT.clone(),
            IngestionMessage::OperationEvent { feature },
        ))
        .await
        .map_err(|e| SourceError::GltfReader(format!("Failed to send feature: {e}")))?;

    Ok(())
}

async fn load_buffers(
    gltf: &gltf::Gltf,
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    base_uri: &Uri,
    _content: &Bytes,
) -> Result<Vec<Vec<u8>>, SourceError> {
    let mut buffer_data = Vec::new();

    for buffer in gltf.buffers() {
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => {
                let blob = gltf
                    .blob
                    .as_ref()
                    .ok_or_else(|| SourceError::GltfReader("GLB blob not found".to_string()))?;
                blob.clone()
            }
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    decode_data_uri(uri)?
                } else {
                    load_external_buffer(ctx, storage_resolver.clone(), base_uri, uri).await?
                }
            }
        };

        buffer_data.push(data);
    }

    Ok(buffer_data)
}

fn decode_data_uri(uri: &str) -> Result<Vec<u8>, SourceError> {
    let data_prefix = "data:";
    if !uri.starts_with(data_prefix) {
        return Err(SourceError::GltfReader(format!("Invalid data URI: {uri}")));
    }

    let uri = &uri[data_prefix.len()..];
    let parts: Vec<&str> = uri.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(SourceError::GltfReader(format!(
            "Invalid data URI format: {uri}"
        )));
    }

    let metadata = parts[0];
    let data = parts[1];

    if metadata.contains("base64") {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| SourceError::GltfReader(format!("Failed to decode base64 data: {e}")))
    } else {
        Ok(data.as_bytes().to_vec())
    }
}

async fn load_external_buffer(
    _ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    base_uri: &Uri,
    buffer_uri: &str,
) -> Result<Vec<u8>, SourceError> {
    let buffer_uri_str = if let Some(slash_pos) = base_uri.to_string().rfind('/') {
        format!("{}/{}", &base_uri.to_string()[..slash_pos], buffer_uri)
    } else {
        buffer_uri.to_string()
    };

    let uri = Uri::from_str(&buffer_uri_str)
        .map_err(|e| SourceError::GltfReader(format!("Invalid buffer URI: {e}")))?;

    let storage = storage_resolver
        .resolve(&uri)
        .map_err(|e| SourceError::GltfReader(format!("Failed to resolve buffer storage: {e}")))?;

    let result = storage
        .get(&uri.path())
        .await
        .map_err(|e| SourceError::GltfReader(format!("Failed to read buffer file: {e}")))?;

    let content = result
        .bytes()
        .await
        .map_err(|e| SourceError::GltfReader(format!("Failed to read buffer content: {e}")))?;

    Ok(content.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64_data_uri() {
        let uri = "data:application/octet-stream;base64,AAABAAIAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAACAPwAAAAA=";
        let result = decode_data_uri(uri).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_decode_plain_data_uri() {
        let uri = "data:text/plain,Hello%20World";
        let result = decode_data_uri(uri).unwrap();
        assert_eq!(result, b"Hello%20World");
    }
}
