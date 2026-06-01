//! Static attribute-schema propagation over a [`DagSchemas`] graph.
//!
//! Walks the DAG in topological order, threading inferred per-port
//! [`AttrSchema`]s from producers to consumers, and collects validation
//! diagnostics (e.g. references to attributes that no upstream node produces).

use std::collections::HashMap;

use petgraph::visit::EdgeRef;
use reearth_flow_types::attr_schema::{AttrRef, AttrSchema, Presence};

use crate::dag_schemas::{DagSchemas, SchemaEdgeType, SchemaNodeType};
use crate::node::{NodeKind, Port, ProcessorFactory, SinkFactory, SourceFactory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub node_id: String,
    pub node_name: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct InferResult {
    pub diagnostics: Vec<Diagnostic>,
    /// Inferred output schema per node id, keyed by output port name (port.to_string()).
    pub node_outputs: HashMap<String, HashMap<String, AttrSchema>>,
}

impl InferResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
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

    fn referenced_input_attributes(
        &self,
        with: &Option<HashMap<String, serde_json::Value>>,
    ) -> Vec<AttrRef> {
        match self {
            FactoryRef::Source(f) => f.referenced_input_attributes(with),
            FactoryRef::Processor(f) => f.referenced_input_attributes(with),
            FactoryRef::Sink(f) => f.referenced_input_attributes(with),
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

/// Statically propagate attribute schemas through the DAG and collect validation diagnostics.
pub fn infer_and_validate(
    dag: &DagSchemas,
) -> Result<InferResult, crate::errors::ExecutionError> {
    let graph = dag.graph();
    let order = petgraph::algo::toposort(graph, None)
        .map_err(|_| crate::errors::ExecutionError::SchemaInferenceCycle)?;

    let mut outputs_by_index: HashMap<
        petgraph::graph::NodeIndex,
        HashMap<Port, AttrSchema>,
    > = HashMap::new();
    let mut result = InferResult::default();

    for idx in order {
        let node = &graph[idx];

        // (b) Gather inputs per consumer input-port, joining when multiple
        // producers feed the same port.
        let inputs = gather_inputs(graph, idx, &outputs_by_index);

        let factory = node.kind.as_ref().map(FactoryRef::of);

        // (c) Reference validation.
        if let Some(factory) = &factory {
            check_references(factory, node, &inputs, &mut result);
        }

        // (d) Compute this node's outputs.
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

/// Validate a node's referenced input attributes against the schemas reaching
/// it, pushing diagnostics onto `result` for absent (Error) or maybe-present
/// (Warning) references.
fn check_references(
    factory: &FactoryRef<'_>,
    node: &SchemaNodeType,
    inputs: &HashMap<Port, AttrSchema>,
    result: &mut InferResult,
) {
    for AttrRef { name, port } in factory.referenced_input_attributes(&node.with) {
        let Some(schema) = inputs.get(&Port::new(port.clone())) else {
            // Input port absent: cannot disprove the reference.
            continue;
        };
        if schema.open {
            // Open schema: any attribute may exist, so no diagnostic.
            continue;
        }
        match schema.fields.get(&name) {
            None => result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                node_id: node.handle.id.to_string(),
                node_name: node.name.clone(),
                message: format!(
                    "references attribute `{name}` which is not produced by any upstream node on port `{port}`"
                ),
            }),
            Some(field) if field.presence == Presence::Maybe => {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    node_id: node.handle.id.to_string(),
                    node_name: node.name.clone(),
                    message: format!(
                        "references attribute `{name}` on port `{port}` which may not always be present"
                    ),
                });
            }
            Some(_) => {}
        }
    }
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
    use crate::node::{
        Processor, Source, DEFAULT_PORT,
    };

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
            // Force closed so downstream reference checks can fire.
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

    /// Produces a CLOSED schema with a single field "missing" (String, Maybe).
    #[derive(Debug, Clone)]
    struct MaybeProc;

    impl ProcessorFactory for MaybeProc {
        fn name(&self) -> &str {
            "Maybe"
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
            _inputs: &HashMap<Port, AttrSchema>,
            _with: &Option<HashMap<String, serde_json::Value>>,
        ) -> Option<HashMap<Port, AttrSchema>> {
            let mut schema = AttrSchema::empty();
            schema.insert(
                Attribute::new("missing".to_string()),
                AttrField::maybe(AttrType::String),
            );
            let mut out = HashMap::new();
            out.insert(DEFAULT_PORT.clone(), schema);
            Some(out)
        }
    }

    /// Reads attribute "missing" from its default input. Output inference unimplemented.
    #[derive(Debug, Clone)]
    struct NeedsProc;

    impl ProcessorFactory for NeedsProc {
        fn name(&self) -> &str {
            "Needs"
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
        fn referenced_input_attributes(
            &self,
            _with: &Option<HashMap<String, serde_json::Value>>,
        ) -> Vec<AttrRef> {
            vec![AttrRef {
                name: Attribute::new("missing".to_string()),
                port: "default".to_string(),
            }]
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
        m.insert("StubSource".to_string(), NodeKind::Source(Box::new(StubSource)));
        m.insert("Adder".to_string(), NodeKind::Processor(Box::new(AdderProc)));
        m.insert("Maybe".to_string(), NodeKind::Processor(Box::new(MaybeProc)));
        m.insert("Needs".to_string(), NodeKind::Processor(Box::new(NeedsProc)));
        m
    }

    /// source -> Adder -> Needs. `adder_action` selects whether the middle node
    /// closes the schema ("Adder") or stays transparent/open ("StubSource"-style
    /// via Adder None fallback). We just vary the middle action name.
    fn build_dag(middle_action: &str) -> DagSchemas {
        let src_id = Uuid::new_v4();
        let mid_id = Uuid::new_v4();
        let needs_id = Uuid::new_v4();
        let graph_id = Uuid::new_v4();

        let graph = Graph {
            id: graph_id,
            name: "g".to_string(),
            nodes: vec![
                action_node(src_id, "src", "StubSource"),
                action_node(mid_id, "mid", middle_action),
                action_node(needs_id, "needs", "Needs"),
            ],
            edges: vec![edge(src_id, mid_id), edge(mid_id, needs_id)],
        };

        DagSchemas::from_graphs(graph_id, vec![graph], factories(), None)
            .expect("dag construction")
    }

    // ---- Tests ----------------------------------------------------------

    #[test]
    fn propagates_and_flags_missing_reference() {
        let dag = build_dag("Adder");
        let result = infer_and_validate(&dag).expect("infer");

        assert!(result.has_errors(), "expected an error diagnostic");
        let errors: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1, "exactly one error expected");
        assert!(errors[0].message.contains("missing"));
        assert_eq!(errors[0].node_name, "needs");

        // The Adder node's output schema must carry "foo" on the default port.
        let adder_out = result
            .node_outputs
            .values()
            .find(|m| m.get("default").is_some_and(|s| s.fields.contains_key(&Attribute::new("foo".to_string()))))
            .expect("an output with field foo");
        assert!(adder_out["default"]
            .fields
            .contains_key(&Attribute::new("foo".to_string())));
    }

    #[test]
    fn flags_maybe_present_reference_as_warning() {
        // source -> Maybe -> Needs. The "Maybe" node emits a closed schema with
        // field "missing" present only conditionally (Presence::Maybe). "Needs"
        // references "missing", so the reference is satisfiable but not
        // guaranteed -> exactly one Warning, no errors.
        let dag = build_dag("Maybe");
        let result = infer_and_validate(&dag).expect("infer");

        assert!(!result.has_errors(), "maybe-present reference must not be an error");
        let warnings: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect();
        assert_eq!(warnings.len(), 1, "exactly one warning expected, got {warnings:?}");
        assert!(warnings[0].message.contains("missing"));
        assert_eq!(warnings[0].node_name, "needs");
    }

    #[test]
    fn open_input_suppresses_reference_error() {
        // Middle node is "StubSource" action -> NodeKind::Source whose
        // infer_output_schema is None -> fallback. As a source-style factory in
        // the middle it still has a default output port; with an open input
        // joined through, the schema reaching "Needs" is open, suppressing errors.
        let dag = build_dag("StubSource");
        let result = infer_and_validate(&dag).expect("infer");

        let errors: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "open schema must suppress reference errors, got {errors:?}");
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
