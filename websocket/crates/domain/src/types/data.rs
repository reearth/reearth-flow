use serde::{Deserialize, Serialize};

use super::snapshot::SnapshotInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotData {
    pub info: SnapshotInfo,
    pub state: Vec<u8>,
}
