use std::sync::Arc;

use crate::repository::ProjectSnapshotRepository;
use crate::types::data::SnapshotData;
use crate::types::snapshot::{Metadata, ObjectDelete, ObjectTenant, ProjectSnapshot, SnapshotInfo};
use crate::utils::generate_id;
use chrono::Utc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEditingSession {
    pub project_id: String,
    pub session_id: Option<String>,
    pub session_setup_complete: bool,
    pub tenant: ObjectTenant,
}

#[derive(Error, Debug)]
pub enum ProjectEditingSessionError<E> {
    #[error("Session not setup")]
    SessionNotSetup,
    #[error(transparent)]
    SnapshotRepository(#[from] E),
}

impl ProjectEditingSession {
    pub fn new(project_id: String, tenant: ObjectTenant) -> Self {
        Self {
            project_id,
            session_id: None,
            tenant,
            session_setup_complete: false,
        }
    }

    pub async fn start_or_join_session<R>(
        &mut self,
        snapshot_repo: &R,
    ) -> Result<String, ProjectEditingSessionError<R::Error>>
    where
        R: ProjectSnapshotRepository + ?Sized,
    {
        // Logic to start or join a session
        let session_id = generate_id(14, "editor-session");
        self.session_id = Some(session_id.clone());
        if !self.session_setup_complete {
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
        _state_vector: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<u8>), ProjectEditingSessionError<()>> {
        self.check_session_setup()?;
        // Logic to get the diff update
        Ok((vec![], vec![]))
    }

    pub async fn merge_updates(&self) -> Result<(), ProjectEditingSessionError<()>> {
        self.check_session_setup()?;
        // Logic to merge updates
        Ok(())
    }

    pub async fn get_state_update(&self) -> Result<Vec<u8>, ProjectEditingSessionError<()>> {
        self.check_session_setup()?;
        // Logic to get the state update
        Ok(vec![])
    }

    pub async fn push_update(
        &self,
        _update: Vec<u8>,
        _updated_by: Option<String>,
    ) -> Result<(), ProjectEditingSessionError<()>> {
        self.check_session_setup()?;
        // Logic to push an update
        Ok(())
    }

    pub async fn create_snapshot<R: ProjectSnapshotRepository>(
        &self,
        snapshot_repo: &R,
        data: SnapshotData,
        skip_lock: bool,
    ) -> Result<(), ProjectEditingSessionError<R::Error>> {
        self.check_session_setup()?;
        if skip_lock {
            self.create_snapshot_internal(snapshot_repo, data).await
        } else {
            // Logic to lock the session before creating a snapshot
            self.create_snapshot_internal(snapshot_repo, data).await
        }
    }

    async fn create_snapshot_internal<R: ProjectSnapshotRepository>(
        &self,
        snapshot_repo: &R,
        data: SnapshotData,
    ) -> Result<(), ProjectEditingSessionError<R::Error>> {
        self.merge_updates()
            .await
            .map_err(|_| ProjectEditingSessionError::SessionNotSetup)?;

        let now = Utc::now();

        let metadata = Metadata::new(
            generate_id(14, "snap"),
            self.project_id.clone(),
            self.session_id.clone(),
            data.name.unwrap_or_default(),
            String::new(), // path
        );

        let state = SnapshotInfo::new(
            data.created_by,
            vec![],
            self.tenant.clone(), // use tenant from project
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(now), // created_at
            None,      // updated_at
        );

        let snapshot = ProjectSnapshot::new(metadata, state);

        snapshot_repo.create_snapshot(snapshot).await?;
        Ok(())
    }

    pub async fn end_session(&self) -> Result<(), ProjectEditingSessionError<()>> {
        self.check_session_setup()?;
        // Logic to end the session
        Ok(())
    }

    #[inline]
    fn check_session_setup<E>(&self) -> Result<(), ProjectEditingSessionError<E>> {
        if !self.session_setup_complete {
            Err(ProjectEditingSessionError::SessionNotSetup)
        } else {
            Ok(())
        }
    }

    pub async fn load_session<R: ProjectSnapshotRepository>(
        &mut self,
        snapshot_repo: &R,
        session_id: &str,
    ) -> Result<(), ProjectEditingSessionError<R::Error>> {
        let snapshot = snapshot_repo.get_latest_snapshot(session_id).await?;
        if let Some(snapshot) = snapshot {
            self.project_id = snapshot.metadata.project_id;
        }
        self.session_id = Some(session_id.to_string());
        //self.session_setup_complete = true;
        Ok(())
    }

    pub async fn active_editing_session(
        &self,
    ) -> Result<Option<String>, ProjectEditingSessionError<()>> {
        self.check_session_setup()?;
        if self.session_setup_complete {
            Ok(self.session_id.clone())
        } else {
            Ok(None)
        }
    }
}
