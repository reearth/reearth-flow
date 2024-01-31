use serde::{Deserialize, Serialize};
use std::time::Duration;

use reearth_flow_workflow::workflow::Property;
use reearth_flow_workflow::{graph::Node, id::Id};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Pending,
    Queued,
    InProgress,
    Succeeded,
    Failed,
    Cancelled,
    Skipped,
}

impl JobState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Skipped
        )
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self, Self::InProgress)
    }

    pub fn is_queued(&self) -> bool {
        matches!(self, Self::Queued)
    }
}

pub struct Job {
    pub id: Id,
    pub node: Node,
    pub workflow_parameters: Property,
    pub timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_terminal() {
        assert!(!JobState::Pending.is_terminal());
        assert!(!JobState::Queued.is_terminal());
        assert!(!JobState::InProgress.is_terminal());
        assert!(JobState::Succeeded.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Cancelled.is_terminal());
        assert!(JobState::Skipped.is_terminal());
    }

    #[test]
    fn test_is_in_progress() {
        assert!(!JobState::Pending.is_in_progress());
        assert!(!JobState::Queued.is_in_progress());
        assert!(JobState::InProgress.is_in_progress());
        assert!(!JobState::Succeeded.is_in_progress());
        assert!(!JobState::Failed.is_in_progress());
        assert!(!JobState::Cancelled.is_in_progress());
        assert!(!JobState::Skipped.is_in_progress());
    }

    #[test]
    fn test_is_queued() {
        assert!(!JobState::Pending.is_queued());
        assert!(JobState::Queued.is_queued());
        assert!(!JobState::InProgress.is_queued());
        assert!(!JobState::Succeeded.is_queued());
        assert!(!JobState::Failed.is_queued());
        assert!(!JobState::Cancelled.is_queued());
        assert!(!JobState::Skipped.is_queued());
    }
}
