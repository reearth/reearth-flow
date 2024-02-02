use std::io::Result;
use std::time::Duration;

use opendal::layers::LoggingLayer;
use opendal::layers::MinitraceLayer;
use opendal::layers::RetryLayer;
use opendal::layers::TimeoutLayer;
use opendal::services;
use opendal::Builder;
use opendal::Operator;
use reearth_flow_common::uri::{Protocol, Uri};

/// init_operator will init an opendal operator based on storage config.
pub(crate) fn resolve_operator(uri: &Uri) -> Result<Operator> {
    match uri.protocol() {
        Protocol::File => build_operator(init_fs_operator(uri)),
        Protocol::Ram => build_operator(init_memory_operator()),
        Protocol::Google => build_operator(init_gcs_operator(uri)),
    }
}

pub(crate) fn build_operator<B: Builder>(builder: B) -> Result<Operator> {
    let ob = Operator::new(builder)?;
    let op = ob
        .layer(
            TimeoutLayer::new()
                // Return timeout error if the operation failed to finish in
                // 10s
                .with_timeout(Duration::from_secs(10))
                // Return timeout error if the request speed is less than
                // 1 KiB/s.
                .with_speed(1024),
        )
        // Add retry
        .layer(RetryLayer::new().with_jitter())
        .layer(LoggingLayer::default())
        .layer(MinitraceLayer)
        .finish();
    Ok(op)
}

/// init_fs_operator will init a opendal fs operator.
fn init_fs_operator(uri: &Uri) -> impl Builder {
    let mut builder = services::Fs::default();
    let root = match uri.root() {
        "" => "/",
        _ => uri.root(),
    };
    builder.root(root);
    builder
}

/// init_gcs_operator will init a opendal gcs operator.
fn init_gcs_operator(uri: &Uri) -> impl Builder {
    let mut builder = services::Gcs::default();
    builder.bucket(uri.root());
    builder
}

/// init_memory_operator will init a opendal memory operator.
fn init_memory_operator() -> impl Builder {
    services::Memory::default()
}
