use crate::types::{
    coordinate::{Coordinate2D, Coordinate3D},
    coordnum::CoordFloat,
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString, MultiLineString2D, MultiLineString3D},
    multi_polygon::{MultiPolygon, MultiPolygon2D, MultiPolygon3D},
    point::{Point2D, Point3D},
    polygon::{Polygon, Polygon2D, Polygon3D},
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
        vec![y, x]
    } else {
        let z: f64 = point.z().to_f64().expect("Failed to convert z to f64");
        vec![y, x, z]
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

pub(crate) fn create_geo_coordinate_2d(point_type: &geojson::PointType) -> Coordinate2D<f64> {
    Coordinate2D::new_(point_type[0], point_type[1])
}

pub(crate) fn create_geo_coordinate_3d(point_type: &geojson::PointType) -> Coordinate3D<f64> {
    Coordinate3D::new__(point_type[0], point_type[1], point_type[2])
}

pub fn is_2d(point_type: &geojson::PointType) -> bool {
    point_type.len() == 2
}

pub fn is_3d(point_type: &geojson::PointType) -> bool {
    point_type.len() == 3
}

pub fn is_2d_geojson_value(value: &geojson::Value) -> bool {
    match value {
        geojson::Value::Point(point_type) => is_2d(point_type),
        geojson::Value::LineString(line_type) => line_type.iter().all(is_2d),
        geojson::Value::Polygon(polygon_type) => {
            polygon_type.iter().all(|line| line.iter().all(is_2d))
        }
        geojson::Value::MultiPoint(multi_point_type) => multi_point_type.iter().all(is_2d),
        geojson::Value::MultiLineString(multi_line_type) => {
            multi_line_type.iter().all(|line| line.iter().all(is_2d))
        }
        geojson::Value::MultiPolygon(multi_polygon_type) => multi_polygon_type
            .iter()
            .all(|polygon| polygon.iter().all(|line| line.iter().all(is_2d))),
        _ => false,
    }
}

pub fn is_3d_geojson_value(value: &geojson::Value) -> bool {
    match value {
        geojson::Value::Point(point_type) => is_3d(point_type),
        geojson::Value::LineString(line_type) => line_type.iter().all(is_3d),
        geojson::Value::Polygon(polygon_type) => {
            polygon_type.iter().all(|line| line.iter().all(is_3d))
        }
        geojson::Value::MultiPoint(multi_point_type) => multi_point_type.iter().all(is_3d),
        geojson::Value::MultiLineString(multi_line_type) => {
            multi_line_type.iter().all(|line| line.iter().all(is_3d))
        }
        geojson::Value::MultiPolygon(multi_polygon_type) => multi_polygon_type
            .iter()
            .all(|polygon| polygon.iter().all(|line| line.iter().all(is_3d))),
        _ => false,
    }
}

pub(crate) fn create_geo_point_2d(point_type: &geojson::PointType) -> Point2D<f64> {
    Point2D::from((point_type[1], point_type[0]))
}

pub(crate) fn create_geo_point_3d(point_type: &geojson::PointType) -> Point3D<f64> {
    Point3D::new(point_type[1], point_type[0], point_type[2])
}

pub(crate) fn create_geo_line_string_2d(line_type: &geojson::LineStringType) -> LineString2D<f64> {
    LineString2D::new(line_type.iter().map(create_geo_coordinate_2d).collect())
}

pub(crate) fn create_geo_line_string_3d(line_type: &geojson::LineStringType) -> LineString3D<f64> {
    LineString3D::new(line_type.iter().map(create_geo_coordinate_3d).collect())
}

pub(crate) fn create_geo_multi_line_string_2d(
    multi_line_type: &[geojson::LineStringType],
) -> MultiLineString2D<f64> {
    MultiLineString2D::new(
        multi_line_type
            .iter()
            .map(create_geo_line_string_2d)
            .collect(),
    )
}

pub(crate) fn create_geo_multi_line_string_3d(
    multi_line_type: &[geojson::LineStringType],
) -> MultiLineString3D<f64> {
    MultiLineString3D::new(
        multi_line_type
            .iter()
            .map(create_geo_line_string_3d)
            .collect(),
    )
}

pub(crate) fn create_geo_polygon_2d(polygon_type: &geojson::PolygonType) -> Polygon2D<f64> {
    let exterior = polygon_type
        .first()
        .map(create_geo_line_string_2d)
        .unwrap_or_else(|| create_geo_line_string_2d(&vec![]));

    let interiors = if polygon_type.len() < 2 {
        vec![]
    } else {
        polygon_type[1..]
            .iter()
            .map(create_geo_line_string_2d)
            .collect()
    };

    Polygon2D::new(exterior, interiors)
}

pub(crate) fn create_geo_polygon_3d(polygon_type: &geojson::PolygonType) -> Polygon3D<f64> {
    let exterior = polygon_type
        .first()
        .map(create_geo_line_string_3d)
        .unwrap_or_else(|| create_geo_line_string_3d(&vec![]));

    let interiors = if polygon_type.len() < 2 {
        vec![]
    } else {
        polygon_type[1..]
            .iter()
            .map(create_geo_line_string_3d)
            .collect()
    };

    Polygon3D::new(exterior, interiors)
}

pub(crate) fn create_geo_multi_polygon_2d(
    multi_polygon_type: &[geojson::PolygonType],
) -> MultiPolygon2D<f64> {
    MultiPolygon2D::new(
        multi_polygon_type
            .iter()
            .map(create_geo_polygon_2d)
            .collect(),
    )
}

pub(crate) fn create_geo_multi_polygon_3d(
    multi_polygon_type: &[geojson::PolygonType],
) -> MultiPolygon3D<f64> {
    MultiPolygon3D::new(
        multi_polygon_type
            .iter()
            .map(create_geo_polygon_3d)
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

#[cfg(test)]
mod test {
    use crate::types::conversion::geojson::*;
    use crate::types::no_value::NoValue;

    // generate create_geo_point test
    #[test]
    fn test_create_geo_point() {
        use geojson::PointType;

        let point_type: PointType = vec![1.0, 2.0];
        assert!(is_2d(&point_type));
        assert!(!is_3d(&point_type));
        let point = create_geo_point_2d(&point_type);
        assert_eq!(point.x(), 2.0);
        assert_eq!(point.y(), 1.0);
        assert_eq!(point.z(), NoValue);

        let point_type: PointType = vec![1.0, 2.0, 1.0];
        assert!(!is_2d(&point_type));
        assert!(is_3d(&point_type));
        let point = create_geo_point_3d(&point_type);
        assert_eq!(point.x(), 2.0);
        assert_eq!(point.y(), 1.0);
        assert_eq!(point.z(), 1.0);
    }
}
