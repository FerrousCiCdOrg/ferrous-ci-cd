//! Workspace entity - Represents a build workspace

use crate::domain::value_objects::{
    agent_id::AgentId, build_id::BuildId, workspace_id::WorkspaceId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Workspace entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique identifier
    id: WorkspaceId,

    /// Build using this workspace
    build_id: BuildId,

    /// Agent this workspace is on
    agent_id: Option<AgentId>,

    /// Workspace path
    path: PathBuf,

    /// Workspace status
    status: WorkspaceStatus,

    /// Size in bytes
    size: u64,

    /// Git repository URL
    repository_url: Option<String>,

    /// Git commit
    commit: Option<String>,

    /// Git branch
    branch: Option<String>,

    /// Whether to clean workspace after build
    cleanup_on_completion: bool,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,
}

/// Workspace status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkspaceStatus {
    /// Workspace is being initialized
    Initializing,
    /// Workspace is ready
    Ready,
    /// Workspace is in use
    InUse,
    /// Workspace is being cleaned up
    CleaningUp,
    /// Workspace is cleaned and can be removed
    Cleaned,
    /// Workspace has an error
    Error,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(build_id: BuildId, path: PathBuf) -> Self {
        let now = Utc::now();

        Self {
            id: WorkspaceId::new(),
            build_id,
            agent_id: None,
            path,
            status: WorkspaceStatus::Initializing,
            size: 0,
            repository_url: None,
            commit: None,
            branch: None,
            cleanup_on_completion: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the workspace ID
    pub fn id(&self) -> &WorkspaceId {
        &self.id
    }

    /// Get the workspace path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the workspace status
    pub fn status(&self) -> &WorkspaceStatus {
        &self.status
    }

    /// Get the workspace size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Assign workspace to an agent
    pub fn assign_to_agent(&mut self, agent_id: AgentId) {
        self.agent_id = Some(agent_id);
        self.updated_at = Utc::now();
    }

    /// Set Git repository information
    pub fn set_repository(&mut self, url: String, commit: String, branch: String) {
        self.repository_url = Some(url);
        self.commit = Some(commit);
        self.branch = Some(branch);
        self.updated_at = Utc::now();
    }

    /// Mark workspace as ready
    pub fn mark_ready(&mut self) -> crate::Result<()> {
        if self.status != WorkspaceStatus::Initializing {
            return Err(crate::Error::build("Workspace is not initializing"));
        }

        self.status = WorkspaceStatus::Ready;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Mark workspace as in use
    pub fn mark_in_use(&mut self) -> crate::Result<()> {
        if self.status != WorkspaceStatus::Ready {
            return Err(crate::Error::build("Workspace is not ready"));
        }

        self.status = WorkspaceStatus::InUse;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Start cleanup
    pub fn start_cleanup(&mut self) -> crate::Result<()> {
        if self.status == WorkspaceStatus::CleaningUp || self.status == WorkspaceStatus::Cleaned {
            return Err(crate::Error::build("Workspace is already being cleaned"));
        }

        self.status = WorkspaceStatus::CleaningUp;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Mark cleanup as complete
    pub fn mark_cleaned(&mut self) -> crate::Result<()> {
        if self.status != WorkspaceStatus::CleaningUp {
            return Err(crate::Error::build("Workspace is not being cleaned"));
        }

        self.status = WorkspaceStatus::Cleaned;
        self.size = 0;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Mark workspace as having an error
    pub fn mark_error(&mut self) {
        self.status = WorkspaceStatus::Error;
        self.updated_at = Utc::now();
    }

    /// Update workspace size
    pub fn update_size(&mut self, size: u64) {
        self.size = size;
        self.updated_at = Utc::now();
    }

    /// Set cleanup policy
    pub fn set_cleanup_on_completion(&mut self, cleanup: bool) {
        self.cleanup_on_completion = cleanup;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workspace() -> Workspace {
        Workspace::new(BuildId::new(), PathBuf::from("/tmp/workspace/123"))
    }

    #[test]
    fn test_new_workspace() {
        let workspace = create_test_workspace();

        assert_eq!(workspace.status(), &WorkspaceStatus::Initializing);
        assert_eq!(workspace.size(), 0);
        assert!(workspace.cleanup_on_completion);
    }

    #[test]
    fn test_workspace_lifecycle() {
        let mut workspace = create_test_workspace();

        // Mark as ready
        assert!(workspace.mark_ready().is_ok());
        assert_eq!(workspace.status(), &WorkspaceStatus::Ready);

        // Mark as in use
        assert!(workspace.mark_in_use().is_ok());
        assert_eq!(workspace.status(), &WorkspaceStatus::InUse);

        // Start cleanup
        assert!(workspace.start_cleanup().is_ok());
        assert_eq!(workspace.status(), &WorkspaceStatus::CleaningUp);

        // Mark as cleaned
        assert!(workspace.mark_cleaned().is_ok());
        assert_eq!(workspace.status(), &WorkspaceStatus::Cleaned);
    }

    #[test]
    fn test_workspace_git_info() {
        let mut workspace = create_test_workspace();

        workspace.set_repository(
            "https://github.com/user/repo.git".to_string(),
            "abc123".to_string(),
            "main".to_string(),
        );

        assert_eq!(
            workspace.repository_url,
            Some("https://github.com/user/repo.git".to_string())
        );
        assert_eq!(workspace.commit, Some("abc123".to_string()));
        assert_eq!(workspace.branch, Some("main".to_string()));
    }

    #[test]
    fn test_workspace_agent_assignment() {
        let mut workspace = create_test_workspace();
        let agent_id = AgentId::new();

        workspace.assign_to_agent(agent_id.clone());
        assert_eq!(workspace.agent_id, Some(agent_id));
    }

    #[test]
    fn test_workspace_size_update() {
        let mut workspace = create_test_workspace();

        workspace.update_size(1024);
        assert_eq!(workspace.size(), 1024);

        workspace.update_size(2048);
        assert_eq!(workspace.size(), 2048);
    }
}
