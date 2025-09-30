use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone)]
pub struct CreateSnapshotCommand {
    pub doc_id: String,
    pub version: u64,
    pub name: Option<String>,
}

impl CreateSnapshotCommand {
    pub fn new(doc_id: impl Into<String>, version: u64, name: Option<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
            version,
            name,
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        ensure_not_empty(&self.doc_id, "doc_id")?;
        Ok(())
    }
}

pub fn build_snapshot_name(command: &CreateSnapshotCommand) -> String {
    command
        .name
        .clone()
        .unwrap_or_else(|| format!("snapshot-{}", command.version))
}
