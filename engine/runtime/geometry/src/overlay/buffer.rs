//! Buffering: the region within a distance of a geometry.
//!
//! [`buffer`] and its typed forms [`buffer_2d`] and [`buffer_polygon_3d`]
//! construct the (Minkowski) offset of a geometry by a signed distance, backed
//! by the `i_overlay` offset engine (`outline` for areal leaves, `stroke` for
//! polylines).
//!
//! In 2D the operand is treated like an overlay operand: a `Point`, a
//! `LineString`, a `Polygon`, a `PolygonMesh`, a `TriangularMesh`, or a
//! collection of these in one coordinate frame ([`MixedFrames`] otherwise).
//! The buffer of a collection is the buffer of the point-set union of its
//! leaves, so overlapping member buffers dissolve into one polygon. A point
//! buffers to a disc, a polyline to a stroke with round caps and joins, an
//! areal leaf to its offset with round joins at convex corners; a mesh is
//! dissolved to its union-boundary rings first. A negative distance contracts
//! areal leaves and yields nothing for points and polylines; an areal leaf
//! narrower than twice the contraction vanishes, and a contraction can split
//! one polygon into several.
//!
//! In 3D only a planar `Polygon` is accepted, and it is buffered in its own
//! plane: the face is checked for planarity, rotated to horizontal, offset,
//! and rotated back, so the result lies in the same plane with the same
//! winding sense as the input. Any other 3D leaf is [`Unsupported`].
//!
//! Output rings follow Flow's convention (exterior counter-clockwise, holes
//! clockwise in canonical orientation) whenever the frame's orientation sign
//! can be resolved, else the stored winding of the areal input. Like the rest
//! of the module the construction is not exact: coordinates are snapped to
//! `i_overlay`'s adaptive grid, and arcs are polygonal approximations whose
//! angular step is [`BufferStyle::arc_step`]. Appearance does not propagate.
//!
//! [`MixedFrames`]: PredicateError::MixedFrames
//! [`Unsupported`]: PredicateError::Unsupported

use core::f64::consts::PI;

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::mesh::outline::offset::OutlineOffset;
use i_overlay::mesh::stroke::offset::StrokeOffset;
use i_overlay::mesh::style::{LineCap, LineJoin, OutlineStyle, StrokeStyle};

use super::shapes::{self, Path, Shape};
use crate::collection::{Collection2D, Collection3D};
use crate::coordinate::CoordinateFrame;
use crate::ops::triangulation::normal;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::predicates::view::{flatten_2d, polygon3d_rings, Leaf2D};
use crate::predicates::{PredicateError, Result};
use crate::validation_next::measure::check_planarity_3d;
use crate::validation_next::{open_ring, signed_area_2d, PlanarityThreshold, ValidationReport};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

/// The smallest angular step an arc may be approximated with, in radians.
pub const MIN_ARC_STEP: f64 = 0.01 * PI;
/// The largest angular step an arc may be approximated with, in radians.
pub const MAX_ARC_STEP: f64 = 0.25 * PI;

/// How a geometry is buffered: the offset distance and the arc resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufferStyle {
    /// The signed offset distance, in the frame's coordinate units. Positive
    /// expands, negative contracts. Must be finite; a non-finite distance
    /// buffers to nothing.
    pub distance: f64,
    /// The angular step, in radians, of the segments approximating a round cap,
    /// join, or disc. Clamped to `[MIN_ARC_STEP, MAX_ARC_STEP]`; the default is
    /// `π / 16`.
    pub arc_step: f64,
    /// The planarity tolerance a 3D face must satisfy before it is buffered in
    /// its plane. Unused in 2D.
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

/// The buffer of `geometry`: a 2D geometry buffers to a polygon (or a
/// collection of polygons when the result has several parts) in its frame; a
/// 3D polygon buffers in its own plane; a 3D collection is buffered member by
/// member. An empty result is [`Geometry::None`].
pub fn buffer(geometry: &Geometry, style: &BufferStyle) -> Result<Geometry> {
    match geometry {
        Geometry::None => Ok(Geometry::None),
        Geometry::Euclidean2D(g) => Ok(assemble_2d(buffer_2d(g, style)?)),
        Geometry::Euclidean3D(g) => Ok(assemble_3d(buffer_3d(g, style)?)),
        Geometry::GeometryCollection(c) => buffer_collection(c, style),
    }
}

/// The buffer of a 2D geometry, as disjoint polygons in its frame (empty when
/// the result is empty).
pub fn buffer_2d(geometry: &Euclidean2DGeometry, style: &BufferStyle) -> Result<Vec<Polygon2D>> {
    let mut leaves = Vec::new();
    flatten_2d(geometry, &mut leaves);
    buffer_leaves(&leaves, style)
}

