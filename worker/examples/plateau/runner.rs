use std::sync::Arc;

use rust_embed::RustEmbed;
use tempfile::Builder;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::{id::Id, workflow::Workflow};
use reearth_flow_workflow_runner::dag::DagExecutor;

#[derive(RustEmbed)]
#[folder = "plateau/testdata/"]
struct Asset;

#[tokio::main]
async fn main() {
    let temp_dir = Builder::new().prefix("examples").tempdir_in(".").unwrap();
    for file in Asset::iter() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let yaml = Asset::get(file.as_ref()).unwrap();
        let yaml = std::str::from_utf8(yaml.data.as_ref()).unwrap();
        let state = Arc::new(
            State::new(
                &Uri::for_test(temp_dir.path().to_str().unwrap()),
                &storage_resolver,
            )
            .unwrap(),
        );
        let workflow = Workflow::try_from_str(yaml).unwrap();
        let job_id = Id::new_v4();
        let log_factory = Arc::new(LoggerFactory::new(
            reearth_flow_action_log::ActionLogger::root(
                reearth_flow_action_log::Discard,
                reearth_flow_action_log::o!(),
            ),
            temp_dir.path().to_path_buf(),
        ));
        let executor =
            DagExecutor::new(job_id, &workflow, storage_resolver, state, log_factory).unwrap();
        let result = executor.start().await;
        println!("{:?}", result);
    }
}
