//! Points that stay in a LAS / LAZ file until something reads them.

use std::fmt;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;
use serde::{Deserialize, Serialize};

use super::{Data, PointBuffer, PointCloud};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::error::{Error, Result};

/// A LAS / LAZ file holding a point cloud's points.
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct PointFile {
    uri: Uri,
    /// Read on first successful use and shared by clones.
    #[serde(skip)]
    header: Arc<OnceLock<LasHeader>>,
}

/// What a LAS header says about its points, in the file's own axis order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LasHeader {
    pub(super) count: u64,
    pub(super) min: [f64; 3],
    pub(super) max: [f64; 3],
    /// The CRS the file records, if any.
    pub(super) crs: Option<EpsgCode>,
}

impl PointFile {
    pub(super) fn new(uri: Uri) -> Self {
        PointFile {
            uri,
            header: Arc::default(),
        }
    }

    pub(super) fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Read every point into memory, in `frame`. Fails first if the file
    /// records a CRS other than `frame`'s.
    fn load(&self, frame: &CoordinateFrame) -> Result<PointBuffer> {
        let uri = &self.uri;
        let reader = open(uri).map_err(Error::point_cloud_read)?;
        super::las::open_reader(reader)
            .and_then(|reader| {
                super::las::read_buffer(reader, frame, Some(Arc::from(uri.to_string())))
            })
            .map_err(|e| match e {
                Error::PointCloudRead(message) => {
                    Error::point_cloud_read(format!("{uri}: {message}"))
                }
                e => e,
            })
    }

    /// The file's header, read on the first call that succeeds. A failed read
    /// is not kept, so the next call reads the file again.
    pub(super) fn header(&self) -> Result<&LasHeader> {
        if let Some(header) = self.header.get() {
            return Ok(header);
        }
        let header = read_header(&self.uri).map_err(Error::point_cloud_read)?;
        Ok(self.header.get_or_init(|| header))
    }
}

impl PartialEq for PointFile {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl fmt::Debug for PointFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointFile").field("uri", &self.uri).finish()
    }
}

/// The storage point files are read through.
fn storage_resolver() -> &'static StorageResolver {
    static RESOLVER: OnceLock<StorageResolver> = OnceLock::new();
    RESOLVER.get_or_init(StorageResolver::new)
}

/// Open a LAS / LAZ file in any storage the [`StorageResolver`] reaches.
pub(super) fn open(uri: &Uri) -> std::result::Result<RangeReader, String> {
    let storage = storage_resolver()
        .resolve(uri)
        .map_err(|e| format!("{uri}: {e}"))?;
    let path = uri.path();
    let len = storage
        .head_sync(&path)
        .map_err(|e| format!("{uri}: {e}"))?
        .size;
    Ok(RangeReader {
        storage,
        path,
        len,
        pos: 0,
        block: Bytes::new(),
        block_start: 0,
        block_size: MIN_BLOCK,
    })
}

/// Bytes fetched by the first read, enough for a header and its records.
const MIN_BLOCK: u64 = 64 * 1024;
/// Bytes fetched per read once reading is sequential.
const MAX_BLOCK: u64 = 8 * 1024 * 1024;

/// A file read through ranged requests, one block at a time. A read that
/// continues where the previous block ended fetches a block twice as large.
pub(super) struct RangeReader {
    storage: Arc<Storage>,
    path: PathBuf,
    len: u64,
    pos: u64,
    block: Bytes,
    block_start: u64,
    block_size: u64,
}

impl Read for RangeReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.len || out.is_empty() {
            return Ok(0);
        }
        let block_end = self.block_start + self.block.len() as u64;
        if !(self.block_start..block_end).contains(&self.pos) {
            self.block_size = if self.pos == block_end && !self.block.is_empty() {
                (self.block_size * 2).min(MAX_BLOCK)
            } else {
                MIN_BLOCK
            };
            let want = self.block_size.max(out.len() as u64);
            let end = (self.pos + want).min(self.len);
            self.block = self
                .storage
                .get_range_sync(&self.path, self.pos as usize..end as usize)
                .map_err(io::Error::other)?;
            self.block_start = self.pos;
            if self.block.is_empty() {
                return Ok(0);
            }
        }
        let at = (self.pos - self.block_start) as usize;
        let n = out.len().min(self.block.len() - at);
        out[..n].copy_from_slice(&self.block[at..at + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for RangeReader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let pos = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::End(d) => self.len.checked_add_signed(d),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
        };
        self.pos = pos.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before the start of the file",
            )
        })?;
        Ok(self.pos)
    }
}

