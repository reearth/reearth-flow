use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_int;
use std::sync::Arc;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::broadcast::group::RedisConfig;
use crate::pool::BroadcastPool;
use crate::storage::gcs::{GcsConfig, GcsStore};
use crate::storage::kv::DocOps;

fn create_pool(gcs_config: GcsConfig, redis_config: RedisConfig) -> Option<Arc<BroadcastPool>> {
    // Initialize GCS store
    let store = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(GcsStore::new_with_config(gcs_config))
    {
        Ok(store) => Arc::new(store),
        Err(_) => return None,
    };

    // Create broadcast pool
    Some(Arc::new(BroadcastPool::new(store, redis_config)))
}

#[derive(Serialize, Deserialize)]
struct DocumentResponse {
    doc_id: String,
    content: Vec<u8>,
    clock: i32,
}

#[derive(Serialize, Deserialize)]
struct HistoryVersion {
    version_id: String,
    timestamp: String,
    content: Vec<u8>,
    clock: i32,
}

#[derive(Serialize, Deserialize)]
struct DocumentHistory {
    doc_id: String,
    versions: Vec<HistoryVersion>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    gcs_bucket: String,
    gcs_endpoint: Option<String>,
    redis_url: String,
    redis_ttl: u64,
}

#[no_mangle]
/// Get the latest version of a document.
///
/// # Safety
/// - `doc_id` must be a valid null-terminated C string pointer
/// - `config_json` must be a valid null-terminated C string pointer containing valid JSON
pub unsafe extern "C" fn get_latest_document(
    doc_id: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    let doc_id = unsafe {
        match CStr::from_ptr(doc_id).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let config: Config = unsafe {
        match CStr::from_ptr(config_json)
            .to_str()
            .map_err(|_| "UTF-8 error")
            .and_then(|s| serde_json::from_str(s).map_err(|_| "JSON error"))
        {
            Ok(c) => c,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let gcs_config = GcsConfig {
        bucket_name: config.gcs_bucket,
        endpoint: config.gcs_endpoint,
    };

    let redis_config = RedisConfig {
        url: config.redis_url,
        ttl: config.redis_ttl,
    };

    let pool = match create_pool(gcs_config, redis_config) {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };

    let doc = Doc::new();
    let mut txn = doc.transact_mut();

    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        match pool.get_store().load_doc(&doc_id, &mut txn).await {
            Ok(true) => {
                drop(txn);
                let read_txn = doc.transact();
                let state = read_txn.encode_diff_v1(&StateVector::default());

                // Get the latest clock from updates
                let clock = match pool.get_store().get_updates(&doc_id).await {
                    Ok(updates) if !updates.is_empty() => updates.last().unwrap().clock as i32,
                    _ => 0,
                };

                let response = DocumentResponse {
                    doc_id,
                    content: state,
                    clock,
                };

                serde_json::to_string(&response).ok()
            }
            _ => None,
        }
    });

    match result {
        Some(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
/// Get the version history of a document.
///
/// # Safety
/// - `doc_id` must be a valid null-terminated C string pointer
/// - `config_json` must be a valid null-terminated C string pointer containing valid JSON
pub unsafe extern "C" fn get_document_history(
    doc_id: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    let doc_id = unsafe {
        match CStr::from_ptr(doc_id).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let config: Config = unsafe {
        match CStr::from_ptr(config_json)
            .to_str()
            .map_err(|_| "UTF-8 error")
            .and_then(|s| serde_json::from_str(s).map_err(|_| "JSON error"))
        {
            Ok(c) => c,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let gcs_config = GcsConfig {
        bucket_name: config.gcs_bucket,
        endpoint: config.gcs_endpoint,
    };

    let redis_config = RedisConfig {
        url: config.redis_url,
        ttl: config.redis_ttl,
    };

    let pool = match create_pool(gcs_config, redis_config) {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        match pool.get_store().get_updates(&doc_id).await {
            Ok(updates) => {
                let versions: Vec<HistoryVersion> = updates
                    .into_iter()
                    .map(|info| HistoryVersion {
                        version_id: info.clock.to_string(),
                        timestamp: info
                            .timestamp
                            .format(&time::format_description::well_known::Iso8601::DEFAULT)
                            .unwrap_or_default(),
                        content: info.update.encode_v1(),
                        clock: info.clock as i32,
                    })
                    .collect();

                let history = DocumentHistory { doc_id, versions };

                serde_json::to_string(&history).ok()
            }
            Err(_) => None,
        }
    });

    match result {
        Some(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
/// Rollback a document to a specific version.
///
/// # Safety
/// - `doc_id` must be a valid null-terminated C string pointer
/// - `config_json` must be a valid null-terminated C string pointer containing valid JSON
pub unsafe extern "C" fn rollback_document(
    doc_id: *const c_char,
    target_clock: c_int,
    config_json: *const c_char,
) -> *mut c_char {
    let doc_id = unsafe {
        match CStr::from_ptr(doc_id).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let config: Config = unsafe {
        match CStr::from_ptr(config_json)
            .to_str()
            .map_err(|_| "UTF-8 error")
            .and_then(|s| serde_json::from_str(s).map_err(|_| "JSON error"))
        {
            Ok(c) => c,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let gcs_config = GcsConfig {
        bucket_name: config.gcs_bucket,
        endpoint: config.gcs_endpoint,
    };

    let redis_config = RedisConfig {
        url: config.redis_url,
        ttl: config.redis_ttl,
    };

    let pool = match create_pool(gcs_config, redis_config) {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        match pool
            .get_store()
            .rollback_to(&doc_id, target_clock as u32)
            .await
        {
            Ok(doc) => {
                let txn = doc.transact();
                let state = txn.encode_state_as_update_v1(&StateVector::default());

                let response = DocumentResponse {
                    doc_id,
                    content: state,
                    clock: target_clock,
                };

                serde_json::to_string(&response).ok()
            }
            Err(_) => None,
        }
    });

    match result {
        Some(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
/// Frees a C string pointer that was allocated by Rust.
///
/// # Safety
/// - `ptr` must be null or a pointer previously returned by a function from this module
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}
