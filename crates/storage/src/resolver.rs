use reearth_flow_common::uri::Uri;
use std::io::Result;

use crate::operator::resolve_operator;
use crate::storage::Storage;

/// Resolves the given URI.
pub fn resolve(uri: &Uri) -> Result<Storage> {
    let op = resolve_operator(uri)?;
    Ok(Storage::new(op))
}
