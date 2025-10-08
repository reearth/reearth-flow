use crate::{
    types::{
        coordinate::Coordinate, coordnum::CoordNum, geometry::Geometry,
        geometry_collection::GeometryCollection, line::Line, line_string::LineString,
        multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
        point::Point, polygon::Polygon, rect::Rect, solid::Solid, triangle::Triangle,
    },
    utils::{get_bounding_rect, line_string_bounding_rect},
};

use super::{
    geometry_cow::GeometryCow,
    utils::{partial_max, partial_min},
};

/// Calculation of the bounding rectangle of a geometry.
pub trait BoundingRect<T: CoordNum, Z: CoordNum> {
    type Output: Into<Option<Rect<T, Z>>>;

    fn bounding_rect(&self) -> Self::Output;
}

impl<T, Z> BoundingRect<T, Z> for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Rect<T, Z>;

    fn bounding_rect(&self) -> Self::Output {
        Rect::new(*self, *self)
    }
}

impl<T, Z> BoundingRect<T, Z> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Rect<T, Z>;

    /// Return the bounding rectangle for a `Point`. It will have zero width
    /// and zero height.
    fn bounding_rect(&self) -> Self::Output {
        Rect::new(self.0, self.0)
    }
}

impl<T, Z> BoundingRect<T, Z> for MultiPoint<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    ///
    /// Return the BoundingRect for a MultiPoint
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.0.iter().map(|p| p.0))
    }
}

impl<T, Z> BoundingRect<T, Z> for Line<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Rect<T, Z>;

    fn bounding_rect(&self) -> Self::Output {
        Rect::new(self.start, self.end)
    }
}

impl<T, Z> BoundingRect<T, Z> for LineString<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    ///
    /// Return the BoundingRect for a LineString
    fn bounding_rect(&self) -> Self::Output {
        line_string_bounding_rect(self)
    }
}

impl<T, Z> BoundingRect<T, Z> for MultiLineString<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    ///
    /// Return the BoundingRect for a MultiLineString
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.iter().flat_map(|line| line.0.iter().cloned()))
    }
}

impl<T, Z> BoundingRect<T, Z> for Polygon<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    ///
    /// Return the BoundingRect for a Polygon
    fn bounding_rect(&self) -> Self::Output {
        let line = self.exterior();
        get_bounding_rect(line.0.iter().cloned())
    }
}

impl<T, Z> BoundingRect<T, Z> for Solid<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    fn bounding_rect(&self) -> Self::Output {
        let coords = self.get_all_vertex_coordinates();
        get_bounding_rect(coords)
    }
}

impl<T, Z> BoundingRect<T, Z> for MultiPolygon<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    ///
    /// Return the BoundingRect for a MultiPolygon
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(
            self.iter()
                .flat_map(|poly| poly.exterior().0.iter().cloned()),
        )
    }
}

impl<T, Z> BoundingRect<T, Z> for Triangle<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Rect<T, Z>;

    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.to_array().iter().cloned()).unwrap()
    }
}

impl<T, Z> BoundingRect<T, Z> for Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Rect<T, Z>;

    fn bounding_rect(&self) -> Self::Output {
        *self
    }
}

impl<T, Z> BoundingRect<T, Z> for Vec<Geometry<T, Z>>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    fn bounding_rect(&self) -> Self::Output {
        self.iter().fold(None, |acc, next| {
            let next_bounding_rect = next.bounding_rect();

            match (acc, next_bounding_rect) {
                (None, None) => None,
                (Some(r), None) | (None, Some(r)) => Some(r),
                (Some(r1), Some(r2)) => Some(bounding_rect_merge(r1, r2)),
            }
        })
    }
}

impl<T, Z> BoundingRect<T, Z> for GeometryCow<'_, T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    crate::geometry_cow_delegate_impl! {
       fn bounding_rect(&self) -> Self::Output;
    }
}

impl<T, Z> BoundingRect<T, Z> for Geometry<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    crate::geometry_delegate_impl! {
       fn bounding_rect(&self) -> Self::Output;
    }
}

impl<T, Z> BoundingRect<T, Z> for GeometryCollection<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Output = Option<Rect<T, Z>>;

    fn bounding_rect(&self) -> Self::Output {
        self.iter().fold(None, |acc, next| {
            let next_bounding_rect = next.bounding_rect();

            match (acc, next_bounding_rect) {
                (None, None) => None,
                (Some(r), None) | (None, Some(r)) => Some(r),
                (Some(r1), Some(r2)) => Some(bounding_rect_merge(r1, r2)),
            }
        })
    }
}

