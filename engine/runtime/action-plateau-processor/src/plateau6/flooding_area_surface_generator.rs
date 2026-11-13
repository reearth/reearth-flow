//! Flood-zone surface generation for PLATEAU flood risk models.
//!
//! A flood zone arrives as a patchwork of small triangles, one body per rank and
//! file. For display, each group of bodies (one folder, rank and scale) is
//! dissolved into its connected areas and every area is triangulated afresh: the
//! area's outline is the constraint and a sparse grid of points sampled from the
//! original surface fills its inside. Each area becomes one feature.
//!
//! Lengths are in frame units. The values follow the original workspace, which
//! works in EPSG:3857, so the input is expected in that frame.

use std::collections::HashMap;

use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, ErrorCode, Severity};
use reearth_flow_geometry::collection::Collection3D;
use reearth_flow_geometry::overlay::{BufferStyle, buffer_2d, dissolve_leaves};
use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::predicates::kernel::{CoordPos, Orientation, orient2d};
use reearth_flow_geometry::predicates::point_position_2d;
use reearth_flow_geometry::predicates::view::{Leaf2D, polygon2d_rings, polygon3d_rings};
use reearth_flow_geometry::validation_next::open_ring;
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{FEATURES_PORT, Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use super::errors::Plateau6ProcessorError;
use crate::common::PlateauProfile;

/// Spacing of the grid the inside points are sampled on.
const POINT_SPACING: f64 = 50.0;
/// Only every this many grid points is kept.
const THINNING_INTERVAL: usize = 20;
/// How far an area is shrunk before picking the points inside it, so no point
/// sits on or next to the outline.
const SHRINK_DISTANCE: f64 = 0.5;
/// The inside points are used only when their minimum bounding rectangle is
/// wider than this on its shorter side; nearly collinear points make slivers.
const MIN_POINT_SPREAD: f64 = 5.0;
/// Triangles with two corners this close together are dropped.
const DUPLICATE_VERTEX_TOLERANCE: f64 = 0.01;
/// Outline vertices of neighboring bodies that miss by less than this are
/// treated as one when dissolving.
const SNAP_TOLERANCE: f64 = 0.001;
/// Cell size of the index used to find the original triangle under a point.
const INDEX_CELL_SIZE: f64 = 10.0;

#[derive(Debug, Clone)]
pub(crate) struct FloodingAreaSurfaceGeneratorFactory {
    name: String,
}

impl FloodingAreaSurfaceGeneratorFactory {
    pub(crate) fn new(profile: &PlateauProfile) -> Self {
        Self {
            name: profile.action_name("FloodingAreaSurfaceGenerator"),
        }
    }
}

impl ProcessorFactory for FloodingAreaSurfaceGeneratorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Merges the triangles of flood zone bodies into connected areas and rebuilds each area \
         as one triangulated surface, constrained to the area's outline and filled with sparse \
         points sampled from the original surface. Emits one feature per area, carrying the \
         attributes of the first body in its group. All input must be in one coordinate frame; \
         the action fails otherwise."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FloodingAreaSurfaceGeneratorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FloodingAreaSurfaceGeneratorParam = match with {
            Some(with) => {
                let value = serde_json::to_value(with).map_err(|e| {
                    Plateau6ProcessorError::FloodingAreaSurfaceGeneratorFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    Plateau6ProcessorError::FloodingAreaSurfaceGeneratorFactory(format!(
                        "Failed to deserialize `with` parameter: {e}"
                    ))
                })?
            }
            None => FloodingAreaSurfaceGeneratorParam::default(),
        };
        Ok(Box::new(FloodingAreaSurfaceGenerator {
            group_by: params.group_by,
            groups: Vec::new(),
            group_index: HashMap::new(),
        }))
    }
}

/// # Flooding Area Surface Generator Parameters
///
/// Which bodies are merged into one surface.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FloodingAreaSurfaceGeneratorParam {
    /// # Group By
    /// Attributes whose values partition the bodies: only bodies with the same
    /// values are merged. Empty (the default) merges every body together.
    #[serde(default)]
    group_by: Vec<Attribute>,
}

/// The bodies of one group, in input order.
#[derive(Debug)]
struct Group {
    /// The first body, whose attributes every output feature of the group takes.
    representative: Feature,
    polygons: Vec<Polygon3D>,
}

