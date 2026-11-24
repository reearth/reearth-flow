//! Intermediate-data encoding for the point-cloud leaf: typed per-point
//! positions in place of the packed byte stream, tagged with the
//! position encoding, plus one typed column per present optional field.
//! Attribute columns and the acquisition source pass through unchanged.
//!
//! TODO: represent the normal and ring fields. No reader produces them yet, and
//! a buffer carrying them is rejected.

use std::sync::{Arc, OnceLock};

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use reearth_flow_common::uri::Uri;

use crate::coordinate::CoordinateFrame;

use super::file::PointFile;
use super::{AttributeColumn, FieldMask, PositionEncoding, bit};

/// A 3D point cloud: its points in a coordinate frame.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Point cloud"))]
struct PointCloud {
    #[cfg_attr(feature = "schema", schemars(title = "Coordinate frame"))]
    frame: CoordinateFrame,
    #[cfg_attr(feature = "schema", schemars(title = "Data"))]
    data: Data,
    /// Per-axis factor on every stored position.
    #[serde(default = "unit_scale", skip_serializing_if = "is_unit")]
    #[cfg_attr(feature = "schema", schemars(title = "Scale"))]
    scale: [f64; 3],
    /// Added to every stored position after `scale`.
    #[serde(default, skip_serializing_if = "is_zero")]
    #[cfg_attr(feature = "schema", schemars(title = "Translation"))]
    translation: [f64; 3],
}

/// Either `file`, a LAS / LAZ file holding the points, or their positions in
/// the encoding they are stored in, plus any per-point fields and attribute
/// columns.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", schemars(title = "Point data"))]
struct Data {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Option<String>", title = "File"))]
    file: Option<Uri>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Source"))]
    source: Option<Arc<str>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Positions"))]
    positions: Option<Positions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "RGB"))]
    rgb: Option<Vec<[u16; 3]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Intensity"))]
    intensity: Option<Vec<u16>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Timestamp"))]
    timestamp: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Classification"))]
    classification: Option<Vec<u8>>,
    /// Return number in bits 0-3, number of returns in bits 4-7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Return info"))]
    return_info: Option<Vec<u8>>,
    /// In units of 0.006 degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Scan angle"))]
    scan_angle: Option<Vec<i16>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Point source ID"))]
    point_source_id: Option<Vec<u16>>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::HashMap<String, AttributeColumn>")
    )]
    #[cfg_attr(feature = "schema", schemars(title = "Attribute columns"))]
    attributes: IndexMap<String, AttributeColumn>,
}

fn unit_scale() -> [f64; 3] {
    [1.0; 3]
}

fn is_unit(scale: &[f64; 3]) -> bool {
    *scale == [1.0; 3]
}

fn is_zero(translation: &[f64; 3]) -> bool {
    *translation == [0.0; 3]
}

/// Per-point positions in their stored encoding.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
enum Positions {
    F64(Vec<[f64; 3]>),
    F32(Vec<[f32; 3]>),
    ScaledI32 {
        scale: [f64; 3],
        offset: [f64; 3],
        values: Vec<[i32; 3]>,
    },
}

/// Read a buffer's positions out of its packed byte stream.
fn encode_positions(seg: &super::PointBuffer) -> Positions {
    let stride = seg.stride as usize;
    match &seg.position {
        PositionEncoding::F64 => Positions::F64(
            (0..seg.count)
                .map(|i| {
                    let base = i * stride;
                    let read = |o: usize| {
                        f64::from_le_bytes(seg.data[base + o..base + o + 8].try_into().unwrap())
                    };
                    [read(0), read(8), read(16)]
                })
                .collect(),
        ),
        PositionEncoding::F32 => Positions::F32(
            (0..seg.count)
                .map(|i| {
                    let base = i * stride;
                    let read = |o: usize| {
                        f32::from_le_bytes(seg.data[base + o..base + o + 4].try_into().unwrap())
                    };
                    [read(0), read(4), read(8)]
                })
                .collect(),
        ),
        PositionEncoding::ScaledI32 { scale, offset } => Positions::ScaledI32 {
            scale: *scale,
            offset: *offset,
            values: (0..seg.count)
                .map(|i| {
                    let base = i * stride;
                    let read = |o: usize| {
                        i32::from_le_bytes(seg.data[base + o..base + o + 4].try_into().unwrap())
                    };
                    [read(0), read(4), read(8)]
                })
                .collect(),
        },
    }
}

