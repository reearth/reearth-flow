//! Transitive link resolution.
//!
//! Features that name one another through an ID attribute form a graph. This
//! action resolves that relation transitively. Within each scope it finds the sets
//! of features reachable from one another, and labels every feature with the set
//! it landed in and whether that set spans the whole scope.
use std::cmp::Reverse;
use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;
use super::PlateauProfile;

/// Attribute holding the connectivity verdict (`"full"` / `"partial"` / `"alone"`).
const STATUS_ATTRIBUTE: &str = "_status";
/// Attribute holding the zero-based index of the feature's connected component.
const CONNECTED_ID_ATTRIBUTE: &str = "_connected_id";
/// Attribute holding the number of parts in the feature's connected component.
const CONNECTED_PARTS_ATTRIBUTE: &str = "_connected_parts";

#[derive(Debug, Clone)]
pub(crate) struct TransitiveLinkResolverFactory {
    name: String,
}

impl TransitiveLinkResolverFactory {
    pub(crate) fn new(profile: &PlateauProfile) -> Self {
        Self {
            name: profile.action_name("TransitiveLinkResolver"),
        }
    }
}

impl ProcessorFactory for TransitiveLinkResolverFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Resolves which features link to one another, directly or transitively, through an attribute holding the IDs each feature links to. Within each scope it labels every feature with the index and size of the linked set it belongs to, and whether that set spans one, some, or all of the scope."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(TransitiveLinkResolverParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let Some(with) = with else {
            return Err(PlateauProcessorError::TransitiveLinkResolverFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let value: Value = serde_json::to_value(with).map_err(|e| {
            PlateauProcessorError::TransitiveLinkResolverFactory(format!(
                "Failed to serialize `with` parameter: {e}"
            ))
        })?;
        let params: TransitiveLinkResolverParam = serde_json::from_value(value).map_err(|e| {
            PlateauProcessorError::TransitiveLinkResolverFactory(format!(
                "Failed to deserialize `with` parameter: {e}"
            ))
        })?;

        Ok(Box::new(TransitiveLinkResolver {
            params,
            parts: Vec::new(),
            groups: HashMap::new(),
        }))
    }
}

/// # TransitiveLinkResolver Parameters
/// Names the attribute identifying each feature, the attribute listing the features it links to, and the attributes delimiting the scope a verdict is computed over.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransitiveLinkResolverParam {
    /// # ID Attribute
    /// Attribute holding the identifier of the feature, such as its gml:id. Entries of the linked IDs attribute are matched against this value.
    pub id_attribute: Attribute,

    /// # Linked IDs Attribute
    /// Attribute holding an array of the identifiers of the features this one links to. An absent or null value means it links to nothing; a link recorded on only one side still connects the pair.
    pub linked_ids_attribute: Attribute,

    /// # Group By
    /// Attributes delimiting the scope a verdict is computed over, such as a parent feature, a level of detail and a source file. When omitted, all input features form a single scope. Linked IDs naming a feature outside the scope are ignored.
    pub group_by: Option<Vec<Attribute>>,
}

/// One buffered input feature and the graph data read from it.
#[derive(Debug, Clone)]
struct BufferedPart {
    id: String,
    linked_ids: Vec<String>,
    feature: Feature,
}

/// Where a part ended up once its group's components were resolved.
#[derive(Debug, Clone, Copy, Default)]
struct Verdict {
    status: ConnectivityStatus,
    connected_id: usize,
    connected_parts: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ConnectivityStatus {
    /// The part shares no polygon with any other part of its group.
    #[default]
    Alone,
    /// The part's component covers some, but not all, of its group.
    Partial,
    /// The part's component covers the whole group.
    Full,
}

impl ConnectivityStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Alone => "alone",
            Self::Partial => "partial",
            Self::Full => "full",
        }
    }
}

