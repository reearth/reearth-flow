use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;

/// Owns URI parsing, storage backend acquisition, and bytes write for sink
/// output handling. Expression evaluation is the responsibility of the caller.
#[derive(Clone, Debug)]
pub struct SinkOutput {
    resolved: Uri,
    storage: Arc<Storage>,
}

impl SinkOutput {
    /// Construct a `SinkOutput` from a sandbox root and a relative path string.
    ///
    /// Validates `relative_path` as a strict-relative sink output (empty /
    /// leading-trailing whitespace / `.` / `..` / `scheme://...` / leading `/`
    /// / leading `~` / post-join traversal / post-join root-equality), joins
    /// it against `sandbox_root`, and acquires the storage backend in one shot.
    ///
    /// For migration from the previous absolute-URI API, see the error message
    /// of the URI-scheme rejection path, which names `workerArtifactPath`.
    pub fn new(
        sandbox_root: &Uri,
        relative_path: &str,
        resolver: &StorageResolver,
    ) -> Result<Self, BoxedError> {
        let resolved = validate_and_join(sandbox_root, relative_path)?;
        let storage = resolver.resolve(&resolved).map_err(|e| -> BoxedError {
            format!("SinkOutput: failed to resolve storage for {resolved}: {e}").into()
        })?;
        Ok(Self { resolved, storage })
    }

    /// Return the resolved URI this output writes to.
    pub fn uri(&self) -> &Uri {
        &self.resolved
    }

    /// Write bytes to the resolved URI via the eagerly-acquired storage backend.
    ///
    /// Semantics are "put" (full overwrite), not append.
    pub fn write(&self, bytes: Bytes) -> Result<(), BoxedError> {
        self.storage
            .put_sync(self.resolved.path().as_path(), bytes)?;
        Ok(())
    }
}

