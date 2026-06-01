use std::{collections::HashMap, io};

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_runtime::schema_infer::{self, InferResult, Severity};
use reearth_flow_runtime::{dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS};
use reearth_flow_types::attr_schema::Presence;
use reearth_flow_types::Workflow;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;

pub fn build_build_command() -> Command {
    Command::new("build")
        .visible_alias("check")
        .about("Statically validate a workflow's attribute schemas (no execution).")
        .long_about("Statically validate a workflow's attribute schemas (no execution).")
        .arg(build_cli_arg())
        .arg(vars_arg())
        .arg(show_schema_arg())
}

fn build_cli_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
        .env("REEARTH_FLOW_WORKFLOW")
        .required(true)
        .display_order(1)
}

fn vars_arg() -> Arg {
    Arg::new("var")
        .long("var")
        .help("Workflow variables")
        .required(false)
        .action(ArgAction::Append)
        .display_order(2)
}

fn show_schema_arg() -> Arg {
    Arg::new("show-schema")
        .long("show-schema")
        .action(ArgAction::SetTrue)
        .help("Print the inferred attribute schema per node to stdout.")
        .display_order(3)
}

#[derive(Debug, Eq, PartialEq)]
pub struct BuildCliCommand {
    workflow_path: String,
    vars: HashMap<String, String>,
    show_schema: bool,
}

impl BuildCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let workflow_path = matches
            .remove_one::<String>("workflow")
            .ok_or(crate::errors::Error::init("No workflow uri provided"))?;
        let vars = matches.remove_many::<String>("var");
        let vars = if let Some(vars) = vars {
            vars.into_iter()
                .flat_map(|v| {
                    let parts: Vec<&str> = v.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            HashMap::<String, String>::new()
        };
        let show_schema = matches.get_flag("show-schema");
        Ok(BuildCliCommand {
            workflow_path,
            vars,
            show_schema,
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        let result = self.infer()?;

        if self.show_schema {
            print_node_schemas(&result);
        }

        let mut error_count = 0usize;
        let mut warning_count = 0usize;
        for diagnostic in &result.diagnostics {
            let severity = match diagnostic.severity {
                Severity::Error => {
                    error_count += 1;
                    "ERROR"
                }
                Severity::Warning => {
                    warning_count += 1;
                    "WARNING"
                }
            };
            eprintln!(
                "[{severity}] {} ({}): {}",
                diagnostic.node_name, diagnostic.node_id, diagnostic.message
            );
        }
        eprintln!("{error_count} error(s), {warning_count} warning(s)");

        if result.has_errors() {
            return Err(crate::errors::Error::run(format!(
                "workflow validation failed: {error_count} error(s)"
            )));
        }
        println!("\u{2714} workflow valid");
        Ok(())
    }

    /// Load + expand + parse the workflow, build the static DAG, and run the
    /// schema-inference pass. Returns the full result (diagnostics + per-node
    /// schemas) without printing or deciding the exit code — `execute` does that,
    /// and tests can inspect the result directly.
    fn infer(&self) -> crate::Result<InferResult> {
        debug!(args = ?self, "build");
        let storage_resolver = resolve::StorageResolver::new();
        let (yaml_content, base_dir) = if self.workflow_path == "-" {
            let content = io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?;
            (content, None)
        } else {
            let path = Uri::for_test(self.workflow_path.as_str());

            // Extract base directory for !include resolution
            let base_dir = path.path().parent().map(|p| p.to_path_buf());

            let storage = storage_resolver
                .resolve(&path)
                .map_err(crate::errors::Error::init)?;
            let bytes = storage
                .get_sync(path.path().as_path())
                .map_err(crate::errors::Error::init)?;
            let content = String::from_utf8(bytes.to_vec()).map_err(crate::errors::Error::init)?;
            (content, base_dir)
        };

        // Expand !include directives
        let json = if let Some(base) = base_dir.as_ref() {
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, Some(base))
                .map_err(crate::errors::Error::init)?
        } else {
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, None)
                .map_err(crate::errors::Error::init)?
        };

        let mut factories = HashMap::new();
        factories.extend(ALL_ACTION_FACTORIES.clone());
        factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
        let mut workflow = Workflow::try_from(json.as_str()).map_err(crate::errors::Error::init)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(crate::errors::Error::init)?;
        // NOTE: `DagSchemas::from_graphs` currently `panic!`s on structural
        // malformations (unknown action name, dangling edge endpoints, missing
        // entry/subgraph). A validation command should ideally report those as
        // clean diagnostics instead of aborting; hardening those panics into
        // recoverable errors is deferred. Until then `build` shares `dot`'s
        // panic-on-malformed-input behavior.
        let dag =
            DagSchemas::from_graphs(workflow.entry_graph_id, workflow.graphs, factories, None)
                .map_err(crate::errors::Error::run)?;

        let result = schema_infer::infer_and_validate(&dag)
            .map_err(|e| crate::errors::Error::run(e.to_string()))?;
        Ok(result)
    }
}

