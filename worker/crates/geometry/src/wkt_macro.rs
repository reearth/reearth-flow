#[macro_export]
macro_rules! wkt {
    // Hide distracting implementation details from the generated rustdoc.
    ($($wkt:tt)+) => {
        {
            $crate::wkt_internal!($($wkt)+)
        }
    };
}

#[macro_export]
macro_rules! wkt_internal {
    (POINT EMPTY) => {
        compile_error!("EMPTY points are not supported in geo-types")
    };
    (POINT($x: literal $y: literal)) => {
        $crate::point!(x: $x, y: $y)
    };
    (POINT($x: literal $y: literal $z: literal)) => {
        $crate::point!(x: $x, y: $y, z: $z)
    };
    (POINT $($tail: tt)*) => {
        compile_error!("Invalid POINT wkt");
    };
    (LINESTRING EMPTY) => {
        $crate::line_string![]
    };
    (LINESTRING ($($x: literal $y: literal),+)) => {
        $crate::line_string![
            $($crate::coord!(x: $x, y: $y)),*
        ]
    };
    (LINESTRING ($($x: literal $y: literal $z: literal),+)) => {
        $crate::line_string![
            $($crate::coord!(x: $x, y: $y, z: $z)),*
        ]
    };
    (LINESTRING ()) => {
        compile_error!("use `EMPTY` instead of () for an empty collection")
    };
    (LINESTRING $($tail: tt)*) => {
        compile_error!("Invalid LINESTRING wkt");
    };
    (POLYGON EMPTY) => {
        $crate::polygon![]
    };
    (POLYGON ( $exterior_tt: tt )) => {
        $crate::types::polygon::Polygon::new($crate::wkt!(LINESTRING $exterior_tt), $crate::_alloc::vec![])
    };
    (POLYGON( $exterior_tt: tt, $($interiors_tt: tt),+ )) => {
        $crate::types::polygon::Polygon::new(
            $crate::wkt!(LINESTRING $exterior_tt),
            $crate::_alloc::vec![
               $($crate::wkt!(LINESTRING $interiors_tt)),*
            ]
        )
    };
    (POLYGON ()) => {
        compile_error!("use `EMPTY` instead of () for an empty collection")
    };
    (POLYGON $($tail: tt)*) => {
        compile_error!("Invalid POLYGON wkt");
    };
    (MULTIPOINT EMPTY) => {
        $crate::types::multi_point:: MultiPoint($crate::_alloc::vec![])
    };
    (MULTIPOINT ()) => {
        compile_error!("use `EMPTY` instead of () for an empty collection")
    };
    (MULTIPOINT ($($x: literal $y: literal),* )) => {
        $crate::types::multi_point::MultiPoint(
            $crate::_alloc::vec![$($crate::point!(x: $x, y: $y)),*]
        )
    };
    (MULTIPOINT ($($x: literal $y: literal $z: literal),* )) => {
        $crate::types::multi_point::MultiPoint(
            $crate::_alloc::vec![$($crate::point!(x: $x, y: $y, z: $z)),*]
        )
    };
    (MULTIPOINT $($tail: tt)*) => {
        compile_error!("Invalid MULTIPOINT wkt");
    };
    (MULTILINESTRING EMPTY) => {
        $crate::types::multi_line_string::MultiLineString($crate::_alloc::vec![])
    };
    (MULTILINESTRING ()) => {
        compile_error!("use `EMPTY` instead of () for an empty collection")
    };
    (MULTILINESTRING ( $($line_string_tt: tt),* )) => {
        $crate::types::multi_line_string::MultiLineString($crate::_alloc::vec![
           $($crate::wkt!(LINESTRING $line_string_tt)),*
        ])
    };
    (MULTILINESTRING $($tail: tt)*) => {
        compile_error!("Invalid MULTILINESTRING wkt");
    };
    (MULTIPOLYGON EMPTY) => {
        $crate::types::multi_polygon::MultiPolygon($crate::_alloc::vec![])
    };
    (MULTIPOLYGON ()) => {
        compile_error!("use `EMPTY` instead of () for an empty collection")
    };
    (MULTIPOLYGON ( $($polygon_tt: tt),* )) => {
        $crate::types::multi_polygon::MultiPolygon($crate::_alloc::vec![
           $($crate::wkt!(POLYGON $polygon_tt)),*
        ])
    };
    (MULTIPOLYGON $($tail: tt)*) => {
        compile_error!("Invalid MULTIPOLYGON wkt");
    };
    (GEOMETRYCOLLECTION EMPTY) => {
        $crate::types::geometry_collection::GeometryCollection($crate::_alloc::vec![])
    };
    (GEOMETRYCOLLECTION ()) => {
        compile_error!("use `EMPTY` instead of () for an empty collection")
    };
    (GEOMETRYCOLLECTION ( $($el_type:tt $el_tt: tt),* )) => {
        $crate::types::geometry_collection::GeometryCollection($crate::_alloc::vec![
           $($crate::types::geometry::Geometry::from($crate::wkt!($el_type $el_tt))),*
        ])
    };
    (GEOMETRYCOLLECTION $($tail: tt)*) => {
        compile_error!("Invalid GEOMETRYCOLLECTION wkt");
    };
    ($name: ident ($($tail: tt)*)) => {
        compile_error!("Unknown type. Must be one of POINT, LINESTRING, POLYGON, MULTIPOINT, MULTILINESTRING, MULTIPOLYGON, or GEOMETRYCOLLECTION");
    };
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use crate::types::{
        geometry::Geometry, geometry_collection::GeometryCollection, line_string::LineString,
        multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
        point::Point, polygon::Polygon,
    };

    #[test]
    fn point() {
        let point = wkt! { POINT(1.0 2.0) };
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);

        let point = wkt! { POINT(1.0 2.0 3.0) };
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
        assert_eq!(point.z(), 3.0);
    }

    #[test]
    fn empty_line_string() {
        let line_string: LineString<f64> = wkt! { LINESTRING EMPTY };
        assert_eq!(line_string.0.len(), 0);
    }

    #[test]
    fn line_string() {
        let line_string = wkt! { LINESTRING(1.0 2.0,3.0 4.0) };
        assert_eq!(line_string.0.len(), 2);
        assert_eq!(line_string[0], coord! { x: 1.0, y: 2.0 });

        let line_string = wkt! { LINESTRING(1.0 2.0 3.0,3.0 4.0 5.0) };
        assert_eq!(line_string.0.len(), 2);
        assert_eq!(line_string[0], coord! { x: 1.0, y: 2.0, z: 3.0 });
    }

    #[test]
    fn empty_polygon() {
        let polygon: Polygon = wkt! { POLYGON EMPTY };
        assert_eq!(polygon.exterior().0.len(), 0);
        assert_eq!(polygon.interiors().len(), 0);
    }

    #[test]
    fn polygon() {
        let polygon = wkt! { POLYGON((1.0 2.0)) };
        assert_eq!(polygon.exterior().0.len(), 1);
        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });

        let polygon = wkt! { POLYGON((1.0 2.0 3.0)) };
        assert_eq!(polygon.exterior().0.len(), 1);
        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0, z: 3.0 });

        let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0)) };
        // Note: an extra coord is added to close the linestring
        assert_eq!(polygon.exterior().0.len(), 3);
        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
        assert_eq!(polygon.exterior().0[1], coord! { x: 3.0, y: 4.0 });
        assert_eq!(polygon.exterior().0[2], coord! { x: 1.0, y: 2.0 });

        let polygon = wkt! { POLYGON((1.0 2.0 3.0,3.0 4.0 5.0)) };
        // Note: an extra coord is added to close the linestring
        assert_eq!(polygon.exterior().0.len(), 3);
        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0, z: 3.0 });
        assert_eq!(polygon.exterior().0[1], coord! { x: 3.0, y: 4.0, z: 5.0 });
        assert_eq!(polygon.exterior().0[2], coord! { x: 1.0, y: 2.0, z: 3.0});

        let polygon = wkt! { POLYGON((1.0 2.0), (1.1 2.1)) };
        assert_eq!(polygon.exterior().0.len(), 1);
        assert_eq!(polygon.interiors().len(), 1);

        assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
        assert_eq!(polygon.interiors()[0].0[0], coord! { x: 1.1, y: 2.1 });

        let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)) };
        assert_eq!(polygon.exterior().0.len(), 3);
        assert_eq!(polygon.interiors().len(), 2);
        assert_eq!(polygon.interiors()[1][1], coord! { x: 3.2, y: 4.2 });
    }

    #[test]
    fn empty_multi_point() {
        let multipoint: MultiPoint = wkt! { MULTIPOINT EMPTY };
        assert!(multipoint.0.is_empty());
    }

    #[test]
    fn multi_point() {
        let multi_point = wkt! { MULTIPOINT(1.0 2.0) };
        assert_eq!(multi_point.0, vec![point! { x: 1.0, y: 2.0}]);

        let multi_point = wkt! { MULTIPOINT(1.0 2.0,3.0 4.0) };
        assert_eq!(
            multi_point.0,
            vec![point! { x: 1.0, y: 2.0}, point! { x: 3.0, y: 4.0}]
        );

        let multi_point = wkt! { MULTIPOINT(1.0 2.0 3.0,3.0 4.0 5.0) };
        assert_eq!(
            multi_point.0,
            vec![
                point! { x: 1.0, y: 2.0, z: 3.0},
                point! { x: 3.0, y: 4.0, z: 5.0}
            ]
        );
    }

    #[test]
    fn empty_multi_line_string() {
        let multi_line_string: MultiLineString = wkt! { MULTILINESTRING EMPTY };
        assert_eq!(multi_line_string.0, vec![]);
    }
    #[test]
    fn multi_line_string() {
        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0)) };
        assert_eq!(multi_line_string.0.len(), 1);
        assert_eq!(multi_line_string.0[0].0[1], coord! { x: 3.0, y: 4.0 });
        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0 3.0,3.0 4.0 5.0)) };
        assert_eq!(multi_line_string.0.len(), 1);
        assert_eq!(
            multi_line_string.0[0].0[1],
            coord! { x: 3.0, y: 4.0, z: 5.0 }
        );

        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),(5.0 6.0,7.0 8.0)) };
        assert_eq!(multi_line_string.0.len(), 2);
        assert_eq!(multi_line_string.0[1].0[1], coord! { x: 7.0, y: 8.0 });

        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),EMPTY) };
        assert_eq!(multi_line_string.0.len(), 2);
        assert_eq!(multi_line_string.0[1].0.len(), 0);
    }

    #[test]
    fn empty_multi_polygon() {
        let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON EMPTY };
        assert!(multi_polygon.0.is_empty());
    }

    #[test]
    fn multi_line_polygon() {
        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0))) };
        assert_eq!(multi_polygon.0.len(), 1);
        assert_eq!(multi_polygon.0[0].exterior().0[0], coord! { x: 1.0, y: 2.0});

        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)),((1.0 2.0))) };
        assert_eq!(multi_polygon.0.len(), 2);
        assert_eq!(
            multi_polygon.0[0].interiors()[1].0[0],
            coord! { x: 1.2, y: 2.2}
        );

        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)), EMPTY) };
        assert_eq!(multi_polygon.0.len(), 2);
        assert_eq!(
            multi_polygon.0[0].interiors()[1].0[0],
            coord! { x: 1.2, y: 2.2}
        );
        assert!(multi_polygon.0[1].exterior().0.is_empty());
    }

    #[test]
    fn empty_geometry_collection() {
        let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION EMPTY };
        assert!(geometry_collection.is_empty());
    }

    #[test]
    fn geometry_collection() {
        let geometry_collection = wkt! {
            GEOMETRYCOLLECTION (
                POINT (40.0 10.0),
                LINESTRING (10.0 10.0, 20.0 20.0, 10.0 40.0),
                POLYGON ((40.0 40.0, 20.0 45.0, 45.0 30.0, 40.0 40.0))
            )
        };
        assert_eq!(geometry_collection.len(), 3);

        let line_string = match &geometry_collection[1] {
            Geometry::LineString(line_string) => line_string,
            _ => panic!(
                "unexpected geometry: {geometry:?}",
                geometry = geometry_collection[1]
            ),
        };
        assert_eq!(line_string.0[1], coord! {x: 20.0, y: 20.0 });
    }

    #[test]
    fn other_numeric_types() {
        let point: Point<i32> = wkt!(POINT(1 2));
        assert_eq!(point.x(), 1i32);
        assert_eq!(point.y(), 2i32);

        let point: Point<u64> = wkt!(POINT(1 2));
        assert_eq!(point.x(), 1u64);
        assert_eq!(point.y(), 2u64);

        let point: Point<f32> = wkt!(POINT(1.0 2.0));
        assert_eq!(point.x(), 1.0f32);
        assert_eq!(point.y(), 2.0f32);
    }
}
