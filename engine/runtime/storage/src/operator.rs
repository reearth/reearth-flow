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
        Protocol::File => build_operator(init_fs_operator(uri), false),
        Protocol::Ram => build_operator(init_memory_operator(), false),
        Protocol::Google => build_operator(init_gcs_operator(uri), true),
        Protocol::Http => build_operator(init_http_operator(uri), true),
        Protocol::Https => build_operator(init_https_operator(uri), true),
    }
}

/// Builds an operator with timeout, logging and metrics layers.
///
/// `retry` adds a [`RetryLayer`]. Use it only for remote backends: local IO
/// errors are rarely transient, and retrying a partially completed local write
/// can duplicate data.
pub(crate) fn build_operator<B: Builder>(builder: B, retry: bool) -> Result<Operator> {
    let op = Operator::new(builder)?
        .layer(
            TimeoutLayer::new()
                // Timeout for non-IO operations such as stat, delete and list.
                .with_timeout(Duration::from_secs(30))
                // Timeout for each IO call, e.g. writing one chunk.
                .with_io_timeout(Duration::from_secs(30)),
        )
        .finish();
    let op = if retry {
        op.layer(RetryLayer::new().with_jitter())
    } else {
        op
    };
    Ok(op
        .layer(LoggingLayer::default())
        .layer(MetricsLayer::default()))
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
