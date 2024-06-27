use crate::types::{
    geometry::Geometry, geometry_collection::GeometryCollection, line::Line,
    line_string::LineString, multi_line_string::MultiLineString, multi_point::MultiPoint,
    multi_polygon::MultiPolygon, point::Point, polygon::Polygon, rect::Rect, solid::Solid,
    triangle::Triangle,
};

use super::{
    coords_iter::CoordsIter,
    geometry_cow::GeometryCow,
    kernels::{Orientation, RobustKernel},
    CoordNum, GeoNum,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum Dimensions {
    /// Some geometries, like a `MultiPoint` or `GeometryCollection` may have no elements - thus no
    /// dimensions. Note that this is distinct from being `ZeroDimensional`, like a `Point`.
    Empty,
    /// Dimension of a point
    ZeroDimensional,
    /// Dimension of a line or curve
    OneDimensional,
    /// Dimension of a surface
    TwoDimensional,
}

pub trait HasDimensions {
    fn is_empty(&self) -> bool;

    fn dimensions(&self) -> Dimensions;

    fn boundary_dimensions(&self) -> Dimensions;
}

impl<T: GeoNum, Z: GeoNum> HasDimensions for GeometryCow<'_, T, Z> {
    crate::geometry_cow_delegate_impl! {
        fn is_empty(&self) -> bool;
        fn dimensions(&self) -> Dimensions;
        fn boundary_dimensions(&self) -> Dimensions;
    }
}

impl<T: GeoNum, Z: GeoNum> HasDimensions for Vec<Geometry<T, Z>> {
    fn is_empty(&self) -> bool {
        self.iter().all(Geometry::is_empty)
    }
    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let dimensions = geom.dimensions();
            if dimensions == Dimensions::TwoDimensional {
                // short-circuit since we know none can be larger
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }
    fn boundary_dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let d = geom.boundary_dimensions();

            if d == Dimensions::OneDimensional {
                return Dimensions::OneDimensional;
            }

            max = max.max(d);
        }
        max
    }
}

impl<T: GeoNum, Z: GeoNum> HasDimensions for Geometry<T, Z> {
    crate::geometry_delegate_impl! {
        fn is_empty(&self) -> bool;
        fn dimensions(&self) -> Dimensions;
        fn boundary_dimensions(&self) -> Dimensions;
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for Solid<T, Z> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for Point<T, Z> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for Line<T, Z> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.start == self.end {
            // degenerate line is a point
            Dimensions::ZeroDimensional
        } else {
            Dimensions::OneDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.start == self.end {
            // degenerate line is a point, which has no boundary
            Dimensions::Empty
        } else {
            Dimensions::ZeroDimensional
        }
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for LineString<T, Z> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        let first = self.0[0];
        if self.0.iter().any(|&coord| first != coord) {
            Dimensions::OneDimensional
        } else {
            // all coords are the same - i.e. a point
            Dimensions::ZeroDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string cannot be 2 dimensional"),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for Polygon<T, Z> {
    fn is_empty(&self) -> bool {
        self.exterior().is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        let mut coords = self.exterior_coords_iter();
        match coords.next() {
            None => Dimensions::Empty,
            Some(coord_0) => {
                if coords.all(|coord_n| coord_0 == coord_n) {
                    // all coords are a single point
                    Dimensions::ZeroDimensional
                } else {
                    Dimensions::TwoDimensional
                }
            }
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::OneDimensional
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for MultiPoint<T, Z> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for MultiLineString<T, Z> {
    fn is_empty(&self) -> bool {
        self.iter().all(LineString::is_empty)
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for line in &self.0 {
            match line.dimensions() {
                Dimensions::Empty => {}
                Dimensions::ZeroDimensional => max = Dimensions::ZeroDimensional,
                Dimensions::OneDimensional => {
                    // return early since we know multi line string dimensionality cannot exceed
                    // 1-d
                    return Dimensions::OneDimensional;
                }
                Dimensions::TwoDimensional => unreachable!("MultiLineString cannot be 2d"),
            }
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string cannot be 2 dimensional"),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for MultiPolygon<T, Z> {
    fn is_empty(&self) -> bool {
        self.iter().all(Polygon::is_empty)
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::TwoDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::OneDimensional
    }
}

impl<T: CoordNum, Z: CoordNum> HasDimensions for Rect<T, Z> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.min() == self.max() {
            // degenerate rectangle is a point
            Dimensions::ZeroDimensional
        } else if self.min().x == self.max().x || self.min().y == self.max().y {
            // degenerate rectangle is a line
            Dimensions::OneDimensional
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty => {
                unreachable!("even a degenerate rect should be at least 0-Dimensional")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<T: GeoNum, Z: GeoNum> HasDimensions for Triangle<T, Z> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if Orientation::Collinear == RobustKernel::orient(self.0, self.1, self.2, None) {
            if self.0 == self.1 && self.1 == self.2 {
                // degenerate triangle is a point
                Dimensions::ZeroDimensional
            } else {
                // degenerate triangle is a line
                Dimensions::OneDimensional
            }
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty => {
                unreachable!("even a degenerate triangle should be at least 0-dimensional")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<T: GeoNum, Z: GeoNum> HasDimensions for GeometryCollection<T, Z> {
    fn is_empty(&self) -> bool {
        if self.0.is_empty() {
            true
        } else {
            self.iter().all(Geometry::is_empty)
        }
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let dimensions = geom.dimensions();
            if dimensions == Dimensions::TwoDimensional {
                // short-circuit since we know none can be larger
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let d = geom.boundary_dimensions();

            if d == Dimensions::OneDimensional {
                return Dimensions::OneDimensional;
            }

            max = max.max(d);
        }
        max
    }
}
