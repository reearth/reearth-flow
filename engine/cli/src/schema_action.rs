use std::{collections::HashMap, fs::File, io::BufReader};

use clap::{Arg, ArgMatches, Command};
use reearth_flow_runtime::node::SYSTEM_ACTION_FACTORY_MAPPINGS;
use serde::{Deserialize, Serialize};

use crate::{
    factory::{
        find_action_by_name, BUILTIN_ACTION_FACTORIES, PLATEAU_ACTION_FACTORIES,
        PYTHON_ACTION_FACTORIES,
    },
    utils::{create_action_schema, create_action_schema_with_params, ActionSchema, I18nSchema},
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
        .arg(action_name_cli_arg())
        .arg(with_cli_arg())
        .arg(with_file_cli_arg())
}

fn language_file_path_cli_arg() -> Arg {
    Arg::new("language_file_path")
        .long("language-file-path")
        .help("Language file path")
        .required(false)
        .display_order(1)
}

fn action_name_cli_arg() -> Arg {
    Arg::new("action")
        .long("action")
        .help("Name of the action to generate schema for (e.g. OutputRouter)")
        .required(false)
        .display_order(2)
}

fn with_cli_arg() -> Arg {
    Arg::new("with")
        .long("with")
        .help("Inline JSON string of action parameters")
        .required(false)
        .display_order(3)
}

fn with_file_cli_arg() -> Arg {
    Arg::new("with_file")
        .long("with-file")
        .help("Path to a JSON or YAML file containing action parameters")
        .required(false)
        .display_order(4)
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaActionCliCommand {
    pub language_file_path: Option<String>,
    pub action: Option<String>,
    pub with_json: Option<String>,
    pub with_file: Option<String>,
}

impl SchemaActionCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let language_file_path = matches
            .try_remove_one::<String>("language_file_path")
            .unwrap_or_default();
        let action = matches
            .try_remove_one::<String>("action")
            .unwrap_or_default();
        let with_json = matches.try_remove_one::<String>("with").unwrap_or_default();
        let with_file = matches
            .try_remove_one::<String>("with_file")
            .unwrap_or_default();
        Ok(SchemaActionCliCommand {
            language_file_path,
            action,
            with_json,
            with_file,
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        if self.action.is_none() && (self.with_json.is_some() || self.with_file.is_some()) {
            return Err(crate::errors::Error::parse(
                "The --with and --with-file flags require --action to be specified",
            ));
        }

        let i18n = self.load_i18n()?;

        if let Some(action_name) = &self.action {
            self.execute_single_action(action_name, &i18n)
        } else {
            self.execute_all_actions(&i18n)
        }
    }

    fn load_i18n(&self) -> crate::Result<HashMap<String, I18nSchema>> {
        if let Some(language_file_path) = &self.language_file_path {
            let file = File::open(language_file_path)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            let reader = BufReader::new(file);
            let i18n: RootI18nSchema = serde_json::from_reader(reader)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            Ok(i18n
                .actions
                .into_iter()
                .map(|action| (action.name.clone(), action.clone()))
                .collect::<HashMap<_, _>>())
        } else {
            Ok(HashMap::new())
        }
    }

    fn execute_all_actions(&self, i18n: &HashMap<String, I18nSchema>) -> crate::Result<()> {
        let mut builtin_action_factories = HashMap::new();
        builtin_action_factories.extend(BUILTIN_ACTION_FACTORIES.clone());
        builtin_action_factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
        let mut actions = builtin_action_factories
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, true, i18n))
            .collect::<Vec<_>>();
        let plateau_actions = PLATEAU_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false, i18n))
            .collect::<Vec<_>>();
        actions.extend(plateau_actions);
        let python_actions = PYTHON_ACTION_FACTORIES
            .clone()
            .values()
            .map(|kind| create_action_schema(kind, false, i18n))
            .collect::<Vec<_>>();
        actions.extend(python_actions);
        actions.sort_by(|a, b| a.name.cmp(&b.name));
        let root = RootActionSchema { actions };
        println!("{}", serde_json::to_string_pretty(&root).unwrap());
        Ok(())
    }

    fn execute_single_action(
        &self,
        action_name: &str,
        i18n: &HashMap<String, I18nSchema>,
    ) -> crate::Result<()> {
        let user_with = self.load_user_with()?;

        let (kind, builtin) = find_action_by_name(action_name).ok_or_else(|| {
            crate::errors::Error::init(format!(
                "Action '{}' not found. Run schema-action without --action to see all available actions.",
                action_name
            ))
        })?;

        let schema = create_action_schema_with_params(&kind, builtin, i18n, &user_with);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }

    fn load_user_with(&self) -> crate::Result<HashMap<String, serde_json::Value>> {
        let mut result = HashMap::new();

        if let Some(path) = &self.with_file {
            let content = std::fs::read_to_string(path).map_err(|e| {
                crate::errors::Error::init(format!("Failed to read with-file '{}': {}", path, e))
            })?;
            let file_params: HashMap<String, serde_json::Value> =
                if path.ends_with(".yaml") || path.ends_with(".yml") {
                    serde_yaml::from_str(&content).map_err(|e| {
                        crate::errors::Error::init(format!("Failed to parse YAML with-file: {}", e))
                    })?
                } else {
                    serde_json::from_str(&content).map_err(|e| {
                        crate::errors::Error::init(format!("Failed to parse JSON with-file: {}", e))
                    })?
                };
            result.extend(file_params);
        }

        if let Some(json_str) = &self.with_json {
            let inline_params: HashMap<String, serde_json::Value> = serde_json::from_str(json_str)
                .map_err(|e| {
                    crate::errors::Error::init(format!("Failed to parse --with JSON: {}", e))
                })?;
            result.extend(inline_params);
        }

        Ok(result)
    }
}
