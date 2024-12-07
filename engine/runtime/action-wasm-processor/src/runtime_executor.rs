use super::errors::WasmProcessorError;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::str::FromStr;
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
        "Compiles scripts into .wasm and runs at the wasm runtime"
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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: WasmRuntimeExecutorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                WasmProcessorError::RuntimeExecutorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(WasmProcessorError::RuntimeExecutorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let wasm_binary = self.compile_to_wasm(&params)?;

        let process = WasmRuntimeExecutor {
            params,
            wasm_binary,
        };
        Ok(Box::new(process))
    }
}

impl WasmRuntimeExecutorFactory {
    fn compile_to_wasm(&self, params: &WasmRuntimeExecutorParam) -> super::errors::Result<Vec<u8>> {
        let temp_wasm_file = NamedTempFile::new().map_err(|e| {
            WasmProcessorError::RuntimeExecutorFactory(format!(
                "Failed to create temporary file: {}",
                e
            ))
        })?;
        let temp_wasm_path = temp_wasm_file.path();
        let temp_wasm_path_str = temp_wasm_path.to_str().ok_or_else(|| {
            WasmProcessorError::RuntimeExecutorFactory(
                "Temporary file path is not valid UTF-8".to_string(),
            )
        })?;

        let wasm_binary = match params.programming_language {
            ProgrammingLanguage::Python => {
                let py2wasm_output = std::process::Command::new("py2wasm")
                    .args([&params.source_code_file_path, "-o", temp_wasm_path_str])
                    .output()
                    .map_err(|e| {
                        WasmProcessorError::RuntimeExecutorFactory(format!(
                            "Failed to run py2wasm: {}. Command: py2wasm {} -o {}",
                            e, params.source_code_file_path, temp_wasm_path_str
                        ))
                    })?;

                if !py2wasm_output.status.success() {
                    let error_msg = String::from_utf8_lossy(&py2wasm_output.stderr);
                    return Err(WasmProcessorError::RuntimeExecutorFactory(format!(
                        "Python compilation failed: {}",
                        error_msg
                    )));
                }

                let mut binary = Vec::new();
                std::fs::File::open(temp_wasm_path)
                    .and_then(|mut file| file.read_to_end(&mut binary))
                    .map_err(|e| {
                        WasmProcessorError::RuntimeExecutorFactory(format!(
                            "Failed to read compiled Wasm file: {}",
                            e
                        ))
                    })?;
                binary
            }
        };

        Ok(wasm_binary)
    }
}

