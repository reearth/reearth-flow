use super::{AttributeColumn, PointCloud, PositionEncoding, Segment};
use crate::ops::{Aabb, BoundingBox, Split, UnsupportedOperation};
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