#[derive(Debug)]
pub(crate) struct FloodingAreaSurfaceGenerator {
    group_by: Vec<Attribute>,
    groups: Vec<Group>,
    group_index: HashMap<Vec<AttributeValue>, usize>,
}

impl Clone for FloodingAreaSurfaceGenerator {
    /// The accumulated state belongs to one run, so a clone starts empty the way
    /// the other accumulating actions' do.
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            groups: Vec::new(),
            group_index: HashMap::new(),
        }
    }
}

impl Processor for FloodingAreaSurfaceGenerator {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut polygons = Vec::new();
        collect_polygons(&ctx.feature.geometry, &mut polygons);
        if polygons.is_empty() {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                "flooding area rejected: the geometry carries no 3D face".to_string(),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let key: Vec<AttributeValue> = self
            .group_by
            .iter()
            .map(|attribute| {
                ctx.feature
                    .attributes
                    .get(attribute)
                    .cloned()
                    .unwrap_or(AttributeValue::Null)
            })
            .collect();
        let index = *self.group_index.entry(key).or_insert_with(|| {
            self.groups.push(Group {
                representative: ctx.feature.clone(),
                polygons: Vec::new(),
            });
            self.groups.len() - 1
        });
        self.groups[index].polygons.extend(polygons);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        ensure_one_frame(self.groups.iter().flat_map(|group| &group.polygons))?;
        for group in std::mem::take(&mut self.groups) {
            for geometry in generate_surfaces(&group.polygons)? {
                let mut feature = group.representative.clone();
                feature.refresh_id();
                feature.set_geometry(Geometry::Euclidean3D(geometry));
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    FEATURES_PORT.clone(),
                ));
            }
        }
        self.group_index.clear();
        Ok(())
    }

    fn name(&self) -> &str {
        "Flooding Area Surface Generator"
    }
}

/// Append every 3D face of `geometry` to `out`, recursing into collections.
fn collect_polygons(geometry: &Geometry, out: &mut Vec<Polygon3D>) {
    match geometry {
        Geometry::Euclidean3D(g) => collect_polygons_3d(g, out),
        Geometry::GeometryCollection(c) => {
            for member in c.members() {
                collect_polygons(member, out);
            }
        }
        Geometry::None | Geometry::Euclidean2D(_) => {}
    }
}

fn collect_polygons_3d(geometry: &Euclidean3DGeometry, out: &mut Vec<Polygon3D>) {
    match geometry {
        Euclidean3DGeometry::Polygon(polygon) => out.push((**polygon).clone()),
        Euclidean3DGeometry::Collection(collection) => {
            for member in collection.members() {
                collect_polygons_3d(member, out);
            }
        }
        _ => {}
    }
}

/// One surface per connected area of `polygons`, each a collection of
/// triangles in the polygons' frame. The polygons must share one frame.
fn generate_surfaces(polygons: &[Polygon3D]) -> Result<Vec<Euclidean3DGeometry>, BoxedError> {
    let Some(frame) = polygons.first().map(|p| p.frame().clone()) else {
        return Ok(Vec::new());
    };
    let frame_2d = frame
        .demote_to_2d()
        .map_err(|e| Plateau6ProcessorError::FloodingAreaSurfaceGenerator(e.to_string()))?;
    let flat: Vec<Polygon2D> = polygons
        .iter()
        .map(|p| {
            let mut rings = polygon3d_rings(p).map(|ring| ring.iter().map(|c| [c[0], c[1]]));
            let exterior = rings.next().into_iter().flatten();
            Polygon2D::from_rings(frame_2d.clone(), exterior, rings)
        })
        .collect();
    let leaves: Vec<Leaf2D> = flat.iter().map(Leaf2D::Polygon).collect();
    let areas = dissolve_leaves(&leaves, SNAP_TOLERANCE)
        .map_err(|e| Plateau6ProcessorError::FloodingAreaSurfaceGenerator(e.to_string()))?;

    let surface = OriginalSurface::new(polygons);
    let points = thin(surface.grid_points(POINT_SPACING), THINNING_INTERVAL);

    let mut surfaces = Vec::new();
    for area in areas {
        let inside = points_inside_shrunk(&area, &points)?;
        let inside = if min_rect_shorter_side(&inside) > MIN_POINT_SPREAD {
            inside
        } else {
            Vec::new()
        };
        let triangles = triangulate(&area, &inside, &surface);
        if triangles.is_empty() {
            continue;
        }
        surfaces.push(Euclidean3DGeometry::Collection(Collection3D::new(
            triangles.into_iter().map(|[a, b, c]| {
                Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                    frame.clone(),
                    [a, b, c, a],
                    std::iter::empty::<Vec<[f64; 3]>>(),
                )))
            }),
        )));
    }
    Ok(surfaces)
}

