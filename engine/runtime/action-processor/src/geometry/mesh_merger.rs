use std::collections::HashMap;

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_geometry::ops::merge::{is_mergeable, merge_meshes};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{FEATURES_PORT, Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use crate::accumulation::{AttributeAccumulation, accumulate_attributes};

#[derive(Debug, Clone, Default)]
pub struct MeshMergerFactory;

impl ProcessorFactory for MeshMergerFactory {
    fn name(&self) -> &str {
        "Mesh Merger"
    }

    fn description(&self) -> &str {
        "Merges the polygons and meshes of features that share the same grouping attribute values into one mesh per group, keeping every face as it is. The result is a triangular mesh when every input is one, and a polygon mesh otherwise."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MeshMergerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["aggregation"]
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
        let param: MeshMergerParam = match with {
            Some(with) => {
                let value = serde_json::to_value(with).map_err(|e| {
                    GeometryProcessorError::MeshMergerFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    GeometryProcessorError::MeshMergerFactory(format!(
                        "Failed to deserialize `with` parameter: {e}"
                    ))
                })?
            }
            None => MeshMergerParam::default(),
        };
        Ok(Box::new(MeshMerger {
            group_by: param.group_by,
            attribute_accumulation: param.attribute_accumulation,
            groups: IndexMap::new(),
        }))
    }
}

/// # Mesh Merger Parameters
/// Sets which features are merged together and which of their attributes the merged feature keeps.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeshMergerParam {
    /// # Group By Attributes
    /// Attributes whose values decide which features merge together; features matching on all of
    /// them form one mesh. When omitted, every feature merges into a single mesh.
    #[serde(default)]
    group_by: Option<Vec<Attribute>>,
    /// # Attribute Accumulation
    /// Which attributes the merged feature keeps.
    #[serde(default)]
    attribute_accumulation: AttributeAccumulation,
}

#[derive(Debug, Clone)]
pub struct MeshMerger {
    group_by: Option<Vec<Attribute>>,
    attribute_accumulation: AttributeAccumulation,
    /// Groups in first-seen order, which is the order they are emitted in.
    groups: IndexMap<AttributeValue, Vec<Feature>>,
}

impl MeshMerger {
    fn key(&self, feature: &Feature) -> AttributeValue {
        match &self.group_by {
            Some(group_by) => AttributeValue::Array(
                group_by
                    .iter()
                    .map(|attr| {
                        feature
                            .attributes
                            .get(attr)
                            .cloned()
                            .unwrap_or(AttributeValue::Null)
                    })
                    .collect(),
            ),
            None => AttributeValue::Null,
        }
    }

    fn attributes(&self, features: &[Feature]) -> Attributes {
        accumulate_attributes(
            &self.attribute_accumulation,
            features,
            features.first(),
            self.group_by.as_deref().unwrap_or_default(),
        )
    }
}

