use helper::{create_workflow, BUILTIN_ACTION_FACTORIES};
use reearth_flow_runtime::dag_schemas::DagSchemas;

mod helper;

fn main() {
    let workflow = create_workflow("lod_splitter_with_dm.yml");
    let dag = DagSchemas::from_graphs(
        workflow.entry_graph_id,
        workflow.graphs,
        BUILTIN_ACTION_FACTORIES.clone(),
        workflow.with,
    );
    let dot = dag.to_dot();
    println!("{}", dot);
}
