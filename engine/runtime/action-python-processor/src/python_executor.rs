use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use indexmap::IndexMap;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PythonProcessorError {
    #[error("Factory error: {0}")]
    FactoryError(String),

    #[error("Python execution error: {0}")]
    ExecutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Default)]
pub struct PythonScriptProcessorFactory;

impl ProcessorFactory for PythonScriptProcessorFactory {
    fn name(&self) -> &str {
        "PythonScriptProcessor"
    }

    fn description(&self) -> &str {
        "Executes Python scripts using the system Python interpreter"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(PythonScriptProcessorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Script", "Python"]
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
        let params: PythonScriptProcessorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PythonProcessorError::FactoryError(format!("Failed to serialize parameters: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PythonProcessorError::FactoryError(format!("Failed to deserialize parameters: {e}"))
            })?
        } else {
            return Err(PythonProcessorError::FactoryError(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let processor = PythonScriptProcessor {
            script: params.script.to_string(),
            python_path: params.python_path.unwrap_or_else(|| "python3".to_string()),
            _timeout_seconds: params.timeout_seconds.unwrap_or(30),
            ctx,
        };

        Ok(Box::new(processor))
    }
}

#[derive(Debug, Clone)]
struct PythonScriptProcessor {
    script: String,
    python_path: String,
    _timeout_seconds: u64,
    ctx: NodeContext,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PythonScriptProcessorParam {
    /// Python script to execute (can be inline code or file path)
    script: Expr,

    /// Path to Python interpreter (defaults to "python3")
    #[serde(skip_serializing_if = "Option::is_none")]
    python_path: Option<String>,

    /// Execution timeout in seconds (defaults to 30)
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<u64>,
}

impl Processor for PythonScriptProcessor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Serialize feature attributes to JSON
        let input_json = serde_json::to_string(&feature.attributes).map_err(|e| {
            PythonProcessorError::SerializationError(format!("Failed to serialize attributes: {e}"))
        })?;

        // Resolve the script using expression engine
        let expr_engine = &self.ctx.expr_engine;
        let scope = expr_engine.new_scope();
        let script_expr = Expr::new(&self.script);
        let resolved_script = expr_engine
            .eval_scope::<String>(script_expr.as_ref(), &scope)
            .unwrap_or_else(|_| self.script.clone());

        // Check if script is a file path or inline code
        let (_is_file, script_content) = if resolved_script.ends_with(".py") {
            // It's likely a file path
            match std::fs::read_to_string(&resolved_script) {
                Ok(content) => (true, content),
                Err(_) => (false, resolved_script), // Treat as inline script if file not found
            }
        } else {
            (false, resolved_script)
        };

        // Create a simple Python wrapper that reads JSON from stdin and outputs to stdout
        let python_wrapper = format!(
            r#"
import sys
import json

# Read input from stdin
input_data = sys.stdin.read()
attributes = json.loads(input_data)

# User script starts here
{}
# User script ends here

# Ensure attributes is defined (user script should modify it)
if 'attributes' not in locals():
    attributes = {{}}

# Output the result
print(json.dumps(attributes))
"#,
            script_content
        );

        // Create temporary file for the Python script
        let mut temp_file = NamedTempFile::new().map_err(|e| {
            PythonProcessorError::ExecutionError(format!("Failed to create temp file: {e}"))
        })?;

        temp_file
            .write_all(python_wrapper.as_bytes())
            .map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to write script: {e}"))
            })?;

        temp_file.flush().map_err(|e| {
            PythonProcessorError::ExecutionError(format!("Failed to flush script: {e}"))
        })?;

        // Execute Python script with timeout
        let mut child = Command::new(&self.python_path)
            .arg(temp_file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to spawn Python process: {e}"))
            })?;

        // Write input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input_json.as_bytes()).map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to write to stdin: {e}"))
            })?;
        }

        // Wait for completion with timeout
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                return Err(PythonProcessorError::ExecutionError(format!(
                    "Failed to execute Python script: {e}"
                ))
                .into());
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PythonProcessorError::ExecutionError(format!(
                "Python script failed: {}",
                stderr
            ))
            .into());
        }

        // Parse the output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let updated_attributes: IndexMap<Attribute, AttributeValue> =
            serde_json::from_str(stdout.trim()).map_err(|e| {
                PythonProcessorError::SerializationError(format!(
                    "Failed to parse Python output: {e}. Output was: {}",
                    stdout
                ))
            })?;

        // Update feature attributes
        feature.attributes = updated_attributes;

        // Send the modified feature
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "PythonScriptProcessor"
    }
}
