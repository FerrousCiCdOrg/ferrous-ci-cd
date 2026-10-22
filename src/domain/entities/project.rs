//! Project entity - Represents a software project with CI/CD pipelines

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::project_id::ProjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier
    id: ProjectId,

    /// Project name
    name: String,

    /// Project description
    description: Option<String>,

    /// Repository URL
    repository_url: String,

    /// Default branch
    default_branch: String,

    /// Project visibility
    visibility: ProjectVisibility,

    /// Project settings
    settings: ProjectSettings,

    /// Project metadata
    metadata: HashMap<String, String>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Domain events
    events: Vec<DomainEvent>,
}

/// Project visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectVisibility {
    /// Public project
    Public,
    /// Internal project (visible to authenticated users)
    Internal,
    /// Private project
    Private,
}

/// Project settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Auto-cancel redundant builds
    pub auto_cancel: bool,

    /// Build timeout in seconds
    pub build_timeout: u64,

    /// Maximum concurrent builds
    pub max_concurrent_builds: usize,

    /// Enable pull request builds
    pub pr_builds_enabled: bool,

    /// Protected branches
    pub protected_branches: Vec<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_cancel: true,
            build_timeout: 3600,
            max_concurrent_builds: 5,
            pr_builds_enabled: true,
            protected_branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

impl Project {
    /// Create a new project
    pub fn new(name: String, repository_url: String, default_branch: String) -> Self {
        let now = Utc::now();
        let id = ProjectId::new();

        let mut project = Self {
            id: id.clone(),
            name: name.clone(),
            description: None,
            repository_url: repository_url.clone(),
            default_branch,
            visibility: ProjectVisibility::Private,
            settings: ProjectSettings::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        project.events.push(DomainEvent::ProjectCreated {
            project_id: id,
            name,
            repository_url,
            created_at: now,
        });

        project
    }

    /// Get the project ID
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    /// Get the project name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the repository URL
    pub fn repository_url(&self) -> &str {
        &self.repository_url
    }

    /// Update project settings
    pub fn update_settings(&mut self, settings: ProjectSettings) {
        self.settings = settings;
        self.updated_at = Utc::now();
    }

    /// Set project visibility
    pub fn set_visibility(&mut self, visibility: ProjectVisibility) {
        self.visibility = visibility;
        self.updated_at = Utc::now();
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Get the domain events and clear them
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }

    /// Validate the project
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::Error::validation("Project name cannot be empty"));
        }

        if self.repository_url.is_empty() {
            return Err(crate::Error::validation("Repository URL cannot be empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project() {
        let project = Project::new(
            "test-project".to_string(),
            "https://github.com/user/repo.git".to_string(),
            "main".to_string(),
        );

        assert_eq!(project.name(), "test-project");
        assert_eq!(project.repository_url(), "https://github.com/user/repo.git");
        assert_eq!(project.default_branch, "main");
        assert_eq!(project.visibility, ProjectVisibility::Private);
        assert_eq!(project.events.len(), 1);
    }

    #[test]
    fn test_project_settings() {
        let mut project = Project::new(
            "test".to_string(),
            "https://repo.git".to_string(),
            "main".to_string(),
        );

        let mut settings = ProjectSettings::default();
        settings.max_concurrent_builds = 10;

        project.update_settings(settings);
        assert_eq!(project.settings.max_concurrent_builds, 10);
    }
}
