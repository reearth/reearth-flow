//! LAS / LAZ reading.

use std::io::{Read, Seek};
use std::sync::{Arc, OnceLock};

use smallvec::smallvec;

use crate::coordinate::CoordinateFrame;
use crate::error::{Error, Result};

use super::{PointCloud, PositionEncoding, Segment, bit};

/// Degrees per unit of the stored scan angle.
const SCAN_ANGLE_UNIT: f64 = 0.006;

/// Points decoded per read.
const BATCH: u64 = 65_536;

impl PointCloud {
    /// Read a LAS or LAZ stream into one segment in its native scaled-integer
    /// encoding.
    ///
    /// Keeps RGB, intensity, GPS time, classification, return number and number
    /// of returns, scan angle and point source ID; other LAS fields are dropped.
    /// A legacy (formats 0-5) scan angle is rounded to 0.006 degrees. `frame` is
    /// the frame of the file's coordinates; the file's own CRS records are not
    /// read.
    pub fn from_las<R: Read + Seek + Send + Sync + 'static>(
        frame: CoordinateFrame,
        source: Option<Arc<str>>,
        read: R,
    ) -> Result<Self> {
        let mut reader = ::las::Reader::new(read).map_err(Error::point_cloud_read)?;
        let header = reader.header();
        let format = *header.point_format();
        let t = *header.transforms();
        let count = usize::try_from(header.number_of_points())
            .map_err(|_| Error::point_cloud_read("point count exceeds the address space"))?;
        let mut batch = ::las::PointDataBuilder::new().for_header(header).build();

        let mut fields = 1 << bit::INTENSITY
            | 1 << bit::CLASSIFICATION
            | 1 << bit::RETURN_INFO
            | 1 << bit::SCAN_ANGLE
            | 1 << bit::POINT_SOURCE_ID;
        if format.has_color {
            fields |= 1 << bit::RGB;
        }
        if format.has_gps_time {
            fields |= 1 << bit::TIMESTAMP;
        }
        let position = PositionEncoding::ScaledI32 {
            scale: [t.x.scale, t.y.scale, t.z.scale],
            offset: [t.x.offset, t.y.offset, t.z.offset],
        };
        let mut seg = Segment::zeroed(source, position, fields, count);

        let mut start = 0;
        while start < count {
            let n = reader
                .fill_points(BATCH, &mut batch)
                .map_err(Error::point_cloud_read)? as usize;
            if n == 0 {
                return Err(Error::point_cloud_read(format!(
                    "header declares {count} points but the stream ended after {start}"
                )));
            }
            copy_batch(&mut seg, start, &batch);
            start += n;
        }

        Ok(PointCloud {
            frame,
            segments: smallvec![seg],
            kdtree: OnceLock::new(),
        })
    }
}

