//! PointCloud constructors.
//!
//! `from_positions` builds the basic case: a single acquisition segment of bare
//! XYZ points in `f64` encoding — no optional fields, no user attributes — the
//! shape a plain XYZ reader produces. The richer machinery (RGB / intensity /
//! normals, `f32` or scaled-`i32` encodings, per-point attribute columns, multiple
//! segments) is deferred until a reader needs it.

use std::sync::OnceLock;

use indexmap::IndexMap;
use smallvec::SmallVec;

use crate::coordinate::Coordinate;

use super::{PointCloud, PositionEncoding, Segment};

/// Bytes per point in `f64` XYZ encoding (`3 * size_of::<f64>()`).
const F64_STRIDE: u16 = 24;

impl PointCloud {
    /// Build a point cloud from bare XYZ positions: one `f64`-encoded segment with
    /// no optional fields or attributes.
    pub fn from_positions(
        coordinate: Coordinate,
        positions: impl IntoIterator<Item = [f64; 3]>,
    ) -> Self {
        let positions = positions.into_iter();
        let mut data = Vec::with_capacity(positions.size_hint().0 * F64_STRIDE as usize);
        let mut count = 0usize;
        for [x, y, z] in positions {
            data.extend_from_slice(&x.to_le_bytes());
            data.extend_from_slice(&y.to_le_bytes());
            data.extend_from_slice(&z.to_le_bytes());
            count += 1;
        }
        let mut segments = SmallVec::new();
        segments.push(Segment {
            source: None,
            position: PositionEncoding::F64,
            fields: 0,
            stride: F64_STRIDE,
            offsets: [0; 9],
            data,
            count,
            attributes: IndexMap::new(),
        });
        Self {
            coordinate,
            segments,
            kdtree: OnceLock::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_positions_packs_one_f64_segment() {
        let pc = PointCloud::from_positions(
            Coordinate::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        );
        assert_eq!(pc.segments.len(), 1);
        let seg = &pc.segments[0];
        assert_eq!(seg.count, 3);
        assert_eq!(seg.stride, F64_STRIDE);
        assert_eq!(seg.data.len(), 3 * F64_STRIDE as usize);
        assert_eq!(seg.fields, 0);
        assert!(matches!(seg.position, PositionEncoding::F64));
        assert!(seg.attributes.is_empty());
        // The third point's X decodes back from its little-endian bytes.
        let x2 = f64::from_le_bytes(seg.data[48..56].try_into().unwrap());
        assert_eq!(x2, 4.0);
    }
}
