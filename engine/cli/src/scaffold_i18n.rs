use std::{
    collections::HashMap,
    fs,
    path::Path,
};

use clap::{Arg, ArgMatches, Command};
use reearth_flow_runtime::node::SYSTEM_ACTION_FACTORY_MAPPINGS;
use crate::{
    factory::{BUILTIN_ACTION_FACTORIES, PLATEAU_ACTION_FACTORIES, PYTHON_ACTION_FACTORIES},
    schema_action::RootI18nSchema,
    utils::{ActionSchema, I18nSchema, PropertyI18n},
};

/// Collects all property names at the top level of `schema["properties"]`.
fn collect_top_level_properties(parameter: &serde_json::Value) -> Vec<String> {
    parameter
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

/// Collects all definition names and their property names from
/// `schema["definitions"][def_name]["properties"]`.
fn collect_definitions(
    parameter: &serde_json::Value,
) -> HashMap<String, Vec<String>> {
    parameter
        .get("definitions")
        .and_then(|d| d.as_object())
        .map(|defs| {
            defs.iter()
                .filter_map(|(def_name, def_schema)| {
                    let props = def_schema
                        .get("properties")
                        .and_then(|p| p.as_object())
                        .map(|obj| obj.keys().cloned().collect::<Vec<_>>())?;
                    Some((def_name.clone(), props))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Reconciles a single `I18nSchema` entry against the current action schema.
/// - Adds missing `parameterI18n` / `definitionI18n` keys with empty values.
/// - Removes stale keys that no longer exist in the schema.
/// - Preserves existing non-empty values.
fn reconcile_action(existing: &mut I18nSchema, action: &ActionSchema) {
    if action.parameter.is_null() {
        existing.parameter_i18n = None;
        existing.definition_i18n = None;
        return;
    }

    let top_level = collect_top_level_properties(&action.parameter);
    let definitions = collect_definitions(&action.parameter);

    // --- parameterI18n ---
    let mut param_i18n = existing
        .parameter_i18n
        .take()
        .unwrap_or_default();

    // Remove stale keys (keep "" root key always if present)
    param_i18n.retain(|k, _| k.is_empty() || top_level.contains(k));

    // Add missing keys
    for prop in &top_level {
        param_i18n.entry(prop.clone()).or_insert_with(PropertyI18n::default);
    }

    existing.parameter_i18n = if param_i18n.is_empty() {
        None
    } else {
        // Stable sort: root key "" first, then alphabetical
        let mut sorted: Vec<(String, PropertyI18n)> = param_i18n.into_iter().collect();
        sorted.sort_by(|a, b| match (a.0.is_empty(), b.0.is_empty()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });
        Some(sorted.into_iter().collect())
    };

    // --- definitionI18n ---
    let mut def_i18n = existing
        .definition_i18n
        .take()
        .unwrap_or_default();

    // Remove stale definition names
    def_i18n.retain(|def_name, _| definitions.contains_key(def_name));

    for (def_name, props) in &definitions {
        let entry = def_i18n.entry(def_name.clone()).or_insert_with(HashMap::new);
        // Remove stale property keys within the definition
        entry.retain(|k, _| props.contains(k));
        // Add missing property keys
        for prop in props {
            entry.entry(prop.clone()).or_insert_with(PropertyI18n::default);
        }
    }

    existing.definition_i18n = if def_i18n.is_empty() {
        None
    } else {
        // Sort definition entries alphabetically for stable output
        let mut sorted_defs: Vec<(String, HashMap<String, PropertyI18n>)> =
            def_i18n.into_iter().collect();
        sorted_defs.sort_by(|a, b| a.0.cmp(&b.0));
        Some(
            sorted_defs
                .into_iter()
                .map(|(def_name, props)| {
                    let mut sorted_props: Vec<(String, PropertyI18n)> =
                        props.into_iter().collect();
                    sorted_props.sort_by(|a, b| a.0.cmp(&b.0));
                    (def_name, sorted_props.into_iter().collect())
                })
                .collect(),
        )
    };
}

pub fn build_scaffold_i18n_command() -> Command {
    Command::new("scaffold-i18n")
        .about("Sync i18n skeleton files for action parameter translations.")
        .long_about(
            "Reads all {lang}.json files in the i18n directory and reconciles them \
             against the current action schemas: adds missing parameterI18n / \
             definitionI18n keys (with empty strings), removes stale keys, and \
             preserves existing translations.",
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .help("Path to the i18n actions directory")
                .default_value("./schema/i18n/actions")
                .display_order(1),
        )
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScaffoldI18nCliCommand {
    pub dir: String,
}

impl ScaffoldI18nCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let dir = matches
            .remove_one::<String>("dir")
            .unwrap_or_else(|| "./schema/i18n/actions".to_string());
        Ok(ScaffoldI18nCliCommand { dir })
    }

    pub fn execute(&self) -> crate::Result<()> {
        // Build the full set of current action schemas (no i18n applied)
        let mut builtin = HashMap::new();
        builtin.extend(BUILTIN_ACTION_FACTORIES.clone());
        builtin.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());

        let empty_i18n: HashMap<String, I18nSchema> = HashMap::new();

        let mut actions: HashMap<String, ActionSchema> = builtin
            .values()
            .map(|kind| {
                let s = crate::utils::create_action_schema(kind, true, &empty_i18n);
                (s.name.clone(), s)
            })
            .collect();

        for (name, kind) in PLATEAU_ACTION_FACTORIES.clone().iter() {
            let s = crate::utils::create_action_schema(kind, false, &empty_i18n);
            actions.insert(name.to_string(), s);
        }
        for (name, kind) in PYTHON_ACTION_FACTORIES.clone().iter() {
            let s = crate::utils::create_action_schema(kind, false, &empty_i18n);
            actions.insert(name.to_string(), s);
        }

        let dir = Path::new(&self.dir);
        if !dir.exists() {
            return Err(crate::errors::Error::init(format!(
                "i18n directory not found: {}",
                dir.display()
            )));
        }

        for entry in fs::read_dir(dir)
            .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?
        {
            let entry = entry.map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = fs::read_to_string(&path)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            let mut root: RootI18nSchema = serde_json::from_str(&content)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;

            // Build a lookup of existing entries by action name
            let mut by_name: HashMap<String, I18nSchema> = root
                .actions
                .into_iter()
                .map(|a| (a.name.clone(), a))
                .collect();

            // Reconcile each known action
            for (action_name, action_schema) in &actions {
                let entry = by_name.entry(action_name.clone()).or_insert_with(|| I18nSchema {
                    name: action_name.clone(),
                    description: String::new(),
                    parameter: None,
                    parameter_i18n: None,
                    definition_i18n: None,
                });
                reconcile_action(entry, action_schema);
            }

            // Remove entries for actions that no longer exist
            by_name.retain(|name, _| actions.contains_key(name));

            // Re-sort alphabetically for stable output
            let mut updated: Vec<I18nSchema> = by_name.into_values().collect();
            updated.sort_by(|a, b| a.name.cmp(&b.name));

            root = RootI18nSchema { actions: updated };

            let output = serde_json::to_string_pretty(&root)
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;
            fs::write(&path, output + "\n")
                .map_err(|e| crate::errors::Error::init(format!("{e:?}")))?;

            eprintln!("  updated {}", path.display());
        }

        Ok(())
    }
}
