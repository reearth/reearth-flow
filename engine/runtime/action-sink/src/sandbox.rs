use std::fmt;

use reearth_flow_common::uri::Uri;

/// Error returned when a candidate URI is rejected for sandbox violation.
#[derive(Debug, Clone)]
pub enum SandboxError {
    /// The candidate URI resolves outside the configured sandbox root.
    OutsideRoot { resolved: Uri, root: Uri },
    /// The candidate cannot be normalized (e.g. malformed URI).
    InvalidPath {
        candidate: String,
        reason: &'static str,
    },
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxError::OutsideRoot { resolved, root } => {
                write!(
                    f,
                    "sink output {} is outside the sandbox root {}",
                    resolved, root
                )
            }
            SandboxError::InvalidPath { candidate, reason } => {
                write!(f, "sink output path {:?} rejected: {}", candidate, reason)
            }
        }
    }
}

impl std::error::Error for SandboxError {}

/// Verify that `candidate` resolves under `root`. Same scheme, same authority,
/// segment-aligned path prefix. `..` segments are normalized before the check.
pub fn ensure_under(root: &Uri, candidate: &Uri) -> Result<(), SandboxError> {
    if root.as_str() == candidate.as_str() {
        return Ok(());
    }
    let root_prefix = root.as_str().trim_end_matches('/');
    let candidate_str = candidate.as_str();
    let after_prefix =
        candidate_str
            .strip_prefix(root_prefix)
            .ok_or_else(|| SandboxError::OutsideRoot {
                resolved: candidate.clone(),
                root: root.clone(),
            })?;
    // Segment-aligned: next char must be '/' (so root /abc doesn't match /abcdef).
    if !after_prefix.is_empty() && !after_prefix.starts_with('/') {
        return Err(SandboxError::OutsideRoot {
            resolved: candidate.clone(),
            root: root.clone(),
        });
    }
    // Reject any '..' segment (URI-level normalisation: walk segments).
    for segment in after_prefix.split('/') {
        if segment == ".." {
            return Err(SandboxError::OutsideRoot {
                resolved: candidate.clone(),
                root: root.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn uri(s: &str) -> Uri {
        Uri::from_str(s).expect("test URI parses")
    }

    #[test]
    fn identical_uri_accepted() {
        assert!(ensure_under(&uri("file:///tmp/job"), &uri("file:///tmp/job")).is_ok());
    }

    #[test]
    fn subpath_of_root_accepted() {
        assert!(ensure_under(
            &uri("file:///tmp/job"),
            &uri("file:///tmp/job/group/a.geojson")
        )
        .is_ok());
    }

    #[test]
    fn same_scheme_different_authority_rejected() {
        let err =
            ensure_under(&uri("gs://mine/job"), &uri("gs://other/job/x.geojson")).unwrap_err();
        match err {
            SandboxError::OutsideRoot { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn different_scheme_rejected() {
        let err = ensure_under(&uri("file:///tmp/job"), &uri("gs://bucket/job/x")).unwrap_err();
        assert!(matches!(err, SandboxError::OutsideRoot { .. }));
    }

    #[test]
    fn dotdot_traversal_rejected() {
        // file:///tmp/job/../etc/passwd resolves outside the root after .. normalization
        let err = ensure_under(
            &uri("file:///tmp/job"),
            &uri("file:///tmp/job/../etc/passwd"),
        )
        .unwrap_err();
        assert!(matches!(err, SandboxError::OutsideRoot { .. }));
    }

    #[test]
    fn prefix_but_not_segment_aligned_rejected() {
        // root="/tmp/ab" must NOT accept "/tmp/abcdef/x"
        let err = ensure_under(&uri("file:///tmp/ab"), &uri("file:///tmp/abcdef/x")).unwrap_err();
        assert!(matches!(err, SandboxError::OutsideRoot { .. }));
    }

    #[test]
    fn trailing_slash_root_normalised() {
        // root with trailing slash should accept the same paths as root without
        assert!(ensure_under(&uri("file:///tmp/job/"), &uri("file:///tmp/job/x.geojson")).is_ok());
    }

    #[test]
    fn permissive_file_root_accepts_any_file_uri() {
        // The Default NodeContext uses "file:///" as the permissive root.
        // Any file:// URI must pass.
        assert!(ensure_under(&uri("file:///"), &uri("file:///tmp/foo.bin")).is_ok());
    }
}
