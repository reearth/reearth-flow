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
    /// Validated for path safety and passed to the worker CLI as `--job-id`
    /// (published in the completion event so the server can finalize the job).
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

/// The `/render-view` request body. Field names are the JSON the API sends; see
/// `renderViewRequest` in `server/api/internal/infrastructure/cloudrunworker/worker.go`.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RenderViewRequest {
    pub input_uri: String,
    pub output_uri: String,
    pub report_url: String,
    pub name: String,
    pub shape: String,
    #[serde(default)]
    pub row: Option<usize>,
    #[serde(default)]
    pub filter: Option<String>,
    pub draco: bool,
    pub texel_size: f64,
    pub texture_codec: String,
    pub target_tile_size: u64,
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub extent: i32,
    pub max_tile_bytes: u64,
}

/// The argument vector for `reearth-flow-worker render-view`.
#[cfg(feature = "new-geometry")]
pub fn build_render_view_args(req: &RenderViewRequest) -> Vec<String> {
    let mut args = vec![
        "render-view".to_string(),
        "--input".to_string(),
        req.input_uri.clone(),
        "--output".to_string(),
        req.output_uri.clone(),
        "--report-url".to_string(),
        req.report_url.clone(),
        "--name".to_string(),
        req.name.clone(),
        "--shape".to_string(),
        req.shape.clone(),
        "--texel-size".to_string(),
        req.texel_size.to_string(),
        "--texture-codec".to_string(),
        req.texture_codec.clone(),
        "--target-tile-size".to_string(),
        req.target_tile_size.to_string(),
        "--min-zoom".to_string(),
        req.min_zoom.to_string(),
        "--max-zoom".to_string(),
        req.max_zoom.to_string(),
        "--extent".to_string(),
        req.extent.to_string(),
        "--max-tile-bytes".to_string(),
        req.max_tile_bytes.to_string(),
    ];
    if let Some(row) = req.row {
        args.push("--row".to_string());
        args.push(row.to_string());
    }
    if let Some(filter) = &req.filter {
        args.push("--filter".to_string());
        args.push(filter.clone());
    }
    // Draco is on by default in the subcommand, so only the opt-out is passed.
    if !req.draco {
        args.push("--no-draco".to_string());
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

/// A work root for a request that carries no job id, removed when dropped.
///
/// Removal is tied to `Drop` rather than called at each return so it also runs
/// when the request itself is dropped mid-await. A child that is killed never
/// cleans up after itself, and on Cloud Run local disk is memory.
pub struct ScratchRoot(PathBuf);

impl ScratchRoot {
    pub fn new(base: &Path) -> std::io::Result<Self> {
        make_work_root(base, &uuid::Uuid::new_v4().to_string()).map(Self)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        cleanup_work_root(&self.0);
    }
}

/// How a child run under [`run_bounded`] ended.
#[derive(Debug)]
pub enum Bounded {
    /// The child exited by itself, with everything it wrote to stderr.
    Exited {
        status: std::process::ExitStatus,
        stderr: Vec<u8>,
    },
    /// The child was still running at the limit, and has been killed and reaped.
    TimedOut,
}

/// Run `command` to completion, killing it if it is still running after `limit`.
///
/// stderr is always captured; stdout is left as the caller configured it. The
/// child is also killed if the returned future is dropped, so a request that
/// goes away takes its child with it instead of leaving it running unobserved.
pub async fn run_bounded(
    mut command: tokio::process::Command,
    limit: std::time::Duration,
) -> std::io::Result<Bounded> {
    use tokio::io::AsyncReadExt;

    let mut child = command
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stderr = child.stderr.take();
    // stderr is drained alongside the wait: a child blocked on a full pipe
    // would otherwise never exit, and be reported as timed out.
    let finished = tokio::time::timeout(limit, async {
        let mut buf = Vec::new();
        let read = async {
            if let Some(stderr) = stderr.as_mut() {
                let _ = stderr.read_to_end(&mut buf).await;
            }
        };
        let (status, ()) = tokio::join!(child.wait(), read);
        status.map(|status| (status, buf))
    })
    .await;

    match finished {
        Ok(result) => {
            let (status, stderr) = result?;
            Ok(Bounded::Exited { status, stderr })
        }
        Err(_elapsed) => {
            // `kill` also waits, so the child is gone, not a zombie, on return.
            child.kill().await?;
            Ok(Bounded::TimedOut)
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

#[cfg(all(test, unix))]
mod bounded_tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// A shell that records its pid and then becomes `sleep`, so the pid it
    /// records is the process `run_bounded` has to stop.
    fn sleeper(pid_file: &Path, seconds: u32) -> tokio::process::Command {
        let mut command = tokio::process::Command::new("sh");
        command.arg("-c").arg(format!(
            "echo $$ > '{}'; exec sleep {seconds}",
            pid_file.display()
        ));
        command
    }

    async fn read_pid(pid_file: &Path) -> String {
        for _ in 0..100 {
            if let Ok(pid) = std::fs::read_to_string(pid_file) {
                if !pid.trim().is_empty() {
                    return pid.trim().to_string();
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("the child never wrote its pid");
    }

    /// Alive and not a zombie. A killed child nobody has reaped yet still shows
    /// up as a zombie, but it is no longer running anything.
    fn is_running(pid: &str) -> bool {
        let out = std::process::Command::new("ps")
            .args(["-o", "stat=", "-p", pid])
            .output()
            .expect("ps");
        let stat = String::from_utf8_lossy(&out.stdout);
        let stat = stat.trim();
        !stat.is_empty() && !stat.starts_with('Z')
    }

    #[tokio::test]
    async fn a_child_still_running_at_the_limit_is_killed() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("pid");

        let started = Instant::now();
        let outcome = run_bounded(sleeper(&pid_file, 30), Duration::from_millis(500))
            .await
            .unwrap();

        assert!(matches!(outcome, Bounded::TimedOut), "got {outcome:?}");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "returned at the limit, not when the child finished"
        );
        let pid = read_pid(&pid_file).await;
        assert!(!is_running(&pid), "pid {pid} outlived its limit");
    }

    #[tokio::test]
    async fn dropping_the_wait_kills_the_child() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("pid");

        // The outer timeout drops `run_bounded` mid-wait, the way a request
        // that goes away drops its handler.
        let dropped = tokio::time::timeout(
            Duration::from_millis(500),
            run_bounded(sleeper(&pid_file, 30), Duration::from_secs(60)),
        )
        .await;
        assert!(dropped.is_err(), "the wait should still have been pending");

        let pid = read_pid(&pid_file).await;
        // The kill is sent on drop; give the signal a moment to land.
        for _ in 0..50 {
            if !is_running(&pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("pid {pid} outlived the request that started it");
    }

    #[tokio::test]
    async fn a_child_that_finishes_in_time_reports_its_exit_and_stderr() {
        let mut command = tokio::process::Command::new("sh");
        command.arg("-c").arg("echo oops >&2; exit 3");

        let outcome = run_bounded(command, Duration::from_secs(10)).await.unwrap();

        let Bounded::Exited { status, stderr } = outcome else {
            panic!("got {outcome:?}");
        };
        assert_eq!(status.code(), Some(3));
        assert_eq!(stderr, b"oops\n");
    }

    #[tokio::test]
    async fn a_child_writing_more_than_a_pipe_holds_is_not_mistaken_for_a_hang() {
        // 1 MiB of stderr is far past a pipe buffer: unless stderr is drained
        // while waiting, the child blocks writing and never exits.
        let mut command = tokio::process::Command::new("sh");
        command
            .arg("-c")
            .arg("head -c 1048576 /dev/zero | tr '\\0' x >&2");

        let outcome = run_bounded(command, Duration::from_secs(10)).await.unwrap();

        let Bounded::Exited { status, stderr } = outcome else {
            panic!("got {outcome:?}");
        };
        assert!(status.success());
        assert_eq!(stderr.len(), 1_048_576);
    }

    #[tokio::test]
    async fn a_scratch_root_is_removed_with_what_a_killed_child_left_in_it() {
        let base = tempfile::tempdir().unwrap();
        let scratch = ScratchRoot::new(base.path()).unwrap();
        let root = scratch.path().to_path_buf();
        let pid_file = root.join("pid");

        let outcome = run_bounded(sleeper(&pid_file, 30), Duration::from_millis(300))
            .await
            .unwrap();
        assert!(matches!(outcome, Bounded::TimedOut));
        assert!(pid_file.exists(), "the child wrote into its scratch root");

        drop(scratch);
        assert!(!root.exists());
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod render_view_args_tests {
    use super::*;

    fn tiles_request() -> RenderViewRequest {
        RenderViewRequest {
            input_uri: "gs://b/in.jsonl.zst".to_string(),
            output_uri: "gs://b/out".to_string(),
            report_url: "gs://b/out/report.json".to_string(),
            name: "view".to_string(),
            shape: "tiles".to_string(),
            row: None,
            filter: None,
            draco: true,
            texel_size: 0.0,
            texture_codec: "jpeg".to_string(),
            target_tile_size: 1_048_576,
            min_zoom: 0,
            max_zoom: 15,
            extent: 4096,
            max_tile_bytes: 500_000,
        }
    }

    #[test]
    fn a_tiles_request_becomes_the_subcommand_line() {
        let mut req = tiles_request();
        req.filter = Some("foo > 1".to_string());
        let args = build_render_view_args(&req);
        assert_eq!(args[0], "render-view");
        assert!(args.contains(&"--shape".to_string()));
        assert!(args.contains(&"tiles".to_string()));
        assert!(args.contains(&"--filter".to_string()));
        assert!(!args.contains(&"--row".to_string()), "tiles carries no row");
        // Draco is on by default in the subcommand, so the opt-out must be absent.
        assert!(!args.contains(&"--no-draco".to_string()));
    }

    #[test]
    fn draco_off_becomes_the_opt_out_flag() {
        let mut req = tiles_request();
        req.draco = false;
        let args = build_render_view_args(&req);
        assert!(args.contains(&"--no-draco".to_string()));
    }

    #[test]
    fn a_gltf_request_carries_its_row_and_no_filter() {
        let mut req = tiles_request();
        req.shape = "gltf".to_string();
        req.row = Some(4);
        let args = build_render_view_args(&req);
        assert!(args.contains(&"--row".to_string()));
        assert!(args.contains(&"4".to_string()));
        assert!(!args.contains(&"--filter".to_string()));
    }

    /// Parses `argv` (as `try_get_matches_from` would see it, program name
    /// included) all the way through `render_view::args::parse`, which is
    /// where all four validation rules actually live (shape/row/filter
    /// combinations, min-zoom vs. max-zoom). Stopping at
    /// `try_get_matches_from` would pass even if the route built argv that
    /// `parse` rejects.
    fn parse_argv(argv: Vec<String>) -> Result<crate::render_view::args::RenderViewArgs, String> {
        let matches = crate::render_view::build_render_view_command()
            .try_get_matches_from(argv)
            .map_err(|e| e.to_string())?;
        crate::render_view::args::parse(matches)
    }

    #[test]
    fn a_tiles_request_round_trips_through_the_validator() {
        let mut req = tiles_request();
        req.filter = Some("foo > 1".to_string());
        let args = build_render_view_args(&req);
        let argv = std::iter::once("render-view".to_string())
            .chain(args.into_iter().skip(1))
            .collect();
        let parsed = parse_argv(argv);
        assert!(
            parsed.is_ok(),
            "built args must pass validation: {parsed:?}"
        );
    }

    #[test]
    fn a_gltf_request_round_trips_through_the_validator() {
        // Validation is where a shape mismatch actually bites (`gltf`
        // requires `row` and rejects `filter`), and the tiles case above
        // exercises none of that: it never gets exercised if every seam test
        // only ever builds a tiles request.
        let mut req = tiles_request();
        req.shape = "gltf".to_string();
        req.row = Some(4);
        let args = build_render_view_args(&req);
        let argv = std::iter::once("render-view".to_string())
            .chain(args.into_iter().skip(1))
            .collect();
        let parsed = parse_argv(argv);
        assert!(
            parsed.is_ok(),
            "built args must pass validation: {parsed:?}"
        );
    }
}
