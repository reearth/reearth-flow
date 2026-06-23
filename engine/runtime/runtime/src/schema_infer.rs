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

/// Output ports for a node including dynamically-configured ones (OutputRouter
/// `routingPort`, FeatureFilter `conditions[].outputPort`) that are derivable
/// from `with` but not reported by the factory's static `get_output_ports()`.
/// Mirrors the dynamic-port derivation in `builder_dag`.
fn effective_output_ports(
    factory: &FactoryRef<'_>,
    action: &str,
    with: &Option<HashMap<String, serde_json::Value>>,
) -> Vec<Port> {
    let mut ports = factory.output_ports();
    if let Some(with) = with {
        if action == crate::node::OUTPUT_ROUTING_ACTION {
            if let Some(serde_json::Value::String(rp)) = with.get(crate::node::ROUTING_PARAM_KEY) {
                let p = Port::new(rp.clone());
                if !ports.contains(&p) {
                    ports.push(p);
                }
            }
        } else if action == crate::node::FEATURE_FILTER_ACTION {
            if let Some(serde_json::Value::Array(conditions)) = with.get("conditions") {
                for c in conditions {
                    if let Some(serde_json::Value::String(port)) = c.get("outputPort") {
                        let p = Port::new(port.clone());
                        if !ports.contains(&p) {
                            ports.push(p);
                        }
                    }
                }
            }
        }
    }
    ports
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
                    // through to every effective output port (including dynamic
                    // router/filter ports derived from `with`).
                    let joined = join_all_inputs(&inputs);
                    effective_output_ports(factory, node.node.action(), &node.with)
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

/// Like [`infer_and_validate`], but seeds source nodes by sampling their
/// datasets (up to `sample_size` features; `0` = unbounded). Non-source nodes
/// propagate exactly as in [`infer_and_validate`]. Per-source failures degrade
/// to an `open` schema with a recorded note (in [`InferResult::notes`], keyed by
/// node id).
///
/// `vars` are the workflow's global `with:` variables (already merged with any
/// CLI `--var` overrides). They seed the expression engine used while sampling,
/// so a source `dataset` expression like `env.get("path")` resolves to the same
/// value it would under `run`.
pub fn infer_with_sampling(
    dag: &DagSchemas,
    sample_size: usize,
    vars: serde_json::Map<String, serde_json::Value>,
) -> Result<InferResult, crate::errors::ExecutionError> {
    // Build the expression engine from the workflow's global `with:` vars,
    // exactly as the runtime does in `Orchestrator`
    // (`Engine::with_vars(workflow.with...)`). Threading it into the source
    // sampling below lets `dataset` expressions such as `env.get("path")`
    // resolve during sampling — the same way `run` resolves them.
    let env_vars = std::sync::Arc::new(vars);
    let graph = dag.graph();
    let order = petgraph::algo::toposort(graph, None)
        .map_err(|_| crate::errors::ExecutionError::SchemaInferenceCycle)?;

    let mut outputs_by_index: HashMap<petgraph::graph::NodeIndex, HashMap<Port, AttrSchema>> =
        HashMap::new();
    let mut result = InferResult::default();

    for idx in order {
        let node = &graph[idx];
        let inputs = gather_inputs(graph, idx, &outputs_by_index);

        let outputs: HashMap<Port, AttrSchema> = match node.kind.as_ref() {
            // Source node: sample its dataset; map the sampled schema onto every
            // declared output port.
            Some(kind @ NodeKind::Source(_)) => {
                let factory = FactoryRef::of(kind);
                let outcome = crate::schema_sample::sample_source(
                    kind,
                    &node.with,
                    sample_size,
                    env_vars.clone(),
                );
                if let Some(note) = outcome.note {
                    result.notes.insert(node.handle.id.to_string(), note);
                }
                factory
                    .output_ports()
                    .into_iter()
                    .map(|p| (p, outcome.schema.clone()))
                    .collect()
            }
            // Processor/Sink: existing inference + passthrough fallback.
            Some(kind) => {
                let factory = FactoryRef::of(kind);
                match factory.infer_output_schema(&inputs, &node.with) {
                    Some(map) => map,
                    None => {
                        // Schema-transparent fallback: pass the join of all inputs
                        // through to every effective output port (including dynamic
                        // router/filter ports derived from `with`).
                        let joined = join_all_inputs(&inputs);
                        effective_output_ports(&factory, node.node.action(), &node.with)
                            .into_iter()
                            .map(|p| (p, joined.clone()))
                            .collect()
                    }
                }
            }
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
            // Not a real reader: returning an error makes `sample_source`
            // degrade to an open schema plus a note (instead of panicking),
            // which is exactly the behaviour the sampling test exercises.
            Err("StubSource is not built in schema inference tests".into())
        }
        // For static inference, rely on the None -> open fallback for output.
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

    /// Mirrors OutputRouter: declares NO static output ports, has no
    /// `infer_output_schema` (so it hits the passthrough fallback). Its routed
    /// output port is carried only via `with["routingPort"]`.
    #[derive(Debug, Clone)]
    struct RouterStub;

    impl ProcessorFactory for RouterStub {
        fn name(&self) -> &str {
            "OutputRouter"
        }
        fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
            None
        }
        fn get_input_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn get_output_ports(&self) -> Vec<Port> {
            // OutputRouter declares no static output ports.
            vec![]
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
        // No infer_output_schema: rely on the None -> passthrough fallback,
        // which must surface the dynamic routingPort.
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

    fn action_node_with(id: Uuid, name: &str, action: &str, with: serde_json::Value) -> Node {
        let with = match with {
            serde_json::Value::Object(map) => Some(map),
            _ => panic!("with must be a JSON object"),
        };
        Node::Action {
            entity: NodeEntity {
                id,
                name: name.to_string(),
                with,
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
        m.insert(
            "OutputRouter".to_string(),
            NodeKind::Processor(Box::new(RouterStub)),
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
    fn infer_with_sampling_records_source_note_and_propagates() {
        // Reuse the StubSource -> Adder dag from propagates_node_outputs.
        let dag = build_dag("Adder");
        let result = infer_with_sampling(&dag, 10, serde_json::Map::new()).expect("inference ok");

        // 1) The StubSource node gets a note: it is not a real reader, so
        //    sampling falls back to open + note. This proves the sampling path
        //    ran for the source node.
        assert!(
            !result.notes.is_empty(),
            "source sampling should record a note for the stub source"
        );

        // 2) Downstream propagation still works: the Adder still produces "foo"
        //    on its default port (mirrors propagates_node_outputs).
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
    fn infer_with_sampling_includes_output_router_dynamic_port() {
        // StubSource -> OutputRouter. OutputRouter declares no static output
        // ports; its routed port is configured via with["routingPort"]. The
        // passthrough fallback must surface that dynamic port.
        let src_id = Uuid::new_v4();
        let router_id = Uuid::new_v4();
        let graph_id = Uuid::new_v4();

        let graph = Graph {
            id: graph_id,
            name: "g".to_string(),
            nodes: vec![
                action_node(src_id, "src", "StubSource"),
                action_node_with(
                    router_id,
                    "router",
                    "OutputRouter",
                    serde_json::json!({ "routingPort": "myroute" }),
                ),
            ],
            edges: vec![edge(src_id, router_id)],
        };

        let dag = DagSchemas::from_graphs(graph_id, vec![graph], factories(), None)
            .expect("dag construction");

        // Exercise both entrypoints: the router port must appear in each.
        let sampled = infer_with_sampling(&dag, 10, serde_json::Map::new()).expect("inference ok");
        let router_out = sampled
            .node_outputs
            .get(&router_id.to_string())
            .expect("router node outputs present");
        assert!(
            router_out.contains_key("myroute"),
            "infer_with_sampling: routed port `myroute` should appear, got {:?}",
            router_out.keys().collect::<Vec<_>>()
        );

        let validated = infer_and_validate(&dag).expect("inference ok");
        let router_out = validated
            .node_outputs
            .get(&router_id.to_string())
            .expect("router node outputs present");
        assert!(
            router_out.contains_key("myroute"),
            "infer_and_validate: routed port `myroute` should appear, got {:?}",
            router_out.keys().collect::<Vec<_>>()
        );
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
