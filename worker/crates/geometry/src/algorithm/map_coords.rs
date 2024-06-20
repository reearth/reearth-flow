use crate::types::{
    coordinate::Coordinate, coordnum::CoordNum, geometry::Geometry,
    geometry_collection::GeometryCollection, line::Line, line_string::LineString,
    multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
    point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
};

/// Map a function over all the coordinates in an object, returning a new one
pub trait MapCoords<T, Z, NT, NZ> {
    type Output;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output
    where
        T: CoordNum,
        Z: CoordNum,
        NT: CoordNum,
        NZ: CoordNum;

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E>
    where
        T: CoordNum,
        Z: CoordNum,
        NT: CoordNum,
        NZ: CoordNum;
}

pub trait MapCoordsInPlace<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy)
    where
        T: CoordNum,
        Z: CoordNum;

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E>
    where
        T: CoordNum,
        Z: CoordNum;
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ> for Point<T, Z> {
    type Output = Point<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        Point(func(self.0))
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E>,
    ) -> Result<Self::Output, E> {
        Ok(Point(func(self.0)?))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for Point<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z>) {
        self.0 = func(self.0);
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        self.0 = func(self.0)?;
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ> for Line<T, Z> {
    type Output = Line<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        Line::new_(
            self.start_point().map_coords(func).0,
            self.end_point().map_coords(func).0,
        )
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(Line::new_(
            self.start_point().try_map_coords(func)?.0,
            self.end_point().try_map_coords(func)?.0,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for Line<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z>) {
        self.start = func(self.start);
        self.end = func(self.end);
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        self.start = func(self.start)?;
        self.end = func(self.end)?;

        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for LineString<T, Z>
{
    type Output = LineString<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        LineString::from(
            self.points()
                .map(|p| p.map_coords(func))
                .collect::<Vec<_>>(),
        )
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(LineString::from(
            self.points()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for LineString<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z>) {
        for p in &mut self.0 {
            *p = func(*p);
        }
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        for p in &mut self.0 {
            *p = func(*p)?;
        }
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for Polygon<T, Z>
{
    type Output = Polygon<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        Polygon::new(
            self.exterior().map_coords(func),
            self.interiors()
                .iter()
                .map(|l| l.map_coords(func))
                .collect(),
        )
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(Polygon::new(
            self.exterior().try_map_coords(func)?,
            self.interiors()
                .iter()
                .map(|l| l.try_map_coords(func))
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for Polygon<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy) {
        self.exterior_mut(|line_string| {
            line_string.map_coords_in_place(func);
        });

        self.interiors_mut(|line_strings| {
            for line_string in line_strings {
                line_string.map_coords_in_place(func);
            }
        });
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        let mut result = Ok(());

        self.exterior_mut(|line_string| {
            if let Err(e) = line_string.try_map_coords_in_place(&func) {
                result = Err(e);
            }
        });

        if result.is_ok() {
            self.interiors_mut(|line_strings| {
                for line_string in line_strings {
                    if let Err(e) = line_string.try_map_coords_in_place(&func) {
                        result = Err(e);
                        break;
                    }
                }
            });
        }

        result
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for MultiPoint<T, Z>
{
    type Output = MultiPoint<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        MultiPoint::new(self.iter().map(|p| p.map_coords(func)).collect())
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(MultiPoint::new(
            self.0
                .iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for MultiPoint<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy) {
        for p in &mut self.0 {
            p.map_coords_in_place(func);
        }
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        for p in &mut self.0 {
            p.try_map_coords_in_place(&func)?;
        }
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for MultiLineString<T, Z>
{
    type Output = MultiLineString<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        MultiLineString::new(self.iter().map(|l| l.map_coords(func)).collect())
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(MultiLineString::new(
            self.0
                .iter()
                .map(|l| l.try_map_coords(func))
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for MultiLineString<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy) {
        for p in &mut self.0 {
            p.map_coords_in_place(func);
        }
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        for p in &mut self.0 {
            p.try_map_coords_in_place(&func)?;
        }
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for MultiPolygon<T, Z>
{
    type Output = MultiPolygon<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        MultiPolygon::new(self.iter().map(|p| p.map_coords(func)).collect())
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(MultiPolygon::new(
            self.0
                .iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for MultiPolygon<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy) {
        for p in &mut self.0 {
            p.map_coords_in_place(func);
        }
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        for p in &mut self.0 {
            p.try_map_coords_in_place(&func)?;
        }
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for Geometry<T, Z>
{
    type Output = Geometry<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        match *self {
            Geometry::Point(ref x) => Geometry::Point(x.map_coords(func)),
            Geometry::Line(ref x) => Geometry::Line(x.map_coords(func)),
            Geometry::LineString(ref x) => Geometry::LineString(x.map_coords(func)),
            Geometry::Polygon(ref x) => Geometry::Polygon(x.map_coords(func)),
            Geometry::MultiPoint(ref x) => Geometry::MultiPoint(x.map_coords(func)),
            Geometry::MultiLineString(ref x) => Geometry::MultiLineString(x.map_coords(func)),
            Geometry::MultiPolygon(ref x) => Geometry::MultiPolygon(x.map_coords(func)),
            Geometry::GeometryCollection(ref x) => {
                let mut result = Vec::new();
                for g in x.iter() {
                    result.push(g.map_coords(func));
                }
                Geometry::GeometryCollection(result)
            }
            Geometry::Rect(ref x) => Geometry::Rect(x.map_coords(func)),
            Geometry::Triangle(ref x) => Geometry::Triangle(x.map_coords(func)),
            _ => unimplemented!(),
        }
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        match *self {
            Geometry::Point(ref x) => Ok(Geometry::Point(x.try_map_coords(func)?)),
            Geometry::Line(ref x) => Ok(Geometry::Line(x.try_map_coords(func)?)),
            Geometry::LineString(ref x) => Ok(Geometry::LineString(x.try_map_coords(func)?)),
            Geometry::Polygon(ref x) => Ok(Geometry::Polygon(x.try_map_coords(func)?)),
            Geometry::MultiPoint(ref x) => Ok(Geometry::MultiPoint(x.try_map_coords(func)?)),
            Geometry::MultiLineString(ref x) => {
                Ok(Geometry::MultiLineString(x.try_map_coords(func)?))
            }
            Geometry::MultiPolygon(ref x) => Ok(Geometry::MultiPolygon(x.try_map_coords(func)?)),
            Geometry::GeometryCollection(ref x) => {
                let mut result = Vec::new();
                for g in x.iter() {
                    result.push(g.try_map_coords(func)?);
                }
                Ok(Geometry::GeometryCollection(result))
            }
            Geometry::Rect(ref x) => Ok(Geometry::Rect(x.try_map_coords(func)?)),
            Geometry::Triangle(ref x) => Ok(Geometry::Triangle(x.try_map_coords(func)?)),
            _ => unimplemented!(),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for Geometry<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy) {
        match *self {
            Geometry::Point(ref mut x) => x.map_coords_in_place(func),
            Geometry::Line(ref mut x) => x.map_coords_in_place(func),
            Geometry::LineString(ref mut x) => x.map_coords_in_place(func),
            Geometry::Polygon(ref mut x) => x.map_coords_in_place(func),
            Geometry::MultiPoint(ref mut x) => x.map_coords_in_place(func),
            Geometry::MultiLineString(ref mut x) => x.map_coords_in_place(func),
            Geometry::MultiPolygon(ref mut x) => x.map_coords_in_place(func),
            Geometry::GeometryCollection(ref mut x) => {
                for g in x.iter_mut() {
                    g.map_coords_in_place(func);
                }
            }
            Geometry::Rect(ref mut x) => x.map_coords_in_place(func),
            Geometry::Triangle(ref mut x) => x.map_coords_in_place(func),
            _ => unimplemented!(),
        }
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        match *self {
            Geometry::Point(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::Line(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::LineString(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::Polygon(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::MultiPoint(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::MultiLineString(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::MultiPolygon(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::GeometryCollection(ref mut x) => {
                for g in x.iter_mut() {
                    g.try_map_coords_in_place(&func)?;
                }
                Ok(())
            }
            Geometry::Rect(ref mut x) => x.try_map_coords_in_place(func),
            Geometry::Triangle(ref mut x) => x.try_map_coords_in_place(func),
            _ => unimplemented!(),
        }
    }
}

//------------------------------------//
// GeometryCollection implementations //
//------------------------------------//

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for GeometryCollection<T, Z>
{
    type Output = GeometryCollection<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        GeometryCollection::from(self.iter().map(|g| g.map_coords(func)).collect::<Vec<_>>())
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E> + Copy,
    ) -> Result<Self::Output, E> {
        Ok(GeometryCollection::from(
            self.0
                .iter()
                .map(|g| g.try_map_coords(func))
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for GeometryCollection<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z> + Copy) {
        for p in &mut self.0 {
            p.map_coords_in_place(func);
        }
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        for p in &mut self.0 {
            p.try_map_coords_in_place(&func)?;
        }
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ> for Rect<T, Z> {
    type Output = Rect<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        Rect::new(func(self.min()), func(self.max()))
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E>,
    ) -> Result<Self::Output, E> {
        Ok(Rect::new(func(self.min())?, func(self.max())?))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for Rect<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z>) {
        let mut new_rect = Rect::new(func(self.min()), func(self.max()));
        ::std::mem::swap(self, &mut new_rect);
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        let mut new_rect = Rect::new(func(self.min())?, func(self.max())?);
        ::std::mem::swap(self, &mut new_rect);
        Ok(())
    }
}

impl<T: CoordNum, Z: CoordNum, NT: CoordNum, NZ: CoordNum> MapCoords<T, Z, NT, NZ>
    for Triangle<T, Z>
{
    type Output = Triangle<NT, NZ>;

    fn map_coords(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Coordinate<NT, NZ> + Copy,
    ) -> Self::Output {
        Triangle::new(func(self.0), func(self.1), func(self.2))
    }

    fn try_map_coords<E>(
        &self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<NT, NZ>, E>,
    ) -> Result<Self::Output, E> {
        Ok(Triangle::new(func(self.0)?, func(self.1)?, func(self.2)?))
    }
}

impl<T: CoordNum, Z: CoordNum> MapCoordsInPlace<T, Z> for Triangle<T, Z> {
    fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T, Z>) -> Coordinate<T, Z>) {
        let mut new_triangle = Triangle::new(func(self.0), func(self.1), func(self.2));

        ::std::mem::swap(self, &mut new_triangle);
    }

    fn try_map_coords_in_place<E>(
        &mut self,
        func: impl Fn(Coordinate<T, Z>) -> Result<Coordinate<T, Z>, E>,
    ) -> Result<(), E> {
        let mut new_triangle = Triangle::new(func(self.0)?, func(self.1)?, func(self.2)?);

        ::std::mem::swap(self, &mut new_triangle);

        Ok(())
    }
}
