//! Lossless intermediate-data encoding for the point-cloud leaf.
//!
//! The wire form presents each segment's packed little-endian byte stream decoded
//! into typed per-point positions, keeping the position encoding
//! (`F64` / `F32` / scaled-`i32`) so the exact stored bytes are recovered on
//! decode. User attribute columns and the acquisition source pass through as-is;
//! their `IndexMap` order is preserved. Decoding recomputes the stride and repacks
//! the byte stream.
//!
//! Optional per-point fields (RGB, intensity, ...) are not yet produced by any
//! reader, so a segment carrying them is rejected rather than silently lost; the
//! wire form must grow a typed representation for them when they land.

use std::sync::{Arc, OnceLock};

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;

use crate::coordinate::CoordinateFrame;

use super::{AttributeColumn, PointCloud, PositionEncoding, Segment};

/// Decoded wire form of a [`PointCloud`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct PointCloudWire {
    frame: CoordinateFrame,
    segments: Vec<SegmentWire>,
}

/// Decoded wire form of one acquisition [`Segment`].
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
struct SegmentWire {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<Arc<str>>,
    positions: PositionsWire,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::HashMap<String, AttributeColumn>")
    )]
    attributes: IndexMap<String, AttributeColumn>,
}

/// Per-point positions in their stored encoding. Raw `i32` values (with the
/// segment's scale / offset) are kept rather than the derived `f64`, so a scaled
/// segment round-trips byte-for-byte.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize)]
enum PositionsWire {
    F64(Vec<[f64; 3]>),
    F32(Vec<[f32; 3]>),
    ScaledI32 {
        scale: [f64; 3],
        offset: [f64; 3],
        values: Vec<[i32; 3]>,
    },
}

/// Decode a segment's positions out of its packed byte stream.
fn encode_positions(seg: &Segment) -> PositionsWire {
    let stride = seg.stride as usize;
    match &seg.position {
        PositionEncoding::F64 => PositionsWire::F64(
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
        PositionEncoding::F32 => PositionsWire::F32(
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
        PositionEncoding::ScaledI32 { scale, offset } => PositionsWire::ScaledI32 {
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

/// Repack decoded positions into the `(encoding, stride, byte stream, count)` a
/// [`Segment`] stores.
fn decode_positions(positions: PositionsWire) -> (PositionEncoding, u16, Vec<u8>, usize) {
    match positions {
        PositionsWire::F64(values) => {
            let mut data = Vec::with_capacity(values.len() * 24);
            for [x, y, z] in &values {
                data.extend_from_slice(&x.to_le_bytes());
                data.extend_from_slice(&y.to_le_bytes());
                data.extend_from_slice(&z.to_le_bytes());
            }
            (PositionEncoding::F64, 24, data, values.len())
        }
        PositionsWire::F32(values) => {
            let mut data = Vec::with_capacity(values.len() * 12);
            for [x, y, z] in &values {
                data.extend_from_slice(&x.to_le_bytes());
                data.extend_from_slice(&y.to_le_bytes());
                data.extend_from_slice(&z.to_le_bytes());
            }
            (PositionEncoding::F32, 12, data, values.len())
        }
        PositionsWire::ScaledI32 {
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

impl Serialize for PointCloud {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut segments = Vec::with_capacity(self.segments.len());
        for seg in &self.segments {
            if seg.fields != 0 {
                return Err(serde::ser::Error::custom(
                    "point-cloud optional fields are not yet supported by the intermediate-data encoder",
                ));
            }
            segments.push(SegmentWire {
                source: seg.source.clone(),
                positions: encode_positions(seg),
                attributes: seg.attributes.clone(),
            });
        }
        PointCloudWire {
            frame: self.frame.clone(),
            segments,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PointCloud {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PointCloudWire::deserialize(deserializer)?;
        let segments: SmallVec<[Segment; 1]> = wire
            .segments
            .into_iter()
            .map(|s| {
                let (position, stride, data, count) = decode_positions(s.positions);
                Segment {
                    source: s.source,
                    position,
                    fields: 0,
                    stride,
                    offsets: [0; 9],
                    data,
                    count,
                    attributes: s.attributes,
                }
            })
            .collect();
        Ok(PointCloud {
            frame: wire.frame,
            segments,
            kdtree: OnceLock::new(),
        })
    }
}

// The intermediate-data schema is the wire form, so the leaf's schema is its
// wire struct's.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for PointCloud {
    fn schema_name() -> String {
        "PointCloud".to_string()
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <PointCloudWire as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(pc: &PointCloud) {
        let json = serde_json::to_string(pc).unwrap();
        let back: PointCloud = serde_json::from_str(&json).unwrap();
        assert_eq!(pc, &back);
    }

    #[test]
    fn f64_segment_round_trips() {
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
    fn f32_and_scaled_i32_multi_segment_round_trip() {
        let f32_pts = [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let f32_data = packed(
            &f32_pts
                .iter()
                .map(|p| p.map(f32::to_le_bytes))
                .collect::<Vec<_>>(),
        );
        let f32_seg = Segment {
            source: Some(Arc::from("scan-a")),
            position: PositionEncoding::F32,
            fields: 0,
            stride: 12,
            offsets: [0; 9],
            data: f32_data,
            count: 2,
            attributes: IndexMap::new(),
        };

        let i32_pts = [[10i32, 20, 30], [40, 50, 60]];
        let i32_data = packed(
            &i32_pts
                .iter()
                .map(|p| p.map(i32::to_le_bytes))
                .collect::<Vec<_>>(),
        );
        let mut attributes = IndexMap::new();
        attributes.insert(
            "intensity".to_string(),
            AttributeColumn::UInt16(vec![100, 200]),
        );
        attributes.insert(
            "source_id".to_string(),
            AttributeColumn::String(vec![Some(Arc::from("x")), None]),
        );
        let i32_seg = Segment {
            source: None,
            position: PositionEncoding::ScaledI32 {
                scale: [0.001, 0.001, 0.001],
                offset: [100.0, 200.0, 0.0],
            },
            fields: 0,
            stride: 12,
            offsets: [0; 9],
            data: i32_data,
            count: 2,
            attributes,
        };

        let mut segments: SmallVec<[Segment; 1]> = SmallVec::new();
        segments.push(f32_seg);
        segments.push(i32_seg);
        round_trip(&PointCloud {
            frame: CoordinateFrame::Euclidean,
            segments,
            kdtree: OnceLock::new(),
        });
    }
}
