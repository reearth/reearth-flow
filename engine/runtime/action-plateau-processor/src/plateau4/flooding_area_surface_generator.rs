//! Flood-zone surface generation for PLATEAU flood risk models.
//!
//! A flood zone arrives as a patchwork of small triangles, one body per rank and
//! file. Each group of bodies (one folder, rank and scale) is dissolved into its
//! connected areas, and every area becomes one feature carrying the original
//! triangles that fall in it. The triangles themselves are kept as they are.
//!
//! The areas are found in EPSG:3857, the frame the original workspace dissolves
//! in, so the snapping tolerance is in its units.

use std::collections::HashMap;
use std::sync::Arc;

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ContourDirection;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::core::solver::Solver;
use i_overlay::float::overlay::{FloatOverlay, OverlayOptions};
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{
    Attribute, AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryType, GeometryValue,
    GmlGeometry,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;

/// Outline vertices of neighboring bodies that miss by less than this (EPSG:3857
/// units) are treated as one when dissolving.
const SNAP_TOLERANCE: f64 = 0.001;
/// Radius of the sphere EPSG:3857 projects from.
const WEB_MERCATOR_RADIUS: f64 = 6_378_137.0;

#[derive(Debug, Clone, Default)]
pub struct FloodingAreaSurfaceGeneratorFactory;

impl ProcessorFactory for FloodingAreaSurfaceGeneratorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.FloodingAreaSurfaceGenerator"
    }

    fn description(&self) -> &str {
        "Merges the triangles of flood zone bodies into connected areas and emits one feature \
         per area, carrying the original triangles of the area and the attributes of the first \
         body in its group."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FloodingAreaSurfaceGeneratorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
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
                    PlateauProcessorError::FloodingAreaSurfaceGeneratorFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    PlateauProcessorError::FloodingAreaSurfaceGeneratorFactory(format!(
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

/// # FloodingAreaSurfaceGenerator Parameters
///
/// Which bodies are merged together.
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
    /// The first body, whose attributes and geometry metadata every output
    /// feature of the group takes.
    representative: Feature,
    /// The triangles of every body, in longitude/latitude/height.
    triangles: Vec<Polygon3D<f64>>,
}

#[derive(Debug)]
pub struct FloodingAreaSurfaceGenerator {
    group_by: Vec<Attribute>,
    groups: Vec<Group>,
    group_index: HashMap<Vec<AttributeValue>, usize>,
}

impl Clone for FloodingAreaSurfaceGenerator {
    /// The accumulated state belongs to one run, so a clone starts empty.
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
        let triangles = collect_polygons(&ctx.feature.geometry.value);
        if triangles.is_empty() {
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
                triangles: Vec::new(),
            });
            self.groups.len() - 1
        });
        self.groups[index].triangles.extend(triangles);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for group in std::mem::take(&mut self.groups) {
            for triangles in merge_into_areas(&group.triangles) {
                let feature = area_feature(&group.representative, triangles);
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    DEFAULT_PORT.clone(),
                ));
            }
        }
        self.group_index.clear();
        Ok(())
    }

    fn name(&self) -> &str {
        "FloodingAreaSurfaceGenerator"
    }
}

/// The surface polygons of `geometry`.
fn collect_polygons(geometry: &GeometryValue) -> Vec<Polygon3D<f64>> {
    match geometry {
        GeometryValue::CityGmlGeometry(city_gml) => city_gml
            .gml_geometries
            .iter()
            .filter(|g| {
                matches!(
                    g.ty,
                    GeometryType::Surface | GeometryType::Triangle | GeometryType::Solid
                )
            })
            .flat_map(|g| g.polygons.iter().cloned())
            .collect(),
        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)) => vec![polygon.clone()],
        GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(polygons)) => polygons.0.clone(),
        _ => Vec::new(),
    }
}

