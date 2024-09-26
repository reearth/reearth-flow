use crate::types::{
    coordnum::{CoordFloat, CoordNum},
    geometry::Geometry3D,
    geometry_collection::GeometryCollection3D,
    line::Line3D,
    line_string::LineString3D,
    multi_line_string::MultiLineString3D,
    multi_point::MultiPoint3D,
    multi_polygon::MultiPolygon3D,
    point::Point3D,
    polygon::Polygon3D,
    rect::Rect3D,
    triangle::Triangle3D,
};

use super::map_coords::MapCoords;

pub(crate) fn twice_signed_ring_area3d<T>(linestring: &LineString3D<T>) -> T
where
    T: CoordNum,
{
    // LineString with less than 3 points is empty, or a
    // single point, or is not closed.
    if linestring.0.len() < 3 {
        return T::zero();
    }

    // Above test ensures the vector has at least 2 elements.
    // We check if linestring is closed, and return 0 otherwise.
    if linestring.0.first().unwrap() != linestring.0.last().unwrap() {
        return T::zero();
    }

    // Use a reasonable shift for the line-string coords
    // to avoid numerical-errors when summing the
    // determinants.
    //
    // Note: we can't use the `Centroid` trait as it
    // requires `T: Float` and in fact computes area in the
    // implementation. Another option is to use the average
    // of the coordinates, but it is not fool-proof to
    // divide by the length of the linestring (eg. a long
    // line-string with T = u8)
    let shift = linestring.0[0];

    let mut tmp = T::zero();
    for line in linestring.lines() {
        let line = line.map_coords(|c| c - shift);
        tmp = tmp + line.determinant3d();
    }

    tmp
}

pub trait Area3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T;

    fn unsigned_area3d(&self) -> T;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area3d<T>(linestring: &LineString3D<T>) -> T
where
    T: CoordFloat,
{
    twice_signed_ring_area3d(linestring) / (T::one() + T::one())
}

impl<T> Area3D<T> for Point3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T {
        T::zero()
    }

    fn unsigned_area3d(&self) -> T {
        T::zero()
    }
}

impl<T> Area3D<T> for LineString3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T {
        T::zero()
    }

    fn unsigned_area3d(&self) -> T {
        T::zero()
    }
}

impl<T> Area3D<T> for Line3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T {
        T::zero()
    }

    fn unsigned_area3d(&self) -> T {
        T::zero()
    }
}

/// **Note.** The implementation handles polygons whose
/// holes do not all have the same orientation. The sign of
/// the output is the same as that of the exterior shell.
impl<T> Area3D<T> for Polygon3D<T>
where
    T: CoordFloat,
{
    fn signed_area3d(&self) -> T {
        let area = get_linestring_area3d(self.exterior());

        // We could use winding order here, but that would
        // result in computing the shoelace formula twice.
        let is_negative = area < T::zero();

        let area = self.interiors().iter().fold(area.abs(), |total, next| {
            total - get_linestring_area3d(next).abs()
        });

        if is_negative {
            -area
        } else {
            area
        }
    }

    fn unsigned_area3d(&self) -> T {
        self.signed_area3d().abs()
    }
}

impl<T> Area3D<T> for MultiPoint3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T {
        T::zero()
    }

    fn unsigned_area3d(&self) -> T {
        T::zero()
    }
}

impl<T> Area3D<T> for MultiLineString3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T {
        T::zero()
    }

    fn unsigned_area3d(&self) -> T {
        T::zero()
    }
}

impl<T> Area3D<T> for MultiPolygon3D<T>
where
    T: CoordFloat,
{
    fn signed_area3d(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.signed_area3d())
    }

    fn unsigned_area3d(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.signed_area3d().abs())
    }
}

/// Because a `Rect` has no winding order, the area will always be positive.
impl<T> Area3D<T> for Rect3D<T>
where
    T: CoordNum,
{
    fn signed_area3d(&self) -> T {
        self.width() * self.height()
    }

    fn unsigned_area3d(&self) -> T {
        self.width() * self.height()
    }
}

impl<T> Area3D<T> for Triangle3D<T>
where
    T: CoordFloat,
{
    fn signed_area3d(&self) -> T {
        self.to_lines()
            .iter()
            .fold(T::zero(), |total, line| total + line.determinant3d())
            / (T::one() + T::one())
    }

    fn unsigned_area3d(&self) -> T {
        self.signed_area3d().abs()
    }
}

impl<T> Area3D<T> for Geometry3D<T>
where
    T: CoordFloat,
{
    fn signed_area3d(&self) -> T {
        match self {
            Geometry3D::Point(p) => p.signed_area3d(),
            Geometry3D::Line(l) => l.signed_area3d(),
            Geometry3D::LineString(ls) => ls.signed_area3d(),
            Geometry3D::Polygon(p) => p.signed_area3d(),
            Geometry3D::MultiPoint(mp) => mp.signed_area3d(),
            Geometry3D::MultiLineString(mls) => mls.signed_area3d(),
            Geometry3D::MultiPolygon(mp) => mp.signed_area3d(),
            Geometry3D::Rect(r) => r.signed_area3d(),
            Geometry3D::Triangle(t) => t.signed_area3d(),
            _ => unimplemented!(),
        }
    }
    fn unsigned_area3d(&self) -> T {
        match self {
            Geometry3D::Point(p) => p.unsigned_area3d(),
            Geometry3D::Line(l) => l.unsigned_area3d(),
            Geometry3D::LineString(ls) => ls.unsigned_area3d(),
            Geometry3D::Polygon(p) => p.unsigned_area3d(),
            Geometry3D::MultiPoint(mp) => mp.unsigned_area3d(),
            Geometry3D::MultiLineString(mls) => mls.unsigned_area3d(),
            Geometry3D::MultiPolygon(mp) => mp.unsigned_area3d(),
            Geometry3D::Rect(r) => r.unsigned_area3d(),
            Geometry3D::Triangle(t) => t.unsigned_area3d(),
            _ => unimplemented!(),
        }
    }
}

impl<T> Area3D<T> for GeometryCollection3D<T>
where
    T: CoordFloat,
{
    fn signed_area3d(&self) -> T {
        self.0
            .iter()
            .map(|g| g.signed_area3d())
            .fold(T::zero(), |acc, next| acc + next)
    }

    fn unsigned_area3d(&self) -> T {
        self.0
            .iter()
            .map(|g| g.unsigned_area3d())
            .fold(T::zero(), |acc, next| acc + next)
    }
}
