use std::sync::OnceLock;

use super::{AttributeColumn, Data, PointBuffer, PointCloud, PositionEncoding};
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
        let buffer = self.data.buffer("split")?;
        for (i, position) in buffer_positions(buffer).enumerate() {
            let mut attributes = Attributes::with_capacity(buffer.attributes.len());
            for (name, column) in &buffer.attributes {
                attributes.insert(Attribute::new(name.clone()), column_value(column, i));
            }
            let point = Point3D::new(self.frame.clone(), position);
            emit(
                Geometry::Euclidean3D(Euclidean3DGeometry::Point(point)),
                attributes,
            );
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
    /// A file's extent is read from its header, not its points.
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let unbounded = UnsupportedOperation {
            geometry: "PointCloud",
            operation: "bounding_box",
        };
        let file = match &self.data {
            Data::InMemory(buffer) => {
                return Aabb::from_points_3d(buffer_positions(buffer)).ok_or(unbounded);
            }
            Data::File(file) => file,
        };
        let header = file.header().map_err(|_| UnsupportedOperation {
            geometry: "PointCloud (unreadable file)",
            ..unbounded
        })?;
        if header.count == 0 {
            return Err(unbounded);
        }
        let swap = super::las::swaps_xy(&self.frame).map_err(|_| unbounded)?;
        let corners = [header.min, header.max].map(|mut corner| {
            if swap {
                corner.swap(0, 1);
            }
            for (axis, d) in corner.iter_mut().zip(file.shift()) {
                *axis += d;
            }
            corner
        });
        Aabb::from_points_3d(corners).ok_or(unbounded)
    }
}

impl Translate for PointCloud {
    /// Shift every point, keeping the position encoding. `F32` positions
    /// therefore stay at `f32` precision, which the shifted coordinates must
    /// still be representable in.
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        match &mut self.data {
            Data::InMemory(buffer) => translate_buffer(buffer, delta),
            Data::File(file) => file.translate(delta),
        }
        self.kdtree = OnceLock::new();
        Ok(())
    }
}

