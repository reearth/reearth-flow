//! Regression tests for: "Canvas showing not most recent contents"
//!
//! Root cause: `KVStore::get()` previously mapped ALL GCS errors to `Ok(None)`,
//! meaning a transient 500 from GCS was treated as "document does not exist".
//! When `load_doc_v2` saw `None`, it returned an empty document — the user saw
//! a blank canvas instead of their most recent work.
//!
//! The fix: `is_not_found()` discriminates 404 (legitimate not-found) from all
//! other errors (500, 403, network failures), which now propagate as `Err`.

use google_cloud_storage::http::error::ErrorResponse;
use google_cloud_storage::http::Error as GcsError;

/// Reproduce the helper that guards the KVStore::get() path.
/// This is extracted from `infrastructure/gcs/mod.rs`.
fn is_not_found(err: &GcsError) -> bool {
    match err {
        GcsError::Response(resp) => resp.code == 404,
        GcsError::HttpClient(reqwest_err) => reqwest_err.status().is_some_and(|s| s.as_u16() == 404),
        _ => false,
    }
}

fn make_response_error(code: u16, message: &str) -> GcsError {
    GcsError::Response(ErrorResponse {
        code,
        errors: vec![],
        message: message.to_string(),
    })
}

// ─── Bug reproduction: the OLD behaviour ────────────────────────────────────

/// BEFORE THE FIX: Any GCS error was swallowed → `Ok(None)` → empty doc → stale canvas.
///
/// ```rust
/// // OLD CODE in KVStore::get():
/// match self.client.download_object(&request, &Range::default()).await {
///     Ok(data) => Ok(Some(data)),
///     Err(_) => Ok(None),   // <── BUG: 500 treated as "not found"
/// }
/// ```
///
/// This test shows that a 500 error IS NOT a "not found" — if we treated it as
/// one, the caller would load an empty document and the user would see stale/blank content.
#[test]
fn bug_gcs_500_must_not_be_treated_as_not_found() {
    let err = make_response_error(500, "Internal Server Error");

    // Under the old code, ANY error → Ok(None). This was wrong.
    // Under the fix, only 404 → Ok(None). Everything else propagates.
    assert!(
        !is_not_found(&err),
        "BUG REGRESSION: A 500 error was treated as 'not found', \
         causing load_doc_v2 to return an empty document (stale canvas)"
    );
}

#[test]
fn bug_gcs_503_must_not_be_treated_as_not_found() {
    let err = make_response_error(503, "Service Unavailable");
    assert!(
        !is_not_found(&err),
        "BUG REGRESSION: A 503 error was treated as 'not found'"
    );
}

#[test]
fn bug_gcs_403_must_not_be_treated_as_not_found() {
    let err = make_response_error(403, "Forbidden");
    assert!(
        !is_not_found(&err),
        "BUG REGRESSION: A 403 (permission denied) was treated as 'not found'"
    );
}

// ─── Fix verification: correct behaviour ────────────────────────────────────

#[test]
fn fix_gcs_404_is_correctly_identified_as_not_found() {
    let err = make_response_error(404, "Not Found");
    assert!(
        is_not_found(&err),
        "A genuine 404 should return Ok(None) — this is a new document"
    );
}

/// Simulates the full decision path that `KVStore::get()` now takes.
/// Returns `Ok(None)` only for 404; returns `Err` for everything else.
fn simulated_kv_get(gcs_result: Result<Vec<u8>, GcsError>) -> Result<Option<Vec<u8>>, GcsError> {
    match gcs_result {
        Ok(data) => Ok(Some(data)),
        Err(e) if is_not_found(&e) => Ok(None),
        Err(e) => Err(e),
    }
}

#[test]
fn fix_kv_get_returns_data_on_success() {
    let result = simulated_kv_get(Ok(vec![1, 2, 3]));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(vec![1, 2, 3]));
}

#[test]
fn fix_kv_get_returns_none_on_404() {
    let result = simulated_kv_get(Err(make_response_error(404, "Not Found")));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[test]
fn fix_kv_get_propagates_500_as_error() {
    let result = simulated_kv_get(Err(make_response_error(500, "Internal Server Error")));
    assert!(
        result.is_err(),
        "500 must propagate as Err so the caller knows the fetch failed — \
         NOT silently return None (which would load a blank document)"
    );
}

#[test]
fn fix_kv_get_propagates_403_as_error() {
    let result = simulated_kv_get(Err(make_response_error(403, "Forbidden")));
    assert!(
        result.is_err(),
        "403 must propagate — the document exists but we can't read it"
    );
}
