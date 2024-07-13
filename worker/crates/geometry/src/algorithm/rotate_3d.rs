use crate::{
    types::{
        coordinate::Coordinate3D, geometry::Geometry3D, geometry_collection::GeometryCollection3D,
        line::Line3D, line_string::LineString3D, multi_line_string::MultiLineString3D,
        multi_point::MultiPoint3D, multi_polygon::MultiPolygon3D, point::Point3D,
        polygon::Polygon3D, rect::Rect3D, triangle::Triangle3D,
    },
    utils::rotate_3d_custom_axis,
};

pub trait Rotate3D {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self;
}

impl Rotate3D for Coordinate3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        let result = rotate_3d_custom_axis(
            vec![nalgebra::Point3::new(self.x, self.y, self.z)],
            angle_degrees,
            origin,
            direction,
        );
        Coordinate3D::new__(result[0].x, result[0].y, result[0].z)
    }
}

impl Rotate3D for Point3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        let result = rotate_3d_custom_axis(
            vec![nalgebra::Point3::new(self.x(), self.y(), self.z())],
            angle_degrees,
            origin,
            direction,
        );
        Point3D::new_(result[0].x, result[0].y, result[0].z)
    }
}

impl Rotate3D for LineString3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        LineString3D::new(
            self.coords()
                .map(|c| c.rotate_3d(angle_degrees, origin, direction))
                .collect(),
        )
    }
}

impl Rotate3D for Line3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        let start = self.start.rotate_3d(angle_degrees, origin, direction);
        let end = self.end.rotate_3d(angle_degrees, origin, direction);
        Line3D::new_(start, end)
    }
}

impl Rotate3D for Polygon3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        Polygon3D::new(
            self.exterior().rotate_3d(angle_degrees, origin, direction),
            self.interiors()
                .iter()
                .map(|ls| ls.rotate_3d(angle_degrees, origin, direction))
                .collect(),
        )
    }
}

impl Rotate3D for MultiPoint3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        MultiPoint3D::new(
            self.0
                .iter()
                .map(|p| p.rotate_3d(angle_degrees, origin, direction))
                .collect(),
        )
    }
}

impl Rotate3D for MultiLineString3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        MultiLineString3D::new(
            self.0
                .iter()
                .map(|ls| ls.rotate_3d(angle_degrees, origin, direction))
                .collect(),
        )
    }
}

impl Rotate3D for MultiPolygon3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        MultiPolygon3D::new(
            self.0
                .iter()
                .map(|p| p.rotate_3d(angle_degrees, origin, direction))
                .collect(),
        )
    }
}

impl Rotate3D for Rect3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        Rect3D::new(
            self.min().rotate_3d(angle_degrees, origin, direction),
            self.max().rotate_3d(angle_degrees, origin, direction),
        )
    }
}

impl Rotate3D for Triangle3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        Triangle3D::new(
            self.0.rotate_3d(angle_degrees, origin, direction),
            self.1.rotate_3d(angle_degrees, origin, direction),
            self.2.rotate_3d(angle_degrees, origin, direction),
        )
    }
}

impl Rotate3D for Geometry3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        match self {
            Geometry3D::Point(p) => {
                Geometry3D::Point(p.rotate_3d(angle_degrees, origin, direction))
            }
            Geometry3D::Line(l) => Geometry3D::Line(l.rotate_3d(angle_degrees, origin, direction)),
            Geometry3D::LineString(ls) => {
                Geometry3D::LineString(ls.rotate_3d(angle_degrees, origin, direction))
            }
            Geometry3D::Polygon(p) => {
                Geometry3D::Polygon(p.rotate_3d(angle_degrees, origin, direction))
            }
            Geometry3D::MultiPoint(mp) => {
                Geometry3D::MultiPoint(mp.rotate_3d(angle_degrees, origin, direction))
            }
            Geometry3D::MultiLineString(mls) => {
                Geometry3D::MultiLineString(mls.rotate_3d(angle_degrees, origin, direction))
            }
            Geometry3D::MultiPolygon(mp) => {
                Geometry3D::MultiPolygon(mp.rotate_3d(angle_degrees, origin, direction))
            }
            Geometry3D::Rect(r) => Geometry3D::Rect(r.rotate_3d(angle_degrees, origin, direction)),
            Geometry3D::Triangle(t) => {
                Geometry3D::Triangle(t.rotate_3d(angle_degrees, origin, direction))
            }
            _ => unimplemented!(),
        }
    }
}

impl Rotate3D for GeometryCollection3D<f64> {
    fn rotate_3d(
        &self,
        angle_degrees: f64,
        origin: Option<Point3D<f64>>,
        direction: Point3D<f64>,
    ) -> Self {
        GeometryCollection3D::new(
            self.0
                .iter()
                .map(|g| g.rotate_3d(angle_degrees, origin, direction))
                .collect(),
        )
    }
}
