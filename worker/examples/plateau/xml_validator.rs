use std::collections::HashMap;

use helper::{create_workflow, ALL_ACTION_FACTORIES};
use reearth_flow_runtime::{
    dag_schemas::DagSchemas,
    node::{NodeKind, RouterFactory},
};

mod helper;

fn main() {
    let workflow = create_workflow("quality-check/02-bldg/c_bldg_01.yml");
    let mut factories = HashMap::new();
    factories.extend(ALL_ACTION_FACTORIES.clone());
    factories.insert(
        "Router".to_string(),
        NodeKind::Processor(Box::<RouterFactory>::default()),
    );
    let dag = DagSchemas::from_graphs(
        workflow.entry_graph_id,
        workflow.graphs,
        factories,
        workflow.with,
    );
    let dot = dag.to_dot();
    println!("{}", dot);
}
