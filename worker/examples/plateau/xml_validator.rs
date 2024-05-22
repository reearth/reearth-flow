use helper::create_workflow;
use reearth_flow_runner::executor::ACTION_MAPPINGS;
use reearth_flow_runtime::dag_schemas::DagSchemas;

mod helper;

fn main() {
    let workflow = create_workflow("domain_of_definition_validator.yml");
    let dag = DagSchemas::from_graphs(
        workflow.entry_graph_id,
        workflow.graphs,
        ACTION_MAPPINGS.clone(),
        workflow.with,
    );
    let dot = dag.to_dot();
    println!("{}", dot);
}
