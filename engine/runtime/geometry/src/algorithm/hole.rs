use crate::types::{
    coordinate::Coordinate, coordnum::CoordNum, csg::CSG, face::Face, geometry::Geometry, line::Line, line_string::LineString, multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon, point::Point, polygon::Polygon, rect::Rect, solid::Solid
};

pub trait HoleCounter<T: CoordNum, Z: CoordNum> {
    fn hole_count(&self) -> usize {
        0
    }
}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Coordinate<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for CSG<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Point<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for MultiPoint<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Line<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for LineString<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for MultiLineString<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Polygon<T, Z> {
    fn hole_count(&self) -> usize {
        self.interiors().len()
    }
}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for MultiPolygon<T, Z> {
    fn hole_count(&self) -> usize {
        self.iter().map(|p| p.hole_count()).sum()
    }
}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Face<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Solid<T, Z> {}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Rect<T, Z> {
    fn hole_count(&self) -> usize {
        self.to_polygon().hole_count()
    }
}

impl<T: CoordNum, Z: CoordNum> HoleCounter<T, Z> for Geometry<T, Z> {
    fn hole_count(&self) -> usize {
        match self {
            Geometry::CSG(c) => c.hole_count(),
            Geometry::Point(p) => p.hole_count(),
            Geometry::Line(l) => l.hole_count(),
            Geometry::LineString(ls) => ls.hole_count(),
            Geometry::Polygon(p) => p.hole_count(),
            Geometry::MultiPoint(mp) => mp.hole_count(),
            Geometry::MultiLineString(mls) => mls.hole_count(),
            Geometry::MultiPolygon(mp) => mp.hole_count(),
            Geometry::Rect(rect) => rect.hole_count(),
            Geometry::Triangle(_) => unimplemented!(),
            Geometry::Solid(s) => s.hole_count(),
            Geometry::GeometryCollection(gc) => gc.iter().map(|g| g.hole_count()).sum(),
        }
    }
}
