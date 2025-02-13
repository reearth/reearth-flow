use std::sync::Arc;

#[cfg(feature = "auth")]
use axum::body::Bytes;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, Query,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use time::OffsetDateTime;
use yrs::{updates::encoder::Encode, Doc, ReadTxn, StateVector, Transact};

use crate::{
    group::BroadcastGroup, pool::BroadcastPool, storage::kv::DocOps, ws::WarpConn, AppState,
    RollbackQuery,
};

#[cfg(feature = "auth")]
use crate::AuthQuery;

#[derive(Serialize)]
pub struct UpdateHistoryEntry {
    pub update: Vec<u8>,
    pub clock: u32,
    pub timestamp: OffsetDateTime,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    #[cfg(feature = "auth")] Query(auth): Query<AuthQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Response {
    let doc_id = normalize_doc_id(&doc_id);

    #[cfg(feature = "auth")]
    if !verify_auth(&state, &auth.token, &doc_id).await {
        return Bytes::from("Unauthorized").into_response();
    }

    let bcast = match state.pool.get_or_create_group(&doc_id).await {
        Ok(group) => group,
        Err(e) => {
            tracing::error!("Failed to get or create group for {}: {}", doc_id, e);
            return Response::builder()
                .status(500)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket, bcast, doc_id, state.pool.clone()))
}

pub async fn get_latest_doc(
    Path(doc_id): Path<String>,
    #[cfg(feature = "auth")] Query(auth): Query<AuthQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let doc_id = normalize_doc_id(&doc_id);

    #[cfg(feature = "auth")]
    if !verify_auth(&state, &auth.token, &doc_id).await {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let store = state.pool.get_store();
    let doc = Doc::new();
    let mut txn = doc.transact_mut();

    match store.load_doc(&doc_id, &mut txn).await {
        Ok(true) => {
            drop(txn);
            let read_txn = doc.transact();
            let state = read_txn.encode_diff_v1(&StateVector::default());
            (StatusCode::OK, Json(state)).into_response()
        }
        Ok(false) => (StatusCode::NOT_FOUND, "Document not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get document: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

pub async fn get_doc_history(
    Path(doc_id): Path<String>,
    #[cfg(feature = "auth")] Query(auth): Query<AuthQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let doc_id = normalize_doc_id(&doc_id);

    #[cfg(feature = "auth")]
    if !verify_auth(&state, &auth.token, &doc_id).await {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let store = state.pool.get_store();

    match store.get_updates(&doc_id).await {
        Ok(updates) => {
            let history: Vec<UpdateHistoryEntry> = updates
                .into_iter()
                .map(|info| UpdateHistoryEntry {
                    update: info.update.encode_v1(),
                    clock: info.clock,
                    timestamp: info.timestamp,
                })
                .collect();
            (StatusCode::OK, Json(history)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get document history: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

pub async fn rollback_doc(
    Path(doc_id): Path<String>,
    Query(query): Query<RollbackQuery>,
    #[cfg(feature = "auth")] Query(auth): Query<AuthQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let doc_id = normalize_doc_id(&doc_id);

    #[cfg(feature = "auth")]
    if !verify_auth(&state, &auth.token, &doc_id).await {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let store = state.pool.get_store();

    match store.rollback_to(&doc_id, query.clock).await {
        Ok(doc) => {
            let txn = doc.transact();
            let state = txn.encode_diff_v1(&StateVector::default());
            (StatusCode::OK, Json(state)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to rollback document: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

async fn handle_socket(
    socket: WebSocket,
    bcast: Arc<BroadcastGroup>,
    doc_id: String,
    pool: Arc<BroadcastPool>,
) {
    bcast.increment_connections();
    let conn = WarpConn::new(bcast, socket);
    if let Err(e) = conn.await {
        tracing::error!("WebSocket connection error: {}", e);
    }
    pool.remove_connection(&doc_id).await;
}

fn normalize_doc_id(doc_id: &str) -> String {
    doc_id.strip_suffix(":main").unwrap_or(doc_id).to_string()
}

#[cfg(feature = "auth")]
async fn verify_auth(state: &AppState, token: &str, doc_id: &str) -> bool {
    match state.auth.verify_token(token).await {
        Ok(true) => true,
        Ok(false) => {
            tracing::warn!(
                "Authentication failed for doc_id: {}, token: {}",
                doc_id,
                token
            );
            false
        }
        Err(e) => {
            tracing::error!("Authentication error: {}", e);
            false
        }
    }
}
