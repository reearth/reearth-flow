use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

use crate::{
    factory::{BUILTIN_ACTION_FACTORIES, PLATEAU_ACTION_FACTORIES, PYTHON_ACTION_FACTORIES},
    schema_action::RootI18nSchema,
    utils::{ActionSchema, I18nSchema, PropertyI18n},
};
use clap::{Arg, ArgMatches, Command};
use reearth_flow_runtime::node::SYSTEM_ACTION_FACTORY_MAPPINGS;

/// Extracts a `PropertyI18n` from a JSON Schema node's `title` and `description` fields.
fn property_i18n_from_schema(node: &serde_json::Value) -> PropertyI18n {
    PropertyI18n {
        title: node.get("title").and_then(|v| v.as_str()).map(str::to_string),
        description: node
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    }
}

/// Collects top-level property names and their existing title/description from `schema["properties"]`.
fn collect_top_level_properties(parameter: &serde_json::Value) -> Vec<(String, PropertyI18n)> {
    parameter
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), property_i18n_from_schema(v)))
                .collect()
        })
        .unwrap_or_default()
}

/// Collects definitions that use oneOf/anyOf with enum values, returning
/// `def_name → [(enum_value, PropertyI18n)]` for seeding `enumI18n`.
fn collect_enum_definitions(
    parameter: &serde_json::Value,
) -> BTreeMap<String, Vec<(String, PropertyI18n)>> {
    parameter
        .get("definitions")
        .and_then(|d| d.as_object())
        .map(|defs| {
            defs.iter()
                .filter_map(|(def_name, def_schema)| {
                    let variants = ["oneOf", "anyOf"].iter().find_map(|kw| {
                        def_schema.get(*kw).and_then(|v| v.as_array())
                    })?;
                    let enum_variants: Vec<(String, PropertyI18n)> = variants
                        .iter()
                        .filter_map(|variant| {
                            let enum_val = variant
                                .get("enum")
                                .and_then(|e| e.as_array())
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .map(str::to_string)?;
                            Some((enum_val, property_i18n_from_schema(variant)))
                        })
                        .collect();
                    if enum_variants.is_empty() {
                        None
                    } else {
                        Some((def_name.clone(), enum_variants))
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Collects definition names and their property names + existing title/description from
/// `schema["definitions"][def_name]["properties"]`.
fn collect_definitions(
    parameter: &serde_json::Value,
) -> BTreeMap<String, Vec<(String, PropertyI18n)>> {
    parameter
        .get("definitions")
        .and_then(|d| d.as_object())
        .map(|defs| {
            defs.iter()
                .filter_map(|(def_name, def_schema)| {
                    let props = def_schema
                        .get("properties")
                        .and_then(|p| p.as_object())
                        .map(|obj| {
                            obj.iter()
                                .map(|(k, v)| (k.clone(), property_i18n_from_schema(v)))
                                .collect::<Vec<_>>()
                        })?;
                    Some((def_name.clone(), props))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Reconciles a single `I18nSchema` entry against the current action schema.
/// - Adds missing `parameterI18n` / `definitionI18n` keys, seeding title/description from the base schema.
/// - Removes stale keys that no longer exist in the schema.
/// - Preserves existing values (never overwrites what a translator has already filled in).
fn reconcile_action(existing: &mut I18nSchema, action: &ActionSchema) {
    if action.parameter.is_null() {
        existing.parameter_i18n = None;
        existing.definition_i18n = None;
        existing.enum_i18n = None;
        return;
    }

    let top_level = collect_top_level_properties(&action.parameter);
    let definitions = collect_definitions(&action.parameter);
    let enum_defs = collect_enum_definitions(&action.parameter);

    // --- parameterI18n (BTreeMap keeps keys alphabetical automatically) ---
    let mut param_i18n: BTreeMap<String, PropertyI18n> =
        existing.parameter_i18n.take().unwrap_or_default();

    // Seed root schema title/description into the "" key if not yet set
    let root_seed = property_i18n_from_schema(&action.parameter);
    if root_seed.title.is_some() || root_seed.description.is_some() {
        let root_entry = param_i18n.entry(String::new()).or_default();
        if root_entry.title.is_none() {
            root_entry.title = root_seed.title;
        }
        if root_entry.description.is_none() {
            root_entry.description = root_seed.description;
        }
    }

    // Remove stale keys (keep "" root key always if present)
    param_i18n.retain(|k, _| k.is_empty() || top_level.iter().any(|(p, _)| p == k));

    // Add missing keys or seed empty entries from the base schema's existing title/description.
    // Never overwrite a value a translator has already filled in.
    for (prop, seed) in &top_level {
        let entry = param_i18n.entry(prop.clone()).or_default();
        if entry.title.is_none() {
            entry.title = seed.title.clone();
        }
        if entry.description.is_none() {
            entry.description = seed.description.clone();
        }
    }

    existing.parameter_i18n = if param_i18n.is_empty() { None } else { Some(param_i18n) };

    // --- definitionI18n (BTreeMap keeps keys alphabetical automatically) ---
    let mut def_i18n: BTreeMap<String, BTreeMap<String, PropertyI18n>> =
        existing.definition_i18n.take().unwrap_or_default();

    // Remove stale definition names
    def_i18n.retain(|def_name, _| definitions.contains_key(def_name));

    for (def_name, props) in &definitions {
        let entry: &mut BTreeMap<String, PropertyI18n> =
            def_i18n.entry(def_name.clone()).or_default();
        // Remove stale property keys within the definition
        entry.retain(|k, _| props.iter().any(|(p, _)| p == k));
        // Add missing property keys or seed empty entries from the base schema.
        // Never overwrite a value a translator has already filled in.
        for (prop, seed) in props {
            let e = entry.entry(prop.clone()).or_default();
            if e.title.is_none() {
                e.title = seed.title.clone();
            }
            if e.description.is_none() {
                e.description = seed.description.clone();
            }
        }
    }

    existing.definition_i18n = if def_i18n.is_empty() { None } else { Some(def_i18n) };

    // --- enumI18n (oneOf/anyOf variant labels, keyed by enum value) ---
    let mut enum_i18n: BTreeMap<String, BTreeMap<String, PropertyI18n>> =
        existing.enum_i18n.take().unwrap_or_default();

    // Remove stale definition names
    enum_i18n.retain(|def_name, _| enum_defs.contains_key(def_name));

    for (def_name, variants) in &enum_defs {
        let entry: &mut BTreeMap<String, PropertyI18n> =
            enum_i18n.entry(def_name.clone()).or_default();
        // Remove stale enum values
        entry.retain(|k, _| variants.iter().any(|(v, _)| v == k));
        // Add missing enum values, seeding from base schema
        for (enum_val, seed) in variants {
            let e = entry.entry(enum_val.clone()).or_default();
            if e.title.is_none() {
                e.title = seed.title.clone();
            }
            if e.description.is_none() {
                e.description = seed.description.clone();
            }
        }
    }

    existing.enum_i18n = if enum_i18n.is_empty() { None } else { Some(enum_i18n) };
}

pub fn build_scaffold_i18n_command() -> Command {
    Command::new("scaffold-i18n")
        .about("Sync i18n skeleton files for action parameter translations.")
        .long_about(
            "Reads all {lang}.json files in the i18n directory and reconciles them \
             against the current action schemas: adds missing parameterI18n / \
             definitionI18n keys (as empty objects), removes stale keys, and \
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

        for entry in fs::read_dir(dir).map_err(|e| crate::errors::Error::init(format!("{e:?}")))? {
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
                let entry = by_name
                    .entry(action_name.clone())
                    .or_insert_with(|| I18nSchema {
                        name: action_name.clone(),
                        description: None,
                        parameter: None,
                        parameter_i18n: None,
                        definition_i18n: None,
                        enum_i18n: None,
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
