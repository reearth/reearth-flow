use serde_json::Number;
use std::num::ParseIntError;
use thiserror::Error;

use crate::node::{NodeHandle, Port};

#[derive(Error, Debug)]
pub enum DagError {
    #[error("dag: {0}")]
    DagSchema(String),
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Invalid field index: {0}")]
    InvalidFieldIndex(usize),
    #[error("Invalid field name: {0}")]
    InvalidFieldName(String),
    #[error("Serialization failed: {0}")]
    SerializationError(#[source] SerializationError),
    #[error("Failed to parse the field: {0}")]
    DeserializationError(#[source] DeserializationError),
}

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("custom: {0}")]
    Custom(#[from] BoxedError),
}

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("bson: {0}")]
    Msgpack(#[from] rmp_serde::decode::Error),
    #[error("custom: {0}")]
    Custom(#[from] BoxedError),
    #[error("Empty input")]
    EmptyInput,
    #[error("Unrecognised field type : {0}")]
    UnrecognisedFieldType(u8),
    #[error("Bad data length")]
    BadDataLength,
    #[error("Bad data format: {0}")]
    BadDateFormat(#[from] chrono::ParseError),
    #[error("utf8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Failed to convert type due to json numbers being out of the f64 range: {0}")]
    F64TypeConversionError(Number),
    #[error("Unknown SSL mode: {0}")]
    UnknownSslMode(String),
    #[error("Unable to Parse Postgres configuration: {0}")]
    UnableToParseConnectionUrl(ParseIntError),
    #[error("{0} is missing in Postgres configuration")]
    MissingFieldInPostgresConfig(String),
    #[error("{0} is mismatching in Postgres configuration")]
    MismatchingFieldInPostgresConfig(String),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Invalid port handle: {0}")]
    InvalidPortHandle(Port),
    #[error("Missing input for node {node} on port {port}")]
    MissingInput { node: NodeHandle, port: Port },
    #[error("Duplicate input for node {node} on port {port}")]
    DuplicateInput { node: NodeHandle, port: Port },
    #[error("Cannot send to channel")]
    CannotSendToChannel(String),
    #[error("Cannot receive from channel: {0}")]
    CannotReceiveFromChannel(String),
    #[error("Cannot spawn worker thread: {0}")]
    CannotSpawnWorkerThread(#[source] std::io::Error),
    #[error("Invalid source name {0}")]
    InvalidSourceIdentifier(String),
    #[error("Ambiguous source name {0}")]
    AmbiguousSourceIdentifier(String),
    #[error("Invalid AppSource connection {0}. Already exists.")]
    AppSourceConnectionAlreadyExists(String),
    #[error("Factory error for node {node_id} ({node_name}): {error}")]
    Factory {
        node_id: String,
        node_name: String,
        #[source]
        error: BoxedError,
    },
    #[error("Failed to restore record writer: {0}")]
    RestoreRecordWriter(#[source] DeserializationError),
    #[error("Source error: {0}")]
    Source(#[source] BoxedError),
    #[error("Processor error: {0}")]
    Processor(#[source] BoxedError),
    #[error("Sink error: {0}")]
    Sink(#[source] BoxedError),
    #[error("ChannelManager error: {0}")]
    ChannelManager(#[source] BoxedError),
    #[error("State of {0} is not consistent across sinks")]
    SourceStateConflict(NodeHandle),
    #[error("Action name mismatch for node {0}: {1} != {2}")]
    ActionNameMismatch(String, String, String),
    #[error("Checkpoint writer thread panicked")]
    CheckpointWriterThreadPanicked,
    #[error("Failed to serialize record writer: {0}")]
    SerializeRecordWriter(#[source] SerializationError),
    #[error("InValid Sink: {0}")]
    InvalidSink(String),
}

impl<T> From<crossbeam::channel::SendError<T>> for ExecutionError {
    fn from(_: crossbeam::channel::SendError<T>) -> Self {
        ExecutionError::CannotSendToChannel("SendError".to_string())
    }
}

#[derive(Debug, Error)]
#[error("Cannot convert f64 to json: {0}")]
pub struct CannotConvertF64ToJson(pub f64);

pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
