use clap::Command;
use reearth_flow_runner::executor::ACTION_MAPPINGS;
use reearth_flow_runtime::node::NodeKind;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RootActionSchema {
    pub actions: Vec<ActionSchema>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionSchema {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub parameter: serde_json::Value,
    pub builtin: bool,
    pub input_ports: Vec<String>,
    pub output_ports: Vec<String>,
    pub categories: Vec<String>,
}

impl ActionSchema {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        r#type: String,
        description: String,
        parameter: serde_json::Value,
        builtin: bool,
        input_ports: Vec<String>,
        output_ports: Vec<String>,
        categories: Vec<String>,
    ) -> Self {
        Self {
            name,
            r#type,
            description,
            parameter,
            builtin,
            input_ports,
            output_ports,
            categories,
        }
    }
}

pub fn build_schema_command() -> Command {
    Command::new("schema")
        .about("Show schema.")
        .long_about("Show schema.")
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaCliCommand;

impl SchemaCliCommand {
    pub fn execute(&self) -> crate::Result<()> {
        let actions = ACTION_MAPPINGS
            .clone()
            .values()
            .map(|v| match v {
                NodeKind::Source(factory) => {
                    let parameter = match factory.parameter_schema() {
                        Some(schema) => {
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap()
                        }
                        None => serde_json::Value::Null,
                    };
                    ActionSchema::new(
                        factory.name().to_string(),
                        "source".to_string(),
                        factory.description().to_string(),
                        parameter,
                        true,
                        factory
                            .get_output_ports()
                            .iter()
                            .map(|p| p.to_string())
                            .collect(),
                        vec![],
                        factory.categories().iter().map(|c| c.to_string()).collect(),
                    )
                }
                NodeKind::Processor(factory) => {
                    let parameter = match factory.parameter_schema() {
                        Some(schema) => {
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap()
                        }
                        None => serde_json::Value::Null,
                    };
                    ActionSchema::new(
                        factory.name().to_string(),
                        "processor".to_string(),
                        factory.description().to_string(),
                        parameter,
                        true,
                        factory
                            .get_output_ports()
                            .iter()
                            .map(|p| p.to_string())
                            .collect(),
                        factory
                            .get_input_ports()
                            .iter()
                            .map(|p| p.to_string())
                            .collect(),
                        factory.categories().iter().map(|c| c.to_string()).collect(),
                    )
                }
                NodeKind::Sink(factory) => {
                    let parameter = match factory.parameter_schema() {
                        Some(schema) => {
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap()
                        }
                        None => serde_json::Value::Null,
                    };
                    ActionSchema::new(
                        factory.name().to_string(),
                        "sink".to_string(),
                        factory.description().to_string(),
                        parameter,
                        true,
                        vec![],
                        factory
                            .get_input_ports()
                            .iter()
                            .map(|p| p.to_string())
                            .collect(),
                        factory.categories().iter().map(|c| c.to_string()).collect(),
                    )
                }
            })
            .collect::<Vec<_>>();
        let root = RootActionSchema { actions };
        println!("{}", serde_json::to_string_pretty(&root).unwrap());
        Ok(())
    }
}
