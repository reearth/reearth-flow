//! LAS / LAZ reading.

use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::error::{Error, Result};
use crate::ops::TwoDimensionalCrs;

use super::{Data, PointBuffer, PointCloud, PositionEncoding, bit};

/// Degrees per unit of the stored scan angle.
const SCAN_ANGLE_UNIT: f64 = 0.006;

/// Points decoded per read.
const BATCH: u64 = 65_536;

impl PointCloud {
    /// Read a LAS or LAZ stream into memory in its native scaled-integer
    /// encoding.
    ///
    /// Keeps RGB, intensity, GPS time, classification, return number and number
    /// of returns, scan angle and point source ID; other LAS fields are dropped.
    /// A legacy (formats 0-5) scan angle is rounded to 0.006 degrees. `frame` is
    /// the frame of the file's coordinates and must agree with the CRS the file
    /// records, if any. LAS stores easting before northing, so x and y are
    /// swapped when `frame` is a CRS that stores northing first.
    pub fn from_las<R: Read + Seek + Send + Sync + 'static>(
        frame: CoordinateFrame,
        source: Option<Arc<str>>,
        read: R,
    ) -> Result<Self> {
        let reader = open_reader(read)?;
        let seg = read_buffer(reader, &frame, source)?;
        Ok(PointCloud::new(frame, Data::InMemory(seg)))
    }
}

/// The highest LAS minor version read; later versions change the header.
const LAS_MAX_MINOR: u8 = 4;

/// A LAS reader over `read`, after rejecting a LAS version other than 1.0 to
/// 1.4. A stream too short to hold a version, or without the LAS signature,
/// is left for the LAS reader to reject.
pub(super) fn open_reader<R: Read + Seek + Send + Sync + 'static>(
    mut read: R,
) -> Result<::las::Reader> {
    let start = read.stream_position().map_err(Error::point_cloud_read)?;
    // File signature (4), file source ID (2), global encoding (2), GUID (16),
    // then the major and minor version.
    let mut prefix = [0u8; 26];
    let peeked = read.read_exact(&mut prefix);
    read.seek(SeekFrom::Start(start))
        .map_err(Error::point_cloud_read)?;
    if peeked.is_ok() && &prefix[0..4] == b"LASF" {
        let (major, minor) = (prefix[24], prefix[25]);
        if major != 1 || minor > LAS_MAX_MINOR {
            return Err(Error::point_cloud_read(format!(
                "LAS {major}.{minor} is not supported; only LAS 1.0 to 1.{LAS_MAX_MINOR} can be read"
            )));
        }
    }
    ::las::Reader::new(read).map_err(Error::point_cloud_read)
}

/// Read every point of `reader` into a buffer, after checking the file's
/// CRS against `frame`.
pub(super) fn read_buffer(
    mut reader: ::las::Reader,
    frame: &CoordinateFrame,
    source: Option<Arc<str>>,
) -> Result<PointBuffer> {
    let header = reader.header();
    check_crs(las_crs(header)?, frame)?;
    let format = *header.point_format();
    let t = *header.transforms();
    let swap = swaps_xy(frame)?;
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
    let (tx, ty) = if swap { (t.y, t.x) } else { (t.x, t.y) };
    let position = PositionEncoding::ScaledI32 {
        scale: [tx.scale, ty.scale, t.z.scale],
        offset: [tx.offset, ty.offset, t.z.offset],
    };
    // The count is only the header's claim: reserve for it, but zero and fill
    // memory batch by batch, so a header claiming more points than the file
    // holds fails at the end of the stream instead of allocating them all.
    let mut seg = PointBuffer::with_capacity(source, position, fields, count).ok_or_else(|| {
        Error::point_cloud_read(format!(
            "header declares {count} points, more than can be held in memory"
        ))
    })?;
    while seg.count < count {
        let start = seg.count;
        let n = reader
            .fill_points(BATCH.min((count - start) as u64), &mut batch)
            .map_err(Error::point_cloud_read)? as usize;
        if n == 0 {
            return Err(Error::point_cloud_read(format!(
                "header declares {count} points but the stream ended after {start}"
            )));
        }
        seg.extend_zeroed(n);
        copy_batch(&mut seg, start, &batch, swap);
    }
    Ok(seg)
}

