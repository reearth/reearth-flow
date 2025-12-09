//! B3DM (Batched 3D Model) encoder for Cesium 3D Tiles.
//!
//! This module provides functionality to wrap GLB (binary glTF) data in the B3DM format,
//! which includes a header, feature table, and batch table for metadata.
//!
//! B3DM format specification:
//! - Header (28 bytes): magic, version, byteLength, featureTableJSONByteLength,
//!   featureTableBinaryByteLength, batchTableJSONByteLength, batchTableBinaryByteLength
//! - Feature Table JSON
//! - Feature Table Binary
//! - Batch Table JSON (optional)
//! - Batch Table Binary (optional)
//! - Binary glTF

use std::collections::HashMap;
use std::io::{self, Write};

use indexmap::IndexMap;
use serde_json::{json, Value as JsonValue};

/// B3DM magic bytes: "b3dm"
const B3DM_MAGIC: &[u8; 4] = b"b3dm";

/// B3DM version
const B3DM_VERSION: u32 = 1;

/// B3DM header size in bytes
const B3DM_HEADER_SIZE: u32 = 28;

/// Output format for 3D Tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileOutputFormat {
    /// GLB (binary glTF) format - modern 3D Tiles 1.1 format
    #[default]
    Glb,
    /// B3DM (Batched 3D Model) format - legacy 3D Tiles 1.0 format
    B3dm,
}

impl TileOutputFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            TileOutputFormat::Glb => "glb",
            TileOutputFormat::B3dm => "b3dm",
        }
    }
}

impl std::str::FromStr for TileOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "glb" => Ok(TileOutputFormat::Glb),
            "b3dm" => Ok(TileOutputFormat::B3dm),
            _ => Err(format!("Unknown output format: {}. Use 'glb' or 'b3dm'", s)),
        }
    }
}

/// Padding to align to 8-byte boundary
fn calculate_padding(size: usize) -> usize {
    (8 - (size % 8)) % 8
}

/// Pad a JSON string to align to 8-byte boundary
fn pad_json(json: &str) -> Vec<u8> {
    let mut bytes = json.as_bytes().to_vec();
    let padding = calculate_padding(bytes.len());
    bytes.extend(std::iter::repeat(b' ').take(padding));
    bytes
}

/// Pad binary data to align to 8-byte boundary
fn pad_binary(data: &[u8]) -> Vec<u8> {
    let mut bytes = data.to_vec();
    let padding = calculate_padding(bytes.len());
    bytes.extend(std::iter::repeat(0u8).take(padding));
    bytes
}

/// Batch Table containing per-feature attributes for B3DM
#[derive(Debug, Clone, Default)]
pub struct BatchTable {
    /// Attributes stored as arrays of values, indexed by feature
    /// Each key maps to an array of values, one per feature
    attributes: IndexMap<String, Vec<JsonValue>>,
}

impl BatchTable {
    /// Create a new empty batch table
    pub fn new() -> Self {
        Self {
            attributes: IndexMap::new(),
        }
    }

    /// Add attributes for a single feature
    pub fn add_feature(&mut self, attributes: &HashMap<String, JsonValue>) {
        for (key, value) in attributes {
            self.attributes
                .entry(key.clone())
                .or_default()
                .push(value.clone());
        }
    }

