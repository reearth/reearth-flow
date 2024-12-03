use std::fmt::Debug;

use std::{fmt, iter, marker, slice};

use crate::types::coordinate::Coordinate;
use crate::types::coordnum::CoordNum;
use crate::types::geometry::Geometry;
use crate::types::geometry_collection::GeometryCollection;
use crate::types::line::Line;
use crate::types::line_string::LineString;
use crate::types::multi_line_string::MultiLineString;
use crate::types::multi_point::MultiPoint;
use crate::types::multi_polygon::MultiPolygon;
use crate::types::point::Point;
use crate::types::polygon::Polygon;
use crate::types::rect::Rect;
use crate::types::triangle::Triangle;

type CoordinateChainOnce<T, Z> =
    iter::Chain<iter::Once<Coordinate<T, Z>>, iter::Once<Coordinate<T, Z>>>;

/// Iterate over geometry coordinates.
pub trait CoordsIter {
    type Iter<'a>: Iterator<Item = Coordinate<Self::ScalarXY, Self::ScalarZ>>
    where
        Self: 'a;
    type ExteriorIter<'a>: Iterator<Item = Coordinate<Self::ScalarXY, Self::ScalarZ>>
    where
        Self: 'a;
    type ScalarXY: CoordNum;
    type ScalarZ: CoordNum;

    fn coords_iter(&self) -> Self::Iter<'_>;
    fn coords_count(&self) -> usize;
    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_>;
}