/// `representative` with its geometry replaced by `polygons`, untextured.
fn area_feature(representative: &Feature, polygons: Vec<Polygon3D<f64>>) -> Feature {
    let source = match &representative.geometry.value {
        GeometryValue::CityGmlGeometry(city_gml) => city_gml.gml_geometries.first(),
        _ => None,
    };
    let mut gml = GmlGeometry::new(GeometryType::Surface, source.and_then(|g| g.lod));
    if let Some(source) = source {
        gml.gml_trait = source.gml_trait.clone();
        gml.feature_id = source.feature_id.clone();
        gml.feature_type = source.feature_type.clone();
    }
    let uvs = polygons.iter().map(blank_uv).collect();
    gml.len = polygons.len() as u32;
    let count = polygons.len();
    gml.polygons = polygons;
    let mut city_gml = CityGmlGeometry::new(vec![gml], Vec::new(), Vec::new());
    city_gml.polygon_materials = vec![None; count];
    city_gml.polygon_textures = vec![None; count];
    city_gml.polygon_uvs = MultiPolygon2D::new(uvs);

    let mut feature = representative.clone();
    feature.refresh_id();
    feature.geometry = Arc::new(Geometry {
        epsg: representative.geometry.epsg,
        value: GeometryValue::CityGmlGeometry(city_gml),
    });
    feature
}

/// A zero texture coordinate per vertex of `polygon`.
fn blank_uv(polygon: &Polygon3D<f64>) -> Polygon2D<f64> {
    let ring = |n: usize| LineString2D::new(vec![Coordinate2D::new_(0.0, 0.0); n]);
    Polygon2D::new(
        ring(polygon.exterior().0.len()),
        polygon
            .interiors()
            .iter()
            .map(|r| ring(r.0.len()))
            .collect(),
    )
}

/// The triangles of each connected area of `triangles`, in their original form.
/// An area is a polygon of the dissolved union; a triangle belongs to the area
/// its centroid lies in.
fn merge_into_areas(triangles: &[Polygon3D<f64>]) -> Vec<Vec<Polygon3D<f64>>> {
    let projected: Vec<Vec<[f64; 2]>> = triangles.iter().map(project_exterior).collect();
    let areas = dissolve(&projected);
    let components = edge_components(&projected);

    // Every edge-connected component lies in one area, so one point locates it.
    let index = AreaIndex::new(&areas);
    let mut by_area: HashMap<usize, Vec<Polygon3D<f64>>> = HashMap::new();
    let mut unplaced: Vec<Polygon3D<f64>> = Vec::new();
    for component in components {
        let probe = centroid(&projected[component[0]]);
        let members = component.into_iter().map(|i| triangles[i].clone());
        match index.locate(probe) {
            Some(area) => by_area.entry(area).or_default().extend(members),
            None => unplaced.extend(members),
        }
    }
    let mut areas: Vec<(usize, Vec<Polygon3D<f64>>)> = by_area.into_iter().collect();
    areas.sort_by_key(|(area, _)| *area);
    let mut out: Vec<Vec<Polygon3D<f64>>> = areas.into_iter().map(|(_, t)| t).collect();
    if !unplaced.is_empty() {
        out.push(unplaced);
    }
    out
}

/// The exterior ring of a longitude/latitude polygon in EPSG:3857, open.
fn project_exterior(polygon: &Polygon3D<f64>) -> Vec<[f64; 2]> {
    let ring = &polygon.exterior().0;
    let open = match (ring.first(), ring.last()) {
        (Some(a), Some(b)) if ring.len() > 1 && a.x == b.x && a.y == b.y => &ring[..ring.len() - 1],
        _ => &ring[..],
    };
    open.iter().map(|c| to_web_mercator(c.x, c.y)).collect()
}

fn to_web_mercator(lon: f64, lat: f64) -> [f64; 2] {
    let x = WEB_MERCATOR_RADIUS * lon.to_radians();
    let y = WEB_MERCATOR_RADIUS
        * (std::f64::consts::FRAC_PI_4 + lat.to_radians() / 2.0)
            .tan()
            .ln();
    [x, y]
}

