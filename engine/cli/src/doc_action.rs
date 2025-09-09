use std::collections::HashMap;

use clap::Command;
use indoc::indoc;
use reearth_flow_runtime::node::SYSTEM_ACTION_FACTORY_MAPPINGS;

use crate::{
    factory::{BUILTIN_ACTION_FACTORIES, PLATEAU_ACTION_FACTORIES, PYTHON_ACTION_FACTORIES},
    utils::create_action_schema,
};

pub fn build_doc_action_command() -> Command {
    Command::new("doc-action")
        .about("Show action doc.")
        .long_about("Show action doc.")
}

#[derive(Debug, Eq, PartialEq)]
pub struct DocActionCliCommand;

impl DocActionCliCommand {
    pub fn execute(&self) -> crate::Result<()> {
        let mut builtin_action_factories = HashMap::new();
        let i18n = HashMap::new();
        builtin_action_factories.extend(BUILTIN_ACTION_FACTORIES.clone());
        builtin_action_factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
        let mut actions = builtin_action_factories
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, true, &i18n))
            .collect::<Vec<_>>();
        let plateau_actions = PLATEAU_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false, &i18n))
            .collect::<Vec<_>>();
        actions.extend(plateau_actions);
        let python_actions = PYTHON_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false, &i18n))
            .collect::<Vec<_>>();
        actions.extend(python_actions);
        actions.sort_by(|a, b| a.name.cmp(&b.name));
        println!("# Actions");
        actions.iter().for_each(|action| {
            println!("\n## {}", action.name);
            println!("### Type");
            println!("* {}", action.r#type);
            println!("### Description");
            println!("{}", action.description);
            match action.parameter {
                serde_json::Value::Null => {
                    println!("### Parameters");
                    println!("* No parameters");
                }
                _ => {
                    println!("### Parameters");
                    println!(
                        indoc! {"```json
                    {}
                    ```"},
                        serde_json::to_string_pretty(&action.parameter).unwrap()
                    );
                }
            }
            println!("### Input Ports");
            action.input_ports.iter().for_each(|port| {
                println!("* {port}");
            });
            println!("### Output Ports");
            action.output_ports.iter().for_each(|port| {
                println!("* {port}");
            });
            println!("### Category");
            action.categories.iter().for_each(|category| {
                println!("* {category}");
            });
        });
        Ok(())
    }
}
