use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum WasmProcessorError {
    #[error("Wasm Runtime Executor Factory error: {0}")]
    RuntimeExecutorFactory(String),
    #[error("Wasm Runtime Executor error: {0}")]
    RuntimeExecutor(String),
}

#[allow(dead_code)]
pub(super) type Result<T, E = WasmProcessorError> = std::result::Result<T, E>;
