use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::executor_operation::NodeContext;
use reearth_flow_storage::storage::Storage;
use reearth_flow_types::Expr;

/// Owns expression evaluation, URI parsing, storage backend acquisition,
/// and bytes write for sink output handling.
#[derive(Clone)]
pub struct SinkOutput {
    resolved: Uri,
    storage: Arc<Storage>,
}

impl SinkOutput {
    /// Construct a `SinkOutput` from a fully-formed path string.
    ///
    /// The string must parse as a valid URI (`file://...`, `gs://...`, etc.).
    /// Resolves the storage backend eagerly. Used by sinks that pre-compile
    /// Rhai expression ASTs and evaluate them at write time.
    pub fn from_path(ctx: &NodeContext, path: &str) -> Result<Self, BoxedError> {
        let resolved = Uri::from_str(path).map_err(|e| -> BoxedError {
            format!("SinkOutput: invalid path {:?}: {e}", path).into()
        })?;
        let storage = ctx.storage_resolver.resolve(&resolved)?;
        Ok(Self { resolved, storage })
    }

    /// Construct a `SinkOutput` by evaluating a Rhai expression to a path string.
    ///
    /// If evaluation fails, the raw expression text is used as the path —
    /// this matches the historical per-sink behavior so literal paths work
    /// without writing them as `"\"file://...\""`.
    pub fn from_expr(ctx: &NodeContext, expr: &Expr) -> Result<Self, BoxedError> {
        let scope = ctx.expr_engine.new_scope();
        let path = scope
            .eval::<String>(expr.as_ref())
            .unwrap_or_else(|_| expr.as_ref().to_string());
        Self::from_path(ctx, &path)
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
    fn from_expr_evaluates_then_resolves() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("expr_target.bin");
        let ctx = NodeContext::default();
        let expr = Expr::new(file_uri(&target));
        let out = SinkOutput::from_expr(&ctx, &expr).unwrap();
        out.write(Bytes::from_static(b"x")).unwrap();
        assert!(target.exists());
    }

    #[test]
    fn from_expr_evaluates_rhai_concat_expression() {
        let tmp = tempdir().unwrap();
        // A Rhai expression that concatenates strings — must successfully evaluate
        let dir = tmp.path().display().to_string();
        let expr_src = format!(r#""file://{dir}/" + "concat_out.bin""#);
        let expr = Expr::new(&expr_src);
        let ctx = NodeContext::default();
        let out = SinkOutput::from_expr(&ctx, &expr).unwrap();
        out.write(Bytes::from_static(b"y")).unwrap();
        assert!(tmp.path().join("concat_out.bin").exists());
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
}