/// Fails the node unless every polygon is in the same coordinate frame.
fn ensure_one_frame<'a>(
    mut polygons: impl Iterator<Item = &'a Polygon3D>,
) -> Result<(), BoxedError> {
    let Some(first) = polygons.next() else {
        return Ok(());
    };
    if polygons.all(|p| p.frame() == first.frame()) {
        return Ok(());
    }
    Err(Box::new(Diagnostic::from_draft(
        DiagnosticDraft::new(ErrorCode::GeometryMixedCoordinateFrames)
            .with_severity(Severity::Fatal)
            .with_message("the input is not in one coordinate frame; reproject it upstream"),
        None,
        None,
        None,
    )))
}

/// Every `interval`-th point, starting with the first.
fn thin(points: Vec<[f64; 3]>, interval: usize) -> Vec<[f64; 3]> {
    points.into_iter().step_by(interval.max(1)).collect()
}

/// The points lying inside `area` once it is shrunk by [`SHRINK_DISTANCE`];
/// a point on the shrunk outline counts as inside.
fn points_inside_shrunk(
    area: &Polygon2D,
    points: &[[f64; 3]],
) -> Result<Vec<[f64; 3]>, BoxedError> {
    let shrunk = buffer_2d(
        &Euclidean2DGeometry::Polygon(Box::new(area.clone())),
        &BufferStyle::new(-SHRINK_DISTANCE),
    )
    .map_err(|e| Plateau6ProcessorError::FloodingAreaSurfaceGenerator(e.to_string()))?;
    let shrunk: Vec<Euclidean2DGeometry> = shrunk
        .into_iter()
        .map(|p| Euclidean2DGeometry::Polygon(Box::new(p)))
        .collect();
    Ok(points
        .iter()
        .filter(|p| {
            shrunk
                .iter()
                .any(|s| point_position_2d([p[0], p[1]], s) != CoordPos::Outside)
        })
        .copied()
        .collect())
}

/// The shorter side of the smallest-area rectangle, at any rotation, enclosing
/// `points`; zero when they are fewer than three or collinear.
fn min_rect_shorter_side(points: &[[f64; 3]]) -> f64 {
    let hull = convex_hull(points.iter().map(|p| [p[0], p[1]]).collect());
    if hull.len() < 3 {
        return 0.0;
    }
    let mut best: Option<(f64, f64)> = None;
    for i in 0..hull.len() {
        let a = hull[i];
        let b = hull[(i + 1) % hull.len()];
        let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        if length == 0.0 {
            continue;
        }
        let u = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
        let (mut min_u, mut max_u, mut max_v) = (f64::MAX, f64::MIN, 0.0_f64);
        for p in &hull {
            let d = [p[0] - a[0], p[1] - a[1]];
            let along = d[0] * u[0] + d[1] * u[1];
            let across = (d[0] * u[1] - d[1] * u[0]).abs();
            min_u = min_u.min(along);
            max_u = max_u.max(along);
            max_v = max_v.max(across);
        }
        let (w, h) = (max_u - min_u, max_v);
        if best.is_none_or(|(area, _)| w * h < area) {
            best = Some((w * h, w.min(h)));
        }
    }
    best.map_or(0.0, |(_, shorter)| shorter)
}

