use std::{collections::HashMap, sync::Arc, time::Instant};

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_runtime::{node::NodeKind, shutdown};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tracing::{error, info, info_span};

use crate::orchestrator::Orchestrator;

pub struct Runner;

impl Runner {
    pub fn run(
        job_id: String,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        state: Arc<State>,
    ) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(30)
            .enable_all()
            .build()
            .unwrap();

        let start = Instant::now();
        let span = info_span!(
            "root",
            "otel.name" = workflow.name.as_str(),
            "otel.kind" = "runner",
            "workflow.id" = workflow.id.to_string().as_str(),
        );
        let workflow_name = workflow.name.clone();
        info!(parent: &span, "Start workflow = {:?}", workflow_name.as_str());
        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&runtime);
        let runtime = Arc::new(runtime);
        let orchestraotr = Orchestrator::new(runtime.clone());
        runtime.block_on(async move {
            let result = orchestraotr
                .run_all(
                    job_id,
                    workflow,
                    factories,
                    shutdown_receiver,
                    logger_factory,
                    storage_resolver,
                    state,
                )
                .await;
            if let Err(e) = result {
                error!("Failed to workflow: {:?}", e);
            }
        });
        info!(parent: &span, "Finish workflow = {:?}, duration = {:?}", workflow_name.as_str(), start.elapsed());
    }
}
