use super::errors::WasmProcessorError;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};

use reearth_flow_common::uri::Uri;
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use tempfile::NamedTempFile;
use wasmer::{Module, Store};
use wasmer_wasix::{Pipe, WasiEnv};

#[derive(Debug, Clone, Default)]
pub struct WasmRuntimeExecutorFactory;

impl ProcessorFactory for WasmRuntimeExecutorFactory {
    fn name(&self) -> &str {
        "WasmRuntimeExecutor"
    }

    fn description(&self) -> &str {
        "Compiles scripts (Python) into WebAssembly and executes them in a WASM runtime"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(WasmRuntimeExecutorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Wasm"]
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
        let params: WasmRuntimeExecutorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(WasmProcessorError::RuntimeExecutorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let wasm_binary = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.compile_to_wasm(&ctx, &params))
        })?;
        let process = WasmRuntimeExecutor {
            processor_type: params.processor_type,
            wasm_binary,
        };
        Ok(Box::new(process))
    }
}

impl WasmRuntimeExecutorFactory {
    async fn compile_to_wasm(
        &self,
        ctx: &NodeContext,
        params: &WasmRuntimeExecutorParam,
    ) -> super::errors::Result<Vec<u8>> {
        let temp_wasm_file = NamedTempFile::new().map_err(|e| {
            WasmProcessorError::RuntimeExecutorFactory(format!(
                "Failed to create temporary file: {e}"
            ))
        })?;
        let temp_wasm_path = temp_wasm_file.path();
        let temp_wasm_path_str = temp_wasm_path.to_str().ok_or_else(|| {
            WasmProcessorError::RuntimeExecutorFactory(
                "Temporary file path is not valid UTF-8".to_string(),
            )
        })?;

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        let source = expr_engine
            .eval_scope::<String>(params.source.as_ref(), &scope)
            .unwrap_or_else(|_| params.source.to_string());

        let (local_source_path, _temp_py_file_holder) = if source.starts_with("http://")
            || source.starts_with("https://")
        {
            let source_uri = Uri::from_str(&source).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!("Invalid URL: {e}"))
            })?;

            let storage = ctx.storage_resolver.resolve(&source_uri).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!("Failed to resolve URL: {e}"))
            })?;

            let content = storage
                .get(source_uri.path().as_path())
                .await
                .map_err(|e| {
                    WasmProcessorError::RuntimeExecutorFactory(format!(
                        "Failed to download from URL: {e}"
                    ))
                })?
                .bytes()
                .await
                .map_err(|e| {
                    WasmProcessorError::RuntimeExecutorFactory(format!(
                        "Failed to read content: {e}"
                    ))
                })?;

            let mut temp_py_file = NamedTempFile::new().map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to create temporary Python file: {e}"
                ))
            })?;

            temp_py_file.write_all(&content).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to write Python file: {e}"
                ))
            })?;

            temp_py_file.flush().map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to flush Python file: {e}"
                ))
            })?;

            let temp_path = temp_py_file.path().to_string_lossy().to_string();
            (temp_path, Some(temp_py_file))
        } else {
            (source, None)
        };

        let wasm_binary = match params.programming_language {
            ProgrammingLanguage::Python => {
                let py2wasm_output = std::process::Command::new("py2wasm")
                    .args([&local_source_path, "-o", temp_wasm_path_str])
                    .output()
                    .map_err(|e| {
                        WasmProcessorError::RuntimeExecutorFactory(format!(
                            "Failed to run py2wasm: {e}. Command: py2wasm {local_source_path} -o {temp_wasm_path_str}"
                        ))
                    })?;

                if !py2wasm_output.status.success() {
                    let error_msg = String::from_utf8_lossy(&py2wasm_output.stderr);
                    return Err(WasmProcessorError::RuntimeExecutorFactory(format!(
                        "Python compilation failed: {error_msg}"
                    )));
                }

                let mut binary = Vec::new();
                std::fs::File::open(temp_wasm_path)
                    .and_then(|mut file| file.read_to_end(&mut binary))
                    .map_err(|e| {
                        WasmProcessorError::RuntimeExecutorFactory(format!(
                            "Failed to read compiled Wasm file: {e}"
                        ))
                    })?;
                binary
            }
        };

        Ok(wasm_binary)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WasmRuntimeExecutor {
    processor_type: ProcessorType,
    wasm_binary: Vec<u8>,
}

