use super::utils::calculate_diff;
use std::sync::Arc;

use crate::repository::{ProjectSnapshotRepository, RedisDataManager};
use crate::types::data::SnapshotData;
use crate::types::snapshot::{Metadata, ObjectDelete, ObjectTenant, ProjectSnapshot, SnapshotInfo};
use crate::utils::generate_id;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEditingSession {
    pub project_id: String,
    pub session_id: Option<String>,
    pub session_setup_complete: bool,
    pub tenant: ObjectTenant,
    #[serde(skip)]
    session_lock: Arc<Mutex<()>>,
}

#[derive(Error, Debug)]
pub enum ProjectEditingSessionError {
    #[error("Session not setup")]
    SessionNotSetup,
    #[error("Snapshot not found")]
    SnapshotNotFound,
    #[error("Snapshot error: {0}")]
    Snapshot(String),
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("{0}")]
    Custom(String),
}

impl ProjectEditingSessionError {
    pub fn snapshot<E: std::fmt::Display>(err: E) -> Self {
        Self::Snapshot(err.to_string())
    }

    pub fn redis<E: std::fmt::Display>(err: E) -> Self {
        Self::Redis(err.to_string())
    }
}

impl Default for ProjectEditingSession {
    fn default() -> Self {
        Self {
            project_id: "".to_string(),
            session_id: None,
            session_setup_complete: false,
            session_lock: Arc::new(Mutex::new(())),
            tenant: ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
        }
    }
}

impl ProjectEditingSession {
    pub fn new(project_id: String, tenant: ObjectTenant) -> Self {
        Self {
            project_id,
            session_id: None,
            tenant,
            session_setup_complete: false,
            session_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn start_or_join_session<S, R>(
        &mut self,
        snapshot_repo: &S,
        redis_data_manager: &R,
    ) -> Result<String, ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository + ?Sized,
        R: RedisDataManager,
    {
        let session_id = generate_id(14, "editor-session");
        self.session_id = Some(session_id.clone());
        if !self.session_setup_complete {
            let latest_snapshot_state = snapshot_repo
                .get_latest_snapshot_state(&self.project_id)
                .await
                .map_err(ProjectEditingSessionError::snapshot)?;
            redis_data_manager
                .push_update(latest_snapshot_state, None)
                .await
                .map_err(ProjectEditingSessionError::redis)?;
        }
        self.session_setup_complete = true;
        Ok(session_id)
    }

    pub async fn get_diff_update<R>(
        &self,
        state_vector: Vec<u8>,
        redis_data_manager: &R,
    ) -> Result<(Vec<u8>, Vec<u8>), ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let current_state = redis_data_manager
            .get_current_state()
            .await
            .map_err(ProjectEditingSessionError::redis)?;
        if let Some(current_state) = current_state {
            if current_state == state_vector {
                return Ok((vec![], current_state));
            }
            let (diff, server_state) = calculate_diff(&state_vector, &current_state);
            Ok((diff, server_state))
        } else {
            Ok((state_vector.clone(), vec![]))
        }
    }

    pub async fn merge_updates<R>(
        &self,
        redis_data_manager: &R,
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let result = if skip_lock {
            redis_data_manager
                .merge_updates(false)
                .await
                .map_err(ProjectEditingSessionError::redis)?
        } else {
            let _lock = self.session_lock.lock().await;
            redis_data_manager
                .merge_updates(false)
                .await
                .map_err(ProjectEditingSessionError::redis)?
        };

        Ok(result)
    }

    pub async fn get_state_update<R>(
        &self,
        redis_data_manager: &R,
    ) -> Result<Vec<u8>, ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }
        let current_state = redis_data_manager
            .get_current_state()
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        match current_state {
            Some(state) => Ok(state),
            None => Ok(Vec::new()),
        }
    }

    pub async fn push_update<R>(
        &self,
        update: Vec<u8>,
        updated_by: String,
        redis_data_manager: &R,
        skip_lock: bool,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        if skip_lock {
            redis_data_manager
                .push_update(update, Some(updated_by))
                .await
                .map_err(ProjectEditingSessionError::redis)?;
        } else {
            let _lock = self.session_lock.lock().await;
            redis_data_manager
                .push_update(update, Some(updated_by))
                .await
                .map_err(ProjectEditingSessionError::redis)?;
        }

        Ok(())
    }

    pub async fn create_snapshot<R, S>(
        &self,
        snapshot_repo: &S,
        redis_data_manager: &R,
        data: SnapshotData,
        skip_lock: bool,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository,
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }
        if skip_lock {
            self.create_snapshot_internal(snapshot_repo, redis_data_manager, data)
                .await
        } else {
            let _lock = self.session_lock.lock().await;
            self.create_snapshot_internal(snapshot_repo, redis_data_manager, data)
                .await
        }
    }

    async fn create_snapshot_internal<R, S>(
        &self,
        snapshot_repo: &S,
        redis_data_manager: &R,
        data: SnapshotData,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository,
        R: RedisDataManager,
    {
        self.merge_updates(redis_data_manager, false).await?;

        let now = Utc::now();

        let metadata = Metadata::new(
            generate_id(14, "snap"),
            self.project_id.clone(),
            self.session_id.clone(),
            data.name.unwrap_or_default(),
            String::new(),
        );

        let snapshot_info = SnapshotInfo::new(
            data.created_by,
            vec![],
            self.tenant.clone(),
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(now),
            None,
        );

        let snapshot = ProjectSnapshot::new(metadata, snapshot_info);

        snapshot_repo
            .create_snapshot(snapshot)
            .await
            .map_err(ProjectEditingSessionError::snapshot)?;
        Ok(())
    }

    pub async fn end_session<R, S>(
        &mut self,
        redis_data_manager: &R,
        snapshot_repo: &S,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManager,
        S: ProjectSnapshotRepository,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let _lock = self.session_lock.lock().await;

        let (state, _) = redis_data_manager
            .merge_updates(true)
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        let snapshot_data = SnapshotData::new(self.project_id.clone(), state, None, None);

        snapshot_repo
            .update_latest_snapshot_data(&snapshot_data.project_id, snapshot_data.clone())
            .await
            .map_err(ProjectEditingSessionError::snapshot)?;

        redis_data_manager
            .clear_data()
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        self.session_id = None;
        self.session_setup_complete = false;

        Ok(())
    }

    pub async fn load_session<S>(
        &mut self,
        snapshot_repo: &S,
        session_id: &str,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository,
    {
        let snapshot = snapshot_repo
            .get_latest_snapshot(session_id)
            .await
            .map_err(ProjectEditingSessionError::snapshot)?;
        if let Some(snapshot) = snapshot {
            self.project_id = snapshot.metadata.project_id;
        } else {
            return Err(ProjectEditingSessionError::SnapshotNotFound);
        }
        self.session_id = Some(session_id.to_string());
        self.session_setup_complete = true;
        Ok(())
    }

    pub async fn active_editing_session(
        &self,
    ) -> Result<Option<String>, ProjectEditingSessionError> {
        if !self.session_setup_complete {
            Err(ProjectEditingSessionError::SessionNotSetup)
        } else {
            Ok(self.session_id.clone())
        }
    }
}
