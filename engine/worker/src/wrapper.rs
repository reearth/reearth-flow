use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RunRequest {
    /// Used by the wrapper to build the work-root path; not passed to the worker CLI.
    pub job_id: String,
    pub workflow_url: String,
    pub metadata_path: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub previous_job_id: Option<String>,
    #[serde(default)]
    pub start_node_id: Option<String>,
    /// Polled by the wrapper for cancellation; not passed to the worker CLI.
    pub cancel_flag_uri: String,
}

/// Request body for `POST /probe-schema`.
///
/// Probe is read-only and fast: no work-root, no cancel-flag, no metadata.
#[derive(Debug, Deserialize, Clone)]
pub struct ProbeRequest {
    /// Validated for path safety but not passed to the worker CLI.
    pub job_id: String,
    pub workflow_url: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub sample_size: Option<usize>,
    /// Destination URI (gs://...) where the worker writes the JSON report.
    pub report_url: String,
}

/// Build the argv (excluding the program name) for the
/// `reearth-flow-worker probe-schema` subcommand.
pub fn build_probe_args(req: &ProbeRequest) -> Vec<String> {
    let mut args = vec![
        "probe-schema".to_string(),
        "--workflow".to_string(),
        req.workflow_url.clone(),
        "--report-url".to_string(),
        req.report_url.clone(),
        "--job-id".to_string(),
        req.job_id.clone(),
    ];
    for (k, v) in &req.variables {
        args.push("--var".to_string());
        args.push(format!("{k}={v}"));
    }
    if let Some(n) = req.sample_size {
        args.push("--sample-size".to_string());
        args.push(n.to_string());
    }
    args
}

/// Build the argv (excluding the program name) for the `reearth-flow-worker` CLI.
pub fn build_worker_args(req: &RunRequest) -> Vec<String> {
    let mut args = vec![
        "--workflow".to_string(),
        req.workflow_url.clone(),
        "--metadata-path".to_string(),
        req.metadata_path.clone(),
    ];
    for (k, v) in &req.variables {
        args.push("--var".to_string());
        args.push(format!("{k}={v}"));
    }
    if let Some(prev) = &req.previous_job_id {
        args.push("--previous-job-id".to_string());
        args.push(prev.clone());
    }
    if let Some(node) = &req.start_node_id {
        args.push("--start-node-id".to_string());
        args.push(node.clone());
    }
    args
}

/// Create a unique work root for a request under `base` (e.g. /work).
pub fn make_work_root(base: &Path, job_id: &str) -> std::io::Result<PathBuf> {
    let root = base.join(job_id);
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

/// Returns true if the cancel-flag object exists at `uri`.
/// Uses `head()` (backend-aware) not `exists()` (local-fs only).
pub async fn cancel_requested(resolver: &Arc<StorageResolver>, uri: &Uri) -> bool {
    match resolver.resolve(uri) {
        // head() Err is the normal "no flag yet" case (every poll) — do not log it.
        Ok(storage) => storage.head(uri.path().as_path()).await.is_ok(),
        Err(e) => {
            eprintln!("[wrapper] resolve cancel uri failed: {e}");
            false
        }
    }
}

/// Validate a job_id is safe as a single path segment (prevents rm -rf traversal).
/// Accepts ASCII alphanumeric, `-`, and `_` only (covers UUIDs).
pub fn validate_job_id(job_id: &str) -> Result<(), String> {
    if job_id.is_empty() {
        return Err("job_id must not be empty".to_string());
    }
    if !job_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "job_id must contain only ASCII alphanumeric characters, '-', or '_'".to_string(),
        );
    }
    Ok(())
}

