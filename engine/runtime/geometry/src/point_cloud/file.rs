//! A segment whose points stay in a LAS / LAZ file until something reads them.

use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, OnceLock};

use reearth_flow_common::uri::{Protocol, Uri};
use serde::{Deserialize, Serialize};
use smallvec::smallvec;

use super::{PointBuffer, PointCloud, PositionEncoding, Segment};
use crate::coordinate::CoordinateFrame;
use crate::error::{Error, Result};

/// A LAS / LAZ file holding one segment's points.
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct PointFile {
    uri: Uri,
    /// Added to every position on read, in the cloud's frame.
    #[serde(default, skip_serializing_if = "is_zero")]
    shift: [f64; 3],
    /// Read on first use and shared by clones.
    #[serde(skip)]
    header: Arc<OnceLock<std::result::Result<LasHeader, String>>>,
}

/// What a LAS header says about its points, in the file's own axis order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LasHeader {
    pub(super) count: u64,
    pub(super) min: [f64; 3],
    pub(super) max: [f64; 3],
}

fn is_zero(shift: &[f64; 3]) -> bool {
    *shift == [0.0; 3]
}

impl PointFile {
    pub(super) fn new(uri: Uri) -> Self {
        PointFile {
            uri,
            shift: [0.0; 3],
            header: Arc::default(),
        }
    }

    pub(super) fn uri(&self) -> &Uri {
        &self.uri
    }

    pub(super) fn shift(&self) -> [f64; 3] {
        self.shift
    }

    /// Move every position by `delta` without reading the file.
    pub(super) fn translate(&mut self, delta: [f64; 3]) {
        for (axis, d) in self.shift.iter_mut().zip(delta) {
            *axis += d;
        }
    }

    /// Read every point into memory, in `frame`, with the shift applied.
    /// Fails first if the file records a CRS other than `frame`'s.
    fn load(&self, frame: &CoordinateFrame) -> Result<PointBuffer> {
        let uri = &self.uri;
        let reader = ::las::Reader::new(open(uri).map_err(Error::point_cloud_read)?)
            .map_err(|e| Error::point_cloud_read(format!("{uri}: {e}")))?;
        let mut buffer = super::las::read_buffer(reader, frame, Some(Arc::from(uri.to_string())))
            .map_err(|e| match e {
            Error::PointCloudRead(message) => Error::point_cloud_read(format!("{uri}: {message}")),
            e => e,
        })?;
        if let PositionEncoding::ScaledI32 { offset, .. } = &mut buffer.position {
            for (axis, d) in offset.iter_mut().zip(self.shift) {
                *axis += d;
            }
        }
        Ok(buffer)
    }

    /// The file's header, read on first call.
    pub(super) fn header(&self) -> Result<&LasHeader> {
        self.header
            .get_or_init(|| read_header(&self.uri))
            .as_ref()
            .map_err(Error::point_cloud_read)
    }
}

impl PartialEq for PointFile {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && self.shift == other.shift
    }
}

impl fmt::Debug for PointFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointFile")
            .field("uri", &self.uri)
            .field("shift", &self.shift)
            .finish()
    }
}

/// Open a local LAS / LAZ file for reading.
pub(super) fn open(uri: &Uri) -> std::result::Result<BufReader<File>, String> {
    if uri.protocol() != Protocol::File {
        return Err(format!("{uri} is not a local file"));
    }
    File::open(uri.as_path())
        .map(BufReader::new)
        .map_err(|e| format!("{uri}: {e}"))
}

fn read_header(uri: &Uri) -> std::result::Result<LasHeader, String> {
    let reader = ::las::Reader::new(open(uri)?).map_err(|e| format!("{uri}: {e}"))?;
    let header = reader.header();
    let b = header.bounds();
    Ok(LasHeader {
        count: header.number_of_points(),
        min: [b.min.x, b.min.y, b.min.z],
        max: [b.max.x, b.max.y, b.max.z],
    })
}

impl PointCloud {
    /// A point cloud whose points stay in the LAS / LAZ file at `uri` until an
    /// operation needs them. `frame` is the frame of the file's coordinates.
    pub fn from_las_file(frame: CoordinateFrame, uri: Uri) -> Self {
        PointCloud {
            frame,
            segments: smallvec![Segment::File(PointFile::new(uri))],
            kdtree: OnceLock::new(),
        }
    }

    /// Read every file-backed segment into memory. Each file's own CRS, if it
    /// records one, is checked against the cloud's frame before its points
    /// are read.
    pub fn load(&mut self) -> Result<()> {
        for seg in &mut self.segments {
            if let Segment::File(file) = seg {
                *seg = Segment::InMemory(file.load(&self.frame)?);
                self.kdtree = OnceLock::new();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn a_missing_file_fails_on_first_use_only() {
        let pc = PointCloud::from_las_file(
            CoordinateFrame::Euclidean,
            Uri::from_str("file:///nonexistent/a.las").unwrap(),
        );
        let Segment::File(file) = &pc.segments[0] else {
            unreachable!()
        };
        assert!(matches!(file.header(), Err(Error::PointCloudRead(_))));
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn counting_an_unreadable_file_fails() {
        use crate::ops::CountVertices;
        let pc = PointCloud::from_las_file(
            CoordinateFrame::Euclidean,
            Uri::from_str("file:///nonexistent/a.las").unwrap(),
        );
        assert!(matches!(pc.count_vertices(), Err(Error::PointCloudRead(_))));
    }

    #[test]
    fn a_remote_file_is_not_read() {
        let file = PointFile::new(Uri::from_str("gs://bucket/a.las").unwrap());
        let err = file.header().unwrap_err().to_string();
        assert!(err.contains("not a local file"), "unexpected error: {err}");
    }

    #[test]
    fn load_reads_the_points_with_the_shift() {
        use crate::coordinate::EpsgCode;
        use crate::point_cloud::las::tests::{geotiff_crs, point, write_with};
        use crate::point_cloud::ops::segment_positions;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.las");
        let p = point(0);
        let bytes = write_with(
            3,
            false,
            std::slice::from_ref(&p),
            Some(geotiff_crs(3072, 6677)),
        );
        std::fs::write(&path, bytes.into_inner()).unwrap();
        let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();

        let mut pc =
            PointCloud::from_las_file(CoordinateFrame::Crs(EpsgCode::new(6677)), uri.clone());
        let Segment::File(file) = &mut pc.segments[0] else {
            unreachable!()
        };
        file.translate([1.0, 2.0, 3.0]);
        pc.load().unwrap();
        // EPSG:6677 stores northing first.
        let [x, y, z] = segment_positions(pc.segments[0].as_buffer())
            .next()
            .unwrap();
        assert!((x - (p.y + 1.0)).abs() < 1e-9);
        assert!((y - (p.x + 2.0)).abs() < 1e-9);
        assert!((z - (p.z + 3.0)).abs() < 1e-9);

        let mut wrong = PointCloud::from_las_file(CoordinateFrame::Crs(EpsgCode::new(6676)), uri);
        let err = wrong.load().unwrap_err().to_string();
        assert!(err.contains("EPSG:6677"), "unexpected error: {err}");
        assert!(matches!(wrong.segments[0], Segment::File(_)));
    }
}
