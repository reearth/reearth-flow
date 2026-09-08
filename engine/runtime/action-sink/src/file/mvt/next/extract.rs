use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::reproject::transform_coords_2d;
use reearth_flow_geometry::ops::{ReprojectionCache, Split};
use reearth_flow_geometry::{Euclidean2DGeometry, Geometry};

const WGS84_2D: EpsgCode = EpsgCode::new(4326);

pub(super) enum Leaf {
    Point([f64; 2]),
    LineString(Vec<[f64; 2]>),
    /// Exterior ring first, then holes.
    Polygon(Vec<Vec<[f64; 2]>>),
}

pub(super) fn extract(geometry: &Geometry, cache: &mut ReprojectionCache) -> Vec<Leaf> {
    let mut leaves = Vec::new();
    collect(geometry, cache, &mut leaves);
    leaves
}

fn collect(geometry: &Geometry, cache: &mut ReprojectionCache, out: &mut Vec<Leaf>) {
    match geometry {
        Geometry::None => {}
        Geometry::GeometryCollection(gc) => {
            for member in gc.members() {
                collect(member, cache, out);
            }
        }
        Geometry::Euclidean2D(g) => collect_2d(g, cache, out),
        Geometry::Euclidean3D(_) => tracing::warn!(
            "MVT Writer: 3D geometry is not supported; flatten with Two Dimension Forcer first, skipping"
        ),
    }
}

fn source_crs(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(epsg) => Some(*epsg),
        other => {
            tracing::warn!("MVT Writer: geometry has no geographic CRS ({other:?}); skipping");
            None
        }
    }
}

fn to_lnglat_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    cache: &mut ReprojectionCache,
) -> Option<Vec<[f64; 2]>> {
    let epsg = source_crs(frame)?;
    let mut pts = coords.to_vec();
    if let Err(e) = transform_coords_2d(cache, epsg, WGS84_2D, &mut pts) {
        tracing::warn!("MVT Writer: failed to reproject to WGS84: {e:?}");
        return None;
    }
    Some(pts.into_iter().map(|[lat, lon]| [lon, lat]).collect())
}

// Well-formed rings are closed (first == last); an unclosed ring is corrupt
// input, not an alternate valid form, so it's dropped rather than guessed at.
fn require_closed(mut ring: Vec<[f64; 2]>) -> Option<Vec<[f64; 2]>> {
    if ring.len() < 2 || ring.first() != ring.last() {
        tracing::error!("MVT Writer: polygon ring is not closed (first != last); skipping");
        return None;
    }
    ring.pop();
    Some(ring)
}

fn collect_2d(g: &Euclidean2DGeometry, cache: &mut ReprojectionCache, out: &mut Vec<Leaf>) {
    match g {
        Euclidean2DGeometry::Point(p) => {
            if let Some(mut ll) = to_lnglat_2d(p.frame(), &[p.position()], cache) {
                out.push(Leaf::Point(ll.remove(0)));
            }
        }
        Euclidean2DGeometry::LineString(ls) => {
            if let Some(ll) = to_lnglat_2d(ls.frame(), ls.coords(), cache) {
                out.push(Leaf::LineString(ll));
            }
        }
        Euclidean2DGeometry::Polygon(poly) => {
            let Some(exterior) =
                to_lnglat_2d(poly.frame(), poly.exterior(), cache).and_then(require_closed)
            else {
                return;
            };
            let mut rings = vec![exterior];
            for interior in poly.interiors() {
                if let Some(hole) =
                    to_lnglat_2d(poly.frame(), interior, cache).and_then(require_closed)
                {
                    rings.push(hole);
                }
            }
            out.push(Leaf::Polygon(rings));
        }
        Euclidean2DGeometry::Collection(c) => {
            for member in c.members() {
                collect_2d(member, cache, out);
            }
        }
        Euclidean2DGeometry::PolygonMesh(mesh) => {
            let mut faces = Euclidean2DGeometry::PolygonMesh(mesh.clone());
            let mut emitted = Vec::new();
            if faces.split(&mut |geom, _attrs| emitted.push(geom)).is_err() {
                tracing::warn!("MVT Writer: failed to split polygon mesh into faces, skipping");
                return;
            }
            for face in &emitted {
                collect(face, cache, out);
            }
        }
        other => tracing::warn!("MVT Writer: unsupported 2D geometry, skipping: {other:?}"),
    }
}
