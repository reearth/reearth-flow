use std::{collections::HashMap, io, str::FromStr};

use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;
use reearth_flow_common::uri::{Protocol, Uri};
use reearth_flow_runtime::{
    dag_schemas::DagSchemas, node::SYSTEM_ACTION_FACTORY_MAPPINGS,
    schema_infer::infer_with_sampling,
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{
    attr_schema::{NodeReport, PortReport, SchemaReport},
    Workflow,
};
use tracing::debug;

use crate::factory::ALL_ACTION_FACTORIES;
use reearth_flow_worker::errors::{Error, Result};
use reearth_flow_worker::pubsub::{PubSubBackend, Publisher};
use reearth_flow_worker::types::job_complete_event::{JobCompleteEvent, JobResult};
use uuid::Uuid;

const DEFAULT_SAMPLE_SIZE: usize = 10;

pub fn build_probe_schema_command() -> Command {
    Command::new("probe-schema")
        .about(
            "Probe a workflow's data to infer per-node attribute schemas and write a JSON report.",
        )
        .long_about(
            "Infer the attribute schema produced by each node by PROBING the actual data: the \
             workflow's source readers are run against their datasets (sampling the first N \
             features) to discover real attribute names and types, then transforms are propagated \
             through the DAG. The resulting JSON report is written to `--report-url` (e.g. a \
             gs:// URI) via the storage resolver; the report itself is not printed to stdout \
             (worker logs still go to stdout).",
        )
        .arg(workflow_arg())
        .arg(vars_arg())
        .arg(sample_size_arg())
        .arg(report_url_arg())
        .arg(job_id_arg())
        .arg(pubsub_backend_arg())
}

fn workflow_arg() -> Arg {
    Arg::new("workflow")
        .long("workflow")
        .help("Workflow file location. Use '-' to read from stdin.")
        .env("FLOW_WORKER_WORKFLOW")
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

fn report_url_arg() -> Arg {
    Arg::new("report_url")
        .long("report-url")
        .help("Destination URI (e.g. gs://...) for the JSON schema report")
        .required(true)
        .display_order(4)
}

fn job_id_arg() -> Arg {
    Arg::new("job_id")
        .long("job-id")
        .help("Job id; published in the completion event so the server finalizes the job")
        .required(true)
        .display_order(5)
}

fn pubsub_backend_arg() -> Arg {
    Arg::new("pubsub_backend")
        .long("pubsub-backend")
        .help("Pub/Sub backend for the job-complete event (google|noop)")
        .env("FLOW_WORKER_PUBSUB_BACKEND")
        .default_value("google")
        .required(false)
        .display_order(6)
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProbeSchemaCommand {
    workflow: String,
    vars: HashMap<String, String>,
    sample_size: usize,
    report_url: String,
    job_id: Uuid,
    pubsub_backend: String,
}

impl ProbeSchemaCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> Result<Self> {
        let workflow = matches
            .remove_one::<String>("workflow")
            .ok_or(Error::init("No workflow uri provided"))?;
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
                .map_err(|e| Error::init(format!("Invalid sample-size: {e}")))?,
            None => DEFAULT_SAMPLE_SIZE,
        };
        let report_url = matches
            .remove_one::<String>("report_url")
            .ok_or(Error::init("No report url provided"))?;
        let job_id = matches
            .remove_one::<String>("job_id")
            .ok_or(Error::init("No job id provided"))?;
        let job_id =
            Uuid::from_str(&job_id).map_err(|e| Error::init(format!("Invalid job-id: {e}")))?;
        let pubsub_backend = matches
            .remove_one::<String>("pubsub_backend")
            .unwrap_or_else(|| "google".to_string());
        Ok(Self {
            workflow,
            vars,
            sample_size,
            report_url,
            job_id,
            pubsub_backend,
        })
    }

    /// Build the schema report. Mirrors `cli::probe_schema::build_report` but
    /// uses the worker's `ALL_ACTION_FACTORIES`.
    fn build_report(&self, storage_resolver: &StorageResolver) -> Result<(Uuid, SchemaReport)> {
        let (yaml_content, base_dir) = if self.workflow == "-" {
            let content = io::read_to_string(io::stdin()).map_err(Error::init)?;
            (content, None)
        } else {
            let path = Uri::from_str(self.workflow.as_str()).map_err(Error::init)?;
            // Only derive a base directory for local `file://` workflows. For
            // remote workflows (e.g. `gs://`) leave it `None` so `!include`
            // expansion cannot read worker-local files (incl. via `..`
            // traversal). Mirrors `RunWorkerCommand::download_workflow`.
            let base_dir = if path.protocol() == Protocol::File {
                path.path().parent().map(|p| p.to_path_buf())
            } else {
                None
            };
            let storage = storage_resolver.resolve(&path).map_err(Error::init)?;
            let bytes = storage
                .get_sync(path.path().as_path())
                .map_err(Error::init)?;
            let content = String::from_utf8(bytes.to_vec()).map_err(Error::init)?;
            (content, base_dir)
        };

        let json =
            reearth_flow_common::serde::expand_yaml_includes(&yaml_content, base_dir.as_deref())
                .map_err(Error::init)?;

        let mut workflow = Workflow::try_from(json.as_str()).map_err(Error::init)?;
        workflow
            .merge_with(self.vars.clone())
            .map_err(Error::init)?;

        // Captured before `workflow` is moved into `from_graphs`; used in the event.
        let workflow_id = workflow.id;

        // Capture node id -> display name before `workflow` is moved into `from_graphs`.
        let mut names: HashMap<String, String> = HashMap::new();
        for graph in &workflow.graphs {
            for node in &graph.nodes {
                names.insert(node.id().to_string(), node.name().to_string());
            }
        }

        // Global `with:` vars (already merged with `--var` above) seed the
        // sampling engine so source `dataset` expressions resolve as under `run`.
        let vars = workflow.with.clone().unwrap_or_default();

        let mut factories = HashMap::new();
        factories.extend(ALL_ACTION_FACTORIES.clone());
        factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());

        let dag =
            DagSchemas::from_graphs(workflow.entry_graph_id, workflow.graphs, factories, None)
                .map_err(Error::run)?;

        let inferred = infer_with_sampling(&dag, self.sample_size, vars).map_err(Error::run)?;

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

        Ok((
            workflow_id,
            SchemaReport {
                version: 1,
                sample_size: self.sample_size,
                nodes,
            },
        ))
    }

    pub fn execute(&self) -> Result<()> {
        debug!(args = ?self, "probe-schema");
        let storage_resolver = StorageResolver::new();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(Error::FailedToCreateTokioRuntime)?;

        let outcome = self.run_probe(&storage_resolver, &runtime);
        let (workflow_id, result) = match &outcome {
            Ok(id) => (*id, JobResult::Success),
            Err(_) => (Uuid::nil(), JobResult::Failed),
        };
        let published = runtime.block_on(self.publish_complete(workflow_id, result));

        // This event is the only thing that finalizes the job, so on a
        // successful probe a publish failure is fatal: swallowing it would
        // report COMPLETED to the server while it never sees the event, leaving
        // the job PENDING. A probe failure is the primary error; publishing its
        // failed event is best effort.
        outcome?;
        published
    }

    /// Run the probe and write the report. Returns the workflow id on success.
    fn run_probe(
        &self,
        storage_resolver: &StorageResolver,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<Uuid> {
        let (workflow_id, report) = self.build_report(storage_resolver)?;
        let json = serde_json::to_vec_pretty(&report).map_err(Error::run)?;

        let report_uri = Uri::from_str(self.report_url.as_str()).map_err(Error::init)?;
        let storage = storage_resolver.resolve(&report_uri).map_err(Error::init)?;
        // Write via the async `put`: object-store backends like GCS do not
        // implement OpenDAL's blocking layer, so `put_sync` fails with
        // `Unsupported (persistent) at blocking_write`.
        runtime.block_on(async {
            storage
                .put(report_uri.path().as_path(), json.into())
                .await
                .map_err(Error::run)
        })?;
        tracing::info!("Wrote probe-schema report to {}", self.report_url);
        Ok(workflow_id)
    }

    /// Publish the terminal JobCompleteEvent on the same Pub/Sub backend a run
    /// uses, so the server's monitoring loop finalizes the preview job.
    async fn publish_complete(&self, workflow_id: Uuid, result: JobResult) -> Result<()> {
        let pubsub = PubSubBackend::try_from(self.pubsub_backend.as_str())
            .await
            .map_err(|e| Error::run(format!("{e:?}")))?;
        let event = JobCompleteEvent::new(workflow_id, self.job_id, result);
        match pubsub {
            PubSubBackend::Google(p) => p.publish(event).await.map_err(Error::run),
            PubSubBackend::Noop(p) => p
                .publish(event)
                .await
                .map_err(|e| Error::run(format!("{e:?}"))),
        }
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

    #[test]
    fn execute_writes_report_to_report_url() {
        // Real geojson on disk so the GeoJsonReader can sample it.
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

        let workflow_yaml = format!(
            r#"id: {WORKFLOW_ID}
name: probe-worker-test
entryGraphId: {GRAPH_ID}
with:
graphs:
  - id: {GRAPH_ID}
    name: main
    nodes:
      - id: {READER_ID}
        name: reader
        type: action
        action: GeoJSON Reader
        with:
          dataset:
            type: string
            value: {dataset_uri}
      - id: {MANAGER_ID}
        name: manager
        type: action
        action: Attribute Manager
        with:
          operations:
            - attribute: name
              method: remove
    edges:
      - id: {EDGE_ID}
        from: {READER_ID}
        to: {MANAGER_ID}
        fromPort: features
        toPort: features
"#
        );

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

        // Report destination: a plain file path on disk.
        let report_tmp = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .expect("create temp report");
        let report_path = report_tmp
            .path()
            .to_str()
            .expect("utf8 report path")
            .to_string();
        let report_url = format!("file://{report_path}");

        let cmd = ProbeSchemaCommand {
            workflow: workflow_path,
            vars: HashMap::new(),
            sample_size: 10,
            report_url,
            job_id: Uuid::from_str(WORKFLOW_ID).expect("valid uuid"),
            // noop backend: execute() publishes a (no-op) completion event.
            pubsub_backend: "noop".to_string(),
        };
        cmd.execute().expect("probe-schema execute succeeds");

        // The report was written to disk; parse it back and verify the shape.
        let written = std::fs::read_to_string(&report_path).expect("report written to report_url");
        let value: serde_json::Value =
            serde_json::from_str(&written).expect("report is valid JSON");
        assert_eq!(value["version"], 1);
        assert_eq!(value["sampleSize"], 10);

        let reader = &value["nodes"][READER_ID]["ports"]["features"]["fields"];
        let reader_names: Vec<String> = reader
            .as_array()
            .expect("reader fields array")
            .iter()
            .map(|f| f["name"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(
            reader_names.iter().any(|f| f == "id"),
            "reader default port should include sampled `id`, got {reader_names:?}"
        );
        assert!(
            reader_names.iter().any(|f| f == "name"),
            "reader default port should include sampled `name`, got {reader_names:?}"
        );

        // AttributeManager removal propagates: `name` gone, `id` survives.
        let manager = &value["nodes"][MANAGER_ID]["ports"]["features"]["fields"];
        let manager_names: Vec<String> = manager
            .as_array()
            .expect("manager fields array")
            .iter()
            .map(|f| f["name"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(
            manager_names.iter().any(|f| f == "id"),
            "manager default port should still include `id`, got {manager_names:?}"
        );
        assert!(
            !manager_names.iter().any(|f| f == "name"),
            "manager default port should NOT include removed `name`, got {manager_names:?}"
        );
    }
}