/// The convex hull in counter-clockwise order, without collinear points.
fn convex_hull(mut points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();
    if points.len() < 3 {
        return points;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut hull: Vec<[f64; 2]> = Vec::with_capacity(points.len() * 2);
    let reversed: Vec<[f64; 2]> = points.iter().rev().copied().collect();
    for pass in [points, reversed] {
        let start = hull.len();
        for p in pass {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
    }
    hull
}

/// A constrained Delaunay triangulation of `area` with `points` added inside,
/// keeping the triangles whose centroid is inside the area and that have no
/// two corners closer than [`DUPLICATE_VERTEX_TOLERANCE`]. Triangles are wound
/// counter-clockwise seen from above.
fn triangulate(
    area: &Polygon2D,
    points: &[[f64; 3]],
    surface: &OriginalSurface,
) -> Vec<[[f64; 3]; 3]> {
    let mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
        ConstrainedDelaunayTriangulation::new();
    // spade vertex storage index -> elevation; a repeated position keeps the
    // elevation it was first inserted with.
    let mut elevations: Vec<f64> = Vec::new();
    let mut insert = |cdt: &mut ConstrainedDelaunayTriangulation<Point2<f64>>, p: [f64; 3]| {
        let handle = cdt.insert(Point2::new(p[0], p[1])).ok()?;
        if handle.index() >= elevations.len() {
            elevations.resize(handle.index() + 1, p[2]);
        }
        Some(handle)
    };

    for ring in polygon2d_rings(area) {
        let ring = surface.with_vertices_on_edges(open_ring(ring));
        let handles: Vec<_> = ring
            .iter()
            .filter_map(|c| insert(&mut cdt, [c[0], c[1], surface.elevation_at(*c)]))
            .collect();
        for i in 0..handles.len() {
            let (a, b) = (handles[i], handles[(i + 1) % handles.len()]);
            if a != b && cdt.can_add_constraint(a, b) {
                cdt.add_constraint(a, b);
            }
        }
    }
    for p in points {
        insert(&mut cdt, *p);
    }

    let area = Euclidean2DGeometry::Polygon(Box::new(area.clone()));
    let mut triangles = Vec::new();
    for face in cdt.inner_faces() {
        let mut corners = face.vertices().map(|v| {
            let p = v.position();
            [p.x, p.y, elevations[v.fix().index()]]
        });
        let centroid = [
            (corners[0][0] + corners[1][0] + corners[2][0]) / 3.0,
            (corners[0][1] + corners[1][1] + corners[2][1]) / 3.0,
        ];
        if point_position_2d(centroid, &area) == CoordPos::Outside || has_close_corners(&corners) {
            continue;
        }
        if orientation(&corners) == Orientation::Clockwise {
            corners.swap(1, 2);
        }
        triangles.push(corners);
    }
    triangles
}

fn has_close_corners(corners: &[[f64; 3]; 3]) -> bool {
    (0..3).any(|i| {
        let (a, b) = (corners[i], corners[(i + 1) % 3]);
        let d2 = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
        d2 < DUPLICATE_VERTEX_TOLERANCE * DUPLICATE_VERTEX_TOLERANCE
    })
}

/// The winding of the triangle seen from above.
fn orientation(t: &[[f64; 3]; 3]) -> Orientation {
    orient2d([t[0][0], t[0][1]], [t[1][0], t[1][1]], [t[2][0], t[2][1]])
}

/// The input faces, split into triangles, for looking up the elevation under a
/// point.
struct OriginalSurface {
    triangles: Vec<[[f64; 3]; 3]>,
    /// Triangles by the index cells their bounding box covers.
    cells: HashMap<(i64, i64), Vec<usize>>,
    /// Input vertices by snapped position, first occurrence only.
    vertices: HashMap<(i64, i64), [f64; 3]>,
    /// The same vertices by the index cell they lie in.
    vertex_cells: HashMap<(i64, i64), Vec<[f64; 2]>>,
}

impl OriginalSurface {
    fn new(polygons: &[Polygon3D]) -> Self {
        let mut surface = Self {
            triangles: Vec::new(),
            cells: HashMap::new(),
            vertices: HashMap::new(),
            vertex_cells: HashMap::new(),
        };
        for polygon in polygons {
            for v in polygon3d_rings(polygon).flatten() {
                surface.vertices.entry(snap_key(v[0], v[1])).or_insert(*v);
            }
            // The faces are triangles in practice; a larger face is fanned, which
            // is exact for the planar faces the specification allows.
            let exterior = open_ring(polygon.exterior());
            for i in 1..exterior.len().saturating_sub(1) {
                surface.add_triangle([exterior[0], exterior[i], exterior[i + 1]]);
            }
        }
        for v in surface.vertices.values() {
            surface
                .vertex_cells
                .entry((cell(v[0]), cell(v[1])))
                .or_default()
                .push([v[0], v[1]]);
        }
        surface
    }

    fn add_triangle(&mut self, t: [[f64; 3]; 3]) {
        let index = self.triangles.len();
        let (min, max) = bounds(&t);
        for cx in cell(min[0])..=cell(max[0]) {
            for cy in cell(min[1])..=cell(max[1]) {
                self.cells.entry((cx, cy)).or_default().push(index);
            }
        }
        self.triangles.push(t);
    }

    /// `ring` with the input vertices lying on its edges put back in order.
    /// Dissolving drops the vertices along a straight stretch of the outline,
    /// and their elevations with them.
    fn with_vertices_on_edges(&self, ring: &[[f64; 2]]) -> Vec<[f64; 2]> {
        let mut out = Vec::with_capacity(ring.len());
        for (i, &a) in ring.iter().enumerate() {
            out.push(a);
            out.extend(self.vertices_on_segment(a, ring[(i + 1) % ring.len()]));
        }
        out
    }

    /// The input vertices within [`SNAP_TOLERANCE`] of the segment `a`-`b`, ends
    /// excluded, ordered from `a`.
    fn vertices_on_segment(&self, a: [f64; 2], b: [f64; 2]) -> Vec<[f64; 2]> {
        let d = [b[0] - a[0], b[1] - a[1]];
        let length = d[0].hypot(d[1]);
        if length <= 2.0 * SNAP_TOLERANCE {
            return Vec::new();
        }
        let mut found: Vec<(f64, [f64; 2])> = Vec::new();
        let (min_x, max_x) = (
            a[0].min(b[0]) - SNAP_TOLERANCE,
            a[0].max(b[0]) + SNAP_TOLERANCE,
        );
        let (min_y, max_y) = (
            a[1].min(b[1]) - SNAP_TOLERANCE,
            a[1].max(b[1]) + SNAP_TOLERANCE,
        );
        for cx in cell(min_x)..=cell(max_x) {
            for cy in cell(min_y)..=cell(max_y) {
                for &v in self.vertex_cells.get(&(cx, cy)).into_iter().flatten() {
                    // distance along the segment from `a`, and off it
                    let along = ((v[0] - a[0]) * d[0] + (v[1] - a[1]) * d[1]) / length;
                    let off = ((v[1] - a[1]) * d[0] - (v[0] - a[0]) * d[1]) / length;
                    if along > SNAP_TOLERANCE
                        && along < length - SNAP_TOLERANCE
                        && off.abs() <= SNAP_TOLERANCE
                    {
                        found.push((along, v));
                    }
                }
            }
        }
        found.sort_by(|x, y| x.0.total_cmp(&y.0));
        found.into_iter().map(|(_, v)| v).collect()
    }

    /// The elevation at `p`: an input vertex's own when `p` is one, otherwise
    /// the plane of the first triangle containing it, otherwise the nearest
    /// vertex of a nearby triangle.
    fn elevation_at(&self, p: [f64; 2]) -> f64 {
        let (kx, ky) = snap_key(p[0], p[1]);
        let mut best: Option<(f64, f64)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(v) = self.vertices.get(&(kx + dx, ky + dy)) {
                    let d2 = (v[0] - p[0]).powi(2) + (v[1] - p[1]).powi(2);
                    if d2 <= SNAP_TOLERANCE * SNAP_TOLERANCE
                        && best.is_none_or(|(best_d2, _)| d2 < best_d2)
                    {
                        best = Some((d2, v[2]));
                    }
                }
            }
        }
        if let Some((_, z)) = best {
            return z;
        }
        self.interpolate(p)
            .or_else(|| self.nearest_vertex_elevation(p))
            .unwrap_or(0.0)
    }

    /// The elevation on the plane of the first triangle containing `p`.
    fn interpolate(&self, p: [f64; 2]) -> Option<f64> {
        let candidates = self.cells.get(&(cell(p[0]), cell(p[1])))?;
        candidates
            .iter()
            .find_map(|&i| plane_elevation(&self.triangles[i], p))
    }

    fn nearest_vertex_elevation(&self, p: [f64; 2]) -> Option<f64> {
        let (cx, cy) = (cell(p[0]), cell(p[1]));
        (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dy| (cx + dx, cy + dy)))
            .filter_map(|c| self.cells.get(&c))
            .flatten()
            .flat_map(|&i| self.triangles[i])
            .min_by(|a, b| {
                let da = (a[0] - p[0]).powi(2) + (a[1] - p[1]).powi(2);
                let db = (b[0] - p[0]).powi(2) + (b[1] - p[1]).powi(2);
                da.total_cmp(&db)
            })
            .map(|v| v[2])
    }

    /// The nodes of a `spacing` grid aligned to the frame origin that lie on the
    /// surface, each at the elevation of the first triangle containing it, in
    /// row-major order.
    fn grid_points(&self, spacing: f64) -> Vec<[f64; 3]> {
        let mut nodes: HashMap<(i64, i64), f64> = HashMap::new();
        for t in &self.triangles {
            let (min, max) = bounds(t);
            for i in (min[0] / spacing).ceil() as i64..=(max[0] / spacing).floor() as i64 {
                for j in (min[1] / spacing).ceil() as i64..=(max[1] / spacing).floor() as i64 {
                    if nodes.contains_key(&(i, j)) {
                        continue;
                    }
                    if let Some(z) = plane_elevation(t, [i as f64 * spacing, j as f64 * spacing]) {
                        nodes.insert((i, j), z);
                    }
                }
            }
        }
        let mut keys: Vec<(i64, i64)> = nodes.keys().copied().collect();
        keys.sort_by_key(|&(i, j)| (j, i));
        keys.into_iter()
            .map(|(i, j)| [i as f64 * spacing, j as f64 * spacing, nodes[&(i, j)]])
            .collect()
    }
}