/// Print the statically-inferred per-node attribute schemas to stdout in a
/// stable, greppable form. Node ids and port names are sorted for determinism;
/// fields keep their IndexMap insertion order.
fn print_node_schemas(result: &InferResult) {
    let mut node_ids: Vec<&String> = result.node_outputs.keys().collect();
    node_ids.sort();
    for node_id in node_ids {
        let ports = &result.node_outputs[node_id];
        if ports.is_empty() {
            println!("node {node_id}: (no output schema)");
            continue;
        }
        println!("node {node_id}:");
        let mut port_names: Vec<&String> = ports.keys().collect();
        port_names.sort();
        for port_name in port_names {
            let schema = &ports[port_name];
            let open = if schema.open { " (open)" } else { "" };
            println!("  port {port_name}{open}:");
            for (attr, field) in &schema.fields {
                let maybe = if field.presence == Presence::Maybe {
                    " ?"
                } else {
                    ""
                };
                println!("    {attr}: {:?}{maybe}", field.ty);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> String {
        // CARGO_MANIFEST_DIR for the cli crate is <repo>/engine/cli.
        format!(
            "{}/../runtime/tests/fixture/workflow/schema_infer/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        )
    }

    #[test]
    fn build_valid_workflow_succeeds() {
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("valid.yml"),
            vars: HashMap::new(),
            show_schema: false,
        };
        assert!(
            cmd.execute().is_ok(),
            "valid workflow should pass validation"
        );
    }

    #[test]
    fn build_reference_error_fails_end_to_end() {
        // AttributeMapper has REPLACE semantics → its output schema is CLOSED, so
        // a downstream reference to an attribute it does not produce is a hard
        // ERROR reachable from a real workflow (not just a stub test).
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("reference_error.yml"),
            vars: HashMap::new(),
            show_schema: false,
        };
        // execute() must return Err (→ non-zero exit).
        assert!(
            cmd.execute().is_err(),
            "a reference to an attribute no upstream node produces (against a closed \
             schema) must fail validation"
        );
        // And the diagnostic must be a single Error naming the missing attribute.
        let result = cmd.infer().expect("inference itself should succeed");
        assert!(result.has_errors());
        let errors: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1, "exactly one error expected");
        assert!(
            errors[0].message.contains("perimeter"),
            "error should name the missing attribute, got: {}",
            errors[0].message
        );
    }

    #[test]
    fn build_reference_ok_passes_end_to_end() {
        // The downstream node references `name`, which the AttributeMapper DOES
        // produce — guards against false positives.
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("reference_ok.yml"),
            vars: HashMap::new(),
            show_schema: false,
        };
        assert!(
            cmd.execute().is_ok(),
            "a satisfied reference must not be flagged"
        );
        let result = cmd.infer().unwrap();
        assert!(!result.has_errors());
        assert!(
            result.diagnostics.is_empty(),
            "no diagnostics expected, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn build_typed_reference_error_with_known_types() {
        // StatisticsCalculator's `default` port emits a fresh CLOSED *typed* schema
        // (group_by → String, calculations → Number). This proves the validator
        // catches a missing reference end-to-end AND that the surviving fields carry
        // concrete inferred types — not just `Unknown`.
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("typed_reference_error.yml"),
            vars: HashMap::new(),
            show_schema: false,
        };
        assert!(
            cmd.execute().is_err(),
            "reference to an attribute the aggregation does not produce must fail"
        );
        let result = cmd.infer().expect("inference itself should succeed");
        // The error names the missing attribute.
        let errors: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("average"));
        // The Stats node's `default` output carries KNOWN types: region=String, total=Number.
        let stats_out = result
            .node_outputs
            .get("44444444-4444-4444-4444-444444444444")
            .expect("stats node should have inferred output");
        let default = stats_out.get("default").expect("default port present");
        assert!(!default.open, "aggregation output schema is closed");
        use reearth_flow_types::attr_schema::AttrType;
        use reearth_flow_types::Attribute;
        assert_eq!(
            default
                .fields
                .get(&Attribute::new("region".to_string()))
                .map(|f| f.ty),
            Some(AttrType::String),
            "group_by attribute should be inferred as String"
        );
        assert_eq!(
            default
                .fields
                .get(&Attribute::new("total".to_string()))
                .map(|f| f.ty),
            Some(AttrType::Number),
            "calculation result should be inferred as Number"
        );
    }

    #[test]
    fn build_reference_warning_passes_with_warning() {
        // The parent/child mapper records `maybeAttr` with Presence::Maybe; a
        // reference to it is a WARNING (may not always be present), not an error,
        // so the build still succeeds.
        let cmd = BuildCliCommand {
            workflow_path: fixture_path("reference_warning.yml"),
            vars: HashMap::new(),
            show_schema: false,
        };
        assert!(
            cmd.execute().is_ok(),
            "a Maybe-presence reference warns but does not fail the build"
        );
        let result = cmd.infer().unwrap();
        assert!(!result.has_errors(), "warning must not be an error");
        let warnings: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect();
        assert_eq!(warnings.len(), 1, "exactly one warning expected");
        assert!(warnings[0].message.contains("maybeAttr"));
    }

    #[test]
    fn build_plateau_quality_check_workflow_completes() {
        // The real PLATEAU bldg quality-check workflow uses `!include` directives
        // and `with:` vars. This proves the static pass scales end-to-end: the
        // command loads the include-based workflow, expands includes, builds the
        // full subgraph-expanded DAG (~70 nodes, routers, mergers, 20+ action
        // types), runs `infer_and_validate` without panicking, and exits cleanly.
        // CARGO_MANIFEST_DIR for the cli crate is <repo>/engine/cli.
        let workflow_path = format!(
            "{}/../worker/workflow/cms/plateau4/quality-check/bldg/template/workflow.yml",
            env!("CARGO_MANIFEST_DIR")
        );
        assert!(
            std::path::Path::new(&workflow_path).exists(),
            "PLATEAU quality-check workflow fixture should exist at {workflow_path}"
        );
        let cmd = BuildCliCommand {
            workflow_path,
            vars: HashMap::new(),
            show_schema: false,
        };
        assert!(
            cmd.execute().is_ok(),
            "real PLATEAU quality-check workflow should load, expand includes, and validate cleanly"
        );
    }

    #[test]
    fn build_plateau_with_show_schema_succeeds() {
        // Same real PLATEAU bldg quality-check workflow as the e2e test above,
        // but with `--show-schema` enabled. We can't easily assert on stdout from
        // inside `execute()`, so this just proves the schema-dump path doesn't
        // panic or regress on the full subgraph-expanded DAG and still exits Ok.
        let workflow_path = format!(
            "{}/../worker/workflow/cms/plateau4/quality-check/bldg/template/workflow.yml",
            env!("CARGO_MANIFEST_DIR")
        );
        assert!(
            std::path::Path::new(&workflow_path).exists(),
            "PLATEAU quality-check workflow fixture should exist at {workflow_path}"
        );
        let cmd = BuildCliCommand {
            workflow_path,
            vars: HashMap::new(),
            show_schema: true,
        };
        assert!(
            cmd.execute().is_ok(),
            "PLATEAU quality-check workflow should validate cleanly with --show-schema"
        );
    }
}