// ┌──────────────────────────┐
// │ Implementation for Point │
// └──────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for Point<T, Z> {
    type Iter<'a>
        = iter::Once<Coordinate<T, Z>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.0)
    }

    /// Return the number of coordinates in the `Point`.
    fn coords_count(&self) -> usize {
        1
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────┐
// │ Implementation for Line │
// └─────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for Line<T, Z> {
    type Iter<'a>
        = iter::Chain<iter::Once<Coordinate<T, Z>>, iter::Once<Coordinate<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.start).chain(iter::once(self.end))
    }

    /// Return the number of coordinates in the `Line`.
    fn coords_count(&self) -> usize {
        2
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌───────────────────────────────┐
// │ Implementation for LineString │
// └───────────────────────────────┘

type LineStringIter<'a, T, Z> = iter::Copied<slice::Iter<'a, Coordinate<T, Z>>>;

impl<T: CoordNum, Z: CoordNum> CoordsIter for LineString<T, Z> {
    type Iter<'a>
        = LineStringIter<'a, T, Z>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.0.iter().copied()
    }

    /// Return the number of coordinates in the `LineString`.
    fn coords_count(&self) -> usize {
        self.0.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌────────────────────────────┐
// │ Implementation for Polygon │
// └────────────────────────────┘

type PolygonIter<'a, T, Z> = iter::Chain<
    LineStringIter<'a, T, Z>,
    iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T, Z>>, LineString<T, Z>>>,
>;

impl<T: CoordNum, Z: CoordNum> CoordsIter for Polygon<T, Z> {
    type Iter<'a>
        = PolygonIter<'a, T, Z>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = LineStringIter<'a, T, Z>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.exterior()
            .coords_iter()
            .chain(MapCoordsIter(self.interiors().iter(), marker::PhantomData).flatten())
    }

    /// Return the number of coordinates in the `Polygon`.
    fn coords_count(&self) -> usize {
        self.exterior().coords_count()
            + self
                .interiors()
                .iter()
                .map(|i| i.coords_count())
                .sum::<usize>()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.exterior().coords_iter()
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for MultiPoint<T, Z> {
    type Iter<'a>
        = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Point<T, Z>>, Point<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPoint`.
    fn coords_count(&self) -> usize {
        self.0.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for MultiLineString<T, Z> {
    type Iter<'a>
        = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T, Z>>, LineString<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiLineString`.
    fn coords_count(&self) -> usize {
        self.0
            .iter()
            .map(|line_string| line_string.coords_count())
            .sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for MultiPolygon<T, Z> {
    type Iter<'a>
        = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Polygon<T, Z>>, Polygon<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = iter::Flatten<MapExteriorCoordsIter<'a, T, slice::Iter<'a, Polygon<T, Z>>, Polygon<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPolygon`.
    fn coords_count(&self) -> usize {
        self.0.iter().map(|polygon| polygon.coords_count()).sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        MapExteriorCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }
}

// ┌───────────────────────────────────────┐
// │ Implementation for GeometryCollection │
// └───────────────────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for GeometryCollection<T, Z> {
    type Iter<'a>
        = Box<dyn Iterator<Item = Coordinate<T, Z>> + 'a>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Box<dyn Iterator<Item = Coordinate<T, Z>> + 'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        Box::new(self.0.iter().flat_map(|geometry| geometry.coords_iter()))
    }

    /// Return the number of coordinates in the `GeometryCollection`.
    fn coords_count(&self) -> usize {
        self.0.iter().map(|geometry| geometry.coords_count()).sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        Box::new(
            self.0
                .iter()
                .flat_map(|geometry| geometry.exterior_coords_iter()),
        )
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

type RectIter<T, Z> = iter::Chain<
    iter::Chain<CoordinateChainOnce<T, Z>, iter::Once<Coordinate<T, Z>>>,
    iter::Once<Coordinate<T, Z>>,
>;

impl<T: CoordNum, Z: CoordNum> CoordsIter for Rect<T, Z> {
    type Iter<'a>
        = RectIter<T, Z>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        todo!()
    }

    fn coords_count(&self) -> usize {
        8
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for Triangle<T, Z> {
    type Iter<'a>
        = iter::Chain<CoordinateChainOnce<T, Z>, iter::Once<Coordinate<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.0)
            .chain(iter::once(self.1))
            .chain(iter::once(self.2))
    }

    /// Return the number of coordinates in the `Triangle`.
    fn coords_count(&self) -> usize {
        3
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Geometry │
// └─────────────────────────────┘

impl<T: CoordNum, Z: CoordNum> CoordsIter for Geometry<T, Z> {
    type Iter<'a>
        = GeometryCoordsIter<'a, T, Z>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = GeometryExteriorCoordsIter<'a, T, Z>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        match self {
            Geometry::Point(g) => GeometryCoordsIter::Point(g.coords_iter()),
            Geometry::Line(g) => GeometryCoordsIter::Line(g.coords_iter()),
            Geometry::LineString(g) => GeometryCoordsIter::LineString(g.coords_iter()),
            Geometry::Polygon(g) => GeometryCoordsIter::Polygon(g.coords_iter()),
            Geometry::MultiPoint(g) => GeometryCoordsIter::MultiPoint(g.coords_iter()),
            Geometry::MultiLineString(g) => GeometryCoordsIter::MultiLineString(g.coords_iter()),
            Geometry::MultiPolygon(g) => GeometryCoordsIter::MultiPolygon(g.coords_iter()),
            Geometry::Rect(g) => GeometryCoordsIter::Rect(g.coords_iter()),
            Geometry::Triangle(g) => GeometryCoordsIter::Triangle(g.coords_iter()),
            _ => unimplemented!(),
        }
    }
    fn coords_count(&self) -> usize {
        match self {
            Geometry::Point(g) => g.coords_count(),
            Geometry::Line(g) => g.coords_count(),
            Geometry::LineString(g) => g.coords_count(),
            Geometry::Polygon(g) => g.coords_count(),
            Geometry::MultiPoint(g) => g.coords_count(),
            Geometry::MultiLineString(g) => g.coords_count(),
            Geometry::MultiPolygon(g) => g.coords_count(),
            Geometry::Rect(g) => g.coords_count(),
            Geometry::Triangle(g) => g.coords_count(),
            _ => unimplemented!(),
        }
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        match self {
            Geometry::Point(g) => GeometryExteriorCoordsIter::Point(g.exterior_coords_iter()),
            Geometry::Line(g) => GeometryExteriorCoordsIter::Line(g.exterior_coords_iter()),
            Geometry::LineString(g) => {
                GeometryExteriorCoordsIter::LineString(g.exterior_coords_iter())
            }
            Geometry::Polygon(g) => GeometryExteriorCoordsIter::Polygon(g.exterior_coords_iter()),
            Geometry::MultiPoint(g) => {
                GeometryExteriorCoordsIter::MultiPoint(g.exterior_coords_iter())
            }
            Geometry::MultiLineString(g) => {
                GeometryExteriorCoordsIter::MultiLineString(g.exterior_coords_iter())
            }
            Geometry::MultiPolygon(g) => {
                GeometryExteriorCoordsIter::MultiPolygon(g.exterior_coords_iter())
            }
            Geometry::Rect(g) => GeometryExteriorCoordsIter::Rect(g.exterior_coords_iter()),
            Geometry::Triangle(g) => GeometryExteriorCoordsIter::Triangle(g.exterior_coords_iter()),
            _ => unimplemented!(),
        }
    }
}

// ┌──────────────────────────┐
// │ Implementation for Array │
// └──────────────────────────┘

impl<const N: usize, T: CoordNum, Z: CoordNum> CoordsIter for [Coordinate<T, Z>; N] {
    type Iter<'a>
        = iter::Copied<slice::Iter<'a, Coordinate<T, Z>>>
    where
        T: 'a,
        Z: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a,
        Z: 'a;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }

    fn coords_count(&self) -> usize {
        N
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌──────────────────────────┐
// │ Implementation for Slice │
// └──────────────────────────┘

impl<'a, T: CoordNum, Z: CoordNum> CoordsIter for &'a [Coordinate<T, Z>] {
    type Iter<'b>
        = iter::Copied<slice::Iter<'b, Coordinate<T, Z>>>
    where
        T: 'b,
        'a: 'b;
    type ExteriorIter<'b>
        = Self::Iter<'b>
    where
        T: 'b,
        'a: 'b;
    type ScalarXY = T;
    type ScalarZ = Z;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }

    fn coords_count(&self) -> usize {
        self.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌───────────┐
// │ Utilities │
// └───────────┘

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter> Iterator
    for MapCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::Iter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.coords_iter())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapExteriorCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter> Iterator
    for MapExteriorCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::ExteriorIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.exterior_coords_iter())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Geometry into Iterator<Coord>
#[doc(hidden)]
pub enum GeometryCoordsIter<'a, T: CoordNum + 'a, Z: CoordNum + 'a> {
    Point(<Point<T, Z> as CoordsIter>::Iter<'a>),
    Line(<Line<T, Z> as CoordsIter>::Iter<'a>),
    LineString(<LineString<T, Z> as CoordsIter>::Iter<'a>),
    Polygon(<Polygon<T, Z> as CoordsIter>::Iter<'a>),
    MultiPoint(<MultiPoint<T, Z> as CoordsIter>::Iter<'a>),
    MultiLineString(<MultiLineString<T, Z> as CoordsIter>::Iter<'a>),
    MultiPolygon(<MultiPolygon<T, Z> as CoordsIter>::Iter<'a>),
    GeometryCollection(<GeometryCollection<T, Z> as CoordsIter>::Iter<'a>),
    Rect(<Rect<T, Z> as CoordsIter>::Iter<'a>),
    Triangle(<Triangle<T, Z> as CoordsIter>::Iter<'a>),
}

impl<T: CoordNum, Z: CoordNum> Iterator for GeometryCoordsIter<'_, T, Z> {
    type Item = Coordinate<T, Z>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryCoordsIter::Point(g) => g.next(),
            GeometryCoordsIter::Line(g) => g.next(),
            GeometryCoordsIter::LineString(g) => g.next(),
            GeometryCoordsIter::Polygon(g) => g.next(),
            GeometryCoordsIter::MultiPoint(g) => g.next(),
            GeometryCoordsIter::MultiLineString(g) => g.next(),
            GeometryCoordsIter::MultiPolygon(g) => g.next(),
            GeometryCoordsIter::GeometryCollection(g) => g.next(),
            GeometryCoordsIter::Rect(g) => g.next(),
            GeometryCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryCoordsIter::Point(g) => g.size_hint(),
            GeometryCoordsIter::Line(g) => g.size_hint(),
            GeometryCoordsIter::LineString(g) => g.size_hint(),
            GeometryCoordsIter::Polygon(g) => g.size_hint(),
            GeometryCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryCoordsIter::GeometryCollection(g) => g.size_hint(),
            GeometryCoordsIter::Rect(g) => g.size_hint(),
            GeometryCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}

impl<T: CoordNum + Debug, Z: CoordNum + Debug> fmt::Debug for GeometryCoordsIter<'_, T, Z> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryCoordsIter::Point(i) => fmt.debug_tuple("Point").field(i).finish(),
            GeometryCoordsIter::Line(i) => fmt.debug_tuple("Line").field(i).finish(),
            GeometryCoordsIter::LineString(i) => fmt.debug_tuple("LineString").field(i).finish(),
            GeometryCoordsIter::Polygon(i) => fmt.debug_tuple("Polygon").field(i).finish(),
            GeometryCoordsIter::MultiPoint(i) => fmt.debug_tuple("MultiPoint").field(i).finish(),
            GeometryCoordsIter::MultiLineString(i) => {
                fmt.debug_tuple("MultiLineString").field(i).finish()
            }
            GeometryCoordsIter::MultiPolygon(i) => {
                fmt.debug_tuple("MultiPolygon").field(i).finish()
            }
            GeometryCoordsIter::GeometryCollection(_) => fmt
                .debug_tuple("GeometryCollection")
                .field(&String::from("..."))
                .finish(),
            GeometryCoordsIter::Rect(i) => fmt.debug_tuple("Rect").field(i).finish(),
            GeometryCoordsIter::Triangle(i) => fmt.debug_tuple("Triangle").field(i).finish(),
        }
    }
}

// Utility to transform Geometry into Iterator<Coord>
#[doc(hidden)]
pub enum GeometryExteriorCoordsIter<'a, T: CoordNum + 'a, Z: CoordNum + 'a> {
    Point(<Point<T, Z> as CoordsIter>::ExteriorIter<'a>),
    Line(<Line<T, Z> as CoordsIter>::ExteriorIter<'a>),
    LineString(<LineString<T, Z> as CoordsIter>::ExteriorIter<'a>),
    Polygon(<Polygon<T, Z> as CoordsIter>::ExteriorIter<'a>),
    MultiPoint(<MultiPoint<T, Z> as CoordsIter>::ExteriorIter<'a>),
    MultiLineString(<MultiLineString<T, Z> as CoordsIter>::ExteriorIter<'a>),
    MultiPolygon(<MultiPolygon<T, Z> as CoordsIter>::ExteriorIter<'a>),
    GeometryCollection(<GeometryCollection<T, Z> as CoordsIter>::ExteriorIter<'a>),
    Rect(<Rect<T, Z> as CoordsIter>::ExteriorIter<'a>),
    Triangle(<Triangle<T, Z> as CoordsIter>::ExteriorIter<'a>),
}

impl<T: CoordNum, Z: CoordNum> Iterator for GeometryExteriorCoordsIter<'_, T, Z> {
    type Item = Coordinate<T, Z>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryExteriorCoordsIter::Point(g) => g.next(),
            GeometryExteriorCoordsIter::Line(g) => g.next(),
            GeometryExteriorCoordsIter::LineString(g) => g.next(),
            GeometryExteriorCoordsIter::Polygon(g) => g.next(),
            GeometryExteriorCoordsIter::MultiPoint(g) => g.next(),
            GeometryExteriorCoordsIter::MultiLineString(g) => g.next(),
            GeometryExteriorCoordsIter::MultiPolygon(g) => g.next(),
            GeometryExteriorCoordsIter::GeometryCollection(g) => g.next(),
            GeometryExteriorCoordsIter::Rect(g) => g.next(),
            GeometryExteriorCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryExteriorCoordsIter::Point(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Line(g) => g.size_hint(),
            GeometryExteriorCoordsIter::LineString(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Polygon(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryExteriorCoordsIter::GeometryCollection(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Rect(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}

impl<T: CoordNum + Debug, Z: CoordNum + Debug> fmt::Debug for GeometryExteriorCoordsIter<'_, T, Z> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryExteriorCoordsIter::Point(i) => fmt.debug_tuple("Point").field(i).finish(),
            GeometryExteriorCoordsIter::Line(i) => fmt.debug_tuple("Line").field(i).finish(),
            GeometryExteriorCoordsIter::LineString(i) => {
                fmt.debug_tuple("LineString").field(i).finish()
            }
            GeometryExteriorCoordsIter::Polygon(i) => fmt.debug_tuple("Polygon").field(i).finish(),
            GeometryExteriorCoordsIter::MultiPoint(i) => {
                fmt.debug_tuple("MultiPoint").field(i).finish()
            }
            GeometryExteriorCoordsIter::MultiLineString(i) => {
                fmt.debug_tuple("MultiLineString").field(i).finish()
            }
            GeometryExteriorCoordsIter::MultiPolygon(i) => {
                fmt.debug_tuple("MultiPolygon").field(i).finish()
            }
            GeometryExteriorCoordsIter::GeometryCollection(_) => fmt
                .debug_tuple("GeometryCollection")
                .field(&String::from("..."))
                .finish(),
            GeometryExteriorCoordsIter::Rect(i) => fmt.debug_tuple("Rect").field(i).finish(),
            GeometryExteriorCoordsIter::Triangle(i) => {
                fmt.debug_tuple("Triangle").field(i).finish()
            }
        }
    }
}