/// Field `bit` of every point, decoded, or `None` if the buffer lacks it.
fn encode_field<const N: usize, T>(
    seg: &super::PointBuffer,
    bit: u16,
    decode: impl Fn([u8; N]) -> T,
) -> Option<Vec<T>> {
    seg.has(bit)
        .then(|| (0..seg.count).map(|i| decode(seg.field(bit, i))).collect())
}

/// Write `values` into field `bit` of every point, if present.
fn decode_field<const N: usize, T>(
    seg: &mut super::PointBuffer,
    bit: u16,
    values: Option<Vec<T>>,
    encode: impl Fn(T) -> [u8; N],
) {
    for (i, v) in values.into_iter().flatten().enumerate() {
        seg.set_field(bit, i, encode(v));
    }
}

fn rgb_from_bytes(b: [u8; 6]) -> [u16; 3] {
    [
        u16::from_le_bytes([b[0], b[1]]),
        u16::from_le_bytes([b[2], b[3]]),
        u16::from_le_bytes([b[4], b[5]]),
    ]
}

fn rgb_to_bytes([r, g, b]: [u16; 3]) -> [u8; 6] {
    let [r0, r1] = r.to_le_bytes();
    let [g0, g1] = g.to_le_bytes();
    let [b0, b1] = b.to_le_bytes();
    [r0, r1, g0, g1, b0, b1]
}

/// Pack positions back into the `(encoding, stride, byte stream, count)` a
/// [`super::PointBuffer`] stores.
fn decode_positions(positions: Positions) -> (PositionEncoding, u16, Vec<u8>, usize) {
    match positions {
        Positions::F64(values) => {
            let mut data = Vec::with_capacity(values.len() * 24);
            for [x, y, z] in &values {
                data.extend_from_slice(&x.to_le_bytes());
                data.extend_from_slice(&y.to_le_bytes());
                data.extend_from_slice(&z.to_le_bytes());
            }
            (PositionEncoding::F64, 24, data, values.len())
        }
        Positions::F32(values) => {
            let mut data = Vec::with_capacity(values.len() * 12);
            for [x, y, z] in &values {
                data.extend_from_slice(&x.to_le_bytes());
                data.extend_from_slice(&y.to_le_bytes());
                data.extend_from_slice(&z.to_le_bytes());
            }
            (PositionEncoding::F32, 12, data, values.len())
        }
        Positions::ScaledI32 {
            scale,
            offset,
            values,
        } => {
            let mut data = Vec::with_capacity(values.len() * 12);
            for [x, y, z] in &values {
                data.extend_from_slice(&x.to_le_bytes());
                data.extend_from_slice(&y.to_le_bytes());
                data.extend_from_slice(&z.to_le_bytes());
            }
            (
                PositionEncoding::ScaledI32 { scale, offset },
                12,
                data,
                values.len(),
            )
        }
    }
}

impl Serialize for super::PointCloud {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let data = match &self.data {
            super::Data::InMemory(buffer) => encode_buffer(buffer)?,
            super::Data::File(file) => Data {
                file: Some(file.uri().clone()),
                source: None,
                positions: None,
                rgb: None,
                intensity: None,
                timestamp: None,
                classification: None,
                return_info: None,
                scan_angle: None,
                point_source_id: None,
                attributes: IndexMap::new(),
            },
        };
        PointCloud {
            frame: self.frame.clone(),
            data,
            scale: self.scale,
            translation: self.translation,
        }
        .serialize(serializer)
    }
}