fn read_header(uri: &Uri) -> std::result::Result<LasHeader, String> {
    let reader = super::las::open_reader(open(uri)?).map_err(|e| match e {
        Error::PointCloudRead(message) => format!("{uri}: {message}"),
        e => format!("{uri}: {e}"),
    })?;
    let header = reader.header();
    let b = header.bounds();
    let crs = super::las::las_crs(header).map_err(|e| format!("{uri}: {e}"))?;
    Ok(LasHeader {
        count: header.number_of_points(),
        min: [b.min.x, b.min.y, b.min.z],
        max: [b.max.x, b.max.y, b.max.z],
        crs,
    })
}

impl PointCloud {
    /// A point cloud whose points stay in the LAS / LAZ file at `uri` until an
    /// operation needs them. `frame` is the frame of the file's coordinates.
    pub fn from_las_file(frame: CoordinateFrame, uri: Uri) -> Self {
        PointCloud::new(frame, Data::File(PointFile::new(uri)))
    }

    /// Read file-backed points into memory. The file's own CRS, if it records
    /// one, is checked against the cloud's frame before its points are read.
    pub fn load(&mut self) -> Result<()> {
        if let Data::File(file) = &self.data {
            self.data = Data::InMemory(file.load(&self.frame)?);
            self.kdtree = OnceLock::new();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[cfg(feature = "new-geometry")]
    #[test]
    fn counting_retries_an_unreadable_file() {
        use crate::ops::CountVertices;
        use crate::point_cloud::las::tests::{point, write_with};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.las");
        let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();
        let pc = PointCloud::from_las_file(CoordinateFrame::Euclidean, uri);
        assert!(matches!(pc.count_vertices(), Err(Error::PointCloudRead(_))));
        let bytes = write_with(3, false, &[point(0)], None).into_inner();
        std::fs::write(&path, bytes).unwrap();
        assert_eq!(pc.count_vertices().unwrap(), 1);
    }

    #[test]
    fn load_reads_the_points_under_the_translation() {
        use crate::coordinate::EpsgCode;
        use crate::ops::{BoundingBox, Translate};
        use crate::point_cloud::las::tests::{geotiff_crs, point, write_with};

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
        pc.translate([1.0, 2.0, 3.0]).unwrap();
        pc.load().unwrap();
        // EPSG:6677 stores northing first.
        let [x, y, z] = pc.positions("test").unwrap().next().unwrap();
        assert!((x - (p.y + 1.0)).abs() < 1e-9);
        assert!((y - (p.x + 2.0)).abs() < 1e-9);
        assert!((z - (p.z + 3.0)).abs() < 1e-9);

        let mut wrong = PointCloud::from_las_file(CoordinateFrame::Crs(EpsgCode::new(6676)), uri);
        assert!(wrong.bounding_box().is_err());
        let err = wrong.load().unwrap_err().to_string();
        assert!(err.contains("EPSG:6677"), "unexpected error: {err}");
        assert!(matches!(wrong.data, Data::File(_)));
    }

    #[test]
    fn range_reader_reads_and_seeks_like_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let bytes: Vec<u8> = (0..(MIN_BLOCK * 5)).map(|i| (i % 251) as u8).collect();
        std::fs::write(&path, &bytes).unwrap();
        let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();
        let mut reader = open(&uri).unwrap();

        let mut all = Vec::new();
        reader.read_to_end(&mut all).unwrap();
        assert_eq!(all, bytes);
        let mut buf = [0u8; 10];
        reader.seek(SeekFrom::Start(3)).unwrap();
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, bytes[3..13]);
    }
}