    /// Check if the batch table is empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Serialize the batch table to JSON
    pub fn to_json(&self) -> String {
        if self.attributes.is_empty() {
            return "{}".to_string();
        }
        let obj: serde_json::Map<String, JsonValue> = self
            .attributes
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect();
        serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Wrap GLB data in B3DM format.
///
/// # Arguments
/// * `writer` - Output writer
/// * `glb_data` - Binary glTF data
/// * `batch_length` - Number of features in the batch (for feature table)
///
/// # Returns
/// The total number of bytes written
pub fn write_b3dm<W: Write>(
    writer: W,
    glb_data: &[u8],
    batch_length: u32,
) -> io::Result<usize> {
    write_b3dm_with_batch_table(writer, glb_data, batch_length, None)
}

/// Wrap GLB data in B3DM format with an optional batch table.
///
/// # Arguments
/// * `writer` - Output writer
/// * `glb_data` - Binary glTF data
/// * `batch_length` - Number of features in the batch (for feature table)
/// * `batch_table` - Optional batch table containing per-feature attributes
///
/// # Returns
/// The total number of bytes written
pub fn write_b3dm_with_batch_table<W: Write>(
    mut writer: W,
    glb_data: &[u8],
    batch_length: u32,
    batch_table: Option<&BatchTable>,
) -> io::Result<usize> {
    // Create feature table JSON with BATCH_LENGTH
    let feature_table_json = if batch_length > 0 {
        format!(r#"{{"BATCH_LENGTH":{}}}"#, batch_length)
    } else {
        r#"{"BATCH_LENGTH":0}"#.to_string()
    };
    let feature_table_json_padded = pad_json(&feature_table_json);
    let feature_table_json_byte_length = feature_table_json_padded.len() as u32;

    // No feature table binary data
    let feature_table_binary_byte_length: u32 = 0;

    // Batch table JSON (if provided)
    let (batch_table_json_padded, batch_table_json_byte_length) = match batch_table {
        Some(bt) if !bt.is_empty() => {
            let json_str = bt.to_json();
            let padded = pad_json(&json_str);
            let len = padded.len() as u32;
            (Some(padded), len)
        }
        _ => (None, 0),
    };
    let batch_table_binary_byte_length: u32 = 0;

    // Calculate total byte length (must be aligned to 8-byte boundary)
    let glb_padded = pad_binary(glb_data);
    let byte_length = B3DM_HEADER_SIZE
        + feature_table_json_byte_length
        + feature_table_binary_byte_length
        + batch_table_json_byte_length
        + batch_table_binary_byte_length
        + glb_padded.len() as u32;

    // Write header (28 bytes)
    writer.write_all(B3DM_MAGIC)?;
    writer.write_all(&B3DM_VERSION.to_le_bytes())?;
    writer.write_all(&byte_length.to_le_bytes())?;
    writer.write_all(&feature_table_json_byte_length.to_le_bytes())?;
    writer.write_all(&feature_table_binary_byte_length.to_le_bytes())?;
    writer.write_all(&batch_table_json_byte_length.to_le_bytes())?;
    writer.write_all(&batch_table_binary_byte_length.to_le_bytes())?;

    // Write feature table JSON
    writer.write_all(&feature_table_json_padded)?;

    // Write batch table JSON (if provided)
    if let Some(batch_table_json) = batch_table_json_padded {
        writer.write_all(&batch_table_json)?;
    }

    // Write GLB data (already padded)
    writer.write_all(&glb_padded)?;

    Ok(byte_length as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_output_format_from_str() {
        assert_eq!("glb".parse::<TileOutputFormat>().unwrap(), TileOutputFormat::Glb);
        assert_eq!("GLB".parse::<TileOutputFormat>().unwrap(), TileOutputFormat::Glb);
        assert_eq!("b3dm".parse::<TileOutputFormat>().unwrap(), TileOutputFormat::B3dm);
        assert_eq!("B3DM".parse::<TileOutputFormat>().unwrap(), TileOutputFormat::B3dm);
        assert!("invalid".parse::<TileOutputFormat>().is_err());
    }

    #[test]
    fn test_extension() {
        assert_eq!(TileOutputFormat::Glb.extension(), "glb");
        assert_eq!(TileOutputFormat::B3dm.extension(), "b3dm");
    }

    #[test]
    fn test_calculate_padding() {
        assert_eq!(calculate_padding(0), 0);
        assert_eq!(calculate_padding(1), 7);
        assert_eq!(calculate_padding(7), 1);
        assert_eq!(calculate_padding(8), 0);
        assert_eq!(calculate_padding(9), 7);
    }

    #[test]
    fn test_write_b3dm_header() {
        // Minimal GLB data (just glTF magic and version)
        let glb_data = b"glTF\x02\x00\x00\x00\x00\x00\x00\x00";
        let mut output = Vec::new();

        write_b3dm(&mut output, glb_data, 1).unwrap();

        // Check magic
        assert_eq!(&output[0..4], b"b3dm");
        // Check version
        assert_eq!(u32::from_le_bytes(output[4..8].try_into().unwrap()), 1);
        // Check byte length matches actual output
        let byte_length = u32::from_le_bytes(output[8..12].try_into().unwrap()) as usize;
        assert_eq!(output.len(), byte_length);
        // Check feature table JSON byte length is 8-byte aligned (internal alignment)
        let feature_table_json_len =
            u32::from_le_bytes(output[12..16].try_into().unwrap()) as usize;
        assert_eq!(feature_table_json_len % 8, 0);
    }

    #[test]
    fn test_write_b3dm_with_zero_batch() {
        let glb_data = b"glTF\x02\x00\x00\x00\x00\x00\x00\x00";
        let mut output = Vec::new();

        write_b3dm(&mut output, glb_data, 0).unwrap();

        // Should still produce valid B3DM
        assert_eq!(&output[0..4], b"b3dm");
        let byte_length = u32::from_le_bytes(output[8..12].try_into().unwrap()) as usize;
        assert_eq!(output.len(), byte_length);
    }

    #[test]
    fn test_batch_table() {
        let mut batch_table = BatchTable::new();
        assert!(batch_table.is_empty());

        let mut attrs1 = HashMap::new();
        attrs1.insert("city_code".to_string(), json!("33100"));
        attrs1.insert("rank".to_string(), json!(1));
        batch_table.add_feature(&attrs1);

        let mut attrs2 = HashMap::new();
        attrs2.insert("city_code".to_string(), json!("33100"));
        attrs2.insert("rank".to_string(), json!(2));
        batch_table.add_feature(&attrs2);

        assert!(!batch_table.is_empty());

        let json = batch_table.to_json();
        assert!(json.contains("city_code"));
        assert!(json.contains("33100"));
        assert!(json.contains("rank"));
    }

    #[test]
    fn test_write_b3dm_with_batch_table() {
        let glb_data = b"glTF\x02\x00\x00\x00\x00\x00\x00\x00";
        let mut batch_table = BatchTable::new();

        let mut attrs = HashMap::new();
        attrs.insert("city_code".to_string(), json!("33100"));
        batch_table.add_feature(&attrs);

        let mut output = Vec::new();
        write_b3dm_with_batch_table(&mut output, glb_data, 1, Some(&batch_table)).unwrap();

        // Check magic
        assert_eq!(&output[0..4], b"b3dm");
        // Check version
        assert_eq!(u32::from_le_bytes(output[4..8].try_into().unwrap()), 1);

        // Check batch table JSON byte length is non-zero
        let batch_table_json_byte_length =
            u32::from_le_bytes(output[20..24].try_into().unwrap()) as usize;
        assert!(batch_table_json_byte_length > 0);

        // Verify total length
        let byte_length = u32::from_le_bytes(output[8..12].try_into().unwrap()) as usize;
        assert_eq!(output.len(), byte_length);
    }
}
