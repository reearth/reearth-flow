use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::reproject::{transform_coords_2d, transform_coords_3d};
use reearth_flow_geometry::ops::{ReprojectionCache, Split};
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

const WEB_MERCATOR: EpsgCode = EpsgCode::new(3857);

pub(super) enum Leaf {
    /// Web Mercator (EPSG:3857) meters.
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
        Geometry::Euclidean3D(g) => collect_3d(g, cache, out),
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

fn to_mercator_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    cache: &mut ReprojectionCache,
) -> Option<Vec<[f64; 2]>> {
    let epsg = source_crs(frame)?;
    let mut pts = coords.to_vec();
    if let Err(e) = transform_coords_2d(cache, epsg, WEB_MERCATOR, &mut pts) {
        tracing::warn!("MVT Writer: failed to reproject to Web Mercator: {e:?}");
        return None;
    }
    Some(pts)
}

fn to_mercator_3d(
    frame: &CoordinateFrame,
    coords: &[[f64; 3]],
    cache: &mut ReprojectionCache,
) -> Option<Vec<[f64; 2]>> {
    let epsg = source_crs(frame)?;
    let mut pts = coords.to_vec();
    if let Err(e) = transform_coords_3d(cache, epsg, WEB_MERCATOR, &mut pts) {
        tracing::warn!("MVT Writer: failed to reproject to Web Mercator: {e:?}");
        return None;
    }
    Some(pts.into_iter().map(|[x, y, _height]| [x, y]).collect())
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
            if let Some(mut ll) = to_mercator_2d(p.frame(), &[p.position()], cache) {
                out.push(Leaf::Point(ll.remove(0)));
            }
        }
        Euclidean2DGeometry::LineString(ls) => {
            if let Some(ll) = to_mercator_2d(ls.frame(), ls.coords(), cache) {
                out.push(Leaf::LineString(ll));
            }
        }
        Euclidean2DGeometry::Polygon(poly) => {
            let Some(exterior) =
                to_mercator_2d(poly.frame(), poly.exterior(), cache).and_then(require_closed)
            else {
                return;
            };
            let mut rings = vec![exterior];
            for interior in poly.interiors() {
                if let Some(hole) =
                    to_mercator_2d(poly.frame(), interior, cache).and_then(require_closed)
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

fn collect_3d(g: &Euclidean3DGeometry, cache: &mut ReprojectionCache, out: &mut Vec<Leaf>) {
    match g {
        Euclidean3DGeometry::Point(p) => {
            if let Some(mut ll) = to_mercator_3d(p.frame(), &[p.position()], cache) {
                out.push(Leaf::Point(ll.remove(0)));
            }
        }
        Euclidean3DGeometry::LineString(ls) => {
            if let Some(ll) = to_mercator_3d(ls.frame(), ls.coords(), cache) {
                out.push(Leaf::LineString(ll));
            }
        }
        Euclidean3DGeometry::Polygon(poly) => {
            let Some(exterior) =
                to_mercator_3d(poly.frame(), poly.exterior(), cache).and_then(require_closed)
            else {
                return;
            };
            let mut rings = vec![exterior];
            for interior in poly.interiors() {
                if let Some(hole) =
                    to_mercator_3d(poly.frame(), interior, cache).and_then(require_closed)
                {
                    rings.push(hole);
                }
            }
            out.push(Leaf::Polygon(rings));
        }
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                collect_3d(member, cache, out);
            }
        }
        other => tracing::warn!(
            "MVT Writer: only Point/LineString/Polygon (optionally in a Collection) are \
             supported; skipping {other:?}"
        ),
    }
}
