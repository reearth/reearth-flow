use std::sync::Arc;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_runtime::shutdown;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;

use crate::orchestrator::Orchestrator;

pub struct Runner;

impl Runner {
    pub fn run(
        job_id: String,
        workflow: Workflow,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
    ) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&runtime);
        let runtime = Arc::new(runtime);
        let orchestraotr = Orchestrator::new(runtime.clone());
        runtime.block_on(async move {
            orchestraotr
                .run_all(
                    job_id,
                    workflow,
                    shutdown_receiver,
                    logger_factory,
                    storage_resolver,
                )
                .await
                .unwrap()
        });
    }
}
