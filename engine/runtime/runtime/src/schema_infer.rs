//! Static attribute-schema propagation over a [`DagSchemas`] graph.
//!
//! Walks the DAG in topological order, threading inferred per-port
//! [`AttrSchema`]s from producers to consumers.

use std::collections::HashMap;

use petgraph::visit::EdgeRef;
use reearth_flow_types::attr_schema::AttrSchema;

use crate::dag_schemas::{DagSchemas, SchemaEdgeType, SchemaNodeType};
use crate::node::{NodeKind, Port, ProcessorFactory, SinkFactory, SourceFactory};

#[derive(Debug, Default)]
pub struct InferResult {
    /// Inferred output schema per node id, keyed by output port name.
    pub node_outputs: HashMap<String, HashMap<String, AttrSchema>>,
    /// Per-node note (e.g. why a source could not be sampled). Keyed by node id.
    pub notes: HashMap<String, String>,
}

/// The schema-relevant view of a node's factory, regardless of its concrete kind.
enum FactoryRef<'a> {
    Source(&'a dyn SourceFactory),
    Processor(&'a dyn ProcessorFactory),
    Sink(&'a dyn SinkFactory),
}

impl<'a> FactoryRef<'a> {
    fn of(kind: &'a NodeKind) -> Self {
        match kind {
            NodeKind::Source(f) => FactoryRef::Source(f.as_ref()),
            NodeKind::Processor(f) => FactoryRef::Processor(f.as_ref()),
            NodeKind::Sink(f) => FactoryRef::Sink(f.as_ref()),
        }
    }

    fn infer_output_schema(
        &self,
        inputs: &HashMap<Port, AttrSchema>,
        with: &Option<HashMap<String, serde_json::Value>>,
    ) -> Option<HashMap<Port, AttrSchema>> {
        match self {
            FactoryRef::Source(f) => f.infer_output_schema(inputs, with),
            FactoryRef::Processor(f) => f.infer_output_schema(inputs, with),
            FactoryRef::Sink(f) => f.infer_output_schema(inputs, with),
        }
    }

    /// Declared output ports (sinks have none).
    fn output_ports(&self) -> Vec<Port> {
        match self {
            FactoryRef::Source(f) => f.get_output_ports(),
            FactoryRef::Processor(f) => f.get_output_ports(),
            FactoryRef::Sink(_) => Vec::new(),
        }
    }
}

/// Statically propagate attribute schemas through the DAG.
pub fn infer_and_validate(dag: &DagSchemas) -> Result<InferResult, crate::errors::ExecutionError> {
    let graph = dag.graph();
    let order = petgraph::algo::toposort(graph, None)
        .map_err(|_| crate::errors::ExecutionError::SchemaInferenceCycle)?;

    let mut outputs_by_index: HashMap<petgraph::graph::NodeIndex, HashMap<Port, AttrSchema>> =
        HashMap::new();
    let mut result = InferResult::default();

    for idx in order {
        let node = &graph[idx];

        // (b) Gather inputs per consumer input-port, joining when multiple
        // producers feed the same port.
        let inputs = gather_inputs(graph, idx, &outputs_by_index);

        let factory = node.kind.as_ref().map(FactoryRef::of);

        // (c) Compute this node's outputs.
        let outputs: HashMap<Port, AttrSchema> = match &factory {
            Some(factory) => match factory.infer_output_schema(&inputs, &node.with) {
                Some(map) => map,
                None => {
                    // Schema-transparent fallback: pass the join of all inputs
                    // through to every declared output port.
                    let joined = join_all_inputs(&inputs);
                    factory
                        .output_ports()
                        .into_iter()
                        .map(|p| (p, joined.clone()))
                        .collect()
                }
            },
            // No factory (e.g. unexpanded subgraph node): produce nothing.
            None => HashMap::new(),
        };

        result.node_outputs.insert(
            node.handle.id.to_string(),
            outputs
                .iter()
                .map(|(p, s)| (p.to_string(), s.clone()))
                .collect(),
        );
        outputs_by_index.insert(idx, outputs);
    }

    Ok(result)
}

/// Gather inputs per consumer input-port, joining when multiple producers feed
/// the same port. Producers with no recorded output for the referenced port (or
/// no recorded outputs at all) contribute an `open` schema.
fn gather_inputs(
    graph: &petgraph::graph::DiGraph<SchemaNodeType, SchemaEdgeType>,
    idx: petgraph::graph::NodeIndex,
    outputs_by_index: &HashMap<petgraph::graph::NodeIndex, HashMap<Port, AttrSchema>>,
) -> HashMap<Port, AttrSchema> {
    let mut inputs: HashMap<Port, AttrSchema> = HashMap::new();
    for e in graph.edges_directed(idx, petgraph::Direction::Incoming) {
        let src = e.source();
        let ew: &SchemaEdgeType = e.weight();
        let incoming = outputs_by_index
            .get(&src)
            .and_then(|m| m.get(&ew.from))
            .cloned()
            .unwrap_or_else(AttrSchema::open);
        inputs
            .entry(ew.to.clone())
            .and_modify(|existing| *existing = existing.join(&incoming))
            .or_insert(incoming);
    }
    inputs
}

/// Join all input-port schemas into one; `open` if there are no inputs at all.
fn join_all_inputs(inputs: &HashMap<Port, AttrSchema>) -> AttrSchema {
    let mut iter = inputs.values();
    match iter.next() {
        None => AttrSchema::open(),
        Some(first) => iter.fold(first.clone(), |acc, s| acc.join(s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use reearth_flow_types::attr_schema::{AttrField, AttrType};
    use reearth_flow_types::attribute::Attribute;
    use reearth_flow_types::workflow::{Edge, Graph, Node, NodeEntity};
    use uuid::Uuid;

    use crate::event::EventHub;
    use crate::executor_operation::NodeContext;
    use crate::node::{Processor, Source, DEFAULT_PORT};

    // ---- Stub factories -------------------------------------------------

    #[derive(Debug, Clone)]
    struct StubSource;

    impl SourceFactory for StubSource {
        fn name(&self) -> &str {
            "StubSource"
        }
        fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
            None
        }
        fn get_output_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn build(
            &self,
            _ctx: NodeContext,
            _event_hub: EventHub,
            _action: String,
            _with: Option<HashMap<String, serde_json::Value>>,
            _state: Option<Vec<u8>>,
        ) -> Result<Box<dyn Source>, crate::errors::BoxedError> {
            unreachable!("not built in schema inference tests")
        }
        // Rely on the None -> open fallback for output.
    }

    /// Produces a CLOSED schema with a single field "foo" (Number, Always).
    #[derive(Debug, Clone)]
    struct AdderProc;

    impl ProcessorFactory for AdderProc {
        fn name(&self) -> &str {
            "Adder"
        }
        fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
            None
        }
        fn get_input_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn get_output_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn build(
            &self,
            _ctx: NodeContext,
            _event_hub: EventHub,
            _action: String,
            _with: Option<HashMap<String, serde_json::Value>>,
        ) -> Result<Box<dyn Processor>, crate::errors::BoxedError> {
            unreachable!("not built in schema inference tests")
        }
        fn infer_output_schema(
            &self,
            inputs: &HashMap<Port, AttrSchema>,
            _with: &Option<HashMap<String, serde_json::Value>>,
        ) -> Option<HashMap<Port, AttrSchema>> {
            let mut schema = inputs
                .get(&DEFAULT_PORT)
                .cloned()
                .unwrap_or_else(AttrSchema::empty);
            schema.open = false;
            schema.insert(
                Attribute::new("foo".to_string()),
                AttrField::always(AttrType::Number),
            );
            let mut out = HashMap::new();
            out.insert(DEFAULT_PORT.clone(), schema);
            Some(out)
        }
    }

    // ---- Helpers --------------------------------------------------------

    fn action_node(id: Uuid, name: &str, action: &str) -> Node {
        Node::Action {
            entity: NodeEntity {
                id,
                name: name.to_string(),
                with: None,
            },
            action: action.to_string(),
        }
    }

    fn edge(from: Uuid, to: Uuid) -> Edge {
        Edge {
            id: Uuid::new_v4(),
            from,
            to,
            from_port: "default".to_string(),
            to_port: "default".to_string(),
        }
    }

    fn factories() -> HashMap<String, NodeKind> {
        let mut m = HashMap::new();
        m.insert(
            "StubSource".to_string(),
            NodeKind::Source(Box::new(StubSource)),
        );
        m.insert(
            "Adder".to_string(),
            NodeKind::Processor(Box::new(AdderProc)),
        );
        m
    }

    /// source -> mid. `middle_action` selects whether the middle node closes the
    /// schema ("Adder") or stays transparent/open ("StubSource"-style via the
    /// None -> open fallback).
    fn build_dag(middle_action: &str) -> DagSchemas {
        let src_id = Uuid::new_v4();
        let mid_id = Uuid::new_v4();
        let graph_id = Uuid::new_v4();

        let graph = Graph {
            id: graph_id,
            name: "g".to_string(),
            nodes: vec![
                action_node(src_id, "src", "StubSource"),
                action_node(mid_id, "mid", middle_action),
            ],
            edges: vec![edge(src_id, mid_id)],
        };

        DagSchemas::from_graphs(graph_id, vec![graph], factories(), None).expect("dag construction")
    }

    // ---- Tests ----------------------------------------------------------

    #[test]
    fn propagates_node_outputs() {
        let dag = build_dag("Adder");
        let result = infer_and_validate(&dag).expect("infer");

        // The Adder node's output schema must carry "foo" on the default port.
        let adder_out = result
            .node_outputs
            .values()
            .find(|m| {
                m.get("default")
                    .is_some_and(|s| s.fields.contains_key(&Attribute::new("foo".to_string())))
            })
            .expect("an output with field foo");
        assert!(adder_out["default"]
            .fields
            .contains_key(&Attribute::new("foo".to_string())));
    }

    #[test]
    fn cycle_returns_error() {
        let a_id = Uuid::new_v4();
        let b_id = Uuid::new_v4();
        let graph_id = Uuid::new_v4();

        let graph = Graph {
            id: graph_id,
            name: "g".to_string(),
            nodes: vec![
                action_node(a_id, "a", "Adder"),
                action_node(b_id, "b", "Adder"),
            ],
            edges: vec![edge(a_id, b_id), edge(b_id, a_id)],
        };

        let dag = DagSchemas::from_graphs(graph_id, vec![graph], factories(), None)
            .expect("dag construction");

        let err = infer_and_validate(&dag).unwrap_err();
        assert!(matches!(
            err,
            crate::errors::ExecutionError::SchemaInferenceCycle
        ));
    }
}
