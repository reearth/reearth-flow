use crate::types::{
    coordinate::Coordinate3D, geometry::Geometry3D, geometry_collection::GeometryCollection3D,
    line::Line3D, line_string::LineString3D, multi_line_string::MultiLineString3D,
    multi_point::MultiPoint3D, multi_polygon::MultiPolygon3D, point::Point3D, polygon::Polygon3D,
    rect::Rect3D, triangle::Triangle3D,
};

use super::query::RotateQuery3D;
pub trait Rotate3D {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self;
}

impl Rotate3D for Coordinate3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        rotate_coordinates(*self, &query, origin)
    }
}

impl Rotate3D for Point3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        query.rotate(*self, origin)
    }
}

impl Rotate3D for LineString3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        rotate_line_string(self, &query, origin)
    }
}

impl Rotate3D for Line3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        let start = self.start.rotate_3d(query.clone(), origin);
        let end = self.end.rotate_3d(query, origin);
        Line3D::new_(start, end)
    }
}

impl Rotate3D for Polygon3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        rotate_polygon(self, &query, origin)
    }
}

impl Rotate3D for MultiPoint3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        MultiPoint3D::new(
            self.0
                .iter()
                .map(|p| p.rotate_3d(query.clone(), origin))
                .collect(),
        )
    }
}

impl Rotate3D for MultiLineString3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        MultiLineString3D::new(
            self.0
                .iter()
                .map(|ls| ls.rotate_3d(query.clone(), origin))
                .collect(),
        )
    }
}

impl Rotate3D for MultiPolygon3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        MultiPolygon3D::new(
            self.0
                .iter()
                .map(|p| p.rotate_3d(query.clone(), origin))
                .collect(),
        )
    }
}

impl Rotate3D for Rect3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        Rect3D::new(
            self.min().rotate_3d(query.clone(), origin),
            self.max().rotate_3d(query, origin),
        )
    }
}

impl Rotate3D for Triangle3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        Triangle3D::new(
            self.0.rotate_3d(query.clone(), origin),
            self.1.rotate_3d(query.clone(), origin),
            self.2.rotate_3d(query, origin),
        )
    }
}

impl Rotate3D for Geometry3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        match self {
            Geometry3D::Point(p) => Geometry3D::Point(p.rotate_3d(query.clone(), origin)),
            Geometry3D::Line(l) => Geometry3D::Line(l.rotate_3d(query.clone(), origin)),
            Geometry3D::LineString(ls) => {
                Geometry3D::LineString(ls.rotate_3d(query.clone(), origin))
            }
            Geometry3D::Polygon(p) => Geometry3D::Polygon(p.rotate_3d(query.clone(), origin)),
            Geometry3D::MultiPoint(mp) => {
                Geometry3D::MultiPoint(mp.rotate_3d(query.clone(), origin))
            }
            Geometry3D::MultiLineString(mls) => {
                Geometry3D::MultiLineString(mls.rotate_3d(query.clone(), origin))
            }
            Geometry3D::MultiPolygon(mp) => {
                Geometry3D::MultiPolygon(mp.rotate_3d(query.clone(), origin))
            }
            Geometry3D::Rect(r) => Geometry3D::Rect(r.rotate_3d(query.clone(), origin)),
            Geometry3D::Triangle(t) => Geometry3D::Triangle(t.rotate_3d(query, origin)),
            _ => unimplemented!(),
        }
    }
}

impl Rotate3D for GeometryCollection3D<f64> {
    fn rotate_3d(&self, query: RotateQuery3D, origin: Option<Point3D<f64>>) -> Self {
        GeometryCollection3D::new(
            self.0
                .iter()
                .map(|g| g.rotate_3d(query.clone(), origin))
                .collect(),
        )
    }
}

fn rotate_coordinates(
    coords: Coordinate3D<f64>,
    query: &RotateQuery3D,
    origin: Option<Point3D<f64>>,
) -> Coordinate3D<f64> {
    let point = Point3D::new(coords.x, coords.y, coords.z);
    let rotated = query.rotate(point, origin);
    Coordinate3D::new__(rotated.x(), rotated.y(), rotated.z())
}

fn rotate_polygon(
    polygon: &Polygon3D<f64>,
    query: &RotateQuery3D,
    origin: Option<Point3D<f64>>,
) -> Polygon3D<f64> {
    let rotated_exterior = LineString3D::new(
        polygon
            .exterior()
            .coords()
            .map(|c| rotate_coordinates(*c, query, origin))
            .collect(),
    );

    let rotated_interiors = polygon
        .interiors()
        .iter()
        .map(|ls| {
            LineString3D::new(
                ls.coords()
                    .map(|c| rotate_coordinates(*c, query, origin))
                    .collect(),
            )
        })
        .collect();

    Polygon3D::new(rotated_exterior, rotated_interiors)
}

fn rotate_line_string(
    line_string: &LineString3D<f64>,
    query: &RotateQuery3D,
    origin: Option<Point3D<f64>>,
) -> LineString3D<f64> {
    LineString3D::new(
        line_string
            .coords()
            .map(|c| rotate_coordinates(*c, query, origin))
            .collect(),
    )
}
