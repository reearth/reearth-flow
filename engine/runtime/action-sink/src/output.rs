use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::executor_operation::NodeContext;
use reearth_flow_storage::storage::Storage;

/// Owns URI parsing, storage backend acquisition, and bytes write for sink
/// output handling. Expression evaluation is the responsibility of the caller.
#[derive(Clone, Debug)]
pub struct SinkOutput {
    resolved: Uri,
    root: Uri,
    storage: Arc<Storage>,
}

/// Validate `path` as a strict-relative sink output and resolve it against
/// `ctx.sandbox_root`. Returns the joined and sandbox-validated URI without
/// acquiring a storage backend.
///
/// This is the validation half of [`SinkOutput::from_path`]. Use it when you
/// only need the resolved URI (e.g. as a per-feature buffer key) and will
/// acquire the storage handle later via [`SinkOutput::from_resolved_uri`].
///
/// Rejection rules match [`SinkOutput::from_path`]: empty / leading-trailing
/// whitespace / `.` / `..` / `scheme://...` / leading `/` / leading `~` /
/// post-join paths that escape via `..` / post-join paths that resolve to
/// the sandbox root itself.
pub fn ensure_relative_path(ctx: &NodeContext, path: &str) -> Result<Uri, BoxedError> {
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
    let resolved = ctx.sandbox_root.join(path).map_err(|e| -> BoxedError {
        format!("SinkOutput: failed to join {path:?} with sandbox_root: {e}").into()
    })?;

    // ---- 3. Sandbox-validate (catches ".." segments that survived join) ----
    crate::sandbox::ensure_under(&ctx.sandbox_root, &resolved)
        .map_err(|e| -> BoxedError { Box::new(e) })?;

    // ---- 4. Post-join: resolved must not be the root itself ----
    // Compare ignoring trailing slashes — Uri::join may strip them via normalization.
    let resolved_norm = resolved.as_str().trim_end_matches('/');
    let root_norm = ctx.sandbox_root.as_str().trim_end_matches('/');
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

impl SinkOutput {
    /// Construct a `SinkOutput` from a relative path string.
    ///
    /// Sink output paths are **strict-relative**: they must not contain a URI
    /// scheme, must not start with `/` or `~`, must not be empty, must not be
    /// `.` or `..`, and must not have surrounding whitespace. The engine joins
    /// the relative path against `ctx.sandbox_root` and runs the sandbox check
    /// (`ensure_under`) before acquiring the storage backend.
    ///
    /// For migration from the previous absolute-URI API, see the error message
    /// of the URI-scheme rejection path, which names `workerArtifactPath`.
    pub fn from_path(ctx: &NodeContext, path: &str) -> Result<Self, BoxedError> {
        let resolved = ensure_relative_path(ctx, path)?;
        Self::from_resolved_uri(ctx, resolved)
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

    /// Construct a `SinkOutput` from an already-validated, sandbox-bounded URI.
    ///
    /// Intentionally `pub(crate)`: this is the storage-acquisition half of the
    /// chokepoint, exposed only to other modules inside `action-sink` that
    /// already produced `resolved` via [`ensure_relative_path`] (e.g. the
    /// cesium3dtiles and mvt sinks that buffer features keyed by URI and need
    /// to skip a second validation at write time).
    ///
    /// External callers (outside `action-sink`) must go through
    /// [`SinkOutput::from_path`], which always validates. This preserves the
    /// chokepoint property: there is no way to acquire a `SinkOutput` (and
    /// therefore `SinkOutput::write`) from outside this crate without passing
    /// the sandbox gate.
    pub(crate) fn from_resolved_uri(ctx: &NodeContext, resolved: Uri) -> Result<Self, BoxedError> {
        let storage = ctx
            .storage_resolver
            .resolve(&resolved)
            .map_err(|e| -> BoxedError {
                format!("SinkOutput: failed to resolve storage for {resolved}: {e}").into()
            })?;
        Ok(Self {
            resolved,
            root: ctx.sandbox_root.clone(),
            storage,
        })
    }

    /// Derive a child `SinkOutput` whose URI is `self.uri()` joined with `sub`.
    ///
    /// `sub` must be a relative path. Absolute sub-paths will return an error
    /// from `Uri::join`. The joined URI is checked against the captured
    /// sandbox root so traversal like `"../escape"` is hard-rejected.
    pub fn join(&self, sub: &str) -> Result<Self, BoxedError> {
        let joined = self.resolved.join(sub)?;
        crate::sandbox::ensure_under(&self.root, &joined)
            .map_err(|e| -> BoxedError { Box::new(e) })?;
        Ok(Self {
            resolved: joined,
            root: self.root.clone(),
            storage: self.storage.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use reearth_flow_runtime::executor_operation::NodeContext;
    use tempfile::tempdir;

    fn file_uri(path: &std::path::Path) -> String {
        format!("file://{}", path.display())
    }

    #[test]
    fn write_persists_bytes_to_resolved_uri() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext {
            sandbox_root: Uri::from_str(&file_uri(tmp.path())).unwrap(),
            ..NodeContext::default()
        };
        let out = SinkOutput::from_path(&ctx, "write_target.bin").unwrap();
        out.write(Bytes::from_static(b"hello")).unwrap();
        let content = std::fs::read(tmp.path().join("write_target.bin")).unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn join_composes_subpath_under_base() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext {
            sandbox_root: Uri::from_str(&file_uri(tmp.path())).unwrap(),
            ..NodeContext::default()
        };
        let base = SinkOutput::from_path(&ctx, "base_dir").unwrap();
        let sub = base.join("group/a.geojson").unwrap();
        assert_eq!(
            sub.uri().path().as_path(),
            tmp.path().join("base_dir").join("group/a.geojson")
        );
    }

    #[test]
    fn join_rejects_absolute_subpath() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext {
            sandbox_root: Uri::from_str(&file_uri(tmp.path())).unwrap(),
            ..NodeContext::default()
        };
        let base = SinkOutput::from_path(&ctx, "some_dir").unwrap();
        // Absolute sub paths are not allowed by `Uri::join` and must error.
        let result = base.join("/etc/passwd");
        assert!(
            result.is_err(),
            "join should reject absolute subpath, got: {result:?}"
        );
    }

    #[test]
    fn join_with_dotdot_now_rejected_by_sandbox() {
        let tmp = tempdir().unwrap();
        let nested = tmp.path().join("subdir");
        std::fs::create_dir(&nested).unwrap();
        // Critical: set sandbox_root to the nested dir so `../..` actually escapes.
        // base = subdir/out (one level deep), then join("../../escape") goes two
        // levels up, landing outside sandbox_root=subdir.
        let ctx = NodeContext {
            sandbox_root: Uri::from_str(&file_uri(&nested)).unwrap(),
            ..NodeContext::default()
        };
        let base = SinkOutput::from_path(&ctx, "out").unwrap();
        let result = base.join("../../escape.txt");
        assert!(
            result.is_err(),
            "PR2: `../..` traversal must be rejected; got: {result:?}"
        );
    }

    #[test]
    fn clone_shares_storage_backend() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext {
            sandbox_root: Uri::from_str(&file_uri(tmp.path())).unwrap(),
            ..NodeContext::default()
        };
        let original = SinkOutput::from_path(&ctx, "some_output.bin").unwrap();
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

    fn ctx_with_root(root: &str) -> NodeContext {
        NodeContext {
            sandbox_root: Uri::from_str(root).unwrap(),
            ..NodeContext::default()
        }
    }

    #[test]
    fn from_path_accepts_simple_relative() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let out = SinkOutput::from_path(&ctx, "out.gpkg").unwrap();
        assert_eq!(out.uri().path().as_path(), tmp.path().join("out.gpkg"));
    }

    #[test]
    fn from_path_accepts_subdir_relative() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let out = SinkOutput::from_path(&ctx, "group/a.geojson").unwrap();
        assert_eq!(
            out.uri().path().as_path(),
            tmp.path().join("group").join("a.geojson")
        );
    }

    #[test]
    fn from_path_accepts_relative_with_gs_root() {
        let ctx = ctx_with_root("gs://my-bucket/jobs/abc/");
        // Validation and join must succeed even if the default StorageResolver
        // does not have a gs:// backend registered.
        let result = SinkOutput::from_path(&ctx, "out.json");
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
    fn from_path_accepts_relative_with_ram_root() {
        let ctx = ctx_with_root("ram:///jobs/abc/");
        // Validation and join must succeed even if the default StorageResolver
        // does not have a ram:// backend registered.
        let result = SinkOutput::from_path(&ctx, "out.json");
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
    fn from_path_rejects_empty() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "").unwrap_err().to_string();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_leading_whitespace() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, " foo").unwrap_err().to_string();
        assert!(err.contains("whitespace"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_trailing_whitespace() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "foo ").unwrap_err().to_string();
        assert!(err.contains("whitespace"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_dot() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, ".").unwrap_err().to_string();
        assert!(err.contains("not a filename"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_dotdot() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "..").unwrap_err().to_string();
        assert!(err.contains("not a filename"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_network_uri() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "gs://bucket/x")
            .unwrap_err()
            .to_string();
        assert!(err.contains("absolute URIs are not allowed"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_file_uri() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "file:///abs/path")
            .unwrap_err()
            .to_string();
        assert!(err.contains("absolute URIs are not allowed"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_leading_slash() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "/foo").unwrap_err().to_string();
        assert!(err.contains("leading '/'"), "got: {err}");
    }

    #[test]
    fn from_path_rejects_leading_tilde() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "~/foo")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("home expansion") || err.contains("'~'"),
            "got: {err}"
        );
    }

    #[test]
    fn from_path_rejects_traversal_after_normalize() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "foo/../../escape")
            .unwrap_err()
            .to_string();
        // The error comes from ensure_under (SandboxError::OutsideRoot)
        assert!(
            err.contains("outside") || err.contains("sandbox"),
            "got: {err}"
        );
    }

    #[test]
    fn from_path_rejects_path_resolving_to_root() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        // "foo/.." normalizes to "" / root itself — must be rejected.
        let err = SinkOutput::from_path(&ctx, "foo/..")
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
    fn from_path_absolute_error_mentions_workerArtifactPath() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = SinkOutput::from_path(&ctx, "gs://bucket/x")
            .unwrap_err()
            .to_string();
        // Load-bearing: customers searching logs need this keyword to find the migration.
        assert!(
            err.contains("workerArtifactPath"),
            "absolute-URI error must mention workerArtifactPath for migration; got: {err}"
        );
    }

    #[test]
    fn from_path_then_join_is_consistent() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let parent = SinkOutput::from_path(&ctx, "a").unwrap();
        let child = parent.join("b").unwrap();
        assert_eq!(child.uri().path().as_path(), tmp.path().join("a").join("b"));
    }

    // ---- ensure_relative_path tests ----

    #[test]
    fn ensure_relative_path_accepts_simple_relative() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let uri = ensure_relative_path(&ctx, "out.gpkg").unwrap();
        assert_eq!(uri.path().as_path(), tmp.path().join("out.gpkg"));
    }

    #[test]
    fn ensure_relative_path_rejects_absolute_uri() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = ensure_relative_path(&ctx, "gs://bucket/x")
            .unwrap_err()
            .to_string();
        assert!(err.contains("workerArtifactPath"), "got: {err}");
    }

    #[test]
    fn ensure_relative_path_rejects_traversal() {
        let tmp = tempdir().unwrap();
        let ctx = ctx_with_root(&file_uri(tmp.path()));
        let err = ensure_relative_path(&ctx, "foo/../../escape")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("outside") || err.contains("sandbox"),
            "got: {err}"
        );
    }

    #[test]
    fn ensure_relative_path_does_not_acquire_storage() {
        // The helper should succeed even when StorageResolver doesn't have the
        // backend registered — proving it doesn't call resolver.resolve().
        let ctx = ctx_with_root("gs://my-bucket/jobs/abc/");
        let uri = ensure_relative_path(&ctx, "out.json").unwrap();
        assert_eq!(uri.as_str(), "gs://my-bucket/jobs/abc/out.json");
    }
}
