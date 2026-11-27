use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get, routing::post};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_worker::wrapper::{
    ProbeRequest, RunRequest, build_probe_args, serve_run, validate_job_id, worker_failure,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_worker::wrapper::{
    RENDER_VIEW_TIME_LIMIT, RenderViewRequest, STOPPED_REPORT_BUDGET, serve_render_view,
};
use serde_json::json;

#[derive(Clone)]
struct AppState {
    resolver: Arc<StorageResolver>,
    worker_bin: String,
    work_base: PathBuf,
}

async fn health() -> &'static str {
    "ok"
}

/// Handle a `GET /version` request: which engine this Service runs.
///
/// A caller that caches what the engine renders can key on this, so a cached
/// result does not outlive the engine that produced it. Response:
/// `{"engineVersion": string}`.
async fn version() -> Json<serde_json::Value> {
    Json(json!({"engineVersion": reearth_flow_worker::ENGINE_VERSION}))
}

async fn run(
    State(st): State<AppState>,
    Json(req): Json<RunRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let (status, body) = serve_run(&st.worker_bin, &st.work_base, &st.resolver, &req).await;
    (status, Json(body))
}

/// Handle a `/probe-schema` POST request.
/// Probe is read-only and fast: no work-root, no cancel-flag, no metadata.
/// Response: `{"status": "COMPLETED"|"FAILED", "error"?: string}`.
async fn probe_schema(
    State(st): State<AppState>,
    Json(req): Json<ProbeRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = validate_job_id(&req.job_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "FAILED", "error": e})),
        );
    }

    let args = build_probe_args(&req);
    // Killed if the request goes away (e.g. at Cloud Run's request timeout)
    // rather than left running with no one to report to.
    let output = match tokio::process::Command::new(&st.worker_bin)
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "FAILED", "error": e.to_string()})),
            );
        }
    };

    if output.status.success() {
        (StatusCode::OK, Json(json!({"status": "COMPLETED"})))
    } else {
        let (status, body) = worker_failure(output.status, &output.stderr);
        (status, Json(body))
    }
}

/// Handle a `/render-view` POST request. See [`serve_render_view`].
#[cfg(feature = "new-geometry")]
async fn render_view(
    State(st): State<AppState>,
    Json(req): Json<RenderViewRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let (status, body) = serve_render_view(
        &st.worker_bin,
        &st.work_base,
        &st.resolver,
        &req,
        RENDER_VIEW_TIME_LIMIT,
        STOPPED_REPORT_BUDGET,
    )
    .await;
    (status, Json(body))
}

#[tokio::main]
async fn main() {
    reearth_flow_common::tls::install_crypto_provider();
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let worker_bin =
        std::env::var("FLOW_WORKER_BIN").unwrap_or_else(|_| "/bin/reearth-flow-worker".to_string());

    let work_base = std::env::var("FLOW_WRAPPER_WORK_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/work"));

    let state = AppState {
        resolver: Arc::new(StorageResolver::new()),
        worker_bin,
        work_base,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/run", post(run))
        .route("/probe-schema", post(probe_schema));

    #[cfg(feature = "new-geometry")]
    let app = app.route("/render-view", post(render_view));

    let app = app.with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    eprintln!("reearth-flow-worker-server listening on {addr}");
    axum::serve(listener, app).await.expect("axum serve failed");
}