/// Union-find over the distinct part IDs of one group, indexed by node number.
#[derive(Debug)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl UnionFind {
    fn new(nodes: usize) -> Self {
        Self {
            parent: (0..nodes).collect(),
            rank: vec![0; nodes],
        }
    }

    fn find(&mut self, node: usize) -> usize {
        let mut root = node;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = node;
        while self.parent[current] != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        root
    }

    fn union(&mut self, a: usize, b: usize) {
        let (root_a, root_b) = (self.find(a), self.find(b));
        if root_a == root_b {
            return;
        }
        match self.rank[root_a].cmp(&self.rank[root_b]) {
            std::cmp::Ordering::Less => self.parent[root_a] = root_b,
            std::cmp::Ordering::Greater => self.parent[root_b] = root_a,
            std::cmp::Ordering::Equal => {
                self.parent[root_b] = root_a;
                self.rank[root_a] += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransitiveLinkResolver {
    params: TransitiveLinkResolverParam,
    /// Buffered parts in arrival order; the output preserves that order.
    parts: Vec<BufferedPart>,
    /// Group key -> indices into `parts`, each in arrival order.
    groups: HashMap<String, Vec<usize>>,
}

impl Processor for TransitiveLinkResolver {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let id = feature
            .attributes
            .get(&self.params.id_attribute)
            .and_then(|value| value.as_string())
            .ok_or_else(|| {
                PlateauProcessorError::TransitiveLinkResolver(format!(
                    "Feature has no string ID attribute `{}`",
                    self.params.id_attribute
                ))
            })?;

        let linked_ids = match feature.attributes.get(&self.params.linked_ids_attribute) {
            None | Some(AttributeValue::Null) => Vec::new(),
            Some(AttributeValue::Array(values)) => values
                .iter()
                .map(|value| {
                    value.as_string().ok_or_else(|| {
                        PlateauProcessorError::TransitiveLinkResolver(format!(
                            "Linked IDs attribute `{}` holds a non-string entry: {value}",
                            self.params.linked_ids_attribute
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(other) => {
                return Err(PlateauProcessorError::TransitiveLinkResolver(format!(
                    "Linked IDs attribute `{}` is not an array: {other}",
                    self.params.linked_ids_attribute
                ))
                .into())
            }
        };

        let group_key = self
            .params
            .group_by
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|attribute| {
                feature
                    .attributes
                    .get(attribute)
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join("|");

        let index = self.parts.len();
        self.parts.push(BufferedPart {
            id,
            linked_ids,
            feature: feature.clone(),
        });
        self.groups.entry(group_key).or_default().push(index);

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let verdicts = self.resolve();
        let ctx: Context = ctx.as_context();

        self.groups.clear();
        for (part, verdict) in std::mem::take(&mut self.parts).into_iter().zip(verdicts) {
            let mut feature = part.feature;
            let attributes = feature.attributes_mut();
            attributes.insert(
                Attribute::new(STATUS_ATTRIBUTE),
                AttributeValue::String(verdict.status.as_str().to_string()),
            );
            attributes.insert(
                Attribute::new(CONNECTED_ID_ATTRIBUTE),
                AttributeValue::Number(verdict.connected_id.into()),
            );
            attributes.insert(
                Attribute::new(CONNECTED_PARTS_ATTRIBUTE),
                AttributeValue::Number(verdict.connected_parts.into()),
            );

            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "TransitiveLinkResolver"
    }
}

impl TransitiveLinkResolver {
    /// Resolves every group's connected components into one verdict per buffered
    /// part, indexed by position in `parts`.
    fn resolve(&self) -> Vec<Verdict> {
        // Every index belongs to exactly one group and every group writes all of
        // its indices, so no default verdict survives to the output.
        let mut verdicts = vec![Verdict::default(); self.parts.len()];

        for indices in self.groups.values() {
            // Parts are keyed by ID, so two features carrying the same ID are one
            // node of the graph and receive the same verdict.
            let mut node_of_id: HashMap<&str, usize> = HashMap::new();
            for &index in indices {
                let next_node = node_of_id.len();
                node_of_id
                    .entry(self.parts[index].id.as_str())
                    .or_insert(next_node);
            }

            let node_count = node_of_id.len();
            let mut union_find = UnionFind::new(node_count);
            for &index in indices {
                let part = &self.parts[index];
                let node = node_of_id[part.id.as_str()];
                for linked_id in &part.linked_ids {
                    // A link leaving this scope says nothing about the scope's
                    // own connectivity.
                    if let Some(&linked_node) = node_of_id.get(linked_id.as_str()) {
                        union_find.union(node, linked_node);
                    }
                }
            }

            let mut members: Vec<Vec<usize>> = vec![Vec::new(); node_count];
            for node in 0..node_count {
                let root = union_find.find(node);
                members[root].push(node);
            }
            let mut components: Vec<Vec<usize>> = members
                .into_iter()
                .filter(|component| !component.is_empty())
                .collect();
            // Largest component first, ties broken by first arrival, so the
            // reported component index does not depend on iteration order.
            components.sort_by_key(|component| (Reverse(component.len()), component[0]));

            let mut verdict_of_node = vec![Verdict::default(); node_count];
            for (connected_id, component) in components.iter().enumerate() {
                let connected_parts = component.len();
                let status = if connected_parts == 1 {
                    ConnectivityStatus::Alone
                } else if connected_parts == node_count {
                    ConnectivityStatus::Full
                } else {
                    ConnectivityStatus::Partial
                };
                for &node in component {
                    verdict_of_node[node] = Verdict {
                        status,
                        connected_id,
                        connected_parts,
                    };
                }
            }

            for &index in indices {
                verdicts[index] = verdict_of_node[node_of_id[self.parts[index].id.as_str()]];
            }
        }

        verdicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::{create_default_execute_context, create_default_node_context};
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    static TEST_PROFILE: PlateauProfile = PlateauProfile {
        citygml: &crate::citygml::CITYGML2,
        uro_ns: "https://www.geospatial.jp/iur/uro/3.0",
        urf_ns: "https://www.geospatial.jp/iur/urf/3.0",
        action_prefix: "PLATEAU",
    };

    /// One feature: its ID, the IDs it links to, and the scope it belongs to.
    fn part(id: &str, linked: &[&str], building: &str) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert("gmlId".to_string(), AttributeValue::String(id.to_string()));
        attributes.insert(
            "next".to_string(),
            AttributeValue::Array(
                linked
                    .iter()
                    .map(|id| AttributeValue::String((*id).to_string()))
                    .collect(),
            ),
        );
        attributes.insert(
            "parentGmlId".to_string(),
            AttributeValue::String(building.to_string()),
        );
        Feature::from(attributes)
    }

    /// Runs the processor over `features` and returns, in output order, each
    /// feature's `(id, status, connected_id, connected_parts)`.
    fn run(features: Vec<Feature>) -> Result<Vec<(String, String, u64, u64)>, BoxedError> {
        let factory = TransitiveLinkResolverFactory::new(&TEST_PROFILE);
        let params = HashMap::from([
            (
                "idAttribute".to_string(),
                Value::String("gmlId".to_string()),
            ),
            (
                "linkedIdsAttribute".to_string(),
                Value::String("next".to_string()),
            ),
            (
                "groupBy".to_string(),
                Value::Array(vec![Value::String("parentGmlId".to_string())]),
            ),
        ]);
        let mut processor = factory.build(
            create_default_node_context(),
            EventHub::new(1024),
            "test".to_string(),
            Some(params),
        )?;

        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in features {
            processor.process(create_default_execute_context(feature), &fw)?;
        }
        processor.finish(create_default_node_context(), &fw)?;

        let ProcessorChannelForwarder::Noop(noop) = &fw else {
            return Err("Expected Noop forwarder for testing".into());
        };
        let sent = noop.send_features.lock().unwrap();
        Ok(sent
            .iter()
            .map(|feature| {
                let get = |name: &str| feature.attributes.get(&Attribute::new(name)).unwrap();
                let number = |name: &str| match get(name) {
                    AttributeValue::Number(n) => n.as_u64().unwrap(),
                    other => panic!("{name} is not a number: {other}"),
                };
                (
                    get("gmlId").as_string().unwrap(),
                    get(STATUS_ATTRIBUTE).as_string().unwrap(),
                    number(CONNECTED_ID_ATTRIBUTE),
                    number(CONNECTED_PARTS_ATTRIBUTE),
                )
            })
            .collect())
    }

    #[test]
    fn all_parts_in_one_component_are_full() -> Result<(), BoxedError> {
        let output = run(vec![
            part("a", &["b"], "bldg"),
            part("b", &["a", "c"], "bldg"),
            part("c", &["b"], "bldg"),
        ])?;

        assert_eq!(
            output,
            vec![
                ("a".to_string(), "full".to_string(), 0, 3),
                ("b".to_string(), "full".to_string(), 0, 3),
                ("c".to_string(), "full".to_string(), 0, 3),
            ]
        );
        Ok(())
    }

    #[test]
    fn a_lone_part_is_alone() -> Result<(), BoxedError> {
        let output = run(vec![part("a", &[], "bldg")])?;

        assert_eq!(output, vec![("a".to_string(), "alone".to_string(), 0, 1)]);
        Ok(())
    }

    #[test]
    fn a_split_building_reports_partial_and_alone() -> Result<(), BoxedError> {
        // The isolated part sorts after the pair, so the pair takes component 0.
        let output = run(vec![
            part("lonely", &[], "bldg"),
            part("a", &["b"], "bldg"),
            part("b", &["a"], "bldg"),
        ])?;

        assert_eq!(
            output,
            vec![
                ("lonely".to_string(), "alone".to_string(), 1, 1),
                ("a".to_string(), "partial".to_string(), 0, 2),
                ("b".to_string(), "partial".to_string(), 0, 2),
            ]
        );
        Ok(())
    }

    #[test]
    fn equal_sized_components_are_ordered_by_arrival() -> Result<(), BoxedError> {
        let output = run(vec![
            part("c", &["d"], "bldg"),
            part("d", &["c"], "bldg"),
            part("a", &["b"], "bldg"),
            part("b", &["a"], "bldg"),
        ])?;

        let component_ids: Vec<u64> = output.iter().map(|(_, _, id, _)| *id).collect();
        assert_eq!(component_ids, vec![0, 0, 1, 1]);
        Ok(())
    }

    #[test]
    fn a_link_leaving_the_scope_is_ignored() -> Result<(), BoxedError> {
        // `a` and `b` touch, but belong to different Buildings, so neither is
        // connected within its own group.
        let output = run(vec![part("a", &["b"], "left"), part("b", &["a"], "right")])?;

        assert_eq!(
            output,
            vec![
                ("a".to_string(), "alone".to_string(), 0, 1),
                ("b".to_string(), "alone".to_string(), 0, 1),
            ]
        );
        Ok(())
    }

    #[test]
    fn a_link_recorded_on_one_side_only_still_connects() -> Result<(), BoxedError> {
        let output = run(vec![part("a", &["b"], "bldg"), part("b", &[], "bldg")])?;

        assert_eq!(
            output,
            vec![
                ("a".to_string(), "full".to_string(), 0, 2),
                ("b".to_string(), "full".to_string(), 0, 2),
            ]
        );
        Ok(())
    }
}
