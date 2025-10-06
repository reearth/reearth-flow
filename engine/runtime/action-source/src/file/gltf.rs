use std::{collections::HashMap, str::FromStr, sync::Arc};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::{
    coordinate::Coordinate, geometry::Geometry3D as FlowGeometry3D, multi_polygon::MultiPolygon3D,
    polygon::Polygon3D,
};
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
                SourceError::FileReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::FileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::FileReaderFactory(
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

    if params.merge_meshes {
        let mut all_primitives = Vec::new();
        let mut mesh_names = Vec::new();
        let mut node_names = Vec::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                collect_primitives(
                    &node,
                    &gltf,
                    &mut all_primitives,
                    &mut mesh_names,
                    &mut node_names,
                );
            }
        }

        if !all_primitives.is_empty() {
            let geometry = create_geometry_from_primitives(&all_primitives, &buffer_data, params)?;
            let mut attributes = IndexMap::new();

            attributes.insert(
                Attribute::new("source"),
                AttributeValue::String("glTF".to_string()),
            );

            if !mesh_names.is_empty() {
                attributes.insert(
                    Attribute::new("meshes"),
                    AttributeValue::Array(
                        mesh_names
                            .iter()
                            .map(|m| AttributeValue::String(m.clone()))
                            .collect(),
                    ),
                );
            }

            if params.include_nodes && !node_names.is_empty() {
                attributes.insert(
                    Attribute::new("nodes"),
                    AttributeValue::Array(
                        node_names
                            .iter()
                            .map(|n| AttributeValue::String(n.clone()))
                            .collect(),
                    ),
                );
            }

            attributes.insert(
                Attribute::new("primitiveCount"),
                AttributeValue::Number(serde_json::Number::from(all_primitives.len())),
            );

            let feature = Feature {
                geometry,
                attributes,
                ..Default::default()
            };

            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| SourceError::GltfReader(format!("Failed to send feature: {e}")))?;
        }
    } else {
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                process_node(&node, &gltf, &buffer_data, params, &sender).await?;
            }
        }
    }

    Ok(())
}

fn collect_primitives<'a>(
    node: &gltf::Node<'a>,
    _gltf: &gltf::Gltf,
    primitives: &mut Vec<gltf::Primitive<'a>>,
    mesh_names: &mut Vec<String>,
    node_names: &mut Vec<String>,
) {
    if let Some(node_name) = node.name() {
        if !node_names.contains(&node_name.to_string()) {
            node_names.push(node_name.to_string());
        }
    }

    if let Some(mesh) = node.mesh() {
        if let Some(mesh_name) = mesh.name() {
            if !mesh_names.contains(&mesh_name.to_string()) {
                mesh_names.push(mesh_name.to_string());
            }
        }

        for primitive in mesh.primitives() {
            primitives.push(primitive);
        }
    }

    for child in node.children() {
        collect_primitives(&child, _gltf, primitives, mesh_names, node_names);
    }
}

async fn process_node<'a>(
    node: &gltf::Node<'a>,
    _gltf: &gltf::Gltf,
    buffer_data: &[Vec<u8>],
    params: &GltfReaderParam,
    sender: &Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    if let Some(mesh) = node.mesh() {
        let primitives: Vec<_> = mesh.primitives().collect();

        if !primitives.is_empty() {
            let geometry = create_geometry_from_primitives(&primitives, buffer_data, params)?;
            let mut attributes = IndexMap::new();

            attributes.insert(
                Attribute::new("source"),
                AttributeValue::String("glTF".to_string()),
            );

            if let Some(mesh_name) = mesh.name() {
                attributes.insert(
                    Attribute::new("mesh"),
                    AttributeValue::String(mesh_name.to_string()),
                );
            }

            if params.include_nodes {
                if let Some(node_name) = node.name() {
                    attributes.insert(
                        Attribute::new("node"),
                        AttributeValue::String(node_name.to_string()),
                    );
                }
            }

            attributes.insert(
                Attribute::new("primitiveCount"),
                AttributeValue::Number(serde_json::Number::from(primitives.len())),
            );

            let feature = Feature {
                geometry,
                attributes,
                ..Default::default()
            };

            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| SourceError::GltfReader(format!("Failed to send feature: {e}")))?;
        }
    }

    for child in node.children() {
        Box::pin(process_node(&child, _gltf, buffer_data, params, sender)).await?;
    }

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