/// The buffer of a planar 3D polygon in its own plane, as polygons in its
/// frame (empty when the result is empty; several when a contraction splits
/// it). Errs with [`PredicateError::NotPlanar`] when the face is not planar
/// within `style.planarity` or has no fitted plane.
pub fn buffer_polygon_3d(polygon: &Polygon3D, style: &BufferStyle) -> Result<Vec<Polygon3D>> {
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
    let Some(rotation) = PlaneRotation::fit(exterior) else {
        return Err(PredicateError::NotPlanar);
    };
    let shape: Shape = polygon3d_rings(polygon)
        .map(|ring| {
            open_ring(ring)
                .iter()
                .map(|&p| rotation.flatten(p))
                .collect()
        })
        .filter(|path: &Path| !path.is_empty())
        .collect();
    let result = offset_shapes(vec![shape], style);
    Ok(result
        .into_iter()
        .filter_map(|shape| {
            let mut rings = shape.into_iter().map(|path| {
                close(path)
                    .into_iter()
                    .map(|p| rotation.lift(p))
                    .collect::<Vec<_>>()
            });
            let exterior = rings.next()?;
            Some(Polygon3D::from_rings(frame.clone(), exterior, rings))
        })
        .collect())
}

/// Wrap the 2D result polygons as a geometry.
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

/// Wrap the 3D result polygons as a geometry.
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

/// Buffer a 3D geometry: each polygon leaf in its own plane, collected in
/// order. A collection may hold only polygons (nested collections allowed).
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

/// Buffer a heterogeneous collection: its 2D members together as one 2D
/// operand, its 3D members each in their own plane. A collection mixing the two
/// buffers to a collection of both results.
fn buffer_collection(collection: &GeometryCollection, style: &BufferStyle) -> Result<Geometry> {
    let mut leaves_2d = Vec::new();
    let mut polygons_3d = Vec::new();
    let mut nested = Vec::new();
    for member in collection.members() {
        match member {
            Geometry::None => {}
            Geometry::Euclidean2D(g) => flatten_2d(g, &mut leaves_2d),
            Geometry::Euclidean3D(g) => collect_3d(g, style, &mut polygons_3d)?,
            Geometry::GeometryCollection(c) => nested.push(c),
        }
    }
    let mut parts = Vec::new();
    match assemble_2d(buffer_leaves(&leaves_2d, style)?) {
        Geometry::None => {}
        g => parts.push(g),
    }
    match assemble_3d(polygons_3d) {
        Geometry::None => {}
        g => parts.push(g),
    }
    for c in nested {
        match buffer_collection(c, style)? {
            Geometry::None => {}
            g => parts.push(g),
        }
    }
    Ok(match parts.len() {
        0 => Geometry::None,
        1 => parts.pop().expect("one part"),
        _ => Geometry::GeometryCollection(GeometryCollection::new(parts)),
    })
}

// --- 2D leaf-level implementation ---------------------------------------------

