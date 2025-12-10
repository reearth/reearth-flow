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

/// Computes 2D surface area of planar geometries in 3D space (NOT volume).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coordinate::Coordinate;
    use crate::types::line_string::LineString3D;
    use crate::types::polygon::Polygon3D;

    #[test]
    fn test_triangle_area_basic() {
        // Simple right triangle in XY plane: (0,0,0), (1,0,0), (0,1,0)
        // Expected area: 0.5
        let coords = vec![
            (0.0, 0.0, 0.0).into(),
            (1.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            (0.0, 0.0, 0.0).into(), // Close the ring
        ];
        let polygon = Polygon3D::new(LineString3D::new(coords), vec![]);
        let signed_area = polygon.signed_area3d();
        let unsigned_area = polygon.unsigned_area3d();
        println!(
            "Basic triangle signed area: {}, unsigned area: {}",
            signed_area, unsigned_area
        );
        assert!(
            (signed_area - 0.5_f64).abs() < 1e-10,
            "Expected signed area 0.5, got {}",
            signed_area
        );
        assert!(
            (unsigned_area - 0.5_f64).abs() < 1e-10,
            "Expected unsigned area 0.5, got {}",
            unsigned_area
        );
    }

    #[test]
    fn test_triangle_area_with_large_offset() {
        // ECEF-like coordinates with small (1cm) triangle
        let offset = 4000000.0;
        let coords = vec![
            (offset, offset, offset).into(),
            (offset + 0.01, offset, offset).into(),
            (offset, offset + 0.01, offset).into(),
            (offset, offset, offset).into(),
        ];
        let polygon = Polygon3D::new(LineString3D::new(coords), vec![]);
        let signed_area = polygon.signed_area3d();
        let unsigned_area = polygon.unsigned_area3d();
        println!(
            "Large offset triangle signed area: {}, unsigned area: {}",
            signed_area, unsigned_area
        );
        assert!(
            (signed_area - 0.00005_f64).abs() < 1e-6,
            "Expected signed area 0.00005 with offset, got {}",
            signed_area
        );
        assert!(
            (unsigned_area - 0.00005_f64).abs() < 1e-6,
            "Expected unsigned area 0.00005 with offset, got {}",
            unsigned_area
        );
    }

    #[test]
    fn test_polygon_with_hole_all_winding_combinations() {
        // Define coordinate data at the beginning
        // Outer square: 2x2 (area = 4), Inner square: 1x1 (area = 1), Expected net area: 3.0
        let exterior_ccw: Vec<Coordinate<f64, f64>> = vec![
            (0.0, 0.0, 0.0).into(),
            (2.0, 0.0, 0.0).into(),
            (2.0, 2.0, 0.0).into(),
            (0.0, 2.0, 0.0).into(),
            (0.0, 0.0, 0.0).into(),
        ];
        let exterior_cw: Vec<Coordinate<f64, f64>> = vec![
            (0.0, 0.0, 0.0).into(),
            (0.0, 2.0, 0.0).into(),
            (2.0, 2.0, 0.0).into(),
            (2.0, 0.0, 0.0).into(),
            (0.0, 0.0, 0.0).into(),
        ];
        let interior_ccw: Vec<Coordinate<f64, f64>> = vec![
            (0.5, 0.5, 0.0).into(),
            (1.5, 0.5, 0.0).into(),
            (1.5, 1.5, 0.0).into(),
            (0.5, 1.5, 0.0).into(),
            (0.5, 0.5, 0.0).into(),
        ];
        let interior_cw: Vec<Coordinate<f64, f64>> = vec![
            (0.5, 0.5, 0.0).into(),
            (0.5, 1.5, 0.0).into(),
            (1.5, 1.5, 0.0).into(),
            (1.5, 0.5, 0.0).into(),
            (0.5, 0.5, 0.0).into(),
        ];

        // Helper function to test both signed and unsigned area
        let test_winding = |ext: &[Coordinate<f64, f64>],
                            int: &[Coordinate<f64, f64>],
                            expected_signed: f64,
                            name: &str| {
            let polygon = Polygon3D::new(
                LineString3D::new(ext.to_vec()),
                vec![LineString3D::new(int.to_vec())],
            );
            let signed = polygon.signed_area3d();
            let unsigned = polygon.unsigned_area3d();
            assert!(
                (signed - expected_signed).abs() < 1e-10,
                "{} signed: expected {}, got {}",
                name,
                expected_signed,
                signed
            );
            assert!(
                (unsigned - 3.0_f64).abs() < 1e-10,
                "{} unsigned: expected 3.0, got {}",
                name,
                unsigned
            );
        };

        // Test all 4 combinations (CCW exterior = positive, CW exterior = negative)
        test_winding(&exterior_ccw, &interior_ccw, 3.0_f64, "CCW/CCW");
        test_winding(&exterior_ccw, &interior_cw, 3.0_f64, "CCW/CW");
        test_winding(&exterior_cw, &interior_ccw, -3.0_f64, "CW/CCW");
        test_winding(&exterior_cw, &interior_cw, -3.0_f64, "CW/CW");
    }
}
