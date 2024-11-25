use std::collections::HashMap;

use clap::Command;
use reearth_flow_runtime::node::{NodeKind, RouterFactory};
use serde::{Deserialize, Serialize};

use crate::{
    factory::{BUILTIN_ACTION_FACTORIES, PLATEAU_ACTION_FACTORIES, WASM_ACTION_FACTORIES},
    utils::{create_action_schema, ActionSchema},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RootActionSchema {
    pub actions: Vec<ActionSchema>,
}

pub fn build_schema_action_command() -> Command {
    Command::new("schema-action")
        .about("Show action schema.")
        .long_about("Show action schema.")
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaActionCliCommand;

impl SchemaActionCliCommand {
    pub fn execute(&self) -> crate::Result<()> {
        let mut builtin_action_factories = HashMap::new();
        builtin_action_factories.extend(BUILTIN_ACTION_FACTORIES.clone());
        builtin_action_factories.insert(
            "Router".to_string(),
            NodeKind::Processor(Box::<RouterFactory>::default()),
        );
        let mut actions = builtin_action_factories
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, true))
            .collect::<Vec<_>>();
        let plateau_actions = PLATEAU_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false))
            .collect::<Vec<_>>();
        actions.extend(plateau_actions);
        let wasm_actions = WASM_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false))
            .collect::<Vec<_>>();
        actions.extend(wasm_actions);
        actions.sort_by(|a, b| a.name.cmp(&b.name));
        let root = RootActionSchema { actions };
        println!("{}", serde_json::to_string_pretty(&root).unwrap());
        Ok(())
    }
}
