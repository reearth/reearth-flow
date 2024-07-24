use std::collections::HashMap;
use std::{env, fs, path::Path, sync::Arc};

use directories::ProjectDirs;
use once_cell::sync::Lazy;
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use yaml_include::Transformer;

use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
use reearth_flow_action_plateau_processor::mapping::ACTION_MAPPINGS as PLATEAU_MAPPINGS;
use reearth_flow_action_processor::mapping::ACTION_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::runner::Runner;
use reearth_flow_runtime::node::NodeKind;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

pub(crate) static BUILTIN_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut common = HashMap::new();
    let sink = SINK_MAPPINGS.clone();
    let source = SOURCE_MAPPINGS.clone();
    let processor = PROCESSOR_MAPPINGS.clone();
    common.extend(sink);
    common.extend(source);
    common.extend(processor);
    common
});

pub(crate) static PLATEAU_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> =
    Lazy::new(|| PLATEAU_MAPPINGS.clone());

pub(crate) static ALL_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut all = HashMap::new();
    all.extend(BUILTIN_ACTION_FACTORIES.clone());
    all.extend(PLATEAU_ACTION_FACTORIES.clone());
    all
});

#[allow(dead_code)]
pub(crate) fn execute(workflow: &str) {
    env::set_var("RAYON_NUM_THREADS", "10");
    setup_logging_and_tracing();
    let job_id = uuid::Uuid::new_v4();
    let action_log_uri = {
        let p = ProjectDirs::from("reearth", "flow", "worker").unwrap();
        let p = p.cache_dir().to_str().unwrap();
        let p = format!("{}/action-log/{}", p, job_id);
        let _ = fs::create_dir_all(Path::new(p.as_str()));
        Uri::for_test(format!("file://{}", p).as_str())
    };
    let state_uri = {
        let p = ProjectDirs::from("reearth", "flow", "worker").unwrap();
        let p = p.cache_dir().to_str().unwrap();
        let p = format!("{}/feature-store/{}", p, job_id);
        let _ = fs::create_dir_all(Path::new(p.as_str()));
        Uri::for_test(format!("file://{}", p).as_str())
    };
    let storage_resolver = Arc::new(StorageResolver::new());
    let state = Arc::new(State::new(&state_uri, &storage_resolver).unwrap());
    let workflow = create_workflow(workflow);
    let logger_factory = Arc::new(LoggerFactory::new(
        create_root_logger(action_log_uri.path()),
        action_log_uri.path(),
    ));
    Runner::run(
        job_id.to_string(),
        workflow,
        ALL_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    );
}

pub fn create_workflow(workflow: &str) -> Workflow {
    let current_dir = env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .replace("examples", "");
    let current_dir = Path::new(&current_dir);
    let absolute_path = fs::canonicalize(
        current_dir
            .join("examples/plateau/testdata/workflow")
            .join(workflow),
    );
    let path = absolute_path.unwrap();
    let yaml = Transformer::new(path, false).unwrap();
    let yaml = yaml.to_string();
    Workflow::try_from_str(yaml.as_str())
}

pub fn setup_logging_and_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy()
        .add_directive("opendal=error".parse().unwrap());
    let registry = tracing_subscriber::registry().with(env_filter);
    let event_format = tracing_subscriber::fmt::format()
        .with_target(true)
        .with_timer(UtcTime::new(
            time::format_description::parse(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z",
            )
            .expect("Time format invalid."),
        ));
    let _ = registry
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(event_format)
                .with_ansi(true),
        )
        .try_init();
}
