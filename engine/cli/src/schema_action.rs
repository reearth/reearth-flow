use std::{collections::HashMap, fs::File, io::BufReader};

use clap::{Arg, ArgMatches, Command};
use reearth_flow_runtime::node::SYSTEM_ACTION_FACTORY_MAPPINGS;
use serde::{Deserialize, Serialize};

use crate::{
    factory::{
        BUILTIN_ACTION_FACTORIES, PLATEAU_ACTION_FACTORIES, PYTHON_ACTION_FACTORIES,
        WASM_ACTION_FACTORIES,
    },
    utils::{create_action_schema, ActionSchema, I18nSchema},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RootActionSchema {
    pub actions: Vec<ActionSchema>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RootI18nSchema {
    pub actions: Vec<I18nSchema>,
}

pub fn build_schema_action_command() -> Command {
    Command::new("schema-action")
        .about("Show action schema.")
        .long_about("Show action schema.")
        .arg(language_file_path_cli_arg())
}

fn language_file_path_cli_arg() -> Arg {
    Arg::new("language_file_path")
        .long("language-file-path")
        .help("Language file path")
        .required(false)
        .display_order(1)
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaActionCliCommand {
    pub language_file_path: Option<String>,
}

impl SchemaActionCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let language_file_path = matches
            .try_remove_one::<String>("language_file_path")
            .unwrap_or_default();
        Ok(SchemaActionCliCommand { language_file_path })
    }

    pub fn execute(&self) -> crate::Result<()> {
        let i18n = if let Some(language_file_path) = &self.language_file_path {
            let file = File::open(language_file_path)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            let reader = BufReader::new(file);
            let i18n: RootI18nSchema = serde_json::from_reader(reader)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            i18n.actions
                .into_iter()
                .map(|action| (action.name.clone(), action.clone()))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };
        let mut builtin_action_factories = HashMap::new();
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
        let wasm_actions = WASM_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false, &i18n))
            .collect::<Vec<_>>();
        actions.extend(wasm_actions);
        let python_actions = PYTHON_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false, &i18n))
            .collect::<Vec<_>>();
        actions.extend(python_actions);
        actions.sort_by(|a, b| a.name.cmp(&b.name));
        let root = RootActionSchema { actions };
        println!("{}", serde_json::to_string_pretty(&root).unwrap());
        Ok(())
    }
}
