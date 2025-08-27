use std::sync::Arc;

use reearth_flow_action_log::{action_error_log, action_log, factory::LoggerFactory, ActionLogger};
use tracing::{debug_span, error_span, info_span, trace_span, warn_span};

use crate::node_info_tls;

pub(crate) struct LogEventHandler {
    #[allow(dead_code)]
    pub(crate) workflow_id: uuid::Uuid,
    pub(crate) job_id: uuid::Uuid,
    pub(crate) logger: Arc<ActionLogger>,
}

impl LogEventHandler {
    pub(crate) fn new(
        workflow_id: uuid::Uuid,
        job_id: uuid::Uuid,
        logger_factory: Arc<LoggerFactory>,
    ) -> Self {
        let logger = logger_factory.action_logger(&job_id.to_string());
        Self {
            workflow_id,
            job_id,
            logger: Arc::new(logger),
        }
    }
}

#[async_trait::async_trait]
impl reearth_flow_runtime::event::EventHandler for LogEventHandler {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        if let reearth_flow_runtime::event::Event::Log {
            span,
            level,
            node_handle,
            node_name,
            message,
        } = event
        {
            let node_id = node_handle
                .clone()
                .map(|h| h.id.to_string())
                .unwrap_or_else(|| "".to_string());
            match *level {
                tracing::Level::ERROR => {
                    let span = span.clone().unwrap_or_else(|| error_span!(""));

                    if !node_id.is_empty() {
                        node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                    }

                    action_error_log!(parent: span, self.logger, "{:?}", message);

                    node_info_tls::clear_node_info();
                }
                tracing::Level::WARN => {
                    let span = span.clone().unwrap_or_else(|| warn_span!(""));
                    tracing::event!(parent: span, tracing::Level::WARN, "job_id"=self.job_id.to_string(), "node_id"=node_id, "{:?}", message);
                }
                tracing::Level::INFO => {
                    let span = span.clone().unwrap_or_else(|| info_span!(""));

                    if !node_id.is_empty() {
                        node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                    }

                    action_log!(parent: span, self.logger, "{:?}", message);

                    node_info_tls::clear_node_info();
                }
                tracing::Level::DEBUG => {
                    let span = span.clone().unwrap_or_else(|| debug_span!(""));
                    tracing::event!(parent: span, tracing::Level::DEBUG, "job_id"=self.job_id.to_string(), "node_id"=node_id, "{:?}", message);
                }
                tracing::Level::TRACE => {
                    let span = span.clone().unwrap_or_else(|| trace_span!(""));
                    tracing::event!(parent: span, tracing::Level::TRACE, "job_id"=self.job_id.to_string(), "node_id"=node_id, "{:?}", message);
                }
            }
        }
    }
}