/// Copy the points of `batch` into `seg` from point `start` on.
fn copy_batch(seg: &mut Segment, start: usize, batch: &::las::PointData) {
    let xyz = batch.x_raw().zip(batch.y_raw()).zip(batch.z_raw());
    for (i, ((x, y), z)) in (start..).zip(xyz) {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&x.to_le_bytes());
        bytes[4..8].copy_from_slice(&y.to_le_bytes());
        bytes[8..12].copy_from_slice(&z.to_le_bytes());
        seg.set_position(i, &bytes);
    }
    for (i, v) in (start..).zip(batch.intensity()) {
        seg.set_field(bit::INTENSITY, i, v.to_le_bytes());
    }
    for (i, v) in (start..).zip(batch.classification()) {
        seg.set_field(bit::CLASSIFICATION, i, [v]);
    }
    let returns = batch.return_number().zip(batch.number_of_returns());
    for (i, (number, of)) in (start..).zip(returns) {
        seg.set_field(bit::RETURN_INFO, i, [number | of << 4]);
    }
    for (i, degrees) in (start..).zip(batch.scan_angle_degrees()) {
        let units = (f64::from(degrees) / SCAN_ANGLE_UNIT).round() as i16;
        seg.set_field(bit::SCAN_ANGLE, i, units.to_le_bytes());
    }
    for (i, v) in (start..).zip(batch.point_source_id()) {
        seg.set_field(bit::POINT_SOURCE_ID, i, v.to_le_bytes());
    }
    if let Some(times) = batch.gps_time() {
        for (i, v) in (start..).zip(times) {
            seg.set_field(bit::TIMESTAMP, i, v.to_le_bytes());
        }
    }
    if let Some(colors) = batch.rgb() {
        for (i, (r, g, b)) in (start..).zip(colors) {
            let mut bytes = [0u8; 6];
            bytes[0..2].copy_from_slice(&r.to_le_bytes());
            bytes[2..4].copy_from_slice(&g.to_le_bytes());
            bytes[4..6].copy_from_slice(&b.to_le_bytes());
            seg.set_field(bit::RGB, i, bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use ::las::point::{Classification, Format};
    use ::las::{Builder, Color, Point, Transform, Vector, Writer};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::point_cloud::ops::segment_positions;

    const T: Transform = Transform {
        scale: 0.001,
        offset: 1000.0,
    };

    fn point(i: u8) -> Point {
        Point {
            x: 1000.0 + f64::from(i) * 0.5,
            y: -2000.0 + f64::from(i),
            z: 30.125 + f64::from(i),
            intensity: 100 + u16::from(i),
            return_number: i % 3 + 1,
            number_of_returns: 3,
            classification: Classification::new(i + 1).unwrap(),
            scan_angle: -6.0 + f32::from(i),
            point_source_id: 80 + u16::from(i),
            gps_time: Some(1.0e8 + f64::from(i)),
            color: Some(Color::new(257 * u16::from(i), 1, 65535)),
            ..Default::default()
        }
    }

    fn write(format: u8, compressed: bool, points: &[Point]) -> Cursor<Vec<u8>> {
        let mut builder = Builder::from((1, 4));
        let mut f = Format::new(format).unwrap();
        f.is_compressed = compressed;
        builder.point_format = f;
        builder.transforms = Vector { x: T, y: T, z: T };
        let header = builder.into_header().unwrap();
        let mut writer = Writer::new(Cursor::new(Vec::new()), header).unwrap();
        for p in points {
            writer.write_point(p.clone()).unwrap();
        }
        let mut out = writer.into_inner().unwrap();
        out.set_position(0);
        out
    }

    fn check(pc: &PointCloud, points: &[Point], scan_angles: &[i16]) {
        let seg = &pc.segments[0];
        assert_eq!(seg.count, points.len());
        let positions: Vec<_> = segment_positions(seg).collect();
        for (i, p) in points.iter().enumerate() {
            let [x, y, z] = positions[i];
            assert!((x - p.x).abs() < 1e-9 && (y - p.y).abs() < 1e-9 && (z - p.z).abs() < 1e-9);
            assert_eq!(seg.field(bit::INTENSITY, i), p.intensity.to_le_bytes());
            assert_eq!(
                seg.field(bit::CLASSIFICATION, i),
                [u8::from(p.classification)]
            );
            assert_eq!(
                seg.field(bit::RETURN_INFO, i),
                [p.return_number | p.number_of_returns << 4]
            );
            assert_eq!(seg.field(bit::SCAN_ANGLE, i), scan_angles[i].to_le_bytes());
            assert_eq!(
                seg.field(bit::POINT_SOURCE_ID, i),
                p.point_source_id.to_le_bytes()
            );
        }
    }

    #[test]
    fn reads_las_format_3() {
        let points: Vec<_> = (0..5).map(point).collect();
        let pc = PointCloud::from_las(
            CoordinateFrame::Euclidean,
            Some(Arc::from("a.las")),
            write(3, false, &points),
        )
        .unwrap();
        let seg = &pc.segments[0];
        assert_eq!(seg.source.as_deref(), Some("a.las"));
        assert_eq!(
            seg.position,
            PositionEncoding::ScaledI32 {
                scale: [0.001; 3],
                offset: [1000.0; 3]
            }
        );
        // Every field but NORMAL and RING: 12 + 6 + 2 + 8 + 1 + 1 + 2 + 2.
        assert_eq!(seg.fields, 0b1_0111_1011);
        assert_eq!(seg.stride, 34);
        // Whole legacy degrees, rounded to 0.006 degree units.
        check(&pc, &points, &[-1000, -833, -667, -500, -333]);
        for (i, p) in points.iter().enumerate() {
            assert_eq!(
                seg.field(bit::TIMESTAMP, i),
                p.gps_time.unwrap().to_le_bytes()
            );
            let c = p.color.unwrap();
            let rgb: [u8; 6] = seg.field(bit::RGB, i);
            assert_eq!(rgb[0..2], c.red.to_le_bytes());
            assert_eq!(rgb[2..4], c.green.to_le_bytes());
            assert_eq!(rgb[4..6], c.blue.to_le_bytes());
        }
    }

    #[test]
    fn reads_laz_format_6() {
        // Spans more than one read batch.
        let points: Vec<_> = (0..BATCH as usize + 5)
            .map(|i| Point {
                color: None,
                ..point((i % 10) as u8)
            })
            .collect();
        let pc = PointCloud::from_las(CoordinateFrame::Euclidean, None, write(6, true, &points))
            .unwrap();
        let seg = &pc.segments[0];
        assert_eq!(seg.fields, 0b1_0111_1010);
        // Extended scan angles are read back as the units the writer stored.
        let angles: Vec<i16> = points
            .iter()
            .map(|p| (p.scan_angle / 0.006) as i16)
            .collect();
        check(&pc, &points, &angles);
    }

    #[test]
    fn rejects_non_las() {
        let err = PointCloud::from_las(
            CoordinateFrame::Euclidean,
            None,
            Cursor::new(b"not a las file".to_vec()),
        )
        .unwrap_err();
        assert!(matches!(err, Error::PointCloudRead(_)));
    }
}