/// The EPSG code of the CRS a LAS header records, from its GeoTIFF keys or
/// its WKT. `None` if it records none, or one without an EPSG code.
pub(super) fn las_crs(header: &::las::Header) -> Result<Option<EpsgCode>> {
    if let Some(wkt) = header.get_wkt_crs_bytes() {
        let wkt = std::str::from_utf8(wkt).map_err(Error::point_cloud_read)?;
        return Ok(wkt_epsg(wkt.trim_end_matches('\0')));
    }
    let Some(geotiff) = header.get_geotiff_crs().map_err(Error::point_cloud_read)? else {
        return Ok(None);
    };
    let epsg = |key: Option<u16>| key.filter(|k| (1024..=32766).contains(k));
    Ok(epsg(geotiff.get_projected_crs_geo_key_value())
        .or_else(|| epsg(geotiff.get_geodetic_crs_geo_key_value()))
        .map(EpsgCode::new))
}

/// The EPSG code a WKT CRS gives its root object, in either WKT 1
/// `AUTHORITY["EPSG","6677"]` or WKT 2 `ID["EPSG",6677]` form, with brackets
/// or parentheses.
fn wkt_epsg(wkt: &str) -> Option<EpsgCode> {
    let bytes = wkt.as_bytes();
    // The root's own identifier is its last child at depth 1. Quoted text is
    // skipped; a doubled quote inside it toggles twice and stays quoted.
    let mut depth = 0usize;
    let mut quoted = false;
    let mut found = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' => quoted = !quoted,
            _ if quoted => {}
            b'[' | b'(' => depth += 1,
            b']' | b')' => depth = depth.saturating_sub(1),
            _ if depth == 1 && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) => {
                for keyword in [b"AUTHORITY".as_slice(), b"ID"] {
                    let end = i + keyword.len();
                    if bytes
                        .get(i..end)
                        .is_some_and(|w| w.eq_ignore_ascii_case(keyword))
                        && matches!(bytes.get(end), Some(b'[' | b'('))
                    {
                        found = Some(end + 1);
                    }
                }
            }
            _ => {}
        }
    }
    // Follows an ASCII delimiter, so it is a char boundary.
    let body = &wkt[found?..];
    let body = &body[..body.find([']', ')'])?];
    let mut parts = body.split(',').map(|p| p.trim().trim_matches('"'));
    let authority = parts.next()?;
    let code = parts.next()?;
    authority
        .eq_ignore_ascii_case("EPSG")
        .then(|| code.parse::<u16>().ok().map(EpsgCode::new))?
}

/// Fail if a LAS file recording the CRS `recorded` holds points not in
/// `frame`. A file in a 2D CRS agrees with a 3D `frame` sharing its horizontal
/// CRS; a file in a 3D CRS must match exactly, since its heights are in that
/// CRS's vertical datum.
pub(super) fn check_crs(recorded: Option<EpsgCode>, frame: &CoordinateFrame) -> Result<()> {
    let CoordinateFrame::Crs(expected) = *frame else {
        return Ok(());
    };
    let Some(recorded) = recorded else {
        return Ok(());
    };
    if recorded == expected {
        return Ok(());
    }
    let horizontal = |epsg| match crate::ops::crs_demote_to_2d(epsg) {
        Ok(TwoDimensionalCrs::Code(code)) => Some(code),
        _ => None,
    };
    match (horizontal(recorded), horizontal(expected)) {
        (Some(a), Some(b)) if a == b && a == recorded => Ok(()),
        _ => Err(Error::point_cloud_read(format!(
            "the file records EPSG:{} but its points were declared in EPSG:{}",
            recorded.get(),
            expected.get()
        ))),
    }
}

/// Whether `frame` stores northing before easting, the reverse of LAS.
pub(super) fn swaps_xy(frame: &CoordinateFrame) -> Result<bool> {
    match frame {
        CoordinateFrame::Crs(epsg) => Ok(crate::ops::north_axis_index(*epsg)? == 0),
        CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => Ok(false),
    }
}

