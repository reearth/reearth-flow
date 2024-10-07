use std::cmp::min;
use std::sync::Arc;

use crate::repository::{ProjectSnapshotRepository, RedisDataManager};
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

    pub async fn get_diff_update<R>(
        &self,
        state_vector: Vec<u8>,
        redis_data_manager: &R,
    ) -> Result<(Vec<u8>, Vec<u8>), ProjectEditingSessionError<R::Error>>
    where
        R: RedisDataManager,
    {
        self.check_session_setup()?;
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