fn create_geometry_from_primitives(
    primitives: &[gltf::Primitive],
    buffer_data: &[Vec<u8>],
    _params: &GltfReaderParam,
) -> Result<Geometry, SourceError> {
    let mut polygons = Vec::new();

    for primitive in primitives {
        let position_accessor = primitive
            .get(&gltf::Semantic::Positions)
            .ok_or_else(|| SourceError::GltfReader("Primitive has no positions".to_string()))?;

        let positions = read_positions(&position_accessor, buffer_data)?;

        if let Some(indices_accessor) = primitive.indices() {
            let indices = read_indices(&indices_accessor, buffer_data)?;

            match primitive.mode() {
                gltf::mesh::Mode::Triangles => {
                    for chunk in indices.chunks(3) {
                        if chunk.len() == 3 {
                            let triangle = vec![
                                positions[chunk[0]],
                                positions[chunk[1]],
                                positions[chunk[2]],
                                positions[chunk[0]], // Close the ring
                            ];
                            polygons.push(Polygon3D::new(triangle.into(), vec![]));
                        }
                    }
                }
                gltf::mesh::Mode::TriangleStrip => {
                    for i in 0..indices.len().saturating_sub(2) {
                        let triangle = if i % 2 == 0 {
                            vec![
                                positions[indices[i]],
                                positions[indices[i + 1]],
                                positions[indices[i + 2]],
                                positions[indices[i]], // Close the ring
                            ]
                        } else {
                            vec![
                                positions[indices[i]],
                                positions[indices[i + 2]],
                                positions[indices[i + 1]],
                                positions[indices[i]], // Close the ring
                            ]
                        };
                        polygons.push(Polygon3D::new(triangle.into(), vec![]));
                    }
                }
                gltf::mesh::Mode::TriangleFan => {
                    for i in 1..indices.len().saturating_sub(1) {
                        let triangle = vec![
                            positions[indices[0]],
                            positions[indices[i]],
                            positions[indices[i + 1]],
                            positions[indices[0]], // Close the ring
                        ];
                        polygons.push(Polygon3D::new(triangle.into(), vec![]));
                    }
                }
                _ => {
                    return Err(SourceError::GltfReader(format!(
                        "Unsupported primitive mode: {:?}",
                        primitive.mode()
                    )))
                }
            }
        } else {
            // Non-indexed primitives
            match primitive.mode() {
                gltf::mesh::Mode::Triangles => {
                    for chunk in positions.chunks(3) {
                        if chunk.len() == 3 {
                            let triangle = vec![chunk[0], chunk[1], chunk[2], chunk[0]];
                            polygons.push(Polygon3D::new(triangle.into(), vec![]));
                        }
                    }
                }
                _ => {
                    return Err(SourceError::GltfReader(format!(
                        "Unsupported non-indexed primitive mode: {:?}",
                        primitive.mode()
                    )))
                }
            }
        }
    }

    let flow_geometry = if polygons.len() == 1 {
        FlowGeometry3D::Polygon(polygons.into_iter().next().unwrap())
    } else {
        FlowGeometry3D::MultiPolygon(MultiPolygon3D::new(polygons))
    };

    let geometry = Geometry::with_value(GeometryValue::FlowGeometry3D(flow_geometry));

    Ok(geometry)
}

fn read_positions(
    accessor: &gltf::Accessor,
    buffer_data: &[Vec<u8>],
) -> Result<Vec<Coordinate>, SourceError> {
    let view = accessor.view().ok_or_else(|| {
        SourceError::GltfReader("Position accessor has no buffer view".to_string())
    })?;

    let buffer = &buffer_data[view.buffer().index()];
    let start = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(accessor.size());

    let mut positions = Vec::new();

    match accessor.data_type() {
        gltf::accessor::DataType::F32 => {
            if accessor.dimensions() != gltf::accessor::Dimensions::Vec3 {
                return Err(SourceError::GltfReader(
                    "Position accessor must be Vec3".to_string(),
                ));
            }

            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let x = read_f32(buffer, offset)?;
                let y = read_f32(buffer, offset + 4)?;
                let z = read_f32(buffer, offset + 8)?;

                positions.push(Coordinate {
                    x: x as f64,
                    y: y as f64,
                    z: z as f64,
                });
            }
        }
        _ => {
            return Err(SourceError::GltfReader(format!(
                "Unsupported position data type: {:?}",
                accessor.data_type()
            )))
        }
    }

    Ok(positions)
}

fn read_indices(
    accessor: &gltf::Accessor,
    buffer_data: &[Vec<u8>],
) -> Result<Vec<usize>, SourceError> {
    let view = accessor
        .view()
        .ok_or_else(|| SourceError::GltfReader("Index accessor has no buffer view".to_string()))?;

    let buffer = &buffer_data[view.buffer().index()];
    let start = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(accessor.size());

    let mut indices = Vec::new();

    match accessor.data_type() {
        gltf::accessor::DataType::U16 => {
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let idx = read_u16(buffer, offset)?;
                indices.push(idx as usize);
            }
        }
        gltf::accessor::DataType::U32 => {
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let idx = read_u32(buffer, offset)?;
                indices.push(idx as usize);
            }
        }
        gltf::accessor::DataType::U8 => {
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let idx = buffer
                    .get(offset)
                    .ok_or_else(|| SourceError::GltfReader("Index out of bounds".to_string()))?;
                indices.push(*idx as usize);
            }
        }
        _ => {
            return Err(SourceError::GltfReader(format!(
                "Unsupported index data type: {:?}",
                accessor.data_type()
            )))
        }
    }

    Ok(indices)
}

fn read_f32(buffer: &[u8], offset: usize) -> Result<f32, SourceError> {
    let bytes = buffer
        .get(offset..offset + 4)
        .ok_or_else(|| SourceError::GltfReader("Buffer read out of bounds".to_string()))?;

    let mut array = [0u8; 4];
    array.copy_from_slice(bytes);
    Ok(f32::from_le_bytes(array))
}

fn read_u16(buffer: &[u8], offset: usize) -> Result<u16, SourceError> {
    let bytes = buffer
        .get(offset..offset + 2)
        .ok_or_else(|| SourceError::GltfReader("Buffer read out of bounds".to_string()))?;

    let mut array = [0u8; 2];
    array.copy_from_slice(bytes);
    Ok(u16::from_le_bytes(array))
}

fn read_u32(buffer: &[u8], offset: usize) -> Result<u32, SourceError> {
    let bytes = buffer
        .get(offset..offset + 4)
        .ok_or_else(|| SourceError::GltfReader("Buffer read out of bounds".to_string()))?;

    let mut array = [0u8; 4];
    array.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(array))
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
