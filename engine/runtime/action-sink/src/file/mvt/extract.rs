use std::sync::atomic::Ordering;

use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString;
use reearth_flow_geometry::types::polygon::Polygon;
use reearth_flow_types::geometry::{GeometryType, GeometryValue};

use super::Skipped;

pub(super) enum Leaf {
    Point([f64; 2]),
    LineString(Vec<[f64; 2]>),
    /// Exterior ring first, then holes; rings are open (no closing vertex).
    Polygon(Vec<Vec<[f64; 2]>>),
}

/// Geometry is taken as already lng/lat (x = lng, y = lat) and heights are
/// dropped, as the previous MVT Writer did; nothing is reprojected.
pub(super) fn extract(geometry: &GeometryValue, skipped: &Skipped) -> Vec<Leaf> {
    let mut leaves = Vec::new();
    match geometry {
        GeometryValue::CityGmlGeometry(city) => {
            for entry in &city.gml_geometries {
                match entry.ty {
                    GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                        for poly in &entry.polygons {
                            push_polygon(poly, &mut leaves);
                        }
                    }
                    GeometryType::Curve => {
                        for line in &entry.line_strings {
                            leaves.push(Leaf::LineString(coords(line)));
                        }
                    }
                    GeometryType::Point => {
                        for point in &entry.points {
                            leaves.push(Leaf::Point(xy(point)));
                        }
                    }
                }
            }
        }
        GeometryValue::FlowGeometry2D(g) => collect_2d(g, &mut leaves, skipped),
        GeometryValue::FlowGeometry3D(_) => {
            skipped.geometries.fetch_add(1, Ordering::Relaxed);
        }
        GeometryValue::None => {}
    }
    leaves
}

fn collect_2d(g: &Geometry2D<f64>, out: &mut Vec<Leaf>, skipped: &Skipped) {
    match g {
        Geometry2D::Point(p) => out.push(Leaf::Point(xy(&p.0))),
        Geometry2D::MultiPoint(mp) => {
            out.extend(mp.iter().map(|p| Leaf::Point(xy(&p.0))));
        }
        Geometry2D::LineString(line) => out.push(Leaf::LineString(coords(line))),
        Geometry2D::MultiLineString(mls) => {
            out.extend(mls.iter().map(|line| Leaf::LineString(coords(line))));
        }
        Geometry2D::Polygon(poly) => push_polygon(poly, out),
        Geometry2D::MultiPolygon(mpoly) => {
            for poly in mpoly.iter() {
                push_polygon(poly, out);
            }
        }
        Geometry2D::GeometryCollection(members) => {
            for member in members {
                collect_2d(member, out, skipped);
            }
        }
        _ => {
            skipped.geometries.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn xy<Z: CoordNum>(c: &Coordinate<f64, Z>) -> [f64; 2] {
    [c.x, c.y]
}

fn coords<Z: CoordNum>(line: &LineString<f64, Z>) -> Vec<[f64; 2]> {
    line.iter().map(xy).collect()
}

// Flow rings repeat their first vertex at the end; the slicer wants them open.
fn open_ring<Z: CoordNum>(ring: &LineString<f64, Z>) -> Vec<[f64; 2]> {
    let mut ring = coords(ring);
    if ring.len() >= 2 && ring.first() == ring.last() {
        ring.pop();
    }
    ring
}

fn push_polygon<Z: CoordNum>(poly: &Polygon<f64, Z>, out: &mut Vec<Leaf>) {
    let exterior = open_ring(poly.exterior());
    if exterior.len() < 3 {
        return;
    }
    let mut rings = vec![exterior];
    rings.extend(
        poly.interiors()
            .iter()
            .map(open_ring)
            .filter(|hole| hole.len() >= 3),
    );
    out.push(Leaf::Polygon(rings));
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::coordinate::Coordinate2D;
    use reearth_flow_geometry::types::line_string::LineString2D;
    use reearth_flow_geometry::types::polygon::Polygon2D;

    use super::*;

    #[test]
    fn a_closed_flow_ring_becomes_an_open_leaf_ring() {
        let ring: Vec<Coordinate2D<f64>> = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]
            .iter()
            .map(|&[x, y]| Coordinate2D::new_(x, y))
            .collect();
        let poly = Polygon2D::new(LineString2D::new(ring), vec![]);
        let leaves = extract(
            &GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)),
            &Skipped::default(),
        );

        let [Leaf::Polygon(rings)] = leaves.as_slice() else {
            panic!("expected one polygon leaf");
        };
        assert_eq!(rings, &vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]]);
    }
}