fn encode_buffer<E: serde::ser::Error>(seg: &super::PointBuffer) -> Result<Data, E> {
    if seg.has(bit::NORMAL) || seg.has(bit::RING) {
        return Err(E::custom(
            "point-cloud normal and ring fields are not yet supported by the \
             intermediate-data encoder",
        ));
    }
    Ok(Data {
        file: None,
        source: seg.source.clone(),
        positions: Some(encode_positions(seg)),
        rgb: encode_field(seg, bit::RGB, rgb_from_bytes),
        intensity: encode_field(seg, bit::INTENSITY, u16::from_le_bytes),
        timestamp: encode_field(seg, bit::TIMESTAMP, f64::from_le_bytes),
        classification: encode_field(seg, bit::CLASSIFICATION, |[b]| b),
        return_info: encode_field(seg, bit::RETURN_INFO, |[b]| b),
        scan_angle: encode_field(seg, bit::SCAN_ANGLE, i16::from_le_bytes),
        point_source_id: encode_field(seg, bit::POINT_SOURCE_ID, u16::from_le_bytes),
        attributes: seg.attributes.clone(),
    })
}

impl<'de> Deserialize<'de> for super::PointCloud {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PointCloud::deserialize(deserializer)?;
        Ok(super::PointCloud {
            frame: wire.frame,
            data: decode_data(wire.data)?,
            scale: wire.scale,
            translation: wire.translation,
            kdtree: OnceLock::new(),
        })
    }
}

fn decode_data<E: serde::de::Error>(s: Data) -> Result<super::Data, E> {
    let Some(uri) = s.file else {
        return decode_buffer(s).map(super::Data::InMemory);
    };
    let inline = s.source.is_some()
        || s.positions.is_some()
        || s.rgb.is_some()
        || s.intensity.is_some()
        || s.timestamp.is_some()
        || s.classification.is_some()
        || s.return_info.is_some()
        || s.scan_angle.is_some()
        || s.point_source_id.is_some()
        || !s.attributes.is_empty();
    if inline {
        return Err(E::custom(
            "point data with `file` cannot also carry inline points, fields or columns",
        ));
    }
    Ok(super::Data::File(PointFile::new(uri)))
}

