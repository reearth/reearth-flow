use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_action_plateau_processor::mapping::ACTION_FACTORY_MAPPINGS as PLATEAU_MAPPINGS;
use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

pub fn run_workflow(
    workflow_path: &Path,
    citygml_path: &Path,
    output_dir: &Path,
    codelists_path: Option<&Path>,
    schemas_path: Option<&Path>,
) {
    let yaml_transformer =
        yaml_include::Transformer::new(workflow_path.to_path_buf(), false).unwrap();
    let yaml_str = yaml_transformer.to_string();

    // Save workflow as JSON for reference
    let workflow_json_path = output_dir.join("workflow.json");
    if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_str) {
        if let Ok(json_str) = serde_json::to_string_pretty(&yaml_value) {
            let _ = fs::write(&workflow_json_path, json_str);
        }
    }

    let mut workflow: Workflow = Workflow::try_from(yaml_str.as_str()).unwrap();

    let mut variables = HashMap::new();
    variables.insert(
        "cityGmlPath".to_string(),
        format!("{}", citygml_path.display()),
    );

    let flow_dir = output_dir.join("flow");
    fs::create_dir_all(&flow_dir).unwrap();
    variables.insert(
        "workerArtifactPath".to_string(),
        flow_dir.display().to_string(),
    );

    if let Some(codelists_path) = codelists_path {
        variables.insert(
            "codelistsPath".to_string(),
            format!("{}", codelists_path.display()),
        );
    }

    if let Some(schemas_path) = schemas_path {
        variables.insert(
            "schemasPath".to_string(),
            format!("{}", schemas_path.display()),
        );
    }

    workflow.extend_with(variables).unwrap();

    let mut action_factories = HashMap::new();
    action_factories.extend(SINK_MAPPINGS.clone());
    action_factories.extend(SOURCE_MAPPINGS.clone());
    action_factories.extend(PROCESSOR_MAPPINGS.clone());
    action_factories.extend(PLATEAU_MAPPINGS.clone());

    let job_id = uuid::Uuid::new_v4();

    let runtime_dir = output_dir.join("runtime");
    fs::create_dir_all(&runtime_dir).unwrap();
    let action_log_path = runtime_dir.join("action-log");
    fs::create_dir_all(&action_log_path).unwrap();
    let feature_state_path = runtime_dir.join("feature-store");
    fs::create_dir_all(&feature_state_path).unwrap();

    let logger_factory = Arc::new(LoggerFactory::new(
        create_root_logger(action_log_path.clone()),
        action_log_path,
    ));

    let storage_resolver = Arc::new(StorageResolver::new());
    let feature_state_uri = format!("file://{}", feature_state_path.display());
    let feature_state_uri = reearth_flow_common::uri::Uri::from_str(&feature_state_uri).unwrap();
    let feature_state = Arc::new(State::new(&feature_state_uri, &storage_resolver).unwrap());
    let ingress_state = Arc::clone(&feature_state);

    tracing::info!("Starting workflow run...");
    Runner::run(
        job_id,
        workflow,
        action_factories,
        logger_factory,
        storage_resolver,
        ingress_state,
        feature_state,
        None,
    )
    .unwrap();
}
