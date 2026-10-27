//! Assigning geometry to Japanese standard mesh cells by how much of it each
//! cell holds: the largest area for areal geometry, the longest length for
//! linear geometry, plus a plain point lookup.

use std::cell::RefCell;

use reearth_flow_geometry::{
    coordinate::{CoordinateFrame, EpsgCode},
    ops::{reproject::transform_coords_3d, Aabb, BoundingBox, ReprojectionCache},
    overlay::{clip_2d, overlay_2d, OverlayOp},
    polygon::Polygon2D,
    Euclidean2DGeometry,
};
// The mesh grid is still described with the pre-migration rectangle types.
use reearth_flow_geometry::types::{coordinate::Coordinate2D, rect::Rect2D};

use super::{in_jis_range, JPMeshCode, JPMeshType};

/// The geographic CRS the Japanese mesh grid is described in: the 2D horizontal
/// counterpart of EPSG:6697, sharing its datum and its `[latitude, longitude]`
/// axis order. [`JPMeshCode`], in contrast, works in `(x = longitude,
/// y = latitude)`, so coordinates are swapped at every crossing between them.
pub const GEOGRAPHIC_EPSG: EpsgCode = EpsgCode::new(6668);

/// How many pieces each edge of a mesh cell is cut into before it is projected.
const SEGMENTS_PER_EDGE: usize = 16;

thread_local! {
    /// The live PROJ transforms, kept thread-local because they wrap
    /// non-shareable PROJ pointers. One per direction: a cache holds a single
    /// `(from, to)` pair, so sharing one between the two directions would
    /// rebuild the transform every call.
    static TO_GEOGRAPHIC: RefCell<ReprojectionCache> = RefCell::new(ReprojectionCache::new());
    static FROM_GEOGRAPHIC: RefCell<ReprojectionCache> = RefCell::new(ReprojectionCache::new());
}

/// Why a mesh assignment could not be computed.
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    /// A coordinate transform failed.
    #[error("{0}")]
    Transform(String),
    /// Intersecting or clipping against a mesh cell failed.
    #[error("{0}")]
    Overlay(String),
}

/// The outcome of assigning a geometry to the mesh cell holding most of it.
#[derive(Debug, Clone)]
pub struct MeshOverlap {
    pub selected: JPMeshCode,
    /// How many cells the geometry has a positive measure in.
    pub mesh_count: usize,
    /// The selected cell's measure (square metres or metres), to two decimals.
    pub max_measure: f64,
    /// `(mesh code number, measure)` for every cell with a positive measure.
    pub per_mesh: Vec<(u64, f64)>,
}

/// The mesh the areal geometry `area` has the most area in, with the per-mesh
/// areas behind that choice. `None` when it meets no mesh, or lies outside the
/// range mesh codes can express.
///
/// Areas are measured in `epsg`, the projected CRS the geometry is already
/// expressed in, so they come out in square metres.
pub fn largest_area_overlap(
    area: &Euclidean2DGeometry,
    epsg: EpsgCode,
    mesh_type: JPMeshType,
) -> Result<Option<MeshOverlap>, MeshError> {
    let Ok(Aabb::D2 { min, max }) = area.bounding_box() else {
        return Ok(None);
    };
    let bounds = geographic_bounds(min, max, epsg)?;
    largest_overlap(bounds, epsg, mesh_type, |cell, number| {
        let pieces = overlay_2d(area, cell, OverlayOp::Intersection).map_err(|e| {
            MeshError::Overlay(format!(
                "Failed to intersect the footprint with mesh {number}: {e}"
            ))
        })?;
        Ok(pieces.iter().map(Polygon2D::area).sum())
    })
}

/// The mesh the linear geometry `lines` has the most length in, with the
/// per-mesh lengths behind that choice. `None` when it meets no mesh, or lies
/// outside the range mesh codes can express.
///
/// Lengths are measured in `epsg`, which should be a projected CRS.
pub fn largest_length_overlap(
    lines: &Euclidean2DGeometry,
    epsg: EpsgCode,
    mesh_type: JPMeshType,
) -> Result<Option<MeshOverlap>, MeshError> {
    let Ok(Aabb::D2 { min, max }) = lines.bounding_box() else {
        return Ok(None);
    };
    let bounds = geographic_bounds(min, max, epsg)?;
    largest_overlap(bounds, epsg, mesh_type, |cell, number| {
        let pieces = clip_2d(lines, cell, false).map_err(|e| {
            MeshError::Overlay(format!("Failed to clip the line with mesh {number}: {e}"))
        })?;
        Ok(pieces
            .iter()
            .map(|line| {
                line.coords()
                    .windows(2)
                    .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
                    .sum::<f64>()
            })
            .sum())
    })
}

