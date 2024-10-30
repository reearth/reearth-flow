use std::env;
use std::io::Result;
use std::time::Duration;

use opendal::layers::LoggingLayer;
use opendal::layers::MetricsLayer;
use opendal::layers::RetryLayer;
use opendal::layers::TimeoutLayer;
use opendal::services;
use opendal::Builder;
use opendal::Operator;
use reearth_flow_common::uri::{Protocol, Uri};
use tracing::debug;

/// init_operator will init an opendal operator based on storage config.
pub(crate) fn resolve_operator(uri: &Uri) -> Result<Operator> {
    match uri.protocol() {
        Protocol::File => build_operator(init_fs_operator(uri)),
        Protocol::Ram => build_operator(init_memory_operator()),
        Protocol::Google => build_operator(init_gcs_operator(uri)),
        Protocol::Http => build_operator(init_http_operator(uri)),
        Protocol::Https => build_operator(init_https_operator(uri)),
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
                // Return timeout error if the operation failed to finish in
                // 5s
                .with_io_timeout(Duration::from_secs(5)),
        )
        // Add retry
        .layer(RetryLayer::new().with_jitter())
        .layer(LoggingLayer::default())
        .layer(MetricsLayer::default())
        .finish();
    Ok(op)
}

/// init_fs_operator will init a opendal fs operator.
fn init_fs_operator(uri: &Uri) -> impl Builder {
    let builder = services::Fs::default();
    let root = match uri.root() {
        "" => "/",
        _ => uri.root(),
    };
    builder.root(root)
}

/// init_gcs_operator will init a opendal gcs operator.
fn init_gcs_operator(uri: &Uri) -> impl Builder {
    if let Ok(host) = env::var("STORAGE_EMULATOR_HOST") {
        let builder = services::Gcs::default().endpoint(host.as_str());
        let builder = builder.disable_vm_metadata();
        let builder = builder.disable_config_load();
        let builder = builder.allow_anonymous();
        return builder.bucket(uri.root());
    }
    let builder = services::Gcs::default();
    let builder = if env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok() {
        builder.disable_vm_metadata()
    } else {
        builder
    };
    builder.bucket(uri.root())
}

/// init_memory_operator will init a opendal memory operator.
fn init_memory_operator() -> impl Builder {
    services::Memory::default()
}

fn init_https_operator(uri: &Uri) -> impl Builder {
    let builder = services::Http::default();
    debug!("init_https_operator: {}", uri.root());
    builder
        .endpoint(&format!("https://{}", uri.root()))
        .root("/")
}

fn init_http_operator(uri: &Uri) -> impl Builder {
    let builder = services::Http::default();
    debug!("init_http_operator: {}", uri.root());
    builder
        .endpoint(&format!("http://{}", uri.root()))
        .root("/")
}