fn decode_buffer<E: serde::de::Error>(s: Data) -> Result<super::PointBuffer, E> {
    let Some(positions) = s.positions else {
        return Err(E::missing_field("positions"));
    };
    let (position, stride, data, count) = decode_positions(positions);
    // Columns are read by point index, so a column that disagrees with the
    // point count is rejected here rather than left to index out of bounds on
    // the first read.
    let check = |name: &str, len: usize| {
        if len == count {
            Ok(())
        } else {
            Err(E::custom(format!(
                "{name} has {len} entries but there are {count} points"
            )))
        }
    };
    for (name, column) in &s.attributes {
        check(&format!("attribute column `{name}`"), column.len())?;
    }
    let mut fields: FieldMask = 0;
    for (b, name, len) in [
        (bit::RGB, "rgb", s.rgb.as_ref().map(Vec::len)),
        (
            bit::INTENSITY,
            "intensity",
            s.intensity.as_ref().map(Vec::len),
        ),
        (
            bit::TIMESTAMP,
            "timestamp",
            s.timestamp.as_ref().map(Vec::len),
        ),
        (
            bit::CLASSIFICATION,
            "classification",
            s.classification.as_ref().map(Vec::len),
        ),
        (
            bit::RETURN_INFO,
            "return_info",
            s.return_info.as_ref().map(Vec::len),
        ),
        (
            bit::SCAN_ANGLE,
            "scan_angle",
            s.scan_angle.as_ref().map(Vec::len),
        ),
        (
            bit::POINT_SOURCE_ID,
            "point_source_id",
            s.point_source_id.as_ref().map(Vec::len),
        ),
    ] {
        if let Some(len) = len {
            check(&format!("field `{name}`"), len)?;
            fields |= 1 << b;
        }
    }
    if fields == 0 {
        return Ok(super::PointBuffer {
            source: s.source,
            position,
            fields,
            stride,
            offsets: [0; 9],
            data,
            count,
            attributes: s.attributes,
        });
    }
    let mut seg = super::PointBuffer::zeroed(s.source, position, fields, count);
    for (i, bytes) in data.chunks_exact(stride as usize).enumerate() {
        seg.set_position(i, bytes);
    }
    decode_field(&mut seg, bit::RGB, s.rgb, rgb_to_bytes);
    decode_field(&mut seg, bit::INTENSITY, s.intensity, u16::to_le_bytes);
    decode_field(&mut seg, bit::TIMESTAMP, s.timestamp, f64::to_le_bytes);
    decode_field(&mut seg, bit::CLASSIFICATION, s.classification, |b| [b]);
    decode_field(&mut seg, bit::RETURN_INFO, s.return_info, |b| [b]);
    decode_field(&mut seg, bit::SCAN_ANGLE, s.scan_angle, i16::to_le_bytes);
    decode_field(
        &mut seg,
        bit::POINT_SOURCE_ID,
        s.point_source_id,
        u16::to_le_bytes,
    );
    seg.attributes = s.attributes;
    Ok(seg)
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for super::PointCloud {
    fn schema_name() -> String {
        "PointCloud".to_string()
    }
    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        <PointCloud as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::point_cloud::{PointBuffer, PointCloud};

    fn round_trip(pc: &PointCloud) {
        let json = serde_json::to_string(pc).unwrap();
        let back: PointCloud = serde_json::from_str(&json).unwrap();
        assert_eq!(pc, &back);
    }

    #[test]
    fn f64_positions_round_trip() {
        round_trip(&PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [4.0, -1.0, 2.0], [1.0, 0.0, 9.0]],
        ));
    }

    fn packed<const B: usize>(values: &[[[u8; B]; 3]]) -> Vec<u8> {
        let mut data = Vec::new();
        for point in values {
            for component in point {
                data.extend_from_slice(component);
            }
        }
        data
    }

    #[test]
    fn f32_and_scaled_i32_positions_round_trip() {
        let f32_pts = [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        round_trip(&PointCloud {
            frame: CoordinateFrame::Euclidean,
            data: PointBuffer {
                source: Some(Arc::from("scan-a")),
                position: PositionEncoding::F32,
                fields: 0,
                stride: 12,
                offsets: [0; 9],
                data: packed(&f32_pts.map(|p| p.map(f32::to_le_bytes))),
                count: 2,
                attributes: IndexMap::new(),
            }
            .into(),
            scale: [1.0; 3],
            translation: [0.0; 3],
            kdtree: OnceLock::new(),
        });

        let i32_pts = [[10i32, 20, 30], [40, 50, 60]];
        let mut attributes = IndexMap::new();
        attributes.insert(
            "intensity".to_string(),
            AttributeColumn::UInt16(vec![100, 200]),
        );
        attributes.insert(
            "source_id".to_string(),
            AttributeColumn::String(vec![Some(Arc::from("x")), None]),
        );
        round_trip(&PointCloud {
            frame: CoordinateFrame::Euclidean,
            data: PointBuffer {
                source: None,
                position: PositionEncoding::ScaledI32 {
                    scale: [0.001, 0.001, 0.001],
                    offset: [100.0, 200.0, 0.0],
                },
                fields: 0,
                stride: 12,
                offsets: [0; 9],
                data: packed(&i32_pts.map(|p| p.map(i32::to_le_bytes))),
                count: 2,
                attributes,
            }
            .into(),
            scale: [1.0; 3],
            translation: [0.0; 3],
            kdtree: OnceLock::new(),
        });
    }

    #[test]
    fn optional_fields_round_trip() {
        let mut seg = PointBuffer::zeroed(
            Some(Arc::from("a.las")),
            PositionEncoding::ScaledI32 {
                scale: [0.01; 3],
                offset: [5.0, 6.0, 7.0],
            },
            1 << bit::RGB | 1 << bit::TIMESTAMP | 1 << bit::SCAN_ANGLE | 1 << bit::POINT_SOURCE_ID,
            2,
        );
        for i in 0..2 {
            let n = i as i32 + 1;
            let mut pos = [0u8; 12];
            pos[0..4].copy_from_slice(&n.to_le_bytes());
            pos[4..8].copy_from_slice(&(-n).to_le_bytes());
            pos[8..12].copy_from_slice(&(10 * n).to_le_bytes());
            seg.set_position(i, &pos);
            seg.set_field(bit::RGB, i, rgb_to_bytes([n as u16, 2, 65535]));
            seg.set_field(bit::TIMESTAMP, i, (0.5 * n as f64).to_le_bytes());
            seg.set_field(bit::SCAN_ANGLE, i, (-1000 * n as i16).to_le_bytes());
            seg.set_field(bit::POINT_SOURCE_ID, i, (80 + n as u16).to_le_bytes());
        }
        seg.attributes
            .insert("user".to_string(), AttributeColumn::UInt8(vec![0, 1]));
        let pc = PointCloud {
            frame: CoordinateFrame::Euclidean,
            data: seg.into(),
            scale: [1.0; 3],
            translation: [0.0; 3],
            kdtree: OnceLock::new(),
        };
        let json = serde_json::to_value(&pc).unwrap();
        assert_eq!(
            json["data"]["rgb"],
            serde_json::json!([[1, 2, 65535], [2, 2, 65535]])
        );
        assert_eq!(
            json["data"]["scan_angle"],
            serde_json::json!([-1000, -2000])
        );
        assert!(json["data"].get("intensity").is_none());
        round_trip(&pc);
    }

    #[test]
    fn file_data_round_trips_as_its_uri() {
        let mut pc = PointCloud::from_las_file(
            CoordinateFrame::Euclidean,
            Uri::from_str("file:///data/a.las").unwrap(),
        );
        crate::ops::Translate::translate(&mut pc, [1.0, 0.0, -2.0]).unwrap();
        pc.scale = [2.0, 2.0, 1.0];
        let json = serde_json::to_value(&pc).unwrap();
        assert_eq!(
            json["data"],
            serde_json::json!({"file": "file:///data/a.las"})
        );
        assert_eq!(json["translation"], serde_json::json!([1.0, 0.0, -2.0]));
        assert_eq!(json["scale"], serde_json::json!([2.0, 2.0, 1.0]));
        round_trip(&pc);
    }

    /// Columns are read by point index, so a short one would index out of
    /// bounds on the first read; it is rejected while decoding instead.
    #[test]
    fn short_attribute_column_is_rejected() {
        let json = r#"{
            "frame": "Euclidean",
            "data": {
                "positions": {"F64": [[0.0,0.0,0.0],[1.0,1.0,1.0],[2.0,2.0,2.0]]},
                "attributes": {"intensity": {"UInt16": [100, 200]}}
            }
        }"#;
        let err = serde_json::from_str::<PointCloud>(json)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("attribute column `intensity` has 2 entries but there are 3 points"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn matching_attribute_column_is_accepted() {
        let json = r#"{
            "frame": "Euclidean",
            "data": {
                "positions": {"F64": [[0.0,0.0,0.0],[1.0,1.0,1.0]]},
                "attributes": {"intensity": {"UInt16": [100, 200]}}
            }
        }"#;
        let pc: PointCloud = serde_json::from_str(json).unwrap();
        round_trip(&pc);
    }
}
