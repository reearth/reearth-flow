use crate::generate_id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Option<String>,
    pub owner_id: String,
    pub members: Vec<WorkspaceMember>,
    pub settings: WorkspaceSettings,
    pub projects: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceMember {
    pub user_id: String,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum WorkspaceRole {
    Admin,
    Member,
    Guest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceSettings {
    pub allow_member_create_project: bool,
    pub default_project_visibility: ProjectVisibility,
    pub allow_external_collaborators: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectVisibility {
    Public,
    Private,
}

impl Workspace {
    pub fn new(
        name: String,
        description: Option<String>,
        organization_id: Option<String>,
        owner_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id!("workspace"),
            name,
            description,
            organization_id,
            owner_id: owner_id.clone(),
            members: vec![WorkspaceMember {
                user_id: owner_id,
                role: WorkspaceRole::Admin,
                joined_at: now,
            }],
            settings: WorkspaceSettings::default(),
            projects: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_member(&mut self, user_id: String, role: WorkspaceRole) -> Result<(), &'static str> {
        if self.members.iter().any(|m| m.user_id == user_id) {
            return Err("User is already a member");
        }

        self.members.push(WorkspaceMember {
            user_id,
            role,
            joined_at: Utc::now(),
        });
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_member(&mut self, user_id: &str) -> Result<(), &'static str> {
        if user_id == self.owner_id {
            return Err("Cannot remove workspace owner");
        }

        let initial_len = self.members.len();
        self.members.retain(|m| m.user_id != user_id);

        if self.members.len() == initial_len {
            return Err("Member not found");
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_member_role(
        &mut self,
        user_id: &str,
        new_role: WorkspaceRole,
    ) -> Result<(), &'static str> {
        if user_id == self.owner_id {
            return Err("Cannot change workspace owner's role");
        }

        if let Some(member) = self.members.iter_mut().find(|m| m.user_id == user_id) {
            member.role = new_role;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Member not found")
        }
    }

    pub fn is_member(&self, user_id: &str) -> bool {
        self.members.iter().any(|m| m.user_id == user_id)
    }

    pub fn get_member_role(&self, user_id: &str) -> Option<&WorkspaceRole> {
        self.members
            .iter()
            .find(|m| m.user_id == user_id)
            .map(|m| &m.role)
    }

    pub fn update_settings(&mut self, new_settings: WorkspaceSettings) {
        self.settings = new_settings;
        self.updated_at = Utc::now();
    }

    pub fn transfer_ownership(&mut self, new_owner_id: String) -> Result<(), &'static str> {
        if !self.is_member(&new_owner_id) {
            return Err("New owner must be a workspace member");
        }

        if let Some(old_owner) = self.members.iter_mut().find(|m| m.user_id == self.owner_id) {
            old_owner.role = WorkspaceRole::Admin;
        }

        if let Some(new_owner) = self.members.iter_mut().find(|m| m.user_id == new_owner_id) {
            new_owner.role = WorkspaceRole::Admin;
        }

        self.owner_id = new_owner_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_project(&mut self, project_id: String) -> Result<(), &'static str> {
        if self.projects.contains(&project_id) {
            return Err("Project already exists in workspace");
        }
        self.projects.push(project_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_project(&mut self, project_id: &str) -> Result<(), &'static str> {
        let initial_len = self.projects.len();
        self.projects.retain(|p| p != project_id);

        if self.projects.len() == initial_len {
            return Err("Project not found in workspace");
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_projects(&self) -> &Vec<String> {
        &self.projects
    }
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            allow_member_create_project: true,
            default_project_visibility: ProjectVisibility::Private,
            allow_external_collaborators: false,
        }
    }
}

impl WorkspaceMember {
    pub fn new(user_id: String, role: WorkspaceRole) -> Self {
        Self {
            user_id,
            role,
            joined_at: Utc::now(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == WorkspaceRole::Admin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let owner_id = "user1".to_string();
        let workspace = Workspace::new(
            "Test Workspace".to_string(),
            Some("Description".to_string()),
            None,
            owner_id.clone(),
        );

        assert!(workspace.id.starts_with("workspace"));
        assert_eq!(workspace.name, "Test Workspace");
        assert_eq!(workspace.owner_id, owner_id);
        assert_eq!(workspace.members.len(), 1);
        assert_eq!(workspace.members[0].role, WorkspaceRole::Admin);
    }

    #[test]
    fn test_member_management() {
        let owner_id = "user1".to_string();
        let mut workspace = Workspace::new("Test Workspace".to_string(), None, None, owner_id);

        assert!(workspace
            .add_member("user2".to_string(), WorkspaceRole::Member)
            .is_ok());
        assert_eq!(workspace.members.len(), 2);

        assert!(workspace
            .add_member("user2".to_string(), WorkspaceRole::Member)
            .is_err());

        assert!(workspace
            .update_member_role("user2", WorkspaceRole::Admin)
            .is_ok());
        assert_eq!(
            workspace.get_member_role("user2"),
            Some(&WorkspaceRole::Admin)
        );

        assert!(workspace.remove_member("user2").is_ok());
        assert_eq!(workspace.members.len(), 1);
    }
}
