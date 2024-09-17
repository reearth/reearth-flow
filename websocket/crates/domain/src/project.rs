use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::repository::ProjectSnapshotRepository;
use crate::snapshot::{ObjectDelete, ObjectTenant, ProjectMetadata, ProjectSnapshot};
use crate::utils::generate_id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAllowedActions {
    pub id: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    pub action: String,
    pub allowed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEditingSession {
    pub project_id: String,
    pub session_id: Option<String>,
    pub session_setup_complete: bool,
    pub redis_client: String, // Redis connection string or identifier
}

impl ProjectEditingSession {
    pub fn new(project_id: String, redis_client: String) -> Self {
        Self {
            project_id,
            session_id: None,
            session_setup_complete: false,
            redis_client,
        }
    }

    pub async fn start_or_join_session(
        &mut self,
        snapshot_repo: &impl ProjectSnapshotRepository,
    ) -> Result<String, Box<dyn Error>> {
        // Logic to start or join a session
        let session_id = generate_id(14, "editor-session");
        self.session_id = Some(session_id.clone());
        if (!self.session_setup_complete) {
            let _latest_snapshot_state = snapshot_repo
                .get_latest_snapshot_state(&self.project_id)
                .await?;
            // Initialize Redis with latest snapshot state
        }
        self.session_setup_complete = true;
        Ok(session_id)
    }

    pub async fn get_diff_update(
        &self,
        state_vector: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
        self.check_session_setup()?;
        // Logic to get the diff update
        Ok((vec![], vec![]))
    }

    pub async fn merge_updates(&self) -> Result<(), Box<dyn Error>> {
        self.check_session_setup()?;
        // Logic to merge updates
        Ok(())
    }

    pub async fn get_state_update(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.check_session_setup()?;
        // Logic to get the state update
        Ok(vec![])
    }

    pub async fn push_update(
        &self,
        update: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        self.check_session_setup()?;
        // Logic to push an update
        Ok(())
    }

    pub async fn create_snapshot(
        &self,
        snapshot_repo: &impl ProjectSnapshotRepository,
        data: SnapshotData,
        skip_lock: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.check_session_setup()?;
        if skip_lock {
            self.create_snapshot_internal(snapshot_repo, data).await
        } else {
            // Logic to lock the session before creating a snapshot
            self.create_snapshot_internal(snapshot_repo, data).await
        }
    }

    async fn create_snapshot_internal(
        &self,
        snapshot_repo: &impl ProjectSnapshotRepository,
        data: SnapshotData,
    ) -> Result<(), Box<dyn Error>> {
        self.merge_updates().await?;

        let metadata = ProjectMetadata {
            id: generate_id(14, "snap"),
            project_id: self.project_id.clone(),
            session_id: self.session_id.clone(),
            name: data.name.unwrap_or_default(),
            path: String::new(),
        };

        let snapshot = ProjectSnapshot {
            metadata,
            created_by: data.created_by.clone(),
            changes_by: vec![], // populate changes_by appropriately
            tenant: ObjectTenant {
                id: "tenant_id_example".to_string(),
                key: "tenant_key_example".to_string(),
            },
            delete: ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            created_at: None,
            updated_at: None,
        };

        snapshot_repo.create_snapshot(snapshot).await?;
        Ok(())
    }

    pub async fn end_session(&self) -> Result<(), Box<dyn Error>> {
        self.check_session_setup()?;
        // Logic to end the session
        Ok(())
    }

    fn check_session_setup(&self) -> Result<(), Box<dyn Error>> {
        if !self.session_setup_complete {
            Err("Session not setup".into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotData {
    pub name: Option<String>,
    pub created_by: Option<String>,
}