// Return a new rectangle that encompasses the provided rectangles
fn bounding_rect_merge<T: CoordNum, Z: CoordNum>(a: Rect<T, Z>, b: Rect<T, Z>) -> Rect<T, Z> {
    Rect::new(
        Coordinate::new__(
            partial_min(a.min().x, b.min().x),
            partial_min(a.min().y, b.min().y),
            partial_min(a.min().z, b.min().z),
        ),
        Coordinate::new__(
            partial_max(a.max().x, b.max().x),
            partial_max(a.max().y, b.max().y),
            partial_max(a.max().z, b.max().z),
        ),
    )
}

#[cfg(test)]
mod test {
    use super::bounding_rect_merge;
    use super::*;
    use crate::{coord, line_string, point, polygon, types::line_string::LineString};

    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32, f32> = line_string![];
        let bounding_rect = linestring.bounding_rect();
        assert!(bounding_rect.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let linestring = line_string![(x: 40.02f64, y: 116.34)];
        let bounding_rect = Rect::new(
            coord! {
                x: 40.02f64,
                y: 116.34,
            },
            coord! {
                x: 40.02,
                y: 116.34,
            },
        );
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 2., y: -2.),
            (x: -3., y: -3.),
            (x: -4., y: 4.)
        ];
        let bounding_rect = Rect::new(coord! { x: -4., y: -3. }, coord! { x: 2., y: 4. });
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let multiline = MultiLineString::new(vec![
            line_string![(x: 1., y: 1.), (x: -40., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 50., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 1., y: -60.)],
            line_string![(x: 1., y: 1.), (x: 1., y: 70.)],
        ]);
        let bounding_rect = Rect::new(coord! { x: -40., y: -60. }, coord! { x: 50., y: 70. });
        assert_eq!(bounding_rect, multiline.bounding_rect().unwrap());
    }
    #[test]
    fn multipoint_test() {
        let multipoint = MultiPoint::from(vec![(1., 1.), (2., -2.), (-3., -3.), (-4., 4.)]);
        let bounding_rect = Rect::new(coord! { x: -4., y: -3. }, coord! { x: 2., y: 4. });
        assert_eq!(bounding_rect, multipoint.bounding_rect().unwrap());
    }
    #[test]
    fn polygon_test() {
        let linestring = line_string![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.),
        ];
        let line_bounding_rect = linestring.bounding_rect().unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(line_bounding_rect, poly.bounding_rect().unwrap());
    }
    #[test]
    fn multipolygon_test() {
        let mpoly = MultiPolygon::new(vec![
            polygon![(x: 0., y: 0.), (x: 50., y: 0.), (x: 0., y: -70.), (x: 0., y: 0.)],
            polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 0., y: 80.), (x: 0., y: 0.)],
            polygon![(x: 0., y: 0.), (x: -60., y: 0.), (x: 0., y: 6.), (x: 0., y: 0.)],
        ]);
        let bounding_rect = Rect::new(coord! { x: -60., y: -70. }, coord! { x: 50., y: 80. });
        assert_eq!(bounding_rect, mpoly.bounding_rect().unwrap());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(coord! { x: 0., y: 1. }, coord! { x: 2., y: 3. });
        let line2 = Line::new(coord! { x: 2., y: 3. }, coord! { x: 0., y: 1. });
        assert_eq!(
            line1.bounding_rect(),
            Rect::new(coord! { x: 0., y: 1. }, coord! { x: 2., y: 3. },)
        );
        assert_eq!(
            line2.bounding_rect(),
            Rect::new(coord! { x: 0., y: 1. }, coord! { x: 2., y: 3. },)
        );
    }

    #[test]
    fn bounding_rect_merge_test() {
        assert_eq!(
            bounding_rect_merge(
                Rect::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }),
                Rect::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. }),
            ),
            Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 2. }),
        );
    }

    #[test]
    fn point_bounding_rect_test() {
        assert_eq!(
            Rect::new(coord! { x: 1., y: 2. }, coord! { x: 1., y: 2. }),
            point! { x: 1., y: 2. }.bounding_rect(),
        );
    }

    #[test]
    fn geometry_collection_bounding_rect_test() {
        assert_eq!(
            Some(Rect::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 2. })),
            GeometryCollection::from(vec![
                Geometry::Point(point! { x: 0., y: 0. }),
                Geometry::Point(point! { x: 1., y: 2. }),
            ])
            .bounding_rect(),
        );
    }
}
