//! Round trip through the Shapefile Reader and Writer.
//!
//! New-geometry only: the old world's writer emits nothing but `PolygonZ`
//! records, so it cannot write back the 2D areas the fixture holds.
#![cfg(feature = "new-geometry")]

use std::path::{Path, PathBuf};

use crate::helper::execute;

/// Records in the Natural Earth fixture the round-trip workflow reads.
const FIXTURE_RECORDS: usize = 177;

/// The shapefile the round-trip workflow wrote, under the output directory the
/// test ran in.
fn written_shapefile(dir: &Path) -> PathBuf {
    let mut written: Vec<PathBuf> = std::fs::read_dir(dir.join("roundtrip"))
        .expect("the writer is expected to create its output directory")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "shp"))
        .collect();
    assert_eq!(
        written.len(),
        1,
        "features of one shape type are expected to produce one file, got {written:?}"
    );
    written.remove(0)
}

/// Reading a shapefile and writing it back returns the records it had, as the
/// areas it had them.
#[test]
fn test_shapefile_writer_round_trip() {
    let dir = execute(
        "file/writer/shapefile",
        vec!["ne_110m_admin_0_countries.zip"],
    )
    .expect("the workflow is expected to run");
    let path = written_shapefile(dir.path());

    let shapes = shapefile::read_shapes(&path).expect("the output is expected to be readable");
    assert_eq!(shapes.len(), FIXTURE_RECORDS);
    for shape in &shapes {
        assert!(
            matches!(shape, shapefile::Shape::Polygon(_)),
            "expected a polygon, got {shape}"
        );
    }

    // The sibling files a consumer needs to read the output: the index, the
    // attribute table, its encoding, and the CRS.
    for extension in ["shx", "dbf", "cpg", "prj"] {
        assert!(
            path.with_extension(extension).exists(),
            "expected a .{extension} beside the .shp"
        );
    }
}

/// Coordinates come back in the order they went in.
///
/// The fixture is in EPSG:4326, which declares (latitude, longitude), while
/// shapefile positions are easting-first whatever the CRS declares. Reading swaps
/// the pair into the order the frame states and writing swaps it back, so a swap
/// applied on only one side would transpose every coordinate. Natural Earth spans
/// the globe, so longitudes reach beyond the range a latitude can occupy and the
/// transposition is visible in the extent alone.
#[test]
fn test_shapefile_writer_keeps_the_axis_order() {
    let dir = execute(
        "file/writer/shapefile",
        vec!["ne_110m_admin_0_countries.zip"],
    )
    .expect("the workflow is expected to run");

    let shapes = shapefile::read_shapes(written_shapefile(dir.path()))
        .expect("the output is expected to be readable");
    let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
    for shape in &shapes {
        let shapefile::Shape::Polygon(polygon) = shape else {
            panic!("expected a polygon, got {shape}");
        };
        for ring in polygon.rings() {
            for point in ring.points() {
                max_x = max_x.max(point.x.abs());
                max_y = max_y.max(point.y.abs());
            }
        }
    }

    // The fixture's own extent reaches 180.00000000000006, so the range checks
    // allow for the rounding the coordinates were stored with.
    const SLACK: f64 = 1e-9;
    assert!(
        max_x > 90.0,
        "x is expected to hold longitude, reaching past the latitude range; got {max_x}"
    );
    assert!(
        max_x <= 180.0 + SLACK,
        "x is expected to stay within the longitude range; got {max_x}"
    );
    assert!(
        max_y <= 90.0 + SLACK,
        "y is expected to hold latitude; got {max_y}"
    );
}
