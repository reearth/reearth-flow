//! Buffering: the region within a signed distance of a geometry.
//!
//! In 2D the operand is treated like an overlay operand: leaves in one
//! coordinate frame ([`MixedFrames`] otherwise), buffered as the point-set
//! union of the leaves. A point buffers to a disc, a polyline to a stroke with
//! round caps and joins, an areal leaf to its offset with round joins. In 3D
//! only a planar `Polygon` is accepted, buffered in its own plane.
//! Any other 3D leaf is [`Unsupported`].
//!
//! A polygon input whose hole winds the same way as its exterior is
//! [`InvalidHoleWinding`]; a mesh's rings are wound by construction. Output
//! rings follow the frame's orientation convention when it can be resolved,
//! else the stored winding of the areal input. A mesh whose face union falls
//! into several regions buffers region by region, so it can yield several
//! polygons. Coordinates are snapped to `i_overlay`'s adaptive grid, arcs are
//! polygonal approximations stepped by [`BufferStyle::arc_step`], and
//! appearance does not propagate.
//!
//! [`MixedFrames`]: PredicateError::MixedFrames
//! [`Unsupported`]: PredicateError::Unsupported
//! [`InvalidHoleWinding`]: PredicateError::InvalidHoleWinding

use core::f64::consts::PI;

use i_overlay::mesh::outline::offset::OutlineOffset;
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::{LineCap, LineJoin, OutlineStyle, StrokeStyle};

use super::shapes::{self, close_path, dissolve, frame_sign, reverse_shape, Path, Shape};
use super::{common_frame, is_areal, is_line};
use crate::collection::{Collection2D, Collection3D};
use crate::coordinate::{BaseFrame, TangentPlane};
use crate::ops::triangulation::normal;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::predicates::view::{flatten_2d, polygon3d_rings, require_common_frame_leaves, Leaf2D};
use crate::predicates::{PredicateError, Result};
use crate::validation_next::{
    check_face_orientation_3d, check_planarity_3d, open_ring, PlanarityThreshold, ValidationReport,
};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

/// The smallest angular step an arc may be approximated with, in radians.
pub const MIN_ARC_STEP: f64 = 0.01 * PI;
/// The largest angular step an arc may be approximated with, in radians.
pub const MAX_ARC_STEP: f64 = 0.25 * PI;

/// How a geometry is buffered: the offset distance and the arc resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufferStyle {
    /// The signed offset distance in frame units; positive expands, negative
    /// contracts. A non-finite distance buffers to nothing.
    pub distance: f64,
    /// The angular step, in radians, of round caps, joins, and discs. Values
    /// outside `[MIN_ARC_STEP, MAX_ARC_STEP]` are clamped when used.
    pub arc_step: f64,
    /// The planarity tolerance a 3D face must satisfy. Unused in 2D.
    pub planarity: PlanarityThreshold,
}

impl BufferStyle {
    /// A style with `distance` and the default arc step and planarity threshold.
    pub fn new(distance: f64) -> Self {
        Self {
            distance,
            ..Self::default()
        }
    }

    /// This style with `arc_step` (radians).
    pub fn arc_step(mut self, arc_step: f64) -> Self {
        self.arc_step = arc_step;
        self
    }

    /// This style with `planarity`.
    pub fn planarity(mut self, planarity: PlanarityThreshold) -> Self {
        self.planarity = planarity;
        self
    }

    /// The arc step clamped to the supported range.
    fn clamped_arc_step(&self) -> f64 {
        if self.arc_step.is_nan() {
            return Self::default().arc_step;
        }
        self.arc_step.clamp(MIN_ARC_STEP, MAX_ARC_STEP)
    }
}

impl Default for BufferStyle {
    fn default() -> Self {
        Self {
            distance: 1.0,
            arc_step: PI / 16.0,
            planarity: PlanarityThreshold::Ratio(0.001),
        }
    }
}

/// The buffer of `geometry`. A polygon, or a collection of polygons when the
/// result has several parts. A 3D collection is buffered member by member. An
/// empty result is [`Geometry::None`].
pub fn buffer(geometry: &Geometry, style: &BufferStyle) -> Result<Geometry> {
    match geometry {
        Geometry::None => Ok(Geometry::None),
        Geometry::Euclidean2D(g) => Ok(assemble_2d(buffer_2d(g, style)?)),
        Geometry::Euclidean3D(g) => Ok(assemble_3d(buffer_3d(g, style)?)),
        Geometry::GeometryCollection(c) => buffer_collection(c, style),
    }
}