fn snap_key(x: f64, y: f64) -> (i64, i64) {
    (
        (x / SNAP_TOLERANCE).round() as i64,
        (y / SNAP_TOLERANCE).round() as i64,
    )
}

fn cell(v: f64) -> i64 {
    (v / INDEX_CELL_SIZE).floor() as i64
}

fn bounds(t: &[[f64; 3]; 3]) -> ([f64; 2], [f64; 2]) {
    let min = [
        t[0][0].min(t[1][0]).min(t[2][0]),
        t[0][1].min(t[1][1]).min(t[2][1]),
    ];
    let max = [
        t[0][0].max(t[1][0]).max(t[2][0]),
        t[0][1].max(t[1][1]).max(t[2][1]),
    ];
    (min, max)
}

/// The elevation at `p` on the triangle's plane, or `None` when `p` lies outside
/// the triangle's 2D projection or the triangle is degenerate.
fn plane_elevation(t: &[[f64; 3]; 3], p: [f64; 2]) -> Option<f64> {
    let det = (t[1][1] - t[2][1]) * (t[0][0] - t[2][0]) + (t[2][0] - t[1][0]) * (t[0][1] - t[2][1]);
    if det == 0.0 {
        return None;
    }
    let w0 =
        ((t[1][1] - t[2][1]) * (p[0] - t[2][0]) + (t[2][0] - t[1][0]) * (p[1] - t[2][1])) / det;
    let w1 =
        ((t[2][1] - t[0][1]) * (p[0] - t[2][0]) + (t[0][0] - t[2][0]) * (p[1] - t[2][1])) / det;
    let w2 = 1.0 - w0 - w1;
    const EPS: f64 = -1e-12;
    (w0 >= EPS && w1 >= EPS && w2 >= EPS).then(|| w0 * t[0][2] + w1 * t[1][2] + w2 * t[2][2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::polygon::signed_area_2d;

    fn triangle_in(frame: CoordinateFrame, a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Polygon3D {
        Polygon3D::from_rings(frame, [a, b, c, a], std::iter::empty::<Vec<[f64; 3]>>())
    }

    fn triangle(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Polygon3D {
        triangle_in(CoordinateFrame::Crs(EpsgCode::new(3857)), a, b, c)
    }

    /// Two triangles making the square `[x0, x0 + size] x [0, size]` at elevation `z`.
    fn square(x0: f64, size: f64, z: f64) -> [Polygon3D; 2] {
        let (x1, y1) = (x0 + size, size);
        [
            triangle([x0, 0.0, z], [x1, 0.0, z], [x1, y1, z]),
            triangle([x0, 0.0, z], [x1, y1, z], [x0, y1, z]),
        ]
    }

    fn triangles_of(geometry: &Euclidean3DGeometry) -> Vec<[[f64; 3]; 3]> {
        let mut polygons = Vec::new();
        collect_polygons_3d(geometry, &mut polygons);
        polygons
            .iter()
            .map(|p| {
                let r = p.exterior();
                [r[0], r[1], r[2]]
            })
            .collect()
    }

    fn total_area(triangles: &[[[f64; 3]; 3]]) -> f64 {
        triangles
            .iter()
            .map(|t| signed_area_2d(&t.map(|v| [v[0], v[1]])) / 2.0)
            .sum()
    }

    #[test]
    fn adjoining_bodies_become_one_surface() {
        let polygons: Vec<Polygon3D> = square(0.0, 10.0, 1.0)
            .into_iter()
            .chain(square(10.0, 10.0, 1.0))
            .collect();
        let surfaces = generate_surfaces(&polygons).unwrap();
        assert_eq!(surfaces.len(), 1);
        let triangles = triangles_of(&surfaces[0]);
        assert!((total_area(&triangles) - 200.0).abs() < 1e-6);
        assert!(
            triangles
                .iter()
                .all(|t| orientation(t) == Orientation::CounterClockwise)
        );
        assert!(triangles.iter().flatten().all(|v| v[2] == 1.0));
    }

    #[test]
    fn outline_keeps_the_vertices_along_a_straight_edge() {
        // (10, 0) lies on the straight bottom edge, so dissolving drops it
        let (a, m, b, c, d) = (
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 4.0],
            [20.0, 0.0, 0.0],
            [20.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        );
        let polygons = vec![triangle(a, m, d), triangle(m, c, d), triangle(m, b, c)];
        let surfaces = generate_surfaces(&polygons).unwrap();
        assert_eq!(surfaces.len(), 1);
        let triangles = triangles_of(&surfaces[0]);
        assert!(triangles.iter().flatten().any(|v| *v == m));
        assert!((total_area(&triangles) - 200.0).abs() < 1e-6);
    }

    #[test]
    fn separate_bodies_become_separate_surfaces() {
        let polygons: Vec<Polygon3D> = square(0.0, 10.0, 1.0)
            .into_iter()
            .chain(square(100.0, 10.0, 2.0))
            .collect();
        assert_eq!(generate_surfaces(&polygons).unwrap().len(), 2);
    }

    #[test]
    fn inside_points_take_the_original_elevation() {
        // a slope rising with x, 30 grid nodes a row so the thinned nodes do not
        // fall on one line
        let (a, b, c, d) = (
            [0.0, 0.0, 0.0],
            [1450.0, 0.0, 145.0],
            [1450.0, 1450.0, 145.0],
            [0.0, 1450.0, 0.0],
        );
        let polygons = vec![triangle(a, b, c), triangle(a, c, d)];
        let surfaces = generate_surfaces(&polygons).unwrap();
        assert_eq!(surfaces.len(), 1);
        let triangles = triangles_of(&surfaces[0]);
        let inner = triangles
            .iter()
            .flatten()
            .find(|v| v[0] > 0.0 && v[0] < 1000.0 && v[1] > 0.0 && v[1] < 1000.0)
            .expect("an inside point");
        assert!((inner[2] - inner[0] / 10.0).abs() < 1e-9);
        assert!((total_area(&triangles) - 1450.0 * 1450.0).abs() < 1e-3);
    }

    #[test]
    fn mixed_coordinate_frames_are_an_error() {
        let web_mercator = CoordinateFrame::Crs(EpsgCode::new(3857));
        let plane_rectangular = CoordinateFrame::Crs(EpsgCode::new(6677));
        let polygons = [
            triangle_in(
                web_mercator,
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [0.0, 10.0, 0.0],
            ),
            triangle_in(
                plane_rectangular,
                [10.0, 0.0, 0.0],
                [20.0, 0.0, 0.0],
                [20.0, 10.0, 0.0],
            ),
        ];
        assert!(ensure_one_frame(polygons[..1].iter()).is_ok());
        let error = ensure_one_frame(polygons.iter()).unwrap_err();
        let diagnostic = error
            .downcast_ref::<Diagnostic>()
            .expect("a registry-coded diagnostic");
        assert_eq!(diagnostic.code, ErrorCode::GeometryMixedCoordinateFrames);
    }

    #[test]
    fn collinear_points_have_no_spread() {
        let points = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [20.0, 0.0, 0.0]];
        assert_eq!(min_rect_shorter_side(&points), 0.0);
        let points = [
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 6.0, 0.0],
            [0.0, 6.0, 0.0],
        ];
        assert!((min_rect_shorter_side(&points) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn thinning_keeps_every_nth_from_the_first() {
        let points: Vec<[f64; 3]> = (0..45).map(|i| [i as f64, 0.0, 0.0]).collect();
        let kept: Vec<f64> = thin(points, 20).iter().map(|p| p[0]).collect();
        assert_eq!(kept, vec![0.0, 20.0, 40.0]);
    }
}