impl Processor for MeshMerger {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if !is_mergeable(&ctx.feature.geometry) {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }
        let key = self.key(&ctx.feature);
        self.groups.entry(key).or_default().push(ctx.feature);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let merged: Vec<Result<Feature, String>> = self
            .groups
            .par_iter()
            .map(|(_, features)| {
                merge_meshes(features.iter().map(|f| f.geometry.as_ref()))
                    .map(|geometry| {
                        Feature::new_with_attributes_and_geometry(
                            self.attributes(features),
                            geometry,
                        )
                    })
                    .map_err(|e| e.to_string())
            })
            .collect();
        for (result, (_, features)) in merged.into_iter().zip(std::mem::take(&mut self.groups)) {
            match result {
                Ok(feature) => fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    FEATURES_PORT.clone(),
                )),
                Err(e) => {
                    ctx.event_hub.warn_log(
                        None,
                        format!(
                            "Mesh Merger rejected a group of {} features: {e}",
                            features.len()
                        ),
                    );
                    for feature in features {
                        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                            &ctx,
                            feature,
                            REJECTED_PORT.clone(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Mesh Merger"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    fn triangle(frame: CoordinateFrame, corners: [[f64; 3]; 3], attrs: &[(&str, &str)]) -> Feature {
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame, corners, Vec::<Vec<[f64; 3]>>::new()),
        )));
        let mut feature = Feature::from(geometry);
        for (k, v) in attrs {
            feature.insert(*k, AttributeValue::String(v.to_string()));
        }
        feature
    }

    fn lower(attrs: &[(&str, &str)]) -> Feature {
        triangle(
            CoordinateFrame::Euclidean,
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            attrs,
        )
    }

    fn upper(attrs: &[(&str, &str)]) -> Feature {
        triangle(
            CoordinateFrame::Euclidean,
            [[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
            attrs,
        )
    }

    fn merger(group_by: &[&str], strategy: AttributeAccumulation) -> MeshMerger {
        MeshMerger {
            group_by: (!group_by.is_empty())
                .then(|| group_by.iter().map(|a| Attribute::new(*a)).collect()),
            attribute_accumulation: strategy,
            groups: IndexMap::new(),
        }
    }

    fn run(mut merger: MeshMerger, features: Vec<Feature>) -> (Vec<Feature>, Vec<Feature>) {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in &features {
            merger
                .process(create_default_execute_context(feature), &fw)
                .unwrap();
        }
        merger.finish(NodeContext::default(), &fw).unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        let sent = noop.send_features.lock().unwrap();
        let mut merged = Vec::new();
        let mut rejected = Vec::new();
        for (port, feature) in ports.iter().zip(sent.iter()) {
            if *port == *FEATURES_PORT {
                merged.push(feature.clone());
            } else {
                rejected.push(feature.clone());
            }
        }
        (merged, rejected)
    }

    fn face_count(feature: &Feature) -> usize {
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(mesh)) =
            feature.geometry.as_ref()
        else {
            panic!("expected a polygon mesh, got {:?}", feature.geometry);
        };
        mesh.num_faces()
    }

    fn string(v: &str) -> AttributeValue {
        AttributeValue::String(v.to_string())
    }

    #[test]
    fn each_group_becomes_one_mesh() {
        let (merged, rejected) = run(
            merger(&["rank"], AttributeAccumulation::UseOneFeature),
            vec![
                lower(&[("rank", "1")]),
                lower(&[("rank", "2")]),
                upper(&[("rank", "1")]),
            ],
        );
        assert!(rejected.is_empty());
        assert_eq!(merged.len(), 2);
        assert_eq!(face_count(&merged[0]), 2);
        assert_eq!(
            merged[0].attributes.get(&Attribute::new("rank")),
            Some(&string("1"))
        );
        assert_eq!(face_count(&merged[1]), 1);
    }

    #[test]
    fn collect_attributes_keeps_one_map_per_feature() {
        let (merged, _) = run(
            merger(&["rank"], AttributeAccumulation::CollectAttributes),
            vec![
                lower(&[("rank", "1"), ("id", "a")]),
                upper(&[("rank", "1")]),
            ],
        );
        let attributes = &merged[0].attributes;
        assert_eq!(attributes.get(&Attribute::new("rank")), Some(&string("1")));
        let first: Attributes = [(Attribute::new("id"), string("a"))].into_iter().collect();
        assert_eq!(
            attributes.get(&Attribute::new("__members")),
            Some(&AttributeValue::Array(vec![
                AttributeValue::Map(first),
                AttributeValue::Map(Attributes::new()),
            ]))
        );
        assert_eq!(attributes.get(&Attribute::new("id")), None);
    }

    #[test]
    fn group_attributes_only_drops_the_rest() {
        let (merged, _) = run(
            merger(&["rank"], AttributeAccumulation::GroupAttributesOnly),
            vec![lower(&[("rank", "1"), ("id", "a")])],
        );
        assert_eq!(merged[0].attributes.get(&Attribute::new("id")), None);
        assert_eq!(
            merged[0].attributes.get(&Attribute::new("rank")),
            Some(&string("1"))
        );
    }

    #[test]
    fn an_unmergeable_geometry_is_rejected() {
        let point = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
            Point3D::new(CoordinateFrame::Euclidean, [0., 0., 0.]),
        )));
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![point, lower(&[])],
        );
        assert_eq!(merged.len(), 1);
        assert_eq!(rejected.len(), 1);
    }

    #[test]
    fn a_group_in_mixed_frames_is_rejected_whole() {
        let other = triangle(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
            &[],
        );
        let (merged, rejected) = run(
            merger(&[], AttributeAccumulation::UseOneFeature),
            vec![lower(&[]), other],
        );
        assert!(merged.is_empty());
        assert_eq!(rejected.len(), 2);
    }
}