/// The buffer of a 2D geometry, as disjoint polygons in its frame.
pub fn buffer_2d(geometry: &Euclidean2DGeometry, style: &BufferStyle) -> Result<Vec<Polygon2D>> {
    let mut leaves = Vec::new();
    flatten_2d(geometry, &mut leaves);
    buffer_leaves(&leaves, style)
}

/// The buffer of a planar 3D polygon in its own plane, as polygons in its
/// frame. The face must be planar within `style.planarity` ([`NotPlanar`]
/// otherwise) and its holes must wind opposite the exterior
/// ([`InvalidHoleWinding`] otherwise).
///
/// [`NotPlanar`]: PredicateError::NotPlanar
/// [`InvalidHoleWinding`]: PredicateError::InvalidHoleWinding
pub fn buffer_polygon_3d(polygon: &Polygon3D, style: &BufferStyle) -> Result<Vec<Polygon3D>> {
    if !style.distance.is_finite() {
        return Ok(Vec::new());
    }
    let frame = polygon.frame();
    let exterior = polygon.exterior();
    let interiors: Vec<&[[f64; 3]]> = polygon.interiors().collect();
    let report = ValidationReport::ran(|report| {
        check_planarity_3d(
            frame,
            exterior,
            interiors.iter().copied(),
            style.planarity,
            report,
        )
    });
    if report.problem_recorded() {
        return Err(PredicateError::NotPlanar);
    }
    let report = ValidationReport::ran(|report| {
        check_face_orientation_3d(frame, exterior, interiors.iter().copied(), report)
    });
    if report.problem_recorded() {
        return Err(PredicateError::InvalidHoleWinding);
    }
    let plane = fit_plane(exterior).ok_or(PredicateError::NotPlanar)?;
    let shape: Shape = polygon3d_rings(polygon)
        .map(|ring| open_ring(ring).iter().map(|&p| plane.project(p)).collect())
        .filter(|path: &Path| !path.is_empty())
        .collect();
    let result = offset_shapes(vec![shape], style);
    Ok(result
        .into_iter()
        .filter_map(|shape| {
            let mut rings = shape.into_iter().map(|path| {
                close_path(path)
                    .into_iter()
                    .map(|p| plane.lift(p))
                    .collect::<Vec<_>>()
            });
            let exterior = rings.next()?;
            Some(Polygon3D::from_rings(frame.clone(), exterior, rings))
        })
        .collect())
}

/// The tangent plane of a planar ring, oriented so the ring projects
/// counter-clockwise; `None` when the ring has no normal.
fn fit_plane(exterior: &[[f64; 3]]) -> Option<TangentPlane> {
    let ring = open_ring(exterior);
    let n = normal(ring)?;
    let origin = *ring.first()?;
    TangentPlane::from_normal(BaseFrame::Euclidean, origin, n, None).ok()
}

fn assemble_2d(mut polygons: Vec<Polygon2D>) -> Geometry {
    match polygons.len() {
        0 => Geometry::None,
        1 => Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            polygons.pop().expect("one polygon"),
        ))),
        _ => Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
            polygons
                .into_iter()
                .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
        ))),
    }
}

fn assemble_3d(mut polygons: Vec<Polygon3D>) -> Geometry {
    match polygons.len() {
        0 => Geometry::None,
        1 => Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            polygons.pop().expect("one polygon"),
        ))),
        _ => Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(
            polygons
                .into_iter()
                .map(|p| Euclidean3DGeometry::Polygon(Box::new(p))),
        ))),
    }
}

fn buffer_3d(geometry: &Euclidean3DGeometry, style: &BufferStyle) -> Result<Vec<Polygon3D>> {
    let mut out = Vec::new();
    collect_3d(geometry, style, &mut out)?;
    Ok(out)
}