/// The mesh containing `point`, given in `epsg`'s declared axis order, or
/// `None` outside the range mesh codes can express.
pub fn containing(
    point: [f64; 3],
    epsg: EpsgCode,
    mesh_type: JPMeshType,
) -> Result<Option<JPMeshCode>, MeshError> {
    let mut coords = [point];
    if epsg != GEOGRAPHIC_EPSG {
        TO_GEOGRAPHIC
            .with(|cache| {
                transform_coords_3d(&mut cache.borrow_mut(), epsg, GEOGRAPHIC_EPSG, &mut coords)
            })
            .map_err(|e| {
                MeshError::Transform(format!(
                    "Failed to transform the point from EPSG:{epsg} to EPSG:{GEOGRAPHIC_EPSG}: {e}"
                ))
            })?;
    }
    let [lat, lng, _] = coords[0];
    Ok(JPMeshCode::try_new(Coordinate2D::new_(lng, lat), mesh_type))
}

/// The candidate cell with the largest measure, the smaller mesh code winning
/// a tie. `measure` receives the cell as a polygon in `epsg` and its code.
fn largest_overlap(
    operand_bounds: Rect2D<f64>,
    epsg: EpsgCode,
    mesh_type: JPMeshType,
    measure: impl Fn(&Euclidean2DGeometry, u64) -> Result<f64, MeshError>,
) -> Result<Option<MeshOverlap>, MeshError> {
    let frame = CoordinateFrame::Crs(epsg);
    let (mut best, mut best_number, mut max_measure) = (None, u64::MAX, 0.0f64);
    let mut per_mesh = Vec::new();
    // A bounding box outside the JIS range would enumerate meaningless cells
    // (or very many of them), and none could carry a code.
    let (min, max) = (operand_bounds.min(), operand_bounds.max());
    if !in_jis_range(min.x, min.y) || !in_jis_range(max.x, max.y) {
        return Ok(None);
    }
    for mesh_code in JPMeshCode::from_inside_bounds(operand_bounds, mesh_type) {
        let sw = mesh_code.bounds().min();
        if !in_jis_range(sw.x, sw.y) {
            continue;
        }
        let number = mesh_code.to_number();
        let cell =
            Euclidean2DGeometry::Polygon(Box::new(mesh_face(&mesh_code.bounds(), epsg, &frame)?));
        // Two decimal places, as the PLATEAU specification reports them.
        let m = (measure(&cell, number)? * 100.0).round() / 100.0;
        if m <= 0.0 {
            continue;
        }
        per_mesh.push((number, m));
        // The largest measure wins; the smaller mesh code breaks a tie.
        if m > max_measure || (m == max_measure && number < best_number) {
            max_measure = m;
            best_number = number;
            best = Some(mesh_code);
        }
    }
    Ok(best.map(|selected| MeshOverlap {
        selected,
        mesh_count: per_mesh.len(),
        max_measure,
        per_mesh,
    }))
}

/// A projected bounding box carried back to the mesh grid's geographic space,
/// as the `(longitude, latitude)` rectangle [`JPMeshCode`] expects.
///
/// All four corners are transformed: the projection is not axis-aligned with
/// the graticule, so two opposite corners alone would leave out the extremes
/// reached by the other two.
pub(crate) fn geographic_bounds(
    min: [f64; 2],
    max: [f64; 2],
    epsg: EpsgCode,
) -> Result<Rect2D<f64>, MeshError> {
    let mut corners = [
        [min[0], min[1], 0.0],
        [min[0], max[1], 0.0],
        [max[0], min[1], 0.0],
        [max[0], max[1], 0.0],
    ];
    TO_GEOGRAPHIC
        .with(|cache| {
            transform_coords_3d(&mut cache.borrow_mut(), epsg, GEOGRAPHIC_EPSG, &mut corners)
        })
        .map_err(|e| {
            MeshError::Transform(format!(
                "Failed to transform the geometry bounds from EPSG:{epsg} to EPSG:{GEOGRAPHIC_EPSG}: {e}"
            ))
        })?;
    let mut min_lng = f64::INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lng = f64::NEG_INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    for [lat, lng, _] in corners {
        min_lng = min_lng.min(lng);
        max_lng = max_lng.max(lng);
        min_lat = min_lat.min(lat);
        max_lat = max_lat.max(lat);
    }
    Ok(Rect2D::new(
        Coordinate2D::new_(min_lng, min_lat),
        Coordinate2D::new_(max_lng, max_lat),
    ))
}

