use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use nusamai_citygml::GmlGeometryType;
use once_cell::sync::Lazy;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{polygon_mesh::PolygonMesh3D, Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use serde_json::Value;

use super::PlateauProfile;

static PASSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("passed"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

#[derive(Debug, Clone)]
pub(crate) struct CompositeSurfaceContinuityFilterFactory {
    name: String,
}

impl CompositeSurfaceContinuityFilterFactory {
    pub(crate) fn new(profile: &PlateauProfile) -> Self {
        Self {
            name: profile.action_name("CompositeSurfaceContinuityFilter"),
        }
    }
}

impl ProcessorFactory for CompositeSurfaceContinuityFilterFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Checks whether each CompositeSurface is continuous, that is, whether all of its faces \
         are joined to each other through shared edges. A feature whose CompositeSurfaces are \
         all continuous goes to `passed`. Each CompositeSurface that falls apart into several \
         groups of faces goes to `failed` as its own feature, with that CompositeSurface as the \
         geometry and the number of groups in `discontinuous_parts_count`. A feature without a \
         CompositeSurface goes to `rejected`."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
    #[cfg(not(feature = "new-geometry"))]
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

                // Only check CompositeSurface (NOT MultiSurface)
                // MultiSurface does not require surface continuity per CityGML spec
                let is_composite_surface = geom
                    .gml_trait
                    .as_ref()
                    .map(|t| matches!(t.gml_geometry_type, GmlGeometryType::CompositeSurface))
                    .unwrap_or(false);

                if geom.ty.name() != "Surface" || !is_composite_surface {
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

    /// Faces are joined only where they share both pooled vertices of an edge,
    /// which the reader welds by exact coordinates.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let mut meshes = Vec::new();
        collect_meshes(&feature.geometry, &mut meshes);
        if meshes.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let mut continuous = true;
        for mesh in meshes {
            let num_components = mesh.connected_component_count();
            if num_components <= 1 {
                continue;
            }
            continuous = false;
            let mut failed = feature.clone();
            failed.set_geometry(Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(
                Box::new(mesh.clone()),
            )));
            failed.attributes_mut().insert(
                Attribute::new("discontinuous_parts_count"),
                AttributeValue::Number(serde_json::Number::from(num_components)),
            );
            fw.send(ctx.new_with_feature_and_port(failed, FAILED_PORT.clone()));
        }
        if continuous {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), PASSED_PORT.clone()));
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
        "CompositeSurfaceContinuityFilter"
    }
}

/// Collect every CompositeSurface, which the reader turns into a polygon mesh,
/// recursing into collections.
#[cfg(feature = "new-geometry")]
fn collect_meshes<'a>(geometry: &'a Geometry, meshes: &mut Vec<&'a PolygonMesh3D>) {
    match geometry {
        Geometry::Euclidean3D(geometry) => collect_meshes_3d(geometry, meshes),
        Geometry::GeometryCollection(collection) => {
            for member in collection.members() {
                collect_meshes(member, meshes);
            }
        }
        Geometry::None | Geometry::Euclidean2D(_) => {}
    }
}

#[cfg(feature = "new-geometry")]
fn collect_meshes_3d<'a>(geometry: &'a Euclidean3DGeometry, meshes: &mut Vec<&'a PolygonMesh3D>) {
    match geometry {
        Euclidean3DGeometry::PolygonMesh(mesh) => meshes.push(mesh),
        Euclidean3DGeometry::Collection(collection) => {
            for member in collection.members() {
                collect_meshes_3d(member, meshes);
            }
        }
        _ => {}
    }
}

#[cfg(not(feature = "new-geometry"))]
type QuantizedEdge = ([i64; 3], [i64; 3]);

#[cfg(not(feature = "new-geometry"))]
fn quantize(x: f64, y: f64, z: f64) -> [i64; 3] {
    const SCALE: f64 = 1e6;
    [
        (x * SCALE).round() as i64,
        (y * SCALE).round() as i64,
        (z * SCALE).round() as i64,
    ]
}

#[cfg(not(feature = "new-geometry"))]
fn make_edge(a: [i64; 3], b: [i64; 3]) -> QuantizedEdge {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(not(feature = "new-geometry"))]
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

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use reearth_flow_geometry::collection::Collection3D;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    /// A unit square in the z = 0 plane with its lower-left corner at `(x, y)`.
    fn square(x: f64, y: f64) -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [x, y, 0.0],
                [x + 1.0, y, 0.0],
                [x + 1.0, y + 1.0, 0.0],
                [x, y + 1.0, 0.0],
                [x, y, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    fn composite(squares: &[Polygon3D]) -> Euclidean3DGeometry {
        Euclidean3DGeometry::PolygonMesh(Box::new(
            PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, squares).unwrap(),
        ))
    }

    /// Run one feature through the filter and return what it sent with the port
    /// each went to.
    fn run(geometry: Euclidean3DGeometry) -> Vec<(Port, Feature)> {
        let feature = Feature::new_with_attributes_and_geometry(
            Default::default(),
            Geometry::Euclidean3D(geometry),
        );
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let ctx = crate::tests::utils::create_default_execute_context(feature);
        CompositeSurfaceContinuityFilter.process(ctx, &fw).unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    fn parts_count(feature: &Feature) -> Option<&AttributeValue> {
        feature
            .attributes
            .get(&Attribute::new("discontinuous_parts_count"))
    }

    #[test]
    fn faces_joined_through_shared_edges_pass() {
        let sent = run(Euclidean3DGeometry::Collection(Collection3D::new([
            composite(&[square(0.0, 0.0), square(1.0, 0.0), square(1.0, 1.0)]),
        ])));
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, *PASSED_PORT);
        assert_eq!(parts_count(&sent[0].1), None);
    }

    #[test]
    fn each_split_composite_surface_fails_on_its_own() {
        // Touching only at a corner does not join two faces.
        let split = composite(&[square(0.0, 0.0), square(5.0, 0.0), square(1.0, 1.0)]);
        let sent = run(Euclidean3DGeometry::Collection(Collection3D::new([
            composite(&[square(0.0, 0.0), square(1.0, 0.0)]),
            split.clone(),
            Euclidean3DGeometry::Polygon(Box::new(square(9.0, 9.0))),
        ])));
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, *FAILED_PORT);
        assert_eq!(
            parts_count(&sent[0].1),
            Some(&AttributeValue::Number(serde_json::Number::from(3)))
        );
        assert_eq!(*sent[0].1.geometry, Geometry::Euclidean3D(split));
    }

    #[test]
    fn surfaces_without_composite_surface_are_rejected() {
        let sent = run(Euclidean3DGeometry::Collection(Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(square(0.0, 0.0))),
            Euclidean3DGeometry::Polygon(Box::new(square(5.0, 0.0))),
        ])));
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, *REJECTED_PORT);
    }
}
