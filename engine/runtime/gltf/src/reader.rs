use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::coordinate::Coordinate;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GltfReaderError {
    #[error("GLTF parsing error: {0}")]
    Parse(String),
    #[error("Buffer error: {0}")]
    Buffer(String),
    #[error("Accessor error: {0}")]
    Accessor(String),
}

/// Load all buffers from a GLTF file (binary blobs, external URIs, data URIs)
pub async fn load_buffers(
    gltf: &gltf::Gltf,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    base_uri: &Uri,
) -> Result<Vec<Vec<u8>>, GltfReaderError> {
    let mut buffer_data = Vec::new();

    for buffer in gltf.buffers() {
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => gltf
                .blob
                .as_ref()
                .ok_or_else(|| GltfReaderError::Buffer("GLB blob not found".to_string()))?
                .clone(),
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    decode_data_uri(uri)?
                } else {
                    load_external_buffer(storage_resolver.clone(), base_uri, uri).await?
                }
            }
        };

        buffer_data.push(data);
    }

    Ok(buffer_data)
}

/// Decode base64 or plain data URIs
pub fn decode_data_uri(uri: &str) -> Result<Vec<u8>, GltfReaderError> {
    let data_prefix = "data:";
    if !uri.starts_with(data_prefix) {
        return Err(GltfReaderError::Buffer(format!("Invalid data URI: {uri}")));
    }

    let uri = &uri[data_prefix.len()..];
    let parts: Vec<&str> = uri.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(GltfReaderError::Buffer(format!(
            "Invalid data URI format: {uri}"
        )));
    }

    let metadata = parts[0];
    let data = parts[1];

    if metadata.contains("base64") {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| GltfReaderError::Buffer(format!("Failed to decode base64 data: {e}")))
    } else {
        Ok(data.as_bytes().to_vec())
    }
}

async fn load_external_buffer(
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    base_uri: &Uri,
    buffer_uri: &str,
) -> Result<Vec<u8>, GltfReaderError> {
    let buffer_uri_str = if let Some(slash_pos) = base_uri.to_string().rfind('/') {
        format!("{}/{}", &base_uri.to_string()[..slash_pos], buffer_uri)
    } else {
        buffer_uri.to_string()
    };

    let uri = Uri::from_str(&buffer_uri_str)
        .map_err(|e| GltfReaderError::Buffer(format!("Invalid buffer URI: {e}")))?;

    let storage = storage_resolver
        .resolve(&uri)
        .map_err(|e| GltfReaderError::Buffer(format!("Failed to resolve buffer storage: {e}")))?;

    let result = storage
        .get(&uri.path())
        .await
        .map_err(|e| GltfReaderError::Buffer(format!("Failed to read buffer file: {e}")))?;

    let content = result
        .bytes()
        .await
        .map_err(|e| GltfReaderError::Buffer(format!("Failed to read buffer content: {e}")))?;

    Ok(content.to_vec())
}

/// Read Vec3 positions from an accessor
pub fn read_positions(
    accessor: &gltf::Accessor,
    buffer_data: &[Vec<u8>],
) -> Result<Vec<Coordinate>, GltfReaderError> {
    let view = accessor.view().ok_or_else(|| {
        GltfReaderError::Accessor("Position accessor has no buffer view".to_string())
    })?;

    let buffer = &buffer_data[view.buffer().index()];
    let start = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(accessor.size());

    let mut positions = Vec::new();

    match accessor.data_type() {
        gltf::accessor::DataType::F32 => {
            if accessor.dimensions() != gltf::accessor::Dimensions::Vec3 {
                return Err(GltfReaderError::Accessor(
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
            return Err(GltfReaderError::Accessor(format!(
                "Unsupported position data type: {:?}",
                accessor.data_type()
            )))
        }
    }

    Ok(positions)
}

/// Read triangle indices from an accessor (supports U8, U16, U32)
pub fn read_indices(
    accessor: &gltf::Accessor,
    buffer_data: &[Vec<u8>],
) -> Result<Vec<usize>, GltfReaderError> {
    let view = accessor.view().ok_or_else(|| {
        GltfReaderError::Accessor("Index accessor has no buffer view".to_string())
    })?;

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
                let idx = buffer.get(offset).ok_or_else(|| {
                    GltfReaderError::Accessor("Index out of bounds".to_string())
                })?;
                indices.push(*idx as usize);
            }
        }
        _ => {
            return Err(GltfReaderError::Accessor(format!(
                "Unsupported index data type: {:?}",
                accessor.data_type()
            )))
        }
    }

    Ok(indices)
}

pub fn read_f32(buffer: &[u8], offset: usize) -> Result<f32, GltfReaderError> {
    let bytes = buffer.get(offset..offset + 4).ok_or_else(|| {
        GltfReaderError::Buffer("Buffer read out of bounds".to_string())
    })?;

    let mut array = [0u8; 4];
    array.copy_from_slice(bytes);
    Ok(f32::from_le_bytes(array))
}

pub fn read_u16(buffer: &[u8], offset: usize) -> Result<u16, GltfReaderError> {
    let bytes = buffer.get(offset..offset + 2).ok_or_else(|| {
        GltfReaderError::Buffer("Buffer read out of bounds".to_string())
    })?;

    let mut array = [0u8; 2];
    array.copy_from_slice(bytes);
    Ok(u16::from_le_bytes(array))
}

pub fn read_u32(buffer: &[u8], offset: usize) -> Result<u32, GltfReaderError> {
    let bytes = buffer.get(offset..offset + 4).ok_or_else(|| {
        GltfReaderError::Buffer("Buffer read out of bounds".to_string())
    })?;

    let mut array = [0u8; 4];
    array.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(array))
}

/// Parse GLTF from bytes
pub fn parse_gltf(content: &Bytes) -> Result<gltf::Gltf, GltfReaderError> {
    gltf::Gltf::from_slice_without_validation(content)
        .map_err(|e| GltfReaderError::Parse(format!("Failed to parse glTF: {e}")))
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