#[derive(Debug, Clone)]
pub struct WasmRuntimeExecutor {
    params: WasmRuntimeExecutorParam,
    wasm_binary: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WasmRuntimeExecutorParam {
    pub source_code_file_path: String,
    pub processor_type: ProcessorType,
    pub programming_language: ProgrammingLanguage,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum ProgrammingLanguage {
    Python,
}
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum ProcessorType {
    Attribute,
    AttributeWithXML,
}

impl Processor for WasmRuntimeExecutor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match self.params.processor_type {
            ProcessorType::Attribute => self.process_attribute(ctx, fw).map_err(Into::into),
            ProcessorType::AttributeWithXML => {
                self.process_attribute_with_xml(ctx, fw).map_err(Into::into)
            }
        }
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> super::errors::Result<()> {
        let mut feature = ctx.feature.clone();

        let json_input = self.serialize_attributes(&feature.attributes)?;
        let output = self.execute_wasm_module(&json_input, "")?;
        let updated_attributes = self.parse_wasm_output(&output)?;

        feature.attributes = updated_attributes;
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn process_attribute_with_xml(
        &self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> super::errors::Result<()> {
        let mut feature = ctx.feature.clone();

        let city_gml_path = feature
            .attributes
            .get(&Attribute::new("cityGmlPath"))
            .ok_or(WasmProcessorError::RuntimeExecutor(
                "cityGmlPath key empty".to_string(),
            ))?;

        let uri = Uri::from_str(city_gml_path.to_string().as_str()).map_err(|err| {
            WasmProcessorError::RuntimeExecutor(format!("cityGmlPath is not a valid uri: {}", err))
        })?;
        let storage = ctx
            .storage_resolver
            .resolve(&uri)
            .map_err(|e| WasmProcessorError::RuntimeExecutor(format!("{:?}", e)))?;
        let content = storage
            .get_sync(uri.path().as_path())
            .map_err(|e| WasmProcessorError::RuntimeExecutor(format!("{:?}", e)))?;
        let xml_content = String::from_utf8(content.to_vec())
            .map_err(|_| WasmProcessorError::RuntimeExecutor("Invalid UTF-8".to_string()))?;

        let json_input = self.serialize_attributes(&feature.attributes)?;
        let output = self.execute_wasm_module(&json_input, &xml_content)?;
        let updated_attributes = self.parse_wasm_output(&output)?;

        feature.attributes = updated_attributes;
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn serialize_attributes(
        &self,
        attributes: &HashMap<Attribute, AttributeValue>,
    ) -> super::errors::Result<String> {
        serde_json::to_string(attributes).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!(
                "Failed to serialize Feature to JSON: {}",
                e
            ))
        })
    }

    fn execute_wasm_module(&self, arg: &str, stdin: &str) -> super::errors::Result<String> {
        let mut store = Store::default();
        let module = Module::new(&store, &self.wasm_binary).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!("Failed to compile module: {}", e))
        })?;

        let program_name = "WasmRuntimeExecutor";
        let (mut stdin_tx, stdin_rx) = Pipe::channel();
        let (stdout_tx, mut stdout_rx) = Pipe::channel();
        stdin_tx.write_all(stdin.as_bytes()).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!("Failed to write to stdin: {}", e))
        })?;
        drop(stdin_tx);
        WasiEnv::builder(program_name)
            .args([arg])
            .stdin(Box::new(stdin_rx))
            .stdout(Box::new(stdout_tx))
            .run_with_store(module, &mut store)
            .map_err(|e| {
                WasmProcessorError::RuntimeExecutor(format!("Failed to execute module: {}", e))
            })?;

        let mut output = String::new();
        stdout_rx.read_to_string(&mut output).map_err(|e| {
            WasmProcessorError::RuntimeExecutor(format!(
                "Failed to read stdout from Wasm module: {}",
                e
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
                "Failed to parse Wasm module output as JSON. Output: '{}', Error: {}",
                output, e
            ))
        })?;

        match parsed_output.get(FIELD_STATUS) {
            Some(serde_json::Value::String(status)) if status == STATUS_SUCCESS => {
                if let Some(attributes_value) = parsed_output.get(FIELD_ATTRIBUTES) {
                    serde_json::from_value(attributes_value.clone()).map_err(|e| {
                        WasmProcessorError::RuntimeExecutor(format!(
                            "Failed to deserialize '{}': {}",
                            FIELD_ATTRIBUTES, e
                        ))
                    })
                } else {
                    Err(WasmProcessorError::RuntimeExecutor(format!(
                        "Missing '{}' in Wasm module output",
                        FIELD_ATTRIBUTES
                    )))
                }
            }
            Some(serde_json::Value::String(status)) if status == STATUS_ERROR => {
                let error_msg = match parsed_output.get(FIELD_ERROR) {
                    Some(serde_json::Value::String(err)) => {
                        format!("Wasm module execution failed with error: {}", err)
                    }
                    _ => "Wasm module execution failed with an unknown error.".to_string(),
                };
                Err(WasmProcessorError::RuntimeExecutor(error_msg))
            }
            _ => Err(WasmProcessorError::RuntimeExecutor(format!(
                "Unexpected '{}' in Wasm module output",
                FIELD_STATUS
            ))),
        }
    }
}
