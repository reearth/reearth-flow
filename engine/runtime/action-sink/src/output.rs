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
    storage: Arc<Storage>,
}

impl SinkOutput {
    /// Construct a `SinkOutput` from a path string.
    ///
    /// Accepts either a URI (`file://...`, `gs://...`, etc.) or a plain
    /// filesystem path (treated as `file://` by `Uri::from_str`). Resolves
    /// the storage backend eagerly.
    pub fn from_path(ctx: &NodeContext, path: &str) -> Result<Self, BoxedError> {
        let resolved = Uri::from_str(path).map_err(|e| -> BoxedError {
            format!("SinkOutput: invalid path {:?}: {e}", path).into()
        })?;
        let storage = ctx
            .storage_resolver
            .resolve(&resolved)
            .map_err(|e| -> BoxedError {
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

    /// Derive a child `SinkOutput` whose URI is `self.uri()` joined with `sub`.
    ///
    /// `sub` must be a relative path. Absolute sub-paths will return an error
    /// from `Uri::join`. The storage backend is reused (scheme+authority cannot
    /// change via join).
    pub fn join(&self, sub: &str) -> Result<Self, BoxedError> {
        let joined = self.resolved.join(sub)?;
        // join preserves scheme+authority, so the storage backend is the same
        Ok(Self {
            resolved: joined,
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
    fn join_with_dotdot_pins_current_behavior() {
        let tmp = tempdir().unwrap();
        let nested = tmp.path().join("subdir");
        std::fs::create_dir(&nested).unwrap();
        let ctx = NodeContext::default();
        let base = SinkOutput::from_path(&ctx, &file_uri(&nested)).unwrap();
        // Pin current behavior: `Uri::join` normalizes `..` and the result
        // escapes the base directory. PR2 will replace this with a rejection
        // assertion once sandboxing lands.
        let sub = base
            .join("../sibling.txt")
            .expect("Uri::join currently resolves `..` segments");
        let expected = tmp.path().join("sibling.txt");
        assert_eq!(
            sub.uri().path().as_path(),
            expected.as_path(),
            "traversal currently resolves to parent — PR2 will block this"
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
}