/// One mesh cell as a face in `frame`, its edges subdivided before projection.
///
/// A cell is bounded by lines of constant latitude and longitude, which curve
/// under a Transverse Mercator projection. Straight edges between the four
/// corners would cut into the cell, so intermediate vertices are added while
/// the ring is still geographic.
pub(crate) fn mesh_face(
    bounds: &Rect2D<f64>,
    epsg: EpsgCode,
    frame: &CoordinateFrame,
) -> Result<Polygon2D, MeshError> {
    let (min_lng, min_lat) = (bounds.min().x, bounds.min().y);
    let (max_lng, max_lat) = (bounds.max().x, bounds.max().y);
    let step = |i: usize| i as f64 / SEGMENTS_PER_EDGE as f64;

    // South, east, north then west edge: counter-clockwise seen in
    // (East, North). Vertices are held as [latitude, longitude], the geographic
    // CRS's declared axis order, ready for the transform.
    let mut ring = Vec::with_capacity(4 * SEGMENTS_PER_EDGE + 1);
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([min_lat, min_lng + step(i) * (max_lng - min_lng), 0.0]);
    }
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([min_lat + step(i) * (max_lat - min_lat), max_lng, 0.0]);
    }
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([max_lat, max_lng - step(i) * (max_lng - min_lng), 0.0]);
    }
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([max_lat - step(i) * (max_lat - min_lat), min_lng, 0.0]);
    }
    ring.push(ring[0]);

    FROM_GEOGRAPHIC
        .with(|cache| {
            transform_coords_3d(&mut cache.borrow_mut(), GEOGRAPHIC_EPSG, epsg, &mut ring)
        })
        .map_err(|e| {
            MeshError::Transform(format!(
                "Failed to transform a mesh cell from EPSG:{GEOGRAPHIC_EPSG} to EPSG:{epsg}: {e}"
            ))
        })?;

    Ok(Polygon2D::from_rings(
        frame.clone(),
        ring.iter().map(|c| [c[0], c[1]]),
        Vec::<Vec<[f64; 2]>>::new(),
    ))
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::line_string::LineString2D;

    use super::*;

    /// Real coordinates from the `Z-bldg-03_meshcode-extractor_01` fixture, as
    /// `(longitude, latitude)`. Its expected CSV row is mesh 54377085 covering
    /// 9.14 m², a value originally obtained from the original implementation.
    const FOOTPRINT: [(f64, f64); 5] = [
        (137.07022204628032, 36.65423985231743),
        (137.07018801464667, 36.65426289610968),
        (137.0701714804155, 36.654247021162256),
        (137.07020551204704, 36.65422397737467),
        (137.07022204628032, 36.65423985231743),
    ];

    const PRCS: EpsgCode = EpsgCode::new(6675);

    /// A ring of `(longitude, latitude)` pairs as a footprint projected into
    /// EPSG:6675, the shape the workflow hands the extractor.
    fn footprint_in_prcs(ring: &[(f64, f64)]) -> Euclidean2DGeometry {
        let mut coords: Vec<[f64; 3]> = ring.iter().map(|&(lng, lat)| [lat, lng, 0.0]).collect();
        FROM_GEOGRAPHIC
            .with(|cache| {
                transform_coords_3d(&mut cache.borrow_mut(), GEOGRAPHIC_EPSG, PRCS, &mut coords)
            })
            .expect("projection into EPSG:6675");
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            CoordinateFrame::Crs(PRCS),
            coords.iter().map(|c| [c[0], c[1]]),
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    #[test]
    fn coordinates_keep_each_crs_declared_axis_order() {
        let geometry = footprint_in_prcs(&FOOTPRINT);
        let Ok(Aabb::D2 { min, max }) = geometry.bounding_box() else {
            panic!("the footprint has no 2D bounding box");
        };
        // EPSG:6675's origin is 36°N 137°10′E, so this footprint sits ~72.6 km
        // north and ~8.6 km west of it, stored [northing, easting].
        assert!((min[0] - 72_580.0).abs() < 200.0, "northing was {}", min[0]);
        assert!((min[1] + 8_620.0).abs() < 200.0, "easting was {}", min[1]);

        // Back in geographic space the rectangle is (longitude, latitude),
        // which is what JPMeshCode reads.
        let bounds = geographic_bounds(min, max, PRCS).expect("inverse projection");
        assert!((bounds.min().x - 137.0701714804155).abs() < 1e-7);
        assert!((bounds.min().y - 36.65422397737467).abs() < 1e-7);
    }

    #[test]
    fn selects_the_mesh_the_footprint_lies_in() {
        let result =
            largest_area_overlap(&footprint_in_prcs(&FOOTPRINT), PRCS, JPMeshType::Mesh1km)
                .expect("mesh calculation")
                .expect("the footprint meets a mesh");

        assert_eq!(result.selected.to_number(), 54377085);
        assert_eq!(result.mesh_count, 1);
        assert_eq!(result.max_measure, 9.14);
        assert_eq!(result.per_mesh.len(), 1);
    }

    #[test]
    fn a_line_goes_to_the_cell_holding_most_of_its_length() {
        // A west-to-east line crossing the boundary between 3rd meshes
        // 53394611 and 53394612, 3/4 of it in the eastern cell.
        let west = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km).bounds();
        let edge_lng = west.max().x;
        let lat = (west.min().y + west.max().y) / 2.0;
        let span = 45.0 / 3600.0;
        let mut coords = vec![
            [lat, edge_lng - span * 0.25, 0.0],
            [lat, edge_lng + span * 0.75, 0.0],
        ];
        FROM_GEOGRAPHIC
            .with(|c| transform_coords_3d(&mut c.borrow_mut(), GEOGRAPHIC_EPSG, PRCS, &mut coords))
            .unwrap();
        let line = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            CoordinateFrame::Crs(PRCS),
            coords.iter().map(|c| [c[0], c[1]]),
        ));
        let r = largest_length_overlap(&line, PRCS, JPMeshType::Mesh1km)
            .unwrap()
            .unwrap();
        assert_eq!(r.selected.to_number(), 53394612);
        assert_eq!(r.mesh_count, 2);
    }

    #[test]
    fn equal_measures_tie_to_the_smaller_code() {
        // Two neighbouring cells measure the same; every other candidate less.
        let west = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km).bounds();
        let east = JPMeshCode::from_number(53394612, JPMeshType::Mesh1km).bounds();
        let bounds = Rect2D::new(
            Coordinate2D::new_(west.min().x + 1e-4, west.min().y + 1e-4),
            Coordinate2D::new_(east.max().x - 1e-4, east.max().y - 1e-4),
        );
        let r = largest_overlap(bounds, PRCS, JPMeshType::Mesh1km, |_, number| {
            Ok(if number == 53394611 || number == 53394612 {
                5.0
            } else {
                1.0
            })
        })
        .unwrap()
        .unwrap();
        assert_eq!(r.selected.to_number(), 53394611);
        assert_eq!(r.max_measure, 5.0);
        let tied: Vec<u64> = r
            .per_mesh
            .iter()
            .filter(|(_, m)| *m == 5.0)
            .map(|(n, _)| *n)
            .collect();
        assert_eq!(tied.len(), 2);
        assert_eq!(r.mesh_count, r.per_mesh.len());
    }

    #[test]
    fn containing_reprojects_and_respects_axis_order() {
        // A point inside 53394611, given in EPSG:6677 ([northing, easting]).
        let cell = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km).bounds();
        let mut p = [[
            (cell.min().y + cell.max().y) / 2.0,
            (cell.min().x + cell.max().x) / 2.0,
            0.0,
        ]];
        let jprcs9 = EpsgCode::new(6677);
        FROM_GEOGRAPHIC
            .with(|c| transform_coords_3d(&mut c.borrow_mut(), GEOGRAPHIC_EPSG, jprcs9, &mut p))
            .unwrap();
        let code = containing(p[0], jprcs9, JPMeshType::Mesh1km)
            .unwrap()
            .unwrap();
        assert_eq!(code.to_number(), 53394611);
    }

    #[test]
    fn containing_outside_japan_is_none() {
        // London in WGS84 geographic ([lat, lng] for EPSG:4326).
        assert!(
            containing([51.5, -0.12, 0.0], EpsgCode::new(4326), JPMeshType::Mesh1km)
                .unwrap()
                .is_none()
        );
    }
}
