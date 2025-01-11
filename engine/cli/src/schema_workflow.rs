use clap::Command;
use reearth_flow_types::Workflow;

pub fn build_schema_workflow_command() -> Command {
    Command::new("schema-workflow")
        .about("Show workflow schema.")
        .long_about("Show workflow schema.")
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaWorkflowCliCommand;

impl SchemaWorkflowCliCommand {
    pub fn execute(&self) -> crate::Result<()> {
        let schema = schemars::schema_for!(Workflow);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }
}
