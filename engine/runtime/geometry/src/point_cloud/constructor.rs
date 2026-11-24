//! PointCloud constructors.
//!
//! `from_positions` builds the basic case: bare XYZ points in `f64` encoding,
//! with no optional fields or user attributes.
//! LAS / LAZ input is read by `PointCloud::from_las`.

use indexmap::IndexMap;

use crate::coordinate::CoordinateFrame;

use super::{Data, PointBuffer, PointCloud, PositionEncoding};

/// Bytes per point in `f64` XYZ encoding (`3 * size_of::<f64>()`).
const F64_STRIDE: u16 = 24;

impl PointCloud {
    /// Build a point cloud from bare XYZ positions, `f64`-encoded with no
    /// optional fields or attributes.
    pub fn from_positions(
        frame: CoordinateFrame,
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
        let data = Data::InMemory(PointBuffer {
            source: None,
            position: PositionEncoding::F64,
            fields: 0,
            stride: F64_STRIDE,
            offsets: [0; 9],
            data,
            count,
            attributes: IndexMap::new(),
        });
        Self::new(frame, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_positions_packs_f64_points() {
        let pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        );
        let seg = pc.data.as_buffer();
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
