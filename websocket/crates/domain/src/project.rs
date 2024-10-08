use std::cmp::min;
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
pub enum ProjectEditingSessionError<S, R> {
    #[error("Session not setup")]
    SessionNotSetup,
    #[error(transparent)]
    Snapshot(#[from] S),
    #[error(transparent)]
    Redis(R),
    #[error("{0}")]
    Custom(String),
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

    pub async fn start_or_join_session<R, S>(
        &mut self,
        snapshot_repo: &R,
        redis_data_manager: &S,
    ) -> Result<String, ProjectEditingSessionError<R::Error, S::Error>>
    where
        R: ProjectSnapshotRepository + ?Sized,
        S: RedisDataManager,
    {
        // Logic to start or join a session
        let session_id = generate_id(14, "editor-session");
        self.session_id = Some(session_id.clone());
        if !self.session_setup_complete {
            let latest_snapshot_state = snapshot_repo
                .get_latest_snapshot_state(&self.project_id)
                .await?;
            // Initialize Redis with latest snapshot state
            redis_data_manager
                .push_update(latest_snapshot_state, "system".to_string())
                .await
                .map_err(ProjectEditingSessionError::Redis)?;
        }
        self.session_setup_complete = true;
        Ok(session_id)
    }

    pub async fn get_diff_update<R>(
        &self,
        state_vector: Vec<u8>,
        redis_data_manager: &R,
    ) -> Result<(Vec<u8>, Vec<u8>), ProjectEditingSessionError<R::Error, ()>>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let current_state = redis_data_manager.get_current_state().await?;
        if let Some(current_state) = current_state {
            if current_state == state_vector {
                return Ok((vec![], current_state));
            }
            let (diff, server_state) = self.calculate_diff(&current_state, &state_vector);
            Ok((diff, server_state))
        } else {
            Ok((state_vector.clone(), state_vector))
        }
    }

    #[inline]
    fn calculate_diff(&self, client_state: &[u8], server_state: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut diff = Vec::with_capacity(min(client_state.len(), server_state.len()));
        let mut i = 0;
        let mut j = 0;

        while i < client_state.len() && j < server_state.len() {
            if client_state[i] == server_state[j] {
                let start = i;
                while i < client_state.len()
                    && j < server_state.len()
                    && client_state[i] == server_state[j]
                    && i - start < 255
                {
                    i += 1;
                    j += 1;
                }
                diff.push(3);
                diff.push((i - start) as u8);
            } else {
                diff.push(2);
                diff.push(server_state[j]);
                i += 1;
                j += 1;
            }
        }

        while j < server_state.len() {
            diff.push(0);
            diff.push(server_state[j]);
            j += 1;
        }

        while i < client_state.len() {
            diff.push(1);
            i += 1;
        }

        (diff, server_state.to_vec())
    }

    pub async fn merge_updates<R>(
        &self,
        redis_data_manager: &R,
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), ProjectEditingSessionError<R::Error, ()>>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let result = if skip_lock {
            redis_data_manager.merge_updates(false).await?
        } else {
            let _lock = self.session_lock.lock().await;
            redis_data_manager.merge_updates(false).await?
        };

        Ok(result)
    }

    pub async fn get_state_update<R>(
        &self,
        redis_data_manager: &R,
    ) -> Result<Vec<u8>, ProjectEditingSessionError<R::Error, ()>>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }
        let current_state = redis_data_manager.get_current_state().await?;

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
    ) -> Result<(), ProjectEditingSessionError<R::Error, ()>>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        if skip_lock {
            redis_data_manager.push_update(update, updated_by).await?;
        } else {
            let _lock = self.session_lock.lock().await;
            redis_data_manager.push_update(update, updated_by).await?;
        }

        Ok(())
    }

    pub async fn create_snapshot<S, R>(
        &self,
        snapshot_repo: &R,
        redis_data_manager: &S,
        data: SnapshotData,
        skip_lock: bool,
    ) -> Result<(), ProjectEditingSessionError<R::Error, S::Error>>
    where
        R: ProjectSnapshotRepository,
        S: RedisDataManager,
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
        snapshot_repo: &R,
        redis_data_manager: &S,
        data: SnapshotData,
    ) -> Result<(), ProjectEditingSessionError<R::Error, S::Error>>
    where
        R: ProjectSnapshotRepository,
        S: RedisDataManager,
    {
        self.merge_updates(redis_data_manager, false)
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

    pub async fn end_session<R, S>(
        &mut self,
        redis_data_manager: &R,
        snapshot_repo: &S,
    ) -> Result<(), ProjectEditingSessionError<S::Error, R::Error>>
    where
        R: RedisDataManager,
        S: ProjectSnapshotRepository,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        // Acquire the session lock
        let _lock = self.session_lock.lock().await;

        // Merge any pending updates
        let (state, _) = redis_data_manager
            .merge_updates(true)
            .await
            .map_err(ProjectEditingSessionError::Redis)?;

        let snapshot_data = SnapshotData::new(
            self.project_id.clone(), // project_id
            state,
            None, // name (Optional)
            None, // created_by (Optional)
        );

        snapshot_repo
            .update_snapshot_data(&snapshot_data.project_id, snapshot_data.clone())
            .await?;

        // Clear the session data from Redis
        redis_data_manager
            .clear_data()
            .await
            .map_err(ProjectEditingSessionError::Redis)?;

        // Reset session state
        self.session_id = None;
        self.session_setup_complete = false;

        Ok(())
    }

    pub async fn load_session<R>(
        &mut self,
        snapshot_repo: &R,
        session_id: &str,
    ) -> Result<(), ProjectEditingSessionError<R::Error, ()>>
    where
        R: ProjectSnapshotRepository,
    {
        let snapshot = snapshot_repo.get_latest_snapshot(session_id).await?;
        if let Some(snapshot) = snapshot {
            self.project_id = snapshot.metadata.project_id;
        }
        self.session_id = Some(session_id.to_string());
        self.session_setup_complete = true;
        Ok(())
    }

    pub async fn active_editing_session(
        &self,
    ) -> Result<Option<String>, ProjectEditingSessionError<(), ()>> {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }
        if self.session_setup_complete {
            Ok(self.session_id.clone())
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_diff() {
        let session =
            ProjectEditingSession::new("test_project".to_string(), ObjectTenant::default());

        // Test case 1: Identical states
        let client_state = vec![1, 2, 3, 4, 5];
        let server_state = vec![1, 2, 3, 4, 5];
        let (diff, result_server_state) = session.calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 5]);
        assert_eq!(result_server_state, server_state);

        // Test case 2: Server state has additional data
        let client_state = vec![1, 2, 3];
        let server_state = vec![1, 2, 3, 4, 5];
        let (diff, result_server_state) = session.calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 3, 0, 4, 0, 5]);
        assert_eq!(result_server_state, server_state);

        // Test case 3: Client state has additional data
        let client_state = vec![1, 2, 3, 4, 5];
        let server_state = vec![1, 2, 3];
        let (diff, result_server_state) = session.calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 3, 1, 1]);
        assert_eq!(result_server_state, server_state);

        // Test case 4: Different states
        let client_state = vec![1, 2, 3, 4, 5];
        let server_state = vec![1, 2, 6, 7, 8];
        let (diff, result_server_state) = session.calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 2, 2, 6, 2, 7, 2, 8]);
        assert_eq!(result_server_state, server_state);

        // Test case 5: Empty states
        let client_state = vec![];
        let server_state = vec![];
        let (diff, result_server_state) = session.calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![]);
        assert_eq!(result_server_state, server_state);
    }
}