fn snap(p: [f64; 2]) -> [f64; 2] {
    [
        (p[0] / SNAP_TOLERANCE).round() * SNAP_TOLERANCE,
        (p[1] / SNAP_TOLERANCE).round() * SNAP_TOLERANCE,
    ]
}

fn snap_key(p: [f64; 2]) -> (i64, i64) {
    (
        (p[0] / SNAP_TOLERANCE).round() as i64,
        (p[1] / SNAP_TOLERANCE).round() as i64,
    )
}

/// One polygon of a dissolved union: an exterior ring and its holes, open.
struct Area {
    rings: Vec<Vec<[f64; 2]>>,
    min: [f64; 2],
    max: [f64; 2],
}

/// The union of `rings`, each an open ring, as disjoint polygons.
fn dissolve(rings: &[Vec<[f64; 2]>]) -> Vec<Area> {
    let shapes: Vec<Vec<Vec<[f64; 2]>>> = rings
        .iter()
        .filter(|r| r.len() >= 3)
        .map(|r| vec![r.iter().copied().map(snap).collect()])
        .collect();
    if shapes.is_empty() {
        return Vec::new();
    }
    let mut options: OverlayOptions<f64> = OverlayOptions::default();
    options.output_direction = ContourDirection::CounterClockwise;
    options.ogc = true;
    let result = FloatOverlay::with_subj_custom(&shapes, options, Solver::AUTO)
        .overlay(OverlayRule::Union, FillRule::NonZero);
    result
        .into_iter()
        .filter(|shape| !shape.is_empty())
        .map(|rings| {
            let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
            for p in &rings[0] {
                min = [min[0].min(p[0]), min[1].min(p[1])];
                max = [max[0].max(p[0]), max[1].max(p[1])];
            }
            Area { rings, min, max }
        })
        .collect()
}

/// An undirected edge between two snapped vertices, smaller end first.
type Edge = ((i64, i64), (i64, i64));

/// Indices of `triangles` grouped by connection through shared edges.
fn edge_components(triangles: &[Vec<[f64; 2]>]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..triangles.len()).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut first_owner: HashMap<Edge, usize> = HashMap::new();
    for (t, ring) in triangles.iter().enumerate() {
        for k in 0..ring.len() {
            let (a, b) = (snap_key(ring[k]), snap_key(ring[(k + 1) % ring.len()]));
            if a == b {
                continue;
            }
            let edge = if a < b { (a, b) } else { (b, a) };
            match first_owner.get(&edge) {
                Some(&other) => {
                    let (ra, rb) = (find(&mut parent, t), find(&mut parent, other));
                    if ra != rb {
                        parent[ra] = rb;
                    }
                }
                None => {
                    first_owner.insert(edge, t);
                }
            }
        }
    }
    let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
    for t in 0..triangles.len() {
        let root = find(&mut parent, t);
        components.entry(root).or_default().push(t);
    }
    let mut components: Vec<Vec<usize>> = components.into_values().collect();
    components.sort_by_key(|c| c[0]);
    components
}

fn centroid(ring: &[[f64; 2]]) -> [f64; 2] {
    let n = ring.len().max(1) as f64;
    let (x, y) = ring
        .iter()
        .fold((0.0, 0.0), |(x, y), p| (x + p[0], y + p[1]));
    [x / n, y / n]
}

/// Finds the area containing a point.
struct AreaIndex<'a> {
    areas: &'a [Area],
}

impl<'a> AreaIndex<'a> {
    fn new(areas: &'a [Area]) -> Self {
        Self { areas }
    }

    /// The index of the area `p` lies in or on, or `None` when it misses every
    /// area.
    fn locate(&self, p: [f64; 2]) -> Option<usize> {
        let candidates = || {
            self.areas.iter().enumerate().filter(|(_, a)| {
                p[0] >= a.min[0] - SNAP_TOLERANCE
                    && p[0] <= a.max[0] + SNAP_TOLERANCE
                    && p[1] >= a.min[1] - SNAP_TOLERANCE
                    && p[1] <= a.max[1] + SNAP_TOLERANCE
            })
        };
        candidates()
            .find(|(_, a)| point_in_rings(p, &a.rings))
            .or_else(|| {
                // A centroid off by rounding lands next to its area; take the
                // nearest outline among the boxes it falls in.
                candidates().min_by(|(_, a), (_, b)| {
                    distance_to_rings(p, &a.rings).total_cmp(&distance_to_rings(p, &b.rings))
                })
            })
            .map(|(i, _)| i)
    }
}

