use std::sync::OnceLock;

use super::{AttributeColumn, PointCloud, PositionEncoding, Segment};
use crate::ops::{Aabb, BoundingBox, Split, Translate, UnsupportedOperation};
use crate::point::Point3D;
use crate::{Euclidean3DGeometry, Geometry};
use reearth_flow_common::attribute::{Attribute, AttributeValue, Attributes};
use serde_json::Number;

impl Split for PointCloud {
    /// Decode every point as a [`Point3D`] in the cloud's frame, each paired with
    /// its per-point attributes gathered from the typed attribute columns (empty
    /// when the point carries none).
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        for seg in &self.segments {
            for (i, position) in segment_positions(seg).enumerate() {
                let mut attributes = Attributes::with_capacity(seg.attributes.len());
                for (name, column) in &seg.attributes {
                    attributes.insert(Attribute::new(name.clone()), column_value(column, i));
                }
                let point = Point3D::new(self.frame.clone(), position);
                emit(
                    Geometry::Euclidean3D(Euclidean3DGeometry::Point(point)),
                    attributes,
                );
            }
        }
        Ok(())
    }
}

/// Decode one typed column entry into an [`AttributeValue`]. A non-finite float
/// or an unassigned string becomes [`AttributeValue::Null`].
fn column_value(column: &AttributeColumn, i: usize) -> AttributeValue {
    match column {
        AttributeColumn::UInt8(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::UInt16(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::UInt32(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::UInt64(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::Int8(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::Int16(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::Int32(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::Int64(v) => AttributeValue::Number(Number::from(v[i])),
        AttributeColumn::Float32(v) => number_or_null(v[i] as f64),
        AttributeColumn::Float64(v) => number_or_null(v[i]),
        AttributeColumn::String(v) => v[i].as_ref().map_or(AttributeValue::Null, |s| {
            AttributeValue::String(s.to_string())
        }),
    }
}

/// A finite `f64` as a number attribute; `NaN`/infinite becomes null.
fn number_or_null(x: f64) -> AttributeValue {
    Number::from_f64(x).map_or(AttributeValue::Null, AttributeValue::Number)
}

impl BoundingBox for PointCloud {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let points = self.segments.iter().flat_map(segment_positions);
        Aabb::from_points_3d(points).ok_or(UnsupportedOperation {
            geometry: "PointCloud",
            operation: "bounding_box",
        })
    }
}

impl Translate for PointCloud {
    /// Shift every point of every segment, keeping each segment's position
    /// encoding. An `F32` segment therefore stays at `f32` precision, which the
    /// shifted coordinates must still be representable in.
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        for seg in self.segments.iter_mut() {
            translate_segment(seg, delta);
        }
        self.kdtree = OnceLock::new();
        Ok(())
    }
}

/// Add `delta` to every position in a segment. A scaled-integer segment moves
/// its decode offset and leaves the packed bytes untouched; a float segment is
/// rewritten in place, stride by stride.
fn translate_segment(seg: &mut Segment, delta: [f64; 3]) {
    let stride = seg.stride as usize;
    match &mut seg.position {
        PositionEncoding::ScaledI32 { offset, .. } => {
            for (axis, d) in offset.iter_mut().zip(delta) {
                *axis += d;
            }
        }
        PositionEncoding::F64 => {
            for point in 0..seg.count {
                for (axis, d) in delta.iter().enumerate() {
                    let at = point * stride + axis * 8;
                    let shifted = f64::from_le_bytes(seg.data[at..at + 8].try_into().unwrap()) + d;
                    seg.data[at..at + 8].copy_from_slice(&shifted.to_le_bytes());
                }
            }
        }
        PositionEncoding::F32 => {
            for point in 0..seg.count {
                for (axis, d) in delta.iter().enumerate() {
                    let at = point * stride + axis * 4;
                    let shifted =
                        f32::from_le_bytes(seg.data[at..at + 4].try_into().unwrap()) as f64 + d;
                    seg.data[at..at + 4].copy_from_slice(&(shifted as f32).to_le_bytes());
                }
            }
        }
    }
}

/// Decode every point's XYZ from a segment's packed little-endian stride. The
/// position occupies the first bytes of each stride; the encoding fixes the
/// width and any scale/offset. Reads go through `from_le_bytes`, so a bad
/// offset is a bounds panic, never UB (mirrors the field-access contract).
pub(super) fn segment_positions(seg: &Segment) -> impl Iterator<Item = [f64; 3]> + '_ {
    let stride = seg.stride as usize;
    (0..seg.count).map(move |i| {
        let base = i * stride;
        match &seg.position {
            PositionEncoding::F64 => {
                let r = |o: usize| {
                    f64::from_le_bytes(seg.data[base + o..base + o + 8].try_into().unwrap())
                };
                [r(0), r(8), r(16)]
            }
            PositionEncoding::F32 => {
                let r = |o: usize| {
                    f32::from_le_bytes(seg.data[base + o..base + o + 4].try_into().unwrap()) as f64
                };
                [r(0), r(4), r(8)]
            }
            PositionEncoding::ScaledI32 { scale, offset } => {
                let r = |o: usize| {
                    i32::from_le_bytes(seg.data[base + o..base + o + 4].try_into().unwrap()) as f64
                };
                [
                    r(0) * scale[0] + offset[0],
                    r(4) * scale[1] + offset[1],
                    r(8) * scale[2] + offset[2],
                ]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use indexmap::IndexMap;
    use kiddo::ImmutableKdTree;

    /// A segment holding `data` under `position`, with no optional fields or
    /// attribute columns.
    fn segment(position: PositionEncoding, stride: u16, count: usize, data: Vec<u8>) -> Segment {
        Segment {
            source: None,
            position,
            fields: 0,
            stride,
            offsets: [0; 9],
            data,
            count,
            attributes: IndexMap::new(),
        }
    }

    fn scaled_i32_segment(scale: [f64; 3], offset: [f64; 3], raw: &[[i32; 3]]) -> Segment {
        let mut data = Vec::new();
        for point in raw {
            for axis in point {
                data.extend_from_slice(&axis.to_le_bytes());
            }
        }
        segment(
            PositionEncoding::ScaledI32 { scale, offset },
            12,
            raw.len(),
            data,
        )
    }

    fn f32_segment(positions: &[[f32; 3]]) -> Segment {
        let mut data = Vec::new();
        for point in positions {
            for axis in point {
                data.extend_from_slice(&axis.to_le_bytes());
            }
        }
        segment(PositionEncoding::F32, 12, positions.len(), data)
    }

    fn positions(pc: &PointCloud) -> Vec<[f64; 3]> {
        pc.segments.iter().flat_map(segment_positions).collect()
    }

    #[test]
    fn f64_positions_shift() {
        let mut pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [4.0, -1.0, 2.0]],
        );
        pc.translate([10.0, 20.0, 30.0]).unwrap();
        assert_eq!(positions(&pc), [[10.0, 21.0, 32.0], [14.0, 19.0, 32.0]]);
    }

    #[test]
    fn scaled_integer_positions_shift_without_rewriting_the_packed_bytes() {
        let seg = scaled_i32_segment([0.001; 3], [0.0; 3], &[[1000, 2000, 3000]]);
        let packed = seg.data.clone();
        let mut pc = PointCloud {
            frame: CoordinateFrame::Euclidean,
            segments: smallvec::smallvec![seg],
            kdtree: OnceLock::new(),
        };
        pc.translate([0.5, 0.0, -0.25]).unwrap();
        assert_eq!(positions(&pc), [[1.5, 2.0, 2.75]]);
        assert_eq!(pc.segments[0].data, packed);
    }

    #[test]
    fn f32_positions_shift_in_place() {
        let mut pc = PointCloud {
            frame: CoordinateFrame::Euclidean,
            segments: smallvec::smallvec![f32_segment(&[[1.5, 2.5, 3.5]])],
            kdtree: OnceLock::new(),
        };
        // Exactly representable in `f32`, so the round trip is lossless.
        pc.translate([0.25, 0.5, -1.0]).unwrap();
        assert_eq!(positions(&pc), [[1.75, 3.0, 2.5]]);
        assert!(matches!(pc.segments[0].position, PositionEncoding::F32));
    }

    #[test]
    fn every_segment_shifts() {
        let f64_seg = PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]])
            .segments
            .remove(0);
        let mut pc = PointCloud {
            frame: CoordinateFrame::Euclidean,
            segments: smallvec::smallvec![
                f64_seg,
                scaled_i32_segment([1.0; 3], [0.0; 3], &[[5, 5, 5]]),
                f32_segment(&[[9.0, 9.0, 9.0]]),
            ],
            kdtree: OnceLock::new(),
        };
        pc.translate([1.0, 1.0, 1.0]).unwrap();
        assert_eq!(
            positions(&pc),
            [[1.0, 1.0, 1.0], [6.0, 6.0, 6.0], [10.0, 10.0, 10.0]]
        );
    }

    #[test]
    fn translate_drops_the_kdtree_cache() {
        let mut pc = PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]);
        pc.kdtree
            .set(ImmutableKdTree::new_from_slice(&[[0.0, 0.0, 0.0]]))
            .unwrap();
        pc.translate([1.0, 0.0, 0.0]).unwrap();
        assert!(pc.kdtree.get().is_none());
    }

    #[test]
    fn point_cloud_box_spans_all_points() {
        let pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [4.0, -1.0, 2.0], [1.0, 0.0, 9.0]],
        );
        assert_eq!(
            pc.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, -1.0, 2.0],
                max: [4.0, 1.0, 9.0]
            }
        );
    }

    #[test]
    fn empty_point_cloud_has_no_box() {
        let pc = PointCloud::from_positions(CoordinateFrame::Euclidean, Vec::<[f64; 3]>::new());
        assert!(pc.bounding_box().is_err());
    }
}
