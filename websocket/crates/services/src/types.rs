use chrono::{DateTime, Utc};
use flow_websocket_infra::types::{project::Project, user::User, workspace::Workspace};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum SessionCommand {
    Start {
        project_id: String,
        user: User,
    },
    End {
        project_id: String,
        user: User,
    },
    Complete {
        project_id: String,
        user: User,
    },
    CheckStatus {
        project_id: String,
    },
    AddTask {
        project_id: String,
    },
    RemoveTask {
        project_id: String,
    },
    ListAllSnapshotsVersions {
        project_id: String,
    },
    MergeUpdates {
        project_id: String,
        data: Vec<u8>,
        updated_by: Option<String>,
    },
    ProcessStateVector {
        project_id: String,
        state_vector: Vec<u8>,
    },
    CreateWorkspace {
        workspace: Workspace,
    },
    DeleteWorkspace {
        workspace_id: String,
    },
    UpdateWorkspace {
        workspace: Workspace,
    },
    ListWorkspaceProjectsIds {
        workspace_id: String,
    },
    CreateProject {
        project: Project,
    },
    DeleteProject {
        project_id: String,
    },
    UpdateProject {
        project: Project,
    },
}

#[derive(Clone, Debug)]
pub struct ManageProjectEditSessionTaskData {
    pub project_id: String,
    pub last_merged_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    pub last_snapshot_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    pub clients_disconnected_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    pub client_count: Arc<RwLock<Option<usize>>>,
}

impl ManageProjectEditSessionTaskData {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            last_merged_at: Arc::new(RwLock::new(None)),
            last_snapshot_at: Arc::new(RwLock::new(None)),
            clients_disconnected_at: Arc::new(RwLock::new(None)),
            client_count: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_last_merged_at(&self, time: Option<DateTime<Utc>>) {
        let mut last_merged = self.last_merged_at.write().await;
        *last_merged = time;
    }

    pub async fn set_last_snapshot_at(&self, time: Option<DateTime<Utc>>) {
        let mut last_snapshot = self.last_snapshot_at.write().await;
        *last_snapshot = time;
    }

    pub async fn set_clients_disconnected_at(&self, time: Option<DateTime<Utc>>) {
        let mut disconnected_at = self.clients_disconnected_at.write().await;
        *disconnected_at = time;
    }

    pub async fn set_client_count(&self, count: Option<usize>) {
        let mut client_count = self.client_count.write().await;
        *client_count = count;
    }

    pub async fn get_last_merged_at(&self) -> Option<DateTime<Utc>> {
        *self.last_merged_at.read().await
    }

    pub async fn get_last_snapshot_at(&self) -> Option<DateTime<Utc>> {
        *self.last_snapshot_at.read().await
    }

    pub async fn get_clients_disconnected_at(&self) -> Option<DateTime<Utc>> {
        *self.clients_disconnected_at.read().await
    }

    pub async fn get_client_count(&self) -> Option<usize> {
        *self.client_count.read().await
    }

    pub async fn update_last_merged_at(&self) {
        self.set_last_merged_at(Some(Utc::now())).await;
    }

    pub async fn update_last_snapshot_at(&self) {
        self.set_last_snapshot_at(Some(Utc::now())).await;
    }

    pub async fn update_clients_disconnected_at(&self) {
        self.set_clients_disconnected_at(Some(Utc::now())).await;
    }

    pub async fn clear_last_merged_at(&self) {
        self.set_last_merged_at(None).await;
    }

    pub async fn clear_last_snapshot_at(&self) {
        self.set_last_snapshot_at(None).await;
    }

    pub async fn clear_clients_disconnected_at(&self) {
        self.set_clients_disconnected_at(None).await;
    }

    pub async fn clear_client_count(&self) {
        self.set_client_count(None).await;
    }

    pub async fn increment_client_count(&self) {
        let mut count = self.client_count.write().await;
        *count = Some(count.unwrap_or(0) + 1);
    }

    pub async fn decrement_client_count(&self) {
        let mut count = self.client_count.write().await;
        if let Some(current_count) = *count {
            if current_count > 0 {
                *count = Some(current_count - 1);
            }
        }
    }
}
