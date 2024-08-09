use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;
use rust_embed::RustEmbed;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::runner::Runner;
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

#[derive(RustEmbed)]
#[folder = "fixture/testdata/"]
struct Fixtures;

#[derive(RustEmbed)]
#[folder = "fixture/workflow/"]
struct WorkflowFiles;

pub(crate) fn execute(test_id: &str, fixture_files: Vec<&str>) {
    let job_id = uuid::Uuid::new_v4();
    env::set_var("ACTION_LOG_DISABLE", "true");
    let storage_resolver = Arc::new(StorageResolver::new());
    let storage = storage_resolver
        .resolve(&Uri::for_test("ram:///fixture/"))
        .unwrap();
    for fixture in fixture_files {
        let file = Fixtures::get(format!("{}/{}", test_id, fixture).as_str())
            .unwrap()
            .data
            .to_vec();
        storage
            .put_sync(
                PathBuf::from(format!("/fixture/testdata/{}/{}", test_id, fixture)).as_path(),
                bytes::Bytes::from(file),
            )
            .unwrap();
    }
    let workflow_file = WorkflowFiles::get(format!("{}.yaml", test_id).as_str()).unwrap();
    let workflow = std::str::from_utf8(workflow_file.data.as_ref()).unwrap();
    let state = Arc::new(State::new(&Uri::for_test("ram:///state/"), &storage_resolver).unwrap());
    let logger_factory = Arc::new(LoggerFactory::new(
        reearth_flow_action_log::ActionLogger::root(
            reearth_flow_action_log::Discard,
            reearth_flow_action_log::o!(),
        ),
        Uri::for_test("ram:///log/").path(),
    ));
    let workflow = Workflow::try_from_str(workflow);
    Runner::run(
        job_id.to_string(),
        workflow,
        BUILTIN_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    );
}
