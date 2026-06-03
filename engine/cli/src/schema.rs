use std::{collections::HashMap, io};

use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;
use reearth_flow_runtime::{
    dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS,
    schema_infer::infer_with_sampling,
};
use reearth_flow_types::{
    attr_schema::{NodeReport, PortReport, SchemaReport},
    Workflow,
};
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;

use crate::factory::ALL_ACTION_FACTORIES;

const DEFAULT_SAMPLE_SIZE: usize = 10;

pub fn build_schema_command() -> Command {
    Command::new("schema")
        .about("Print per-node attribute schemas as JSON.")
        .long_about(
            "Infer and print the attribute schema produced by each node, sampling source \
             datasets to discover real attribute names and types.",
        )
        .arg(workflow_cli_arg())
        .arg(vars_arg())
        .arg(sample_size_arg())
}

fn workflow_cli_arg() -> Arg {
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

fn sample_size_arg() -> Arg {
    Arg::new("sample_size")
        .long("sample-size")
        .help("Max features read per source to infer its schema (0 = all)")
        .required(false)
        .display_order(3)
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaCliCommand {
    workflow_path: String,
    vars: HashMap<String, String>,
    sample_size: usize,
}

impl SchemaCliCommand {
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
        let sample_size = match matches.remove_one::<String>("sample_size") {
            Some(value) => value
                .parse::<usize>()
                .map_err(|e| crate::errors::Error::init(format!("Invalid sample-size: {e}")))?,
            None => DEFAULT_SAMPLE_SIZE,
        };
        Ok(SchemaCliCommand {
            workflow_path,
            vars,
            sample_size,
        })
    }

    fn build_report(&self) -> crate::Result<SchemaReport> {
        let storage_resolver = resolve::StorageResolver::new();
        let (yaml_content, base_dir) = if self.workflow_path == "-" {
            let content = io::read_to_string(io::stdin()).map_err(crate::errors::Error::init)?;
            (content, None)
        } else {
            let path = Uri::for_test(self.workflow_path.as_str());
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

        let json =
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, base_dir.as_deref())
                .map_err(crate::errors::Error::init)?;

        let mut workflow = Workflow::try_from(json.as_str()).map_err(crate::errors::Error::init)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(crate::errors::Error::init)?;

        // Capture node id -> display name before `workflow` is moved into `from_graphs`.
        // Top-level node ids match the `node_outputs`/`notes` keys
        // (`SchemaNodeType.handle.id == NodeId::new(node.id().to_string())`).
        let mut names: HashMap<String, String> = HashMap::new();
        for graph in &workflow.graphs {
            for node in &graph.nodes {
                names.insert(node.id().to_string(), node.name().to_string());
            }
        }

        let mut factories = HashMap::new();
        factories.extend(ALL_ACTION_FACTORIES.clone());
        factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());

        let dag =
            DagSchemas::from_graphs(workflow.entry_graph_id, workflow.graphs, factories, None)
                .map_err(crate::errors::Error::run)?;

        let inferred =
            infer_with_sampling(&dag, self.sample_size).map_err(crate::errors::Error::run)?;

        let mut node_ids: Vec<&String> = inferred.node_outputs.keys().collect();
        node_ids.sort();

        let mut nodes: IndexMap<String, NodeReport> = IndexMap::new();
        for id in node_ids {
            let ports_map = &inferred.node_outputs[id];
            let mut port_names: Vec<&String> = ports_map.keys().collect();
            port_names.sort();
            let mut ports: IndexMap<String, PortReport> = IndexMap::new();
            for port in port_names {
                ports.insert(port.clone(), PortReport::from_schema(&ports_map[port]));
            }
            nodes.insert(
                id.clone(),
                NodeReport {
                    name: names.get(id).cloned().unwrap_or_default(),
                    ports,
                    note: inferred.notes.get(id).cloned(),
                },
            );
        }

        Ok(SchemaReport {
            version: 1,
            sample_size: self.sample_size,
            nodes,
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        debug!(args = ?self, "schema");
        let report = self.build_report()?;
        let json = serde_json::to_string_pretty(&report).map_err(crate::errors::Error::run)?;
        println!("{json}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const FIXTURE_GEOJSON: &str = r#"{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": { "id": "a", "name": "Alpha" },
      "geometry": { "type": "Point", "coordinates": [0, 0] }
    },
    {
      "type": "Feature",
      "properties": { "id": "b", "name": "Beta", "address": "somewhere" },
      "geometry": { "type": "Point", "coordinates": [1, 1] }
    }
  ]
}"#;

    const READER_ID: &str = "11111111-1111-4111-8111-111111111111";
    const MANAGER_ID: &str = "22222222-2222-4222-8222-222222222222";
    const GRAPH_ID: &str = "33333333-3333-4333-8333-333333333333";
    const WORKFLOW_ID: &str = "44444444-4444-4444-8444-444444444444";
    const EDGE_ID: &str = "55555555-5555-4555-8555-555555555555";

    fn field_names(report: &SchemaReport, node_id: &str) -> Vec<String> {
        let node = report
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("node {node_id} present in report"));
        let port = node.ports.get("default").expect("node has a default port");
        port.fields.iter().map(|f| f.name.clone()).collect()
    }

    #[test]
    fn schema_command_samples_reader_and_reflects_attribute_removal() {
        // 1. Real geojson on disk so the GeoJsonReader can sample it.
        let mut geojson_tmp = tempfile::Builder::new()
            .suffix(".geojson")
            .tempfile()
            .expect("create temp geojson");
        geojson_tmp
            .write_all(FIXTURE_GEOJSON.as_bytes())
            .expect("write geojson fixture");
        let geojson_path = geojson_tmp
            .path()
            .to_str()
            .expect("utf8 geojson path")
            .to_string();
        let dataset_uri = format!("file://{geojson_path}");

        // 2. Inline workflow: GeoJsonReader -> AttributeManager (remove `name`).
        //    `dataset` is a quoted rhai string literal that evaluates to the URI.
        let workflow_yaml = format!(
            r#"id: {WORKFLOW_ID}
name: schema-cli-test
entryGraphId: {GRAPH_ID}
with:
graphs:
  - id: {GRAPH_ID}
    name: main
    nodes:
      - id: {READER_ID}
        name: reader
        type: action
        action: GeoJsonReader
        with:
          dataset: "\"{dataset_uri}\""
      - id: {MANAGER_ID}
        name: manager
        type: action
        action: AttributeManager
        with:
          operations:
            - attribute: name
              method: remove
    edges:
      - id: {EDGE_ID}
        from: {READER_ID}
        to: {MANAGER_ID}
        fromPort: default
        toPort: default
"#
        );

        // 3. Workflow on disk; run the command pipeline.
        let mut workflow_tmp = tempfile::Builder::new()
            .suffix(".yml")
            .tempfile()
            .expect("create temp workflow");
        workflow_tmp
            .write_all(workflow_yaml.as_bytes())
            .expect("write workflow");
        let workflow_path = workflow_tmp
            .path()
            .to_str()
            .expect("utf8 workflow path")
            .to_string();

        let cmd = SchemaCliCommand {
            workflow_path,
            vars: HashMap::new(),
            sample_size: 10,
        };
        let report = cmd.build_report().expect("build_report succeeds");

        // 4a. Reader emits the real sampled attributes.
        let reader_fields = field_names(&report, READER_ID);
        assert!(
            reader_fields.iter().any(|f| f == "id"),
            "reader default port should include sampled `id`, got {reader_fields:?}"
        );
        assert!(
            reader_fields.iter().any(|f| f == "name"),
            "reader default port should include sampled `name`, got {reader_fields:?}"
        );

        // A successfully sampled reader produces a closed schema (not the open
        // fallback). Guards against a silent regression to open schemas.
        let reader_node = report
            .nodes
            .get(READER_ID)
            .expect("reader node present in report");
        let reader_default_port = reader_node
            .ports
            .get("default")
            .expect("reader has a default port");
        assert!(
            !reader_default_port.open,
            "a sampled reader yields a closed schema"
        );

        // 4b. The AttributeManager removal propagates on top of the sampled schema:
        //     `name` is gone, but `id` (and the rest) survive.
        let manager_fields = field_names(&report, MANAGER_ID);
        assert!(
            manager_fields.iter().any(|f| f == "id"),
            "manager default port should still include `id`, got {manager_fields:?}"
        );
        assert!(
            !manager_fields.iter().any(|f| f == "name"),
            "manager default port should NOT include removed `name`, got {manager_fields:?}"
        );
    }
}
