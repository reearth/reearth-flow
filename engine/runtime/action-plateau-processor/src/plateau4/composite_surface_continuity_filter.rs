use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use serde_json::Value;

static PASSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("passed"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

#[derive(Debug, Clone, Default)]
pub struct CompositeSurfaceContinuityFilterFactory;

impl ProcessorFactory for CompositeSurfaceContinuityFilterFactory {
    fn name(&self) -> &str {
        "PLATEAU4.CompositeSurfaceContinuityFilter"
    }

    fn description(&self) -> &str {
        "Checks if a CompositeSurface is continuous (all parts share edges)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            PASSED_PORT.clone(),
            FAILED_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(CompositeSurfaceContinuityFilter))
    }
}

#[derive(Debug, Clone)]
pub struct CompositeSurfaceContinuityFilter;

impl Processor for CompositeSurfaceContinuityFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        match &geometry.value {
            GeometryValue::CityGmlGeometry(gml_geom) => {
                let Some(geom) = gml_geom.gml_geometries.first() else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                // Only check Surface type (CompositeSurface/MultiSurface)
                if geom.ty.name() != "Surface" {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }

                let polygons = &geom.polygons;
                if polygons.len() <= 1 {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), PASSED_PORT.clone()));
                    return Ok(());
                }

                let num_components = count_connected_components(polygons);
                if num_components <= 1 {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), PASSED_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.attributes_mut().insert(
                        Attribute::new("discontinuous_parts_count"),
                        AttributeValue::Number(serde_json::Number::from(num_components)),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }

        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "PLATEAU4.CompositeSurfaceContinuityFilter"
    }
}

type QuantizedEdge = ([i64; 3], [i64; 3]);

fn quantize(x: f64, y: f64, z: f64) -> [i64; 3] {
    const SCALE: f64 = 1e6;
    [
        (x * SCALE).round() as i64,
        (y * SCALE).round() as i64,
        (z * SCALE).round() as i64,
    ]
}

fn make_edge(a: [i64; 3], b: [i64; 3]) -> QuantizedEdge {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn count_connected_components(
    polygons: &[reearth_flow_geometry::types::polygon::Polygon3D<f64>],
) -> usize {
    let n = polygons.len();
    if n == 0 {
        return 0;
    }

    let polygon_edges: Vec<Vec<QuantizedEdge>> = polygons
        .iter()
        .map(|poly| {
            let mut edges = Vec::new();
            let exterior = poly.exterior();
            let coords: Vec<_> = exterior.iter().collect();
            for pair in coords.windows(2) {
                let a = quantize(pair[0].x, pair[0].y, pair[0].z);
                let b = quantize(pair[1].x, pair[1].y, pair[1].z);
                if a != b {
                    edges.push(make_edge(a, b));
                }
            }
            for interior in poly.interiors() {
                let coords: Vec<_> = interior.iter().collect();
                for pair in coords.windows(2) {
                    let a = quantize(pair[0].x, pair[0].y, pair[0].z);
                    let b = quantize(pair[1].x, pair[1].y, pair[1].z);
                    if a != b {
                        edges.push(make_edge(a, b));
                    }
                }
            }
            edges
        })
        .collect();

    let mut edge_to_polygons: HashMap<QuantizedEdge, Vec<usize>> = HashMap::new();
    for (poly_idx, edges) in polygon_edges.iter().enumerate() {
        for edge in edges {
            edge_to_polygons.entry(*edge).or_default().push(poly_idx);
        }
    }

    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    for poly_indices in edge_to_polygons.values() {
        for i in 0..poly_indices.len() {
            for j in (i + 1)..poly_indices.len() {
                let a = poly_indices[i];
                let b = poly_indices[j];
                adjacency[a].push(b);
                adjacency[b].push(a);
            }
        }
    }

    let mut visited = vec![false; n];
    let mut components = 0;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        visited[start] = true;
        while let Some(node) = queue.pop_front() {
            for &neighbor in &adjacency[node] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    components
}
