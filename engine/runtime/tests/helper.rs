use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;
use rust_embed::RustEmbed;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::{errors::Error, runner::Runner};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;
use serde_json::Value;
use tempfile::{tempdir, TempDir};

pub(crate) static BUILTIN_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut common = HashMap::new();
    let sink = SINK_MAPPINGS.clone();
    let source = SOURCE_MAPPINGS.clone();
    let processor = PROCESSOR_MAPPINGS.clone();
    let wasm = reearth_flow_action_wasm_processor::mapping::ACTION_FACTORY_MAPPINGS.clone();
    common.extend(sink);
    common.extend(source);
    common.extend(processor);
    common.extend(wasm);
    common
});

#[derive(RustEmbed)]
#[folder = "fixture/testdata/"]
struct Fixtures;

#[derive(RustEmbed)]
#[folder = "fixture/workflow/"]
struct WorkflowFiles;

pub(crate) fn execute(test_id: &str, fixture_files: Vec<&str>) -> Result<TempDir, Error> {
    env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
    let storage_resolver = Arc::new(StorageResolver::new());
    let storage = storage_resolver
        .resolve(&Uri::for_test("ram:///fixture/"))
        .unwrap();
    for fixture in fixture_files {
        let file = Fixtures::get(format!("{test_id}/{fixture}").as_str())
            .unwrap()
            .data
            .to_vec();
        storage
            .put_sync(
                PathBuf::from(format!("/fixture/testdata/{test_id}/{fixture}")).as_path(),
                bytes::Bytes::from(file),
            )
            .unwrap();
    }
    let workflow_file = WorkflowFiles::get(format!("{test_id}.yaml").as_str()).unwrap();
    let workflow = std::str::from_utf8(workflow_file.data.as_ref()).unwrap();
    let binding = tempdir().unwrap();
    let folder_path = binding.path();
    std::fs::create_dir_all(folder_path).unwrap();
    let state = Arc::new(State::new(&Uri::for_test("ram:///state/"), &storage_resolver).unwrap());
    let logger_factory = Arc::new(LoggerFactory::new(
        reearth_flow_action_log::ActionLogger::root(
            reearth_flow_action_log::Discard,
            reearth_flow_action_log::o!(),
        ),
        Uri::for_test("ram:///log/").path(),
    ));
    let mut workflow = Workflow::try_from(workflow).expect("failed to parse workflow");
    workflow
        .merge_with(HashMap::from([(
            "outputFilePath".to_string(),
            folder_path
                .join("result.json")
                .to_str()
                .unwrap()
                .to_string(),
        )]))
        .unwrap();
    Runner::run(
        "test".to_string(),
        uuid::Uuid::new_v4(),
        workflow,
        BUILTIN_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver,
        state,
    )
    .unwrap();
    Ok(binding)
}

pub(crate) fn execute_with_test_assert(test_id: &str, assert_file: &str) {
    let tempdir = execute(test_id, vec![]).unwrap();
    let storage_resolver = Arc::new(StorageResolver::new());
    let file = Fixtures::get(format!("{test_id}/{assert_file}").as_str())
        .unwrap()
        .data
        .to_vec();
    let expect = bytes::Bytes::from(file);
    let expect_path = &Uri::for_test(
        tempdir
            .path()
            .join("result.json")
            .as_path()
            .to_str()
            .unwrap(),
    );
    let storage = storage_resolver.resolve(expect_path).unwrap();
    let result = storage.get_sync(expect_path.path().as_path());
    let result: Value = if let Ok(result) = result {
        serde_json::from_str(String::from_utf8(result.to_vec()).unwrap().as_str()).unwrap()
    } else {
        serde_json::from_str("[]").unwrap()
    };
    let expect: Value =
        serde_json::from_str(String::from_utf8(expect.to_vec()).unwrap().as_str()).unwrap();
    assert_eq!(expect, result);
}
