use std::{env, fs, sync::Arc};

use tempfile::Builder;
use yaml_include::Transformer;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::{id::Id, workflow::Workflow};
use reearth_flow_workflow_runner::dag::DagExecutor;

#[tokio::main]
async fn main() {
    let temp_dir = Builder::new().prefix("examples").tempdir_in(".").unwrap();
    let relative_path = file!();
    let current_dir = env::current_dir().unwrap();
    let absolute_path =
        fs::canonicalize(current_dir.join(relative_path).join("../testdata/workflow"));
    let paths = fs::read_dir(absolute_path.unwrap()).unwrap();

    for path in paths {
        let yaml = Transformer::new(path.unwrap().path(), false).unwrap();
        let storage_resolver = Arc::new(StorageResolver::new());
        let yaml = yaml.to_string();
        let state = Arc::new(
            State::new(
                &Uri::for_test(temp_dir.path().to_str().unwrap()),
                &storage_resolver,
            )
            .unwrap(),
        );
        let workflow = Workflow::try_from_str(yaml.as_str()).unwrap();
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