/// # WasmRuntimeExecutor Parameters
///
/// Configuration for compiling and executing scripts in WebAssembly runtime.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WasmRuntimeExecutorParam {
    /// Script source code or path to compile to WebAssembly
    source: Expr,
    /// Type of processor to create (Source, Processor, or Sink)
    processor_type: ProcessorType,
    /// Programming language of the source script (currently supports Python)
    programming_language: ProgrammingLanguage,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub(crate) enum ProgrammingLanguage {
    Python,
}
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub(crate) enum ProcessorType {
    Attribute,
}

impl Processor for WasmRuntimeExecutor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match self.processor_type {
            ProcessorType::Attribute => self.process_attribute(ctx, fw).map_err(Into::into),
        }
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "WasmRuntimeExecutor"
    }
}

impl WasmRuntimeExecutor {
    fn process_attribute(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> super::errors::Result<()> {
        let mut feature = ctx.feature.clone();

        let json_input = self.serialize_attributes(
            &feature
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<_, _>>(),
        )?;
        let output = self.execute_wasm_module(&json_input)?;
        let updated_attributes = self.parse_wasm_output(&output)?;

        feature.attributes = updated_attributes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn serialize_attributes(
        &self,
        attributes: &HashMap<Attribute, AttributeValue>,
    ) -> super::errors::Result<String> {
        serde_json::to_string(attributes).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!("Failed to serialize Feature to JSON: {e}"))
        })
    }

    fn execute_wasm_module(&self, input: &str) -> super::errors::Result<String> {
        let mut store = Store::default();
        let module = Module::new(&store, &self.wasm_binary).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!("Failed to compile module: {e}"))
        })?;

        let program_name = "WasmRuntimeExecutor";
        let (stdout_tx, mut stdout_rx) = Pipe::channel();
        WasiEnv::builder(program_name)
            .args([input])
            .stdout(Box::new(stdout_tx))
            .run_with_store(module, &mut store)
            .map_err(|e| {
                WasmProcessorError::RuntimeExecutor(format!("Failed to execute module: {e}"))
            })?;

        let mut output = String::new();
        stdout_rx.read_to_string(&mut output).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!(
                "Failed to read stdout from Wasm module: {e}"
            ))
        })?;
        Ok(output)
    }

    fn parse_wasm_output(
        &self,
        output: &str,
    ) -> super::errors::Result<HashMap<Attribute, AttributeValue>> {
        const STATUS_SUCCESS: &str = "success";
        const STATUS_ERROR: &str = "error";
        const FIELD_ATTRIBUTES: &str = "attributes";
        const FIELD_ERROR: &str = "error";
        const FIELD_STATUS: &str = "status";

        let parsed_output: serde_json::Value = serde_json::from_str(output).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!(
                "Failed to parse Wasm module output as JSON. Output: '{output}', Error: {e}"
            ))
        })?;

        match parsed_output.get(FIELD_STATUS) {
            Some(serde_json::Value::String(status)) if status == STATUS_SUCCESS => {
                if let Some(attributes_value) = parsed_output.get(FIELD_ATTRIBUTES) {
                    serde_json::from_value(attributes_value.clone()).map_err(|e| {
                        WasmProcessorError::RuntimeExecutor(format!(
                            "Failed to deserialize '{FIELD_ATTRIBUTES}': {e}"
                        ))
                    })
                } else {
                    Err(WasmProcessorError::RuntimeExecutor(format!(
                        "Missing '{FIELD_ATTRIBUTES}' in Wasm module output"
                    )))
                }
            }
            Some(serde_json::Value::String(status)) if status == STATUS_ERROR => {
                let error_msg = match parsed_output.get(FIELD_ERROR) {
                    Some(serde_json::Value::String(err)) => {
                        format!("Wasm module execution failed with error: {err}")
                    }
                    _ => "Wasm module execution failed with an unknown error.".to_string(),
                };
                Err(WasmProcessorError::RuntimeExecutor(error_msg))
            }
            _ => Err(WasmProcessorError::RuntimeExecutor(format!(
                "Unexpected '{FIELD_STATUS}' in Wasm module output"
            ))),
        }
    }
}
