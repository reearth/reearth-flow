use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::{extract::State, http::StatusCode, routing::get, routing::post, Json, Router};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_worker::wrapper::{
    build_worker_args, cancel_requested, cleanup_work_root, make_work_root, validate_job_id,
    RunRequest,
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

/// Handle a `/run` POST request.
/// Response: `{"status": "COMPLETED"|"CANCELLED"|"FAILED", "error"?: string}`
async fn run(
    State(st): State<AppState>,
    Json(req): Json<RunRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Validate before touching the filesystem — prevents rm -rf path traversal.
    if let Err(e) = validate_job_id(&req.job_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "FAILED", "error": e})),
        );
    }

    let root = match make_work_root(&st.work_base, &req.job_id) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "FAILED", "error": e.to_string()})),
            );
        }
    };

    let cancel_uri = match Uri::from_str(&req.cancel_flag_uri) {
        Ok(u) => u,
        Err(e) => {
            cleanup_work_root(&root);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "FAILED", "error": format!("bad cancel_flag_uri: {e}")})),
            );
        }
    };

    let args = build_worker_args(&req);
    let mut child = match tokio::process::Command::new(&st.worker_bin)
        .args(&args)
        .env("FLOW_RUNTIME_WORKING_DIRECTORY", &root)
        .env("TMPDIR", &root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            cleanup_work_root(&root);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "FAILED", "error": e.to_string()})),
            );
        }
    };
    eprintln!(
        "[wrapper] spawned job {} pid={}",
        req.job_id,
        child.id().unwrap_or(0)
    );

    // Check exit before cancel flag: COMPLETED wins if both arrive simultaneously.
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                cleanup_work_root(&root);
                return if status.success() {
                    (StatusCode::OK, Json(json!({"status": "COMPLETED"})))
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            json!({"status": "FAILED", "error": format!("worker exit: {status}")}),
                        ),
                    )
                };
            }
            Ok(None) => {}
            Err(e) => {
                cleanup_work_root(&root);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "FAILED", "error": e.to_string()})),
                );
            }
        }

        let cancelled = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            cancel_requested(&st.resolver, &cancel_uri),
        )
        .await
        .unwrap_or(false);
        if cancelled {
            eprintln!("[wrapper] cancel flag seen for {} -> killing", req.job_id);
            let _ = child.kill().await;
            let _ = child.wait().await;
            cleanup_work_root(&root);
            return (StatusCode::OK, Json(json!({"status": "CANCELLED"})));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() {
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
        .route("/run", post(run))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    eprintln!("reearth-flow-worker-server listening on {addr}");
    axum::serve(listener, app).await.expect("axum serve failed");
}