/// Copy the points of `batch` into `seg` from point `start` on, swapping x and
/// y if `swap`.
fn copy_batch(seg: &mut PointBuffer, start: usize, batch: &::las::PointData, swap: bool) {
    let xyz = batch.x_raw().zip(batch.y_raw()).zip(batch.z_raw());
    for (i, ((x, y), z)) in (start..).zip(xyz) {
        let (x, y) = if swap { (y, x) } else { (x, y) };
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
pub(super) mod tests {
    use std::io::Cursor;

    use ::las::point::{Classification, Format};
    use ::las::{Builder, Color, Point, Transform, Vector, Writer};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::point_cloud::ops::buffer_positions;

    const T: Transform = Transform {
        scale: 0.001,
        offset: 1000.0,
    };

    /// A GeoTIFF CRS record holding one model type and one CRS key.
    pub(in crate::point_cloud) fn geotiff_crs(key: u16, epsg: u16) -> ::las::Vlr {
        let model = if key == 3072 { 1 } else { 2 };
        let words = [1u16, 1, 0, 2, 1024, 0, 1, model, key, 0, 1, epsg];
        ::las::Vlr {
            user_id: "LASF_Projection".into(),
            record_id: 34735,
            description: String::new(),
            data: words.iter().flat_map(|w| w.to_le_bytes()).collect(),
        }
    }

    fn read_in(epsg: u16, crs: ::las::Vlr) -> Result<PointCloud> {
        PointCloud::from_las(
            CoordinateFrame::Crs(EpsgCode::new(epsg)),
            None,
            write_with(3, false, &[point(0)], Some(crs)),
        )
    }

    pub(in crate::point_cloud) fn point(i: u8) -> Point {
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
        write_with(format, compressed, points, None)
    }

    pub(in crate::point_cloud) fn write_with(
        format: u8,
        compressed: bool,
        points: &[Point],
        crs: Option<::las::Vlr>,
    ) -> Cursor<Vec<u8>> {
        let mut builder = Builder::from((1, 4));
        builder.vlrs.extend(crs);
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
        let seg = pc.data.as_buffer();
        assert_eq!(seg.count, points.len());
        let positions: Vec<_> = buffer_positions(seg).collect();
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
        let seg = pc.data.as_buffer();
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
        let seg = pc.data.as_buffer();
        assert_eq!(seg.fields, 0b1_0111_1010);
        // Extended scan angles are read back as the units the writer stored.
        let angles: Vec<i16> = points
            .iter()
            .map(|p| (p.scan_angle / 0.006) as i16)
            .collect();
        check(&pc, &points, &angles);
    }

    #[test]
    fn a_recorded_crs_must_agree_with_the_frame() {
        assert!(read_in(6677, geotiff_crs(3072, 6677)).is_ok());
        // EPSG:6668 is the horizontal part of EPSG:6697.
        assert!(read_in(6697, geotiff_crs(2048, 6668)).is_ok());
        // A 3D file's heights are in its own vertical datum.
        assert!(read_in(6668, geotiff_crs(2048, 6697)).is_err());
        let err = read_in(6676, geotiff_crs(3072, 6677)).unwrap_err();
        assert!(err.to_string().contains("EPSG:6677"), "{err}");
    }

    #[test]
    fn wkt_epsg_reads_the_root_identifier() {
        let code = |wkt| wkt_epsg(wkt).map(|e| e.get());
        assert_eq!(
            code(r#"PROJCS["a",GEOGCS["b",AUTHORITY["EPSG","6668"]],AUTHORITY["EPSG","6677"]]"#),
            Some(6677)
        );
        assert_eq!(
            code(r#"PROJCRS["a",BASEGEOGCRS["b",ID["EPSG",6668]],ID["EPSG",6677]]"#),
            Some(6677)
        );
        // Only a nested object carries an identifier.
        assert_eq!(
            code(r#"PROJCS["a",GEOGCS["b",AUTHORITY["EPSG","6668"]]]"#),
            None
        );
        assert_eq!(code(r#"PROJCS["a",AUTHORITY["ESRI","102100"]]"#), None);
        // Parentheses, and a quoted name holding a bracket and non-ASCII text.
        assert_eq!(
            code(r#"PROJCRS("Survey [west 30°",ID("EPSG",6677))"#),
            Some(6677)
        );
    }

    #[test]
    fn a_header_overstating_the_point_count_is_rejected() {
        // More points than the file holds, then more than memory can hold.
        for count in [u64::from(u32::MAX), 1 << 58] {
            let mut bytes = write(3, false, &[point(0)]).into_inner();
            bytes[107..111].copy_from_slice(&0u32.to_le_bytes());
            bytes[247..255].copy_from_slice(&count.to_le_bytes());
            let err = PointCloud::from_las(CoordinateFrame::Euclidean, None, Cursor::new(bytes))
                .unwrap_err();
            assert!(matches!(err, Error::PointCloudRead(_)), "{err}");
        }
    }
}
