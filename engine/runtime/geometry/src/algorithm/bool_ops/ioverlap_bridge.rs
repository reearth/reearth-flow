use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::string::clip::ClipRule;

use crate::algorithm::GeoFloat;
use crate::types::{
    coordinate::Coordinate2D, line_string::LineString2D, multi_line_string::MultiLineString2D,
    multi_polygon::MultiPolygon2D, polygon::Polygon2D,
};

use super::OpType;

/// Newtype wrapper to implement FloatPointCompatible for Coordinate2D.
/// Required to circumvent the orphan rule.
#[derive(Copy, Clone, Debug)]
pub(crate) struct FlowCoord<T: GeoFloat>(pub Coordinate2D<T>);

impl<T: GeoFloat + FloatNumber> FloatPointCompatible<T> for FlowCoord<T> {
    fn from_xy(x: T, y: T) -> Self {
        Self(Coordinate2D::new_(x, y))
    }

    fn x(&self) -> T {
        self.0.x
    }

    fn y(&self) -> T {
        self.0.y
    }
}

/// Convert a flow Polygon2D to iOverlay shape paths.
/// Returns a vector of rings (exterior + holes), where each ring is implicitly closed.
pub(crate) fn polygon_to_shape_paths<T>(poly: &Polygon2D<T>) -> Vec<Vec<FlowCoord<T>>>
where
    T: GeoFloat + FloatNumber,
{
    let mut paths = Vec::with_capacity(1 + poly.interiors().len());

    // Exterior ring
    paths.push(ring_to_path(poly.exterior()));

    // Interior rings (holes)
    for hole in poly.interiors() {
        paths.push(ring_to_path(hole));
    }

    paths
}

/// Convert a flow MultiPolygon2D to iOverlay shape paths.
pub(crate) fn multi_polygon_to_shape_paths<T>(
    multi_poly: &MultiPolygon2D<T>,
) -> Vec<Vec<Vec<FlowCoord<T>>>>
where
    T: GeoFloat + FloatNumber,
{
    multi_poly
        .0
        .iter()
        .map(|poly| polygon_to_shape_paths(poly))
        .collect()
}

/// Convert a LineString2D ring to an iOverlay path.
/// iOverlay expects implicitly closed paths, so we skip the last coordinate
/// if it equals the first (explicitly closed rings).
fn ring_to_path<T: GeoFloat + FloatNumber>(ring: &LineString2D<T>) -> Vec<FlowCoord<T>> {
    if ring.0.is_empty() {
        return vec![];
    }

    // Skip last coordinate if it's a closing point (same as first)
    let coords = &ring.0[..ring.0.len() - 1];
    coords.iter().copied().map(FlowCoord).collect()
}

/// Convert a LineString2D to an iOverlay path (for line clipping).
/// Unlike rings, lines are not implicitly closed.
fn line_string_to_path<T: GeoFloat + FloatNumber>(line: &LineString2D<T>) -> Vec<FlowCoord<T>> {
    line.0.iter().copied().map(FlowCoord).collect()
}

/// Convert iOverlay shapes back to flow MultiPolygon2D.
pub(crate) fn shapes_to_multi_polygon<T>(shapes: Vec<Vec<Vec<FlowCoord<T>>>>) -> MultiPolygon2D<T>
where
    T: GeoFloat,
{
    let polygons: Vec<_> = shapes.into_iter().map(shape_to_polygon).collect();
    MultiPolygon2D::new(polygons)
}

/// Convert a single iOverlay shape to a flow Polygon2D.
fn shape_to_polygon<T: GeoFloat>(shape: Vec<Vec<FlowCoord<T>>>) -> Polygon2D<T> {
    let mut rings = shape.into_iter().map(|path| {
        let mut coords: Vec<_> = path.into_iter().map(|fc| fc.0).collect();

        // Add closing point (iOverlay paths are implicitly closed)
        if !coords.is_empty() {
            coords.push(coords[0]);
        }

        LineString2D::new(coords)
    });

    let exterior = rings.next().unwrap_or_else(|| LineString2D::new(vec![]));
    let interiors: Vec<_> = rings.collect();

    Polygon2D::new(exterior, interiors)
}

/// Convert iOverlay paths back to flow MultiLineString2D (for clipping results).
pub(crate) fn paths_to_multi_line_string<T>(paths: Vec<Vec<FlowCoord<T>>>) -> MultiLineString2D<T>
where
    T: GeoFloat,
{
    let line_strings: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let coords: Vec<_> = path.into_iter().map(|fc| fc.0).collect();
            LineString2D::new(coords)
        })
        .collect();

    MultiLineString2D::new(line_strings)
}

/// Perform boolean operation on two polygons using iOverlay.
pub(crate) fn boolean_op_polygon<T>(
    poly1: &Polygon2D<T>,
    poly2: &Polygon2D<T>,
    op: OpType,
) -> MultiPolygon2D<T>
where
    T: GeoFloat + FloatNumber,
{
    let subject = vec![polygon_to_shape_paths(poly1)];
    let clip = vec![polygon_to_shape_paths(poly2)];

    let shapes = subject.overlay(&clip, op.into(), FillRule::EvenOdd);

    shapes_to_multi_polygon(shapes)
}

/// Perform boolean operation on two multi-polygons using iOverlay.
pub(crate) fn boolean_op_multi_polygon<T>(
    mpoly1: &MultiPolygon2D<T>,
    mpoly2: &MultiPolygon2D<T>,
    op: OpType,
) -> MultiPolygon2D<T>
where
    T: GeoFloat + FloatNumber,
{
    let subject = multi_polygon_to_shape_paths(mpoly1);
    let clip = multi_polygon_to_shape_paths(mpoly2);

    let shapes = subject.overlay(&clip, op.into(), FillRule::EvenOdd);

    shapes_to_multi_polygon(shapes)
}

/// Clip a multi-line-string with a polygon using iOverlay.
pub(crate) fn clip_polygon<T>(
    poly: &Polygon2D<T>,
    mls: &MultiLineString2D<T>,
    invert: bool,
) -> MultiLineString2D<T>
where
    T: GeoFloat + FloatNumber,
{
    let subject: Vec<Vec<_>> = mls.0.iter().map(|line| line_string_to_path(line)).collect();

    let clip_paths = vec![polygon_to_shape_paths(poly)];

    let clip_rule = ClipRule {
        invert,
        boundary_included: true,
    };

    let paths = subject.clip_by(&clip_paths, FillRule::EvenOdd, clip_rule);

    paths_to_multi_line_string(paths)
}

/// Clip a multi-line-string with a multi-polygon using iOverlay.
pub(crate) fn clip_multi_polygon<T>(
    mpoly: &MultiPolygon2D<T>,
    mls: &MultiLineString2D<T>,
    invert: bool,
) -> MultiLineString2D<T>
where
    T: GeoFloat + FloatNumber,
{
    let subject: Vec<Vec<_>> = mls.0.iter().map(|line| line_string_to_path(line)).collect();

    let clip_paths = multi_polygon_to_shape_paths(mpoly);

    let clip_rule = ClipRule {
        invert,
        boundary_included: true,
    };

    let paths = subject.clip_by(&clip_paths, FillRule::EvenOdd, clip_rule);

    paths_to_multi_line_string(paths)
}

/// Map OpType to iOverlay's OverlayRule.
impl From<OpType> for OverlayRule {
    fn from(op: OpType) -> Self {
        match op {
            OpType::Intersection => OverlayRule::Intersect,
            OpType::Union => OverlayRule::Union,
            OpType::Difference => OverlayRule::Difference,
            OpType::Xor => OverlayRule::Xor,
        }
    }
}