/// Whether `p` is inside the polygon of `rings` (exterior first), by the
/// even-odd rule.
fn point_in_rings(p: [f64; 2], rings: &[Vec<[f64; 2]>]) -> bool {
    let mut inside = false;
    for ring in rings {
        let n = ring.len();
        let mut j = n.wrapping_sub(1);
        for i in 0..n {
            let (a, b) = (ring[i], ring[j]);
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
            j = i;
        }
    }
    inside
}

fn distance_to_rings(p: [f64; 2], rings: &[Vec<[f64; 2]>]) -> f64 {
    rings
        .iter()
        .flat_map(|ring| (0..ring.len()).map(move |i| (ring[i], ring[(i + 1) % ring.len()])))
        .map(|(a, b)| segment_distance(p, a, b))
        .fold(f64::MAX, f64::min)
}

fn segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2).clamp(0.0, 1.0)
    };
    let q = [a[0] + t * d[0], a[1] + t * d[1]];
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::Coordinate3D;
    use reearth_flow_geometry::types::line_string::LineString3D;

    /// A triangle given in EPSG:3857 metres near Tokyo, stored as longitude/latitude.
    fn triangle(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Polygon3D<f64> {
        let geo = |p: [f64; 2]| {
            let (x, y) = (p[0] + 15_560_000.0, p[1] + 4_260_000.0);
            let lon = (x / WEB_MERCATOR_RADIUS).to_degrees();
            let lat = (2.0 * (y / WEB_MERCATOR_RADIUS).exp().atan() - std::f64::consts::FRAC_PI_2)
                .to_degrees();
            Coordinate3D::new__(lon, lat, 1.0)
        };
        Polygon3D::new(
            LineString3D::new(vec![geo(a), geo(b), geo(c), geo(a)]),
            Vec::new(),
        )
    }

    fn square(x0: f64) -> [Polygon3D<f64>; 2] {
        [
            triangle([x0, 0.0], [x0 + 10.0, 0.0], [x0 + 10.0, 10.0]),
            triangle([x0, 0.0], [x0 + 10.0, 10.0], [x0, 10.0]),
        ]
    }

    #[test]
    fn adjoining_triangles_make_one_area() {
        let triangles: Vec<_> = square(0.0).into_iter().chain(square(10.0)).collect();
        let areas = merge_into_areas(&triangles);
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].len(), 4);
    }

    #[test]
    fn separate_triangles_make_separate_areas() {
        let triangles: Vec<_> = square(0.0).into_iter().chain(square(100.0)).collect();
        let areas = merge_into_areas(&triangles);
        assert_eq!(areas.len(), 2);
        assert!(areas.iter().all(|a| a.len() == 2));
    }

    #[test]
    fn a_t_junction_still_joins_the_area() {
        // the right square's left edge is split at (10, 5), so no edge is shared
        let triangles = vec![
            triangle([0.0, 0.0], [10.0, 0.0], [10.0, 10.0]),
            triangle([0.0, 0.0], [10.0, 10.0], [0.0, 10.0]),
            triangle([10.0, 0.0], [20.0, 0.0], [10.0, 5.0]),
            triangle([10.0, 5.0], [20.0, 0.0], [20.0, 10.0]),
            triangle([10.0, 5.0], [20.0, 10.0], [10.0, 10.0]),
        ];
        let areas = merge_into_areas(&triangles);
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].len(), 5);
    }

    #[test]
    fn output_keeps_the_original_triangles() {
        let triangles: Vec<_> = square(0.0).into();
        let areas = merge_into_areas(&triangles);
        assert_eq!(areas[0], triangles);
    }
}