fn collect_3d(
    geometry: &Euclidean3DGeometry,
    style: &BufferStyle,
    out: &mut Vec<Polygon3D>,
) -> Result<()> {
    match geometry {
        Euclidean3DGeometry::Polygon(p) => {
            out.extend(buffer_polygon_3d(p, style)?);
            Ok(())
        }
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                collect_3d(member, style, out)?;
            }
            Ok(())
        }
        Euclidean3DGeometry::Point(_) => unsupported("Point3D"),
        Euclidean3DGeometry::PointCloud(_) => unsupported("PointCloud"),
        Euclidean3DGeometry::LineString(_) => unsupported("LineString3D"),
        Euclidean3DGeometry::PolygonMesh(_) => unsupported("PolygonMesh3D"),
        Euclidean3DGeometry::TriangularMesh(_) => unsupported("TriangularMesh3D"),
        Euclidean3DGeometry::Solid(_) => unsupported("Solid"),
        Euclidean3DGeometry::Csg(_) => unsupported("Csg"),
    }
}

fn unsupported<T>(geometry: &'static str) -> Result<T> {
    Err(PredicateError::Unsupported { geometry })
}

/// Buffer a heterogeneous collection: its 2D leaves, at any depth, together
/// as one operand, its 3D polygons each in their own plane.
fn buffer_collection(collection: &GeometryCollection, style: &BufferStyle) -> Result<Geometry> {
    let mut leaves_2d = Vec::new();
    let mut polygons_3d = Vec::new();
    collect_members(collection, style, &mut leaves_2d, &mut polygons_3d)?;
    let mut parts = Vec::new();
    match assemble_2d(buffer_leaves(&leaves_2d, style)?) {
        Geometry::None => {}
        g => parts.push(g),
    }
    match assemble_3d(polygons_3d) {
        Geometry::None => {}
        g => parts.push(g),
    }
    Ok(match parts.len() {
        0 => Geometry::None,
        1 => parts.pop().expect("one part"),
        _ => Geometry::GeometryCollection(GeometryCollection::new(parts)),
    })
}

fn collect_members<'c>(
    collection: &'c GeometryCollection,
    style: &BufferStyle,
    leaves_2d: &mut Vec<Leaf2D<'c>>,
    polygons_3d: &mut Vec<Polygon3D>,
) -> Result<()> {
    for member in collection.members() {
        match member {
            Geometry::None => {}
            Geometry::Euclidean2D(g) => flatten_2d(g, leaves_2d),
            Geometry::Euclidean3D(g) => collect_3d(g, style, polygons_3d)?,
            Geometry::GeometryCollection(c) => collect_members(c, style, leaves_2d, polygons_3d)?,
        }
    }
    Ok(())
}

