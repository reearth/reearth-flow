use std::{env, path::PathBuf, sync::Arc};

use rust_embed::RustEmbed;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::workflow::Workflow;
use reearth_flow_workflow_runner::dag::DagExecutor;

#[derive(RustEmbed)]
#[folder = "fixture/testdata/"]
struct Fixtures;

#[derive(RustEmbed)]
#[folder = "fixture/workflow/"]
struct WorkflowFiles;

pub(crate) async fn init_test_runner(test_id: &str, fixture_files: Vec<&str>) -> DagExecutor {
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
            .put(
                PathBuf::from(format!("/fixture/testdata/{}/{}", test_id, fixture)).as_path(),
                bytes::Bytes::from(file),
            )
            .await
            .unwrap();
    }
    let workflow_file = WorkflowFiles::get(format!("{}.yaml", test_id).as_str()).unwrap();
    let workflow = std::str::from_utf8(workflow_file.data.as_ref()).unwrap();

    let state = Arc::new(State::new(&Uri::for_test("ram:///state/"), &storage_resolver).unwrap());
    let log_factory = Arc::new(LoggerFactory::new(
        reearth_flow_action_log::ActionLogger::root(
            reearth_flow_action_log::Discard,
            reearth_flow_action_log::o!(),
        ),
        Uri::for_test("ram:///log/").path(),
    ));
    let workflow = Workflow::try_from_str(workflow).unwrap();
    DagExecutor::new(job_id, &workflow, storage_resolver, state, log_factory).unwrap()
}
