use std::str::FromStr;
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

impl SinkOutput {
    /// Construct a `SinkOutput` from a path string.
    ///
    /// Accepts either a URI (`file://...`, `gs://...`, etc.) or a plain
    /// filesystem path (treated as `file://` by `Uri::from_str`). Verifies
    /// the resolved URI is within `ctx.output_path` (hard-rejects writes
    /// outside the sandbox), then acquires the storage backend eagerly.
    pub fn from_path(ctx: &NodeContext, path: &str) -> Result<Self, BoxedError> {
        let resolved = Uri::from_str(path).map_err(|e| -> BoxedError {
            format!("SinkOutput: invalid path {:?}: {e}", path).into()
        })?;
        crate::sandbox::ensure_under(&ctx.output_path, &resolved)
            .map_err(|e| -> BoxedError { Box::new(e) })?;
        let storage = ctx
            .storage_resolver
            .resolve(&resolved)
            .map_err(|e| -> BoxedError {
                format!("SinkOutput: failed to resolve storage for {resolved}: {e}").into()
            })?;
        Ok(Self {
            resolved,
            root: ctx.output_path.clone(),
            storage,
        })
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
    use super::*;
    use reearth_flow_runtime::executor_operation::NodeContext;
    use tempfile::tempdir;

    fn file_uri(path: &std::path::Path) -> String {
        format!("file://{}", path.display())
    }

    #[test]
    fn from_path_resolves_file_uri() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("out.bin");
        let ctx = NodeContext::default();
        let out = SinkOutput::from_path(&ctx, &file_uri(&target)).unwrap();
        assert_eq!(out.uri().path().as_path(), target.as_path());
    }

    #[test]
    fn write_persists_bytes_to_resolved_uri() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("write_target.bin");
        let ctx = NodeContext::default();
        let out = SinkOutput::from_path(&ctx, &file_uri(&target)).unwrap();
        out.write(Bytes::from_static(b"hello")).unwrap();
        let content = std::fs::read(&target).unwrap();
        assert_eq!(content, b"hello");
    }

    #[test]
    fn join_composes_subpath_under_base() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext::default();
        let base = SinkOutput::from_path(&ctx, &file_uri(tmp.path())).unwrap();
        let sub = base.join("group/a.geojson").unwrap();
        assert_eq!(
            sub.uri().path().as_path(),
            tmp.path().join("group/a.geojson")
        );
        sub.write(Bytes::from_static(b"{}")).unwrap();
        assert!(tmp.path().join("group/a.geojson").exists());
    }

    #[test]
    fn from_path_rejects_invalid_uri() {
        let ctx = NodeContext::default();
        // An empty string and a bare token are not valid URIs in this codebase's `Uri` type.
        // Try empty first; if `Uri::from_str("")` somehow succeeds, also test a clearly malformed input.
        let result = SinkOutput::from_path(&ctx, "");
        assert!(result.is_err(), "empty string should fail to parse");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("SinkOutput: invalid path"),
            "error should include the context wrapper, got: {err_msg}"
        );
    }

    #[test]
    fn join_rejects_absolute_subpath() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext::default();
        let base = SinkOutput::from_path(&ctx, &file_uri(tmp.path())).unwrap();
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
        // Critical: set output_path to the nested dir so `..` actually escapes.
        let ctx = NodeContext {
            output_path: Uri::from_str(&file_uri(&nested)).unwrap(),
            ..NodeContext::default()
        };
        let base = SinkOutput::from_path(&ctx, &file_uri(&nested)).unwrap();
        let result = base.join("../sibling.txt");
        assert!(
            result.is_err(),
            "PR2: `..` traversal must now be rejected; got: {result:?}"
        );
    }

    #[test]
    fn clone_shares_storage_backend() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext::default();
        let original = SinkOutput::from_path(&ctx, &file_uri(tmp.path())).unwrap();
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

    #[test]
    fn from_path_rejects_uri_outside_output_path() {
        let tmp = tempdir().unwrap();
        let inside = tmp.path().join("inside");
        std::fs::create_dir(&inside).unwrap();
        let outside = tmp.path().join("outside");
        std::fs::create_dir(&outside).unwrap();

        let ctx = NodeContext {
            output_path: Uri::from_str(&file_uri(&inside)).unwrap(),
            ..NodeContext::default()
        };

        let target_outside = outside.join("attack.bin");
        let result = SinkOutput::from_path(&ctx, &file_uri(&target_outside));
        assert!(
            result.is_err(),
            "SinkOutput::from_path must reject URIs outside ctx.output_path"
        );
    }

    #[test]
    fn from_path_accepts_uri_inside_output_path() {
        let tmp = tempdir().unwrap();
        let inside = tmp.path().join("inside");
        std::fs::create_dir(&inside).unwrap();

        let ctx = NodeContext {
            output_path: Uri::from_str(&file_uri(&inside)).unwrap(),
            ..NodeContext::default()
        };

        let target = inside.join("ok.bin");
        let result = SinkOutput::from_path(&ctx, &file_uri(&target));
        assert!(
            result.is_ok(),
            "URI inside output_path must succeed; got: {:?}",
            result.err()
        );
    }
}
