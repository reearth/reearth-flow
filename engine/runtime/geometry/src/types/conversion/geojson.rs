use crate::types::{
    coordinate::Coordinate, coordnum::CoordFloat, line_string::LineString,
    multi_line_string::MultiLineString, multi_polygon::MultiPolygon, point::Point,
    polygon::Polygon,
};

pub(crate) fn create_point_type<T, Z>(
    point: &crate::types::point::Point<T, Z>,
) -> geojson::PointType
where
    T: CoordFloat,
    Z: CoordFloat,
{
    let x: f64 = point.x().to_f64().expect("Failed to convert x to f64");
    let y: f64 = point.y().to_f64().expect("Failed to convert y to f64");

    if point.z().is_nan() {
        vec![x, y]
    } else {
        let z: f64 = point.z().to_f64().expect("Failed to convert z to f64");
        vec![x, y, z]
    }
}

pub(crate) fn create_line_string_type<T, Z>(
    line_string: &crate::types::line_string::LineString<T, Z>,
) -> geojson::LineStringType
where
    T: CoordFloat,
    Z: CoordFloat,
{
    line_string
        .points()
        .map(|point| create_point_type(&point))
        .collect()
}

pub(crate) fn create_from_line_type<T, Z>(
    line_string: &crate::types::line::Line<T, Z>,
) -> geojson::LineStringType
where
    T: CoordFloat,
    Z: CoordFloat,
{
    vec![
        create_point_type(&line_string.start_point()),
        create_point_type(&line_string.end_point()),
    ]
}

pub(crate) fn create_from_triangle_type<T, Z>(
    triangle: &crate::types::triangle::Triangle<T, Z>,
) -> geojson::PolygonType
where
    T: CoordFloat,
    Z: CoordFloat,
{
    create_polygon_type(&triangle.to_polygon())
}

pub(crate) fn create_from_rect_type<T, Z>(
    rect: &crate::types::rect::Rect<T, Z>,
) -> geojson::PolygonType
where
    T: CoordFloat,
    Z: CoordFloat,
{
    create_polygon_type(&rect.to_polygon())
}

pub(crate) fn create_multi_line_string_type<T, Z>(
    multi_line_string: &MultiLineString<T, Z>,
) -> Vec<geojson::LineStringType>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    multi_line_string
        .0
        .iter()
        .map(|line_string| create_line_string_type(line_string))
        .collect()
}

pub(crate) fn create_polygon_type<T, Z>(polygon: &Polygon<T, Z>) -> geojson::PolygonType
where
    T: CoordFloat,
    Z: CoordFloat,
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

pub(crate) fn create_multi_polygon_type<T, Z>(
    multi_polygon: &MultiPolygon<T, Z>,
) -> Vec<geojson::PolygonType>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    multi_polygon
        .0
        .iter()
        .map(|polygon| create_polygon_type(polygon))
        .collect()
}

pub(crate) fn create_geo_coordinate<T, Z>(point_type: &geojson::PointType) -> Coordinate<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    Coordinate::new__(
        T::from(point_type[0]).expect("Failed to convert x to f64"),
        T::from(point_type[1]).expect("Failed to convert y to f64"),
        if point_type.len() > 2 {
            Z::from(point_type[2]).expect("Failed to convert z to f64")
        } else {
            Z::zero()
        },
    )
}

pub(crate) fn create_geo_point<T, Z>(point_type: &geojson::PointType) -> Point<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    Point::new(
        T::from(point_type[0]).expect("Failed to convert x to T"),
        T::from(point_type[1]).expect("Failed to convert y to T"),
        if point_type.len() > 2 {
            Z::from(point_type[2]).expect("Failed to convert z to T")
        } else {
            Z::zero()
        },
    )
}

pub(crate) fn create_geo_line_string<T, Z>(line_type: &geojson::LineStringType) -> LineString<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    LineString::new(
        line_type
            .iter()
            .map(|point_type| create_geo_coordinate(point_type))
            .collect(),
    )
}

pub(crate) fn create_geo_multi_line_string<T, Z>(
    multi_line_type: &[geojson::LineStringType],
) -> MultiLineString<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    MultiLineString::new(
        multi_line_type
            .iter()
            .map(|point_type| create_geo_line_string(point_type))
            .collect(),
    )
}

pub(crate) fn create_geo_polygon<T, Z>(polygon_type: &geojson::PolygonType) -> Polygon<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
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

    Polygon::new(exterior, interiors)
}

pub(crate) fn create_geo_multi_polygon<T, Z>(
    multi_polygon_type: &[geojson::PolygonType],
) -> MultiPolygon<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    MultiPolygon::new(
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