/// Buffer flattened 2D leaves in one frame.
fn buffer_leaves(leaves: &[Leaf2D<'_>], style: &BufferStyle) -> Result<Vec<Polygon2D>> {
    let Some(frame) = common_frame(leaves)? else {
        return Ok(Vec::new());
    };
    if !style.distance.is_finite() {
        return Ok(Vec::new());
    }
    let (areal, others): (Vec<_>, Vec<_>) = leaves.iter().copied().partition(is_areal);
    let (lines, points): (Vec<_>, Vec<_>) = others.into_iter().partition(is_line);

    let areal_shapes = shapes::areal_shapes(&areal).expect("areal leaves only");
    let stored_sign = frame_sign(frame).unwrap_or_else(|| shapes_sign(&areal_shapes));
    let areal_shapes: Vec<Shape> = areal_shapes.into_iter().map(normalize_shape).collect();

    let mut groups: Vec<Vec<Shape>> = Vec::new();
    // Discs are emitted directly, so several of them need the dissolve the
    // backend gives the other groups for free.
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
    let elevation = common_elevation(leaves);
    Ok(result
        .into_iter()
        .filter_map(|shape| {
            let mut rings = shape.into_iter().map(close);
            let exterior = rings.next()?;
            Some(match elevation {
                Some(z) => Polygon2D::from_rings_at_elevation(frame.clone(), exterior, rings, z),
                None => Polygon2D::from_rings(frame.clone(), exterior, rings),
            })
        })
        .collect())
}

/// Offset areal shapes (stored counter-clockwise exteriors, clockwise holes)
/// by `style.distance`, dissolving overlaps. A zero distance dissolves only.
fn offset_shapes(shapes: Vec<Shape>, style: &BufferStyle) -> Vec<Shape> {
    if style.distance == 0.0 {
        return dissolve(shapes);
    }
    let outline =
        OutlineStyle::new(style.distance).line_join(LineJoin::Round(style.clamped_arc_step()));
    shapes.outline(&outline)
}

/// Stroke open polylines with width twice `style.distance`, round caps and
/// joins, dissolving overlaps.
fn stroke_paths(paths: Vec<Path>, style: &BufferStyle) -> Vec<Shape> {
    let step = style.clamped_arc_step();
    let stroke = StrokeStyle::new(2.0 * style.distance)
        .line_join(LineJoin::Round(step))
        .start_cap(LineCap::Round(step))
        .end_cap(LineCap::Round(step));
    paths.stroke(stroke, false)
}

/// The counter-clockwise disc of radius `style.distance` around `center`, as
/// one implicitly closed path.
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

/// The union of `shapes` under the non-zero rule.
fn dissolve(shapes: Vec<Shape>) -> Vec<Shape> {
    let empty: Vec<Shape> = Vec::new();
    shapes.overlay(&empty, OverlayRule::Union, FillRule::NonZero)
}

/// Rewind a shape to a stored counter-clockwise exterior when its rings' total
/// signed area is negative.
fn normalize_shape(shape: Shape) -> Shape {
    if shape_area(&shape) < 0.0 {
        reverse_shape(shape)
    } else {
        shape
    }
}

/// Twice the total signed area of a shape's rings.
fn shape_area(shape: &Shape) -> f64 {
    shape.iter().map(|ring| signed_area_2d(ring)).sum()
}

/// The sign of the total signed area of areal shapes: `-1.0` when the stored
/// winding is clockwise, else `1.0`.
fn shapes_sign(shapes: &[Shape]) -> f64 {
    if shapes.iter().map(shape_area).sum::<f64>() < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// The frame's orientation sign, `None` when it cannot be resolved.
fn frame_sign(frame: &CoordinateFrame) -> Option<f64> {
    frame.orientation_sign().ok().map(f64::from)
}

fn reverse_shape(shape: Shape) -> Shape {
    shape
        .into_iter()
        .map(|mut ring| {
            ring.reverse();
            ring
        })
        .collect()
}

/// Close an implicitly closed path by appending its first vertex.
fn close(mut path: Path) -> Path {
    if let Some(&first) = path.first() {
        path.push(first);
    }
    path
}

/// The leaves' shared frame, or `None` when there are no leaves.
fn common_frame<'l>(leaves: &[Leaf2D<'l>]) -> Result<Option<&'l CoordinateFrame>> {
    let mut frames = leaves.iter().map(Leaf2D::frame);
    let Some(first) = frames.next() else {
        return Ok(None);
    };
    for frame in frames {
        crate::predicates::require_same_frame(first, frame)?;
    }
    Ok(Some(first))
}

/// The elevation every leaf lies at, when they all agree (a point has none).
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

fn is_areal(leaf: &Leaf2D<'_>) -> bool {
    matches!(
        leaf,
        Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_)
    )
}

fn is_line(leaf: &Leaf2D<'_>) -> bool {
    matches!(leaf, Leaf2D::Line(_))
}

// --- 3D plane rotation ---------------------------------------------------------

/// The rigid motion taking a planar face to the horizontal plane `z = 0`
/// and back: a translation to the face's first vertex followed by the rotation
/// carrying the face normal onto `+z`.
struct PlaneRotation {
    origin: [f64; 3],
    /// Row-major rotation matrix.
    m: [[f64; 3]; 3],
}

impl PlaneRotation {
    /// Fit from a closed exterior ring; `None` when the ring has no normal.
    fn fit(exterior: &[[f64; 3]]) -> Option<Self> {
        let ring = open_ring(exterior);
        let [nx, ny, nz] = normal(ring)?;
        let origin = *ring.first()?;
        let horizontal = (nx * nx + ny * ny).sqrt();
        let m = if horizontal < 1e-12 {
            if nz > 0.0 {
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            } else {
                [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]]
            }
        } else {
            // Rodrigues rotation about the unit axis n x ez = (ny, -nx, 0) / |.|
            // by the angle between n and ez.
            let (ax, ay) = (ny / horizontal, -nx / horizontal);
            let cos = nz;
            let sin = horizontal;
            let k = 1.0 - cos;
            [
                [cos + ax * ax * k, ax * ay * k, ay * sin],
                [ax * ay * k, cos + ay * ay * k, -ax * sin],
                [-ay * sin, ax * sin, cos],
            ]
        };
        Some(Self { origin, m })
    }

    /// Rotate `p` into the horizontal frame and drop its (near-zero) height.
    fn flatten(&self, p: [f64; 3]) -> [f64; 2] {
        let d = [
            p[0] - self.origin[0],
            p[1] - self.origin[1],
            p[2] - self.origin[2],
        ];
        let row = |r: [f64; 3]| r[0] * d[0] + r[1] * d[1] + r[2] * d[2];
        [row(self.m[0]), row(self.m[1])]
    }

    /// Lift a horizontal-frame point back onto the face's plane.
    fn lift(&self, [x, y]: [f64; 2]) -> [f64; 3] {
        let m = &self.m;
        [
            m[0][0] * x + m[1][0] * y + self.origin[0],
            m[0][1] * x + m[1][1] * y + self.origin[1],
            m[0][2] * x + m[1][2] * y + self.origin[2],
        ]
    }
}

#[cfg(test)]
mod tests;