/// Remove the work root; never panics. Logs on failure.
pub fn cleanup_work_root(root: &Path) {
    if root.exists() {
        if let Err(e) = std::fs::remove_dir_all(root) {
            eprintln!("[wrapper] failed to cleanup {root:?}: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn maps_required_args() {
        let req = RunRequest {
            job_id: "j1".into(),
            workflow_url: "https://wf".into(),
            metadata_path: "gs://md".into(),
            variables: Default::default(),
            previous_job_id: None,
            start_node_id: None,
            cancel_flag_uri: "gs://b/cancel/j1".into(),
        };
        assert_eq!(
            build_worker_args(&req),
            vec!["--workflow", "https://wf", "--metadata-path", "gs://md"]
        );
    }

    #[test]
    fn maps_optional_args() {
        let req = RunRequest {
            job_id: "j1".into(),
            workflow_url: "https://wf".into(),
            metadata_path: "gs://md".into(),
            variables: HashMap::from([("A".into(), "1".into())]),
            previous_job_id: Some("prev".into()),
            start_node_id: Some("node".into()),
            cancel_flag_uri: "gs://b/cancel/j1".into(),
        };
        let args = build_worker_args(&req);
        assert!(args.windows(2).any(|w| w == ["--var", "A=1"]));
        assert!(args.windows(2).any(|w| w == ["--previous-job-id", "prev"]));
        assert!(args.windows(2).any(|w| w == ["--start-node-id", "node"]));
    }

    #[test]
    fn maps_multiple_variables() {
        let req = RunRequest {
            job_id: "j1".into(),
            workflow_url: "https://wf".into(),
            metadata_path: "gs://md".into(),
            variables: HashMap::from([("A".into(), "1".into()), ("B".into(), "2".into())]),
            previous_job_id: None,
            start_node_id: None,
            cancel_flag_uri: "gs://b/cancel/j1".into(),
        };
        let args = build_worker_args(&req);
        assert!(args.windows(2).any(|w| w == ["--var", "A=1"]));
        assert!(args.windows(2).any(|w| w == ["--var", "B=2"]));
        assert_eq!(args.iter().filter(|a| *a == "--var").count(), 2);
    }

    #[test]
    fn probe_args_required_only() {
        let req = ProbeRequest {
            job_id: "j1".into(),
            workflow_url: "gs://b/wf.yml".into(),
            variables: Default::default(),
            sample_size: None,
            report_url: "gs://b/reports/j1.json".into(),
        };
        assert_eq!(
            build_probe_args(&req),
            vec![
                "probe-schema",
                "--workflow",
                "gs://b/wf.yml",
                "--report-url",
                "gs://b/reports/j1.json",
                "--job-id",
                "j1",
            ]
        );
    }

    #[test]
    fn probe_args_with_sample_size_and_vars() {
        let req = ProbeRequest {
            job_id: "j1".into(),
            workflow_url: "gs://b/wf.yml".into(),
            variables: HashMap::from([("A".into(), "1".into()), ("B".into(), "2".into())]),
            sample_size: Some(25),
            report_url: "gs://b/reports/j1.json".into(),
        };
        let args = build_probe_args(&req);
        // Subcommand first, required flags present including --report-url.
        assert_eq!(args.first().map(String::as_str), Some("probe-schema"));
        assert!(args
            .windows(2)
            .any(|w| w == ["--workflow", "gs://b/wf.yml"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--report-url", "gs://b/reports/j1.json"]));
        // Optional --sample-size mapped.
        assert!(args.windows(2).any(|w| w == ["--sample-size", "25"]));
        // Each variable becomes a repeated --var k=v pair.
        assert!(args.windows(2).any(|w| w == ["--var", "A=1"]));
        assert!(args.windows(2).any(|w| w == ["--var", "B=2"]));
        assert_eq!(args.iter().filter(|a| *a == "--var").count(), 2);
    }

    #[test]
    fn probe_args_omits_sample_size_when_absent() {
        let req = ProbeRequest {
            job_id: "j1".into(),
            workflow_url: "gs://b/wf.yml".into(),
            variables: Default::default(),
            sample_size: None,
            report_url: "gs://b/reports/j1.json".into(),
        };
        let args = build_probe_args(&req);
        assert!(!args.iter().any(|a| a == "--sample-size"));
    }

    #[test]
    fn work_root_create_and_cleanup() {
        let base = std::env::temp_dir().join(format!("wrap-test-{}", uuid::Uuid::new_v4()));
        let root = make_work_root(&base, "job-123").unwrap();
        std::fs::write(root.join("blob"), b"x").unwrap();
        assert!(root.exists());
        cleanup_work_root(&root);
        assert!(!root.exists());
        cleanup_work_root(&base); // tidy parent
    }

    #[test]
    fn validate_job_id_accepts_uuid() {
        assert!(validate_job_id("6b34bf72-1993-450d-b447-60a586a792dc").is_ok());
    }

    #[test]
    fn validate_job_id_rejects_empty_and_traversal() {
        assert!(validate_job_id("").is_err());
        assert!(validate_job_id("../etc").is_err());
        assert!(validate_job_id("a/b").is_err());
        assert!(validate_job_id("a\\b").is_err());
        assert!(validate_job_id(".").is_err());
        assert!(validate_job_id("..").is_err());
    }

    #[tokio::test]
    async fn cancel_flag_detected_via_file_uri() {
        let dir = std::env::temp_dir().join(format!("cancel-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let flag = dir.join("flag");
        // Construct a file:// URI from the absolute path.
        let uri_str = format!("file://{}", flag.display());
        let uri = Uri::for_test(&uri_str);
        let resolver = Arc::new(StorageResolver::new());

        // Flag does not exist yet — cancel_requested must return false.
        assert!(!cancel_requested(&resolver, &uri).await);

        // Write the flag file — cancel_requested must now return true.
        std::fs::write(&flag, b"cancel").unwrap();
        assert!(cancel_requested(&resolver, &uri).await);

        std::fs::remove_dir_all(&dir).ok();
    }
}
