use crate::types::{
    coordinate::Coordinate2D, coordnum::CoordFloat, line_string::LineString2D,
    multi_line_string::MultiLineString2D, multi_polygon::MultiPolygon2D, point::Point2D,
    polygon::Polygon2D,
};

pub(crate) fn create_point_type<T>(point: &crate::types::point::Point2D<T>) -> geojson::PointType
where
    T: CoordFloat,
{
    let x: f64 = point.x().to_f64().expect("Failed to convert x to f64");
    let y: f64 = point.y().to_f64().expect("Failed to convert y to f64");

    vec![x, y]
}

pub(crate) fn create_line_string_type<T>(
    line_string: &crate::types::line_string::LineString2D<T>,
) -> geojson::LineStringType
where
    T: CoordFloat,
{
    line_string
        .points()
        .map(|point| create_point_type(&point))
        .collect()
}

pub(crate) fn create_from_line_type<T>(
    line_string: &crate::types::line::Line2D<T>,
) -> geojson::LineStringType
where
    T: CoordFloat,
{
    vec![
        create_point_type(&line_string.start_point()),
        create_point_type(&line_string.end_point()),
    ]
}

pub(crate) fn create_from_triangle_type<T>(
    triangle: &crate::types::triangle::Triangle2D<T>,
) -> geojson::PolygonType
where
    T: CoordFloat,
{
    create_polygon_type(&triangle.to_polygon())
}

pub(crate) fn create_from_rect_type<T>(rect: &crate::types::rect::Rect2D<T>) -> geojson::PolygonType
where
    T: CoordFloat,
{
    create_polygon_type(&rect.to_polygon())
}

pub(crate) fn create_multi_line_string_type<T>(
    multi_line_string: &MultiLineString2D<T>,
) -> Vec<geojson::LineStringType>
where
    T: CoordFloat,
{
    multi_line_string
        .0
        .iter()
        .map(|line_string| create_line_string_type(line_string))
        .collect()
}

pub(crate) fn create_polygon_type<T>(polygon: &Polygon2D<T>) -> geojson::PolygonType
where
    T: CoordFloat,
{
    let mut coords = vec![polygon
        .exterior()
        .points()
        .map(|point| create_point_type(&point))
        .collect()];

    coords.extend(
        polygon
            .interiors()
            .iter()
            .map(|line_string| create_line_string_type(line_string)),
    );

    coords
}

pub(crate) fn create_multi_polygon_type<T>(
    multi_polygon: &MultiPolygon2D<T>,
) -> Vec<geojson::PolygonType>
where
    T: CoordFloat,
{
    multi_polygon
        .0
        .iter()
        .map(|polygon| create_polygon_type(polygon))
        .collect()
}

pub(crate) fn create_geo_coordinate<T>(point_type: &geojson::PointType) -> Coordinate2D<T>
where
    T: CoordFloat,
{
    Coordinate2D::new_(
        T::from(point_type[0]).expect("Failed to convert x to f64"),
        T::from(point_type[1]).expect("Failed to convert y to f64"),
    )
}

pub(crate) fn create_geo_point<T>(point_type: &geojson::PointType) -> Point2D<T>
where
    T: CoordFloat,
{
    Point2D::new(
        T::from(point_type[0]).expect("Failed to convert x to T"),
        T::from(point_type[1]).expect("Failed to convert y to T"),
    )
}

pub(crate) fn create_geo_line_string<T>(line_type: &geojson::LineStringType) -> LineString2D<T>
where
    T: CoordFloat,
{
    LineString2D::new(
        line_type
            .iter()
            .map(|point_type| create_geo_coordinate(point_type))
            .collect(),
    )
}

pub(crate) fn create_geo_multi_line_string<T>(
    multi_line_type: &[geojson::LineStringType],
) -> MultiLineString2D<T>
where
    T: CoordFloat,
{
    MultiLineString2D::new(
        multi_line_type
            .iter()
            .map(|point_type| create_geo_line_string(point_type))
            .collect(),
    )
}

pub(crate) fn create_geo_polygon<T>(polygon_type: &geojson::PolygonType) -> Polygon2D<T>
where
    T: CoordFloat,
{
    let exterior = polygon_type
        .first()
        .map(|e| create_geo_line_string(e))
        .unwrap_or_else(|| create_geo_line_string(&vec![]));

    let interiors = if polygon_type.len() < 2 {
        vec![]
    } else {
        polygon_type[1..]
            .iter()
            .map(|line_string_type| create_geo_line_string(line_string_type))
            .collect()
    };

    Polygon2D::new(exterior, interiors)
}

pub(crate) fn create_geo_multi_polygon<T>(
    multi_polygon_type: &[geojson::PolygonType],
) -> MultiPolygon2D<T>
where
    T: CoordFloat,
{
    MultiPolygon2D::new(
        multi_polygon_type
            .iter()
            .map(|polygon_type| create_geo_polygon(polygon_type))
            .collect(),
    )
}

pub(crate) fn mismatch_geom_err(
    expected_type: &'static str,
    found: &geojson::Value,
) -> crate::error::Error {
    crate::error::Error::InvalidGeoJsonConversion {
        expected_type,
        found_type: found.type_name(),
    }
}
