use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc},
};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::feature::errors;
use crate::feature::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGmlReaderFactory;

impl ProcessorFactory for FeatureCityGmlReaderFactory {
    fn name(&self) -> &str {
        "FeatureCityGmlReader"
    }

    fn description(&self) -> &str {
        "Reads features from citygml file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGmlReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCityGmlReaderParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGmlReaderFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGmlReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGmlReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let params = CompiledFeatureCityGmlReaderParam {
            dataset: expr_engine
                .compile(params.dataset.as_ref())
                .map_err(|e| FeatureProcessorError::FileCityGmlReaderFactory(format!("{:?}", e)))?,
            flatten: params.flatten,
        };
        let threads_num = {
            let size = (num_cpus::get() as f32 / 4_f32).trunc() as usize;
            if size < 1 {
                1
            } else {
                std::cmp::min(size, 4) as usize
            }
        };
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads_num)
            .build()
            .unwrap();
        let process = FeatureCityGmlReader {
            global_params: with,
            params,
            join_handles: Vec::new(),
            thread_pool: Arc::new(parking_lot::Mutex::new(pool)),
        };
        Ok(Box::new(process))
    }
}

type JoinHandle = Arc<parking_lot::Mutex<Receiver<Result<(), errors::FeatureProcessorError>>>>;

#[derive(Debug, Clone)]
pub struct FeatureCityGmlReader {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledFeatureCityGmlReaderParam,
    join_handles: Vec<JoinHandle>,
    thread_pool: Arc<parking_lot::Mutex<rayon::ThreadPool>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGmlReaderParam {
    /// # Dataset to read
    dataset: Expr,
    /// # Flatten the dataset
    flatten: Option<bool>,
}

#[derive(Debug, Clone)]
struct CompiledFeatureCityGmlReaderParam {
    dataset: rhai::AST,
    flatten: Option<bool>,
}

impl Processor for FeatureCityGmlReader {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let fw = fw.clone();
        let feature = ctx.feature.clone();
        let ctx = ctx.as_context();
        let global_params = self.global_params.clone();
        let dataset = self.params.dataset.clone();
        let flatten = self.params.flatten;
        let pool = self.thread_pool.lock();
        let (tx, rx) = std::sync::mpsc::channel();
        self.join_handles
            .push(Arc::new(parking_lot::Mutex::new(rx)));
        pool.spawn(move || {
            let result = super::reader::read_citygml(
                ctx,
                fw,
                feature,
                dataset,
                flatten,
                global_params.clone(),
            );
            tx.send(result).unwrap();
        });
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let timeout = std::time::Duration::from_secs(60 * 60);
        let mut errors = Vec::new();

        for (i, join) in self.join_handles.iter().enumerate() {
            match join.lock().recv_timeout(timeout) {
                Ok(_) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    errors.push(format!("Worker thread {} timed out after {:?}", i, timeout));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    ctx.event_hub.warn_log(
                        None,
                        format!("Worker thread {} disconnected unexpectedly", i),
                    );
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors::FeatureProcessorError::FileCityGmlReader(format!(
                "Failed to complete all worker threads: {}",
                errors.join("; ")
            ))
            .into());
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCityGmlReader"
    }
}