/// Add `delta` to every position in a buffer. Scaled-integer positions move
/// the decode offset and leave the packed bytes untouched; float positions are
/// rewritten in place, stride by stride.
fn translate_buffer(seg: &mut PointBuffer, delta: [f64; 3]) {
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

#[cfg(feature = "new-geometry")]
impl crate::ops::SetElevation for PointCloud {
    /// Keeps the position encoding, as [`Translate`] does.
    fn set_elevation(&mut self, z: f64) -> Result<(), UnsupportedOperation> {
        set_buffer_elevation(self.data.buffer_mut("set_elevation")?, z);
        self.kdtree = OnceLock::new();
        Ok(())
    }
}

/// Set every position's z in a buffer. Scaled-integer positions pack a zero z
/// and move `z` into the decode offset, so the elevation stays exact.
#[cfg(feature = "new-geometry")]
fn set_buffer_elevation(seg: &mut PointBuffer, z: f64) {
    let stride = seg.stride as usize;
    match &mut seg.position {
        PositionEncoding::ScaledI32 { offset, .. } => {
            offset[2] = z;
            for point in 0..seg.count {
                let at = point * stride + 8;
                seg.data[at..at + 4].copy_from_slice(&0i32.to_le_bytes());
            }
        }
        PositionEncoding::F64 => {
            for point in 0..seg.count {
                let at = point * stride + 16;
                seg.data[at..at + 8].copy_from_slice(&z.to_le_bytes());
            }
        }
        PositionEncoding::F32 => {
            for point in 0..seg.count {
                let at = point * stride + 8;
                seg.data[at..at + 4].copy_from_slice(&(z as f32).to_le_bytes());
            }
        }
    }
}

/// Decode every point's XYZ from a buffer's packed little-endian stride. The
/// position occupies the first bytes of each stride; the encoding fixes the
/// width and any scale/offset. Reads go through `from_le_bytes`, so a bad
/// offset is a bounds panic, never UB (mirrors the field-access contract).
pub(super) fn buffer_positions(seg: &PointBuffer) -> impl Iterator<Item = [f64; 3]> + '_ {
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

#[cfg(feature = "new-geometry")]
use crate::ops::Elevation;

#[cfg(feature = "new-geometry")]
impl Elevation for PointCloud {
    /// `None` while the points are still in a file.
    fn elevation(&self) -> Option<f64> {
        match &self.data {
            Data::InMemory(buffer) => buffer_positions(buffer).next().map(|p| p[2]),
            Data::File(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use indexmap::IndexMap;
    use kiddo::ImmutableKdTree;

    /// A cloud holding `data` under `position`, with no optional fields or
    /// attribute columns.
    fn cloud(position: PositionEncoding, stride: u16, count: usize, data: Vec<u8>) -> PointCloud {
        PointCloud {
            frame: CoordinateFrame::Euclidean,
            data: Data::InMemory(PointBuffer {
                source: None,
                position,
                fields: 0,
                stride,
                offsets: [0; 9],
                data,
                count,
                attributes: IndexMap::new(),
            }),
            kdtree: OnceLock::new(),
        }
    }

    fn scaled_i32_cloud(scale: [f64; 3], offset: [f64; 3], raw: &[[i32; 3]]) -> PointCloud {
        let mut data = Vec::new();
        for point in raw {
            for axis in point {
                data.extend_from_slice(&axis.to_le_bytes());
            }
        }
        cloud(
            PositionEncoding::ScaledI32 { scale, offset },
            12,
            raw.len(),
            data,
        )
    }

    fn f32_cloud(positions: &[[f32; 3]]) -> PointCloud {
        let mut data = Vec::new();
        for point in positions {
            for axis in point {
                data.extend_from_slice(&axis.to_le_bytes());
            }
        }
        cloud(PositionEncoding::F32, 12, positions.len(), data)
    }

    fn positions(pc: &PointCloud) -> Vec<[f64; 3]> {
        buffer_positions(pc.data.as_buffer()).collect()
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
        let mut pc = scaled_i32_cloud([0.001; 3], [0.0; 3], &[[1000, 2000, 3000]]);
        let packed = pc.data.as_buffer().data.clone();
        pc.translate([0.5, 0.0, -0.25]).unwrap();
        assert_eq!(positions(&pc), [[1.5, 2.0, 2.75]]);
        assert_eq!(pc.data.as_buffer().data, packed);
    }

    #[test]
    fn f32_positions_shift_in_place() {
        let mut pc = f32_cloud(&[[1.5, 2.5, 3.5]]);
        // Exactly representable in `f32`, so the round trip is lossless.
        pc.translate([0.25, 0.5, -1.0]).unwrap();
        assert_eq!(positions(&pc), [[1.75, 3.0, 2.5]]);
        assert!(matches!(
            pc.data.as_buffer().position,
            PositionEncoding::F32
        ));
    }

    #[test]
    fn translate_drops_the_kdtree_cache() {
        let mut pc = PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]);
        pc.kdtree
            .set(ImmutableKdTree::new_from_slice(&[[0.0, 0.0, 0.0]]).unwrap())
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

    #[cfg(feature = "new-geometry")]
    #[test]
    fn every_encoding_takes_the_elevation() {
        use crate::ops::SetElevation;

        for (mut pc, expected) in [
            (
                PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 1.0]]),
                [0.0, 0.0, 2.5],
            ),
            (
                scaled_i32_cloud([0.001; 3], [10.0; 3], &[[5000, 6000, 7000]]),
                [15.0, 16.0, 2.5],
            ),
            (f32_cloud(&[[9.0, 9.0, 9.0]]), [9.0, 9.0, 2.5]),
        ] {
            pc.kdtree
                .set(ImmutableKdTree::new_from_slice(&[[0.0, 0.0, 0.0]]).unwrap())
                .unwrap();
            pc.set_elevation(2.5).unwrap();
            assert_eq!(positions(&pc), [expected]);
            assert!(pc.kdtree.get().is_none());
        }
    }
}