/// Validate `path` as a strict-relative sink output and resolve it against
/// `sandbox_root`. Returns the joined and sandbox-validated URI.
///
/// Rejection rules: empty / leading-trailing whitespace / `.` / `..` /
/// `scheme://...` / leading `/` / leading `~` / post-join paths that escape
/// via `..` / post-join paths that resolve to the sandbox root itself.
fn validate_and_join(sandbox_root: &Uri, path: &str) -> Result<Uri, BoxedError> {
    // ---- 1. Validate the relative path ----
    if path.is_empty() {
        return Err("sink output path is empty; provide a relative path \
                    like 'out.gpkg' or 'group/a.geojson'"
            .into());
    }
    if path != path.trim() {
        return Err(format!(
            "sink output {path:?} has leading or trailing whitespace; \
             use 'out.gpkg' not ' out.gpkg ' or 'out.gpkg '"
        )
        .into());
    }
    if path == "." || path == ".." {
        return Err(format!(
            "sink output {path:?} is not a filename; provide a relative \
             path like 'out.gpkg' or 'group/a.geojson'"
        )
        .into());
    }
    if path.contains("://") {
        return Err(format!(
            "sink output {path:?}: absolute URIs are not allowed. \
             Sink paths must be relative to the per-job artifact directory. \
             If your workflow uses Url(env[\"workerArtifactPath\"]) / x, \
             replace the whole expression with just x — the engine joins \
             it internally."
        )
        .into());
    }
    if path.starts_with('/') {
        return Err(format!(
            "sink output {path:?}: leading '/' is ambiguous. \
             Use a relative path without a leading slash, e.g. 'foo/bar'."
        )
        .into());
    }
    if path.starts_with('~') {
        return Err(format!(
            "sink output {path:?}: leading '~' (home expansion) is not \
             supported. Use a relative path under the per-job artifact directory."
        )
        .into());
    }

    // ---- 2. Join against sandbox_root ----
    let resolved = sandbox_root.join(path).map_err(|e| -> BoxedError {
        format!("SinkOutput: failed to join {path:?} with sandbox_root: {e}").into()
    })?;

    // ---- 3. Sandbox-validate (catches ".." segments that survived join) ----
    crate::sandbox::ensure_under(sandbox_root, &resolved)
        .map_err(|e| -> BoxedError { Box::new(e) })?;

    // ---- 4. Post-join: resolved must not be the root itself ----
    // Compare ignoring trailing slashes — Uri::join may strip them via normalization.
    let resolved_norm = resolved.as_str().trim_end_matches('/');
    let root_norm = sandbox_root.as_str().trim_end_matches('/');
    if resolved_norm == root_norm {
        return Err(format!(
            "sink output {path:?} resolves to the artifact directory \
             itself (no filename). Provide a relative path that ends \
             in a filename."
        )
        .into());
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use reearth_flow_storage::resolve::StorageResolver;
    use std::str::FromStr;
    use tempfile::tempdir;

    fn file_uri(path: &std::path::Path) -> String {
        format!("file://{}", path.display())
    }

    fn make_resolver() -> Arc<StorageResolver> {
        Arc::new(StorageResolver::new())
    }

    #[test]
    fn write_persists_bytes_to_resolved_uri() {
        let tmp = tempdir().unwrap();
        let root = Uri::from_str(&file_uri(tmp.path())).unwrap();
        let resolver = make_resolver();
        let out = SinkOutput::new(&root, "write_target.bin", &resolver).unwrap();
        out.write(Bytes::from_static(b"hello")).unwrap();
        let content = std::fs::read(tmp.path().join("write_target.bin")).unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn new_nested_path_composes_correctly() {
        let tmp = tempdir().unwrap();
        let root = Uri::from_str(&file_uri(tmp.path())).unwrap();
        let resolver = make_resolver();
        // Simulate what callers do instead of .join(): compose the string directly.
        let sub = SinkOutput::new(&root, "base_dir/group/a.geojson", &resolver).unwrap();
        assert_eq!(
            sub.uri().path().as_path(),
            tmp.path().join("base_dir").join("group/a.geojson")
        );
    }

    #[test]
    fn new_rejects_traversal_composed_path() {
        let tmp = tempdir().unwrap();
        let root = Uri::from_str(&file_uri(tmp.path())).unwrap();
        let resolver = make_resolver();
        // Composed paths that try to escape must be rejected.
        let result = SinkOutput::new(&root, "some_dir/../../../etc/passwd", &resolver);
        assert!(
            result.is_err(),
            "composed traversal path must be rejected; got: {result:?}"
        );
    }

    #[test]
    fn clone_shares_storage_backend() {
        let tmp = tempdir().unwrap();
        let root = Uri::from_str(&file_uri(tmp.path())).unwrap();
        let resolver = make_resolver();
        let original = SinkOutput::new(&root, "some_output.bin", &resolver).unwrap();
        let cloned = original.clone();
        // `Arc::ptr_eq` confirms both SinkOutputs point at the same underlying storage handle.
        assert!(
            Arc::ptr_eq(&original.storage, &cloned.storage),
            "clone must share the same storage Arc, not create a new one"
        );
    }

    #[test]
    fn sink_output_is_send_sync_clone_debug() {
        fn assert_send_sync_clone_debug<T: Send + Sync + Clone + std::fmt::Debug>() {}
        assert_send_sync_clone_debug::<SinkOutput>();
    }

    // ---- Strict-relative chokepoint tests (issue #2117) ----

    fn root_uri(root: &str) -> Uri {
        Uri::from_str(root).unwrap()
    }

    #[test]
    fn new_accepts_simple_relative() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let out = SinkOutput::new(&root, "out.gpkg", &resolver).unwrap();
        assert_eq!(out.uri().path().as_path(), tmp.path().join("out.gpkg"));
    }

    #[test]
    fn new_accepts_subdir_relative() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let out = SinkOutput::new(&root, "group/a.geojson", &resolver).unwrap();
        assert_eq!(
            out.uri().path().as_path(),
            tmp.path().join("group").join("a.geojson")
        );
    }

    #[test]
    fn new_accepts_relative_with_gs_root() {
        let root = root_uri("gs://my-bucket/jobs/abc/");
        let resolver = StorageResolver::new();
        // Validation and join must succeed even if the default StorageResolver
        // does not have a gs:// backend registered.
        let result = SinkOutput::new(&root, "out.json", &resolver);
        match result {
            Ok(out) => assert_eq!(out.uri().as_str(), "gs://my-bucket/jobs/abc/out.json"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("failed to resolve storage"),
                    "validation should pass; only storage resolution may fail; got: {msg}"
                );
            }
        }
    }

    #[test]
    fn new_accepts_relative_with_ram_root() {
        let root = root_uri("ram:///jobs/abc/");
        let resolver = StorageResolver::new();
        // Validation and join must succeed even if the default StorageResolver
        // does not have a ram:// backend registered.
        let result = SinkOutput::new(&root, "out.json", &resolver);
        match result {
            Ok(out) => assert_eq!(out.uri().as_str(), "ram:///jobs/abc/out.json"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("failed to resolve storage"),
                    "validation should pass; only storage resolution may fail; got: {msg}"
                );
            }
        }
    }

    #[test]
    fn new_rejects_empty() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn new_rejects_leading_whitespace() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, " foo", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("whitespace"), "got: {err}");
    }

    #[test]
    fn new_rejects_trailing_whitespace() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "foo ", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("whitespace"), "got: {err}");
    }

    #[test]
    fn new_rejects_dot() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, ".", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a filename"), "got: {err}");
    }

    #[test]
    fn new_rejects_dotdot() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "..", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a filename"), "got: {err}");
    }

    #[test]
    fn new_rejects_network_uri() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "gs://bucket/x", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("absolute URIs are not allowed"), "got: {err}");
    }

    #[test]
    fn new_rejects_file_uri() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "file:///abs/path", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("absolute URIs are not allowed"), "got: {err}");
    }

    #[test]
    fn new_rejects_leading_slash() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "/foo", &resolver)
            .unwrap_err()
            .to_string();
        assert!(err.contains("leading '/'"), "got: {err}");
    }

    #[test]
    fn new_rejects_leading_tilde() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "~/foo", &resolver)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("home expansion") || err.contains("'~'"),
            "got: {err}"
        );
    }

    #[test]
    fn new_rejects_traversal_after_normalize() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "foo/../../escape", &resolver)
            .unwrap_err()
            .to_string();
        // The error comes from ensure_under (SandboxError::OutsideRoot)
        assert!(
            err.contains("outside") || err.contains("sandbox"),
            "got: {err}"
        );
    }

    #[test]
    fn new_rejects_path_resolving_to_root() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        // "foo/.." normalizes to "" / root itself — must be rejected.
        let err = SinkOutput::new(&root, "foo/..", &resolver)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("resolves to the artifact directory itself")
                || err.contains("no filename"),
            "got: {err}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn new_absolute_error_mentions_workerArtifactPath() {
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let err = SinkOutput::new(&root, "gs://bucket/x", &resolver)
            .unwrap_err()
            .to_string();
        // Load-bearing: customers searching logs need this keyword to find the migration.
        assert!(
            err.contains("workerArtifactPath"),
            "absolute-URI error must mention workerArtifactPath for migration; got: {err}"
        );
    }

    #[test]
    fn two_new_calls_compose_nested_path() {
        // Replaces from_path_then_join_is_consistent: the spirit is that composing
        // "a/b" via new() reaches the same path as two separate calls would imply.
        let tmp = tempdir().unwrap();
        let root = root_uri(&file_uri(tmp.path()));
        let resolver = make_resolver();
        let child = SinkOutput::new(&root, "a/b", &resolver).unwrap();
        assert_eq!(child.uri().path().as_path(), tmp.path().join("a").join("b"));
    }
}