fn buffer_leaves(leaves: &[Leaf2D<'_>], style: &BufferStyle) -> Result<Vec<Polygon2D>> {
    require_common_frame_leaves(leaves, &[])?;
    let Some(frame) = common_frame(leaves, &[]) else {
        return Ok(Vec::new());
    };
    if !style.distance.is_finite() {
        return Ok(Vec::new());
    }
    let (areal, others): (Vec<_>, Vec<_>) = leaves.iter().copied().partition(is_areal);
    let (lines, points): (Vec<_>, Vec<_>) = others.into_iter().partition(is_line);

    let areal_shapes = shapes::areal_shapes(&areal).expect("areal leaves only");
    require_opposing_holes(&areal_shapes)?;
    let stored_sign = frame_sign(frame).unwrap_or_else(|| shapes_sign(&areal_shapes));
    let areal_shapes: Vec<Shape> = areal_shapes.into_iter().map(normalize_shape).collect();

    let mut groups: Vec<Vec<Shape>> = Vec::new();
    let mut needs_dissolve = false;
    if !areal_shapes.is_empty() {
        groups.push(offset_shapes(areal_shapes, style));
    }
    if style.distance > 0.0 {
        let paths = shapes::line_paths(&lines).expect("line leaves only");
        if !paths.is_empty() {
            groups.push(stroke_paths(paths, style));
        }
        let discs: Vec<Shape> = points
            .iter()
            .filter_map(|leaf| match leaf {
                Leaf2D::Point(p) => Some(vec![disc(p.position(), style)]),
                _ => None,
            })
            .collect();
        if !discs.is_empty() {
            needs_dissolve = discs.len() > 1;
            groups.push(discs);
        }
    }
    let result = match groups.len() {
        0 => Vec::new(),
        1 if !needs_dissolve => groups.pop().expect("one group"),
        _ => dissolve(groups.into_iter().flatten().collect()),
    };
    let result = if stored_sign < 0.0 {
        result.into_iter().map(reverse_shape).collect()
    } else {
        result
    };
    // Lines and points contribute nothing at a non-positive distance, so
    // they do not vote on the elevation there.
    let elevation = if style.distance > 0.0 {
        common_elevation(leaves)
    } else {
        common_elevation(&areal)
    };
    Ok(shapes::shapes_to_polygons(result, frame, elevation))
}

/// Err with [`PredicateError::InvalidHoleWinding`] when a hole of any shape
/// winds the same way as its exterior; a zero-area ring is skipped. Only a
/// polygon leaf can carry such a hole: mesh shapes arrive regrouped.
fn require_opposing_holes(shapes: &[Shape]) -> Result<()> {
    for shape in shapes {
        let mut areas = shape.iter().map(|ring| ring_area(ring));
        let Some(exterior) = areas.next() else {
            continue;
        };
        if areas.any(|hole| hole * exterior > 0.0) {
            return Err(PredicateError::InvalidHoleWinding);
        }
    }
    Ok(())
}

/// Offset areal shapes (counter-clockwise exteriors, clockwise holes),
/// dissolving overlaps.
fn offset_shapes(shapes: Vec<Shape>, style: &BufferStyle) -> Vec<Shape> {
    if style.distance == 0.0 {
        return dissolve(shapes);
    }
    let outline =
        OutlineStyle::new(style.distance).line_join(LineJoin::Round(style.clamped_arc_step()));
    shapes.outline(&outline)
}

/// Stroke polylines with width twice `style.distance`, dissolving overlaps.
fn stroke_paths(paths: Vec<Path>, style: &BufferStyle) -> Vec<Shape> {
    let step = style.clamped_arc_step();
    let stroke = StrokeStyle::new(2.0 * style.distance)
        .line_join(LineJoin::Round(step))
        .start_cap(LineCap::Round(step))
        .end_cap(LineCap::Round(step));
    paths.stroke(stroke, false)
}

/// The counter-clockwise disc of radius `style.distance` around `center`.
fn disc(center: [f64; 2], style: &BufferStyle) -> Path {
    let r = style.distance;
    let n = (2.0 * PI / style.clamped_arc_step()).ceil().max(3.0) as usize;
    (0..n)
        .map(|i| {
            let a = 2.0 * PI * i as f64 / n as f64;
            [center[0] + r * a.cos(), center[1] + r * a.sin()]
        })
        .collect()
}

/// Rewind a shape so its total signed area is non-negative.
fn normalize_shape(shape: Shape) -> Shape {
    if shape_area(&shape) < 0.0 {
        reverse_shape(shape)
    } else {
        shape
    }
}

/// Twice the total signed area of a shape's rings.
fn shape_area(shape: &Shape) -> f64 {
    shape.iter().map(|ring| ring_area(ring)).sum()
}

/// Twice the signed area of a ring (shoelace), wrapping the last vertex back to
/// the first. Positive = counter-clockwise, negative = clockwise, zero =
/// degenerate.
fn ring_area(ring: &[[f64; 2]]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let a = ring[i];
            let b = ring[(i + 1) % n];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

/// `-1.0` when the shapes' total signed area is negative, else `1.0`.
fn shapes_sign(shapes: &[Shape]) -> f64 {
    if shapes.iter().map(shape_area).sum::<f64>() < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// The elevation shared by every leaf, if any.
fn common_elevation(leaves: &[Leaf2D<'_>]) -> Option<f64> {
    let mut elevations = leaves.iter().map(|leaf| match leaf {
        Leaf2D::Point(_) => None,
        Leaf2D::Line(l) => l.elevation(),
        Leaf2D::Polygon(p) => p.elevation(),
        Leaf2D::PolygonMesh(m) => m.elevation(),
        Leaf2D::TriangularMesh(m) => m.elevation(),
    });
    let first = elevations.next()??;
    elevations.all(|z| z == Some(first)).then_some(first)
}

#[cfg(test)]
mod tests;
