use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::generate_id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub collaborators: Vec<ProjectCollaborator>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(
        workspace_id: String,
        name: String,
        owner_id: String,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id!("project"),
            workspace_id,
            name,
            description,
            owner_id: owner_id.clone(),
            collaborators: vec![ProjectCollaborator {
                user_id: owner_id,
                role: ProjectRole::Owner,
                joined_at: now,
            }],
            status: ProjectStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_collaborator(
        &mut self,
        user_id: String,
        role: ProjectRole,
    ) -> Result<(), &'static str> {
        if self.collaborators.iter().any(|c| c.user_id == user_id) {
            return Err("User is already a collaborator");
        }

        self.collaborators.push(ProjectCollaborator {
            user_id,
            role,
            joined_at: Utc::now(),
        });
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_collaborator(&mut self, user_id: &str) -> Result<(), &'static str> {
        if user_id == self.owner_id {
            return Err("Cannot remove project owner");
        }

        let initial_len = self.collaborators.len();
        self.collaborators.retain(|c| c.user_id != user_id);

        if self.collaborators.len() == initial_len {
            return Err("Collaborator not found");
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_collaborator_role(
        &mut self,
        user_id: &str,
        new_role: ProjectRole,
    ) -> Result<(), &'static str> {
        if user_id == self.owner_id {
            return Err("Cannot change project owner's role");
        }

        if let Some(collab) = self.collaborators.iter_mut().find(|c| c.user_id == user_id) {
            collab.role = new_role;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Collaborator not found")
        }
    }

    pub fn transfer_ownership(&mut self, new_owner_id: String) -> Result<(), &'static str> {
        if !self.is_collaborator(&new_owner_id) {
            return Err("New owner must be a project collaborator");
        }

        if let Some(old_owner) = self
            .collaborators
            .iter_mut()
            .find(|c| c.user_id == self.owner_id)
        {
            old_owner.role = ProjectRole::Editor;
        }

        if let Some(new_owner) = self
            .collaborators
            .iter_mut()
            .find(|c| c.user_id == new_owner_id)
        {
            new_owner.role = ProjectRole::Owner;
        }

        self.owner_id = new_owner_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_status(&mut self, new_status: ProjectStatus) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }

    pub fn is_collaborator(&self, user_id: &str) -> bool {
        self.collaborators.iter().any(|c| c.user_id == user_id)
    }

    pub fn get_collaborator_role(&self, user_id: &str) -> Option<&ProjectRole> {
        self.collaborators
            .iter()
            .find(|c| c.user_id == user_id)
            .map(|c| &c.role)
    }

    pub fn get_user_permissions(&self, user_id: &str) -> Vec<ProjectActionType> {
        let role = match self.get_collaborator_role(user_id) {
            Some(role) => role,
            None => return vec![],
        };

        match role {
            ProjectRole::Owner => vec![
                ProjectActionType::View,
                ProjectActionType::Edit,
                ProjectActionType::Delete,
                ProjectActionType::ManageUsers,
                ProjectActionType::ChangeSettings,
            ],
            ProjectRole::Editor => vec![ProjectActionType::View, ProjectActionType::Edit],
            ProjectRole::Viewer => vec![ProjectActionType::View],
        }
    }

    pub fn can_user_perform(&self, user_id: &str, action: ProjectActionType) -> bool {
        self.get_user_permissions(user_id).contains(&action)
    }

    pub fn get_editors(&self) -> Vec<String> {
        self.collaborators
            .iter()
            .filter(|c| matches!(c.role, ProjectRole::Owner | ProjectRole::Editor))
            .map(|c| c.user_id.clone())
            .collect()
    }

    pub fn get_active_editors(&self, active_sessions: &HashSet<String>) -> Vec<String> {
        self.get_editors()
            .into_iter()
            .filter(|user_id| active_sessions.contains(user_id))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectCollaborator {
    pub user_id: String,
    pub role: ProjectRole,
    pub joined_at: DateTime<Utc>,
}

impl ProjectCollaborator {
    pub fn new(user_id: String, role: ProjectRole) -> Self {
        Self {
            user_id,
            role,
            joined_at: Utc::now(),
        }
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.role, ProjectRole::Owner)
    }

    pub fn can_edit(&self) -> bool {
        matches!(self.role, ProjectRole::Owner | ProjectRole::Editor)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProjectRole {
    Owner,
    Editor,
    Viewer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectPermission {
    pub project_id: String,
    pub user_id: String,
    pub permissions: Vec<ProjectAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAction {
    pub action_type: ProjectActionType,
    pub allowed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProjectActionType {
    View,
    Edit,
    Delete,
    ManageUsers,
    ChangeSettings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let owner_id = "user1".to_string();
        let project = Project::new(
            "workspace1".to_string(),
            "Test Project".to_string(),
            owner_id.clone(),
            Some("Description".to_string()),
        );

        assert!(project.id.starts_with("project"));
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.owner_id, owner_id);
        assert_eq!(project.collaborators.len(), 1);
        assert!(matches!(project.collaborators[0].role, ProjectRole::Owner));
    }

    #[test]
    fn test_collaborator_management() {
        let mut project = Project::new(
            "workspace1".to_string(),
            "Test Project".to_string(),
            "user1".to_string(),
            None,
        );

        assert!(project
            .add_collaborator("user2".to_string(), ProjectRole::Editor)
            .is_ok());
        assert_eq!(project.collaborators.len(), 2);

        assert!(project.can_user_perform("user2", ProjectActionType::Edit));
        assert!(!project.can_user_perform("user2", ProjectActionType::Delete));

        assert!(project
            .update_collaborator_role("user2", ProjectRole::Viewer)
            .is_ok());
        assert!(!project.can_user_perform("user2", ProjectActionType::Edit));

        assert!(project.remove_collaborator("user2").is_ok());
        assert_eq!(project.collaborators.len(), 1);
    }
}
