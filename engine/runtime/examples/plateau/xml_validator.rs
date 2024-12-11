use std::collections::HashMap;

use helper::{create_workflow, ALL_ACTION_FACTORIES};
use reearth_flow_runtime::{dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS};

mod helper;

fn main() {
    let workflow = create_workflow("quality-check/02-bldg/c_bldg_01.yml");
    let mut factories = HashMap::new();
    factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
    factories.extend(ALL_ACTION_FACTORIES.clone());
    let dag = DagSchemas::from_graphs(
        workflow.entry_graph_id,
        workflow.graphs,
        factories,
        workflow.with,
    );
    let dot = dag.to_dot();
    println!("{}", dot);
}
