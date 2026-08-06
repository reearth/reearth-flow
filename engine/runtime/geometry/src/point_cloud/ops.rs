use std::sync::OnceLock;

use super::{AttributeColumn, PointCloud, PositionEncoding, Segment};
use crate::coordinate::CoordinateFrame;
use crate::ops::{Aabb, Affine3, BoundingBox, Place, Split, Translate, UnsupportedOperation};
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

impl Place for PointCloud {
    /// Apply `affine` to every point of every segment, keeping each segment's
    /// position encoding, then set the frame.
    ///
    /// A `ScaledI32` segment decodes as `raw * scale + offset`: a pure shift
    /// (what `Translate` needs) fits that shape by moving `offset` alone, but
    /// a rotation does not — there is no `scale`/`offset` pair that encodes a
    /// rotated point without first decoding every point to a float, rotating,
    /// and requantizing, which this primitive does not do. So a `ScaledI32`
    /// segment is rejected. `F64` and `F32` segments have no such obstruction:
    /// each point is decoded to `[f64; 3]`, rotated and translated by
    /// `affine.apply`, and re-encoded in place, exactly mirroring how
    /// `translate_segment` already rewrites those two encodings.
    ///
    /// The encoding check runs before any segment is mutated, so a rejected
    /// cloud (one carrying a `ScaledI32` segment) is left untouched.
    fn place(&mut self, affine: &Affine3, frame: &CoordinateFrame) -> crate::error::Result<()> {
        if self
            .segments
            .iter()
            .any(|seg| matches!(seg.position, PositionEncoding::ScaledI32 { .. }))
        {
            return Err(crate::error::Error::invalid_geometry(
                "cannot place a PointCloud segment encoded as ScaledI32: a rotation cannot be \
                 represented in a scaled-integer encoding",
            ));
        }
        for seg in self.segments.iter_mut() {
            place_segment(seg, affine);
        }
        self.kdtree = OnceLock::new();
        self.frame = frame.clone();
        Ok(())
    }
}

/// Apply `affine` to every position in a `F64` or `F32` segment, decoding,
/// transforming, and re-encoding each point in place. Never called on a
/// `ScaledI32` segment: `Place::place` rejects those before any segment is
/// mutated.
fn place_segment(seg: &mut Segment, affine: &Affine3) {
    let stride = seg.stride as usize;
    match &seg.position {
        PositionEncoding::F64 => {
            for point in 0..seg.count {
                let base = point * stride;
                let p = [
                    f64::from_le_bytes(seg.data[base..base + 8].try_into().unwrap()),
                    f64::from_le_bytes(seg.data[base + 8..base + 16].try_into().unwrap()),
                    f64::from_le_bytes(seg.data[base + 16..base + 24].try_into().unwrap()),
                ];
                let out = affine.apply(p);
                seg.data[base..base + 8].copy_from_slice(&out[0].to_le_bytes());
                seg.data[base + 8..base + 16].copy_from_slice(&out[1].to_le_bytes());
                seg.data[base + 16..base + 24].copy_from_slice(&out[2].to_le_bytes());
            }
        }
        PositionEncoding::F32 => {
            for point in 0..seg.count {
                let base = point * stride;
                let p = [
                    f32::from_le_bytes(seg.data[base..base + 4].try_into().unwrap()) as f64,
                    f32::from_le_bytes(seg.data[base + 4..base + 8].try_into().unwrap()) as f64,
                    f32::from_le_bytes(seg.data[base + 8..base + 12].try_into().unwrap()) as f64,
                ];
                let out = affine.apply(p);
                seg.data[base..base + 4].copy_from_slice(&(out[0] as f32).to_le_bytes());
                seg.data[base + 4..base + 8].copy_from_slice(&(out[1] as f32).to_le_bytes());
                seg.data[base + 8..base + 12].copy_from_slice(&(out[2] as f32).to_le_bytes());
            }
        }
        PositionEncoding::ScaledI32 { .. } => {
            unreachable!("Place::place rejects ScaledI32 segments before mutating any segment")
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

    /// Row-major matrix that maps (x, y, z) -> (x, -z, y): the Y-up to Z-up flip.
    fn y_up_to_z_up() -> [[f64; 3]; 3] {
        [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]]
    }

    #[test]
    fn f64_point_cloud_is_placed_and_the_frame_is_set() {
        use crate::coordinate::EpsgCode;

        let mut pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [4.0, -1.0, 2.0]],
        );
        let affine = Affine3::new(y_up_to_z_up(), [10.0, 20.0, 30.0]);
        let target = CoordinateFrame::Crs(EpsgCode::new(4978));
        pc.place(&affine, &target).unwrap();

        assert_eq!(positions(&pc), [[10.0, 18.0, 31.0], [14.0, 18.0, 29.0]]);
        assert_eq!(pc.frame, target);
    }

    #[test]
    fn f32_point_cloud_is_placed_and_the_frame_is_set() {
        use crate::coordinate::EpsgCode;

        let mut pc = PointCloud {
            frame: CoordinateFrame::Euclidean,
            segments: smallvec::smallvec![f32_segment(&[[0.0, 1.0, 2.0]])],
            kdtree: OnceLock::new(),
        };
        let affine = Affine3::new(y_up_to_z_up(), [10.0, 20.0, 30.0]);
        let target = CoordinateFrame::Crs(EpsgCode::new(4978));
        pc.place(&affine, &target).unwrap();

        assert_eq!(positions(&pc), [[10.0, 18.0, 31.0]]);
        assert_eq!(pc.frame, target);
        assert!(matches!(pc.segments[0].position, PositionEncoding::F32));
    }

    #[test]
    fn scaled_i32_point_cloud_place_is_rejected_and_leaves_the_cloud_untouched() {
        let seg = scaled_i32_segment([0.001; 3], [0.0; 3], &[[1000, 2000, 3000]]);
        let packed = seg.data.clone();
        let mut pc = PointCloud {
            frame: CoordinateFrame::Euclidean,
            segments: smallvec::smallvec![seg],
            kdtree: OnceLock::new(),
        };
        let before = positions(&pc);
        let result = pc.place(&Affine3::identity(), &CoordinateFrame::Euclidean);

        assert!(result.is_err());
        // Rejected before any segment is mutated: packed bytes, decoded
        // positions, and frame are all exactly as they started.
        assert_eq!(pc.segments[0].data, packed);
        assert_eq!(positions(&pc), before);
        assert_eq!(pc.frame, CoordinateFrame::Euclidean);
    }
}
