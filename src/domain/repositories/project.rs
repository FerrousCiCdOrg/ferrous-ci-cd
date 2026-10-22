//! Project repository interface

use crate::domain::entities::project::Project;
use crate::domain::value_objects::project_id::ProjectId;
use async_trait::async_trait;

/// Project repository interface
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Save a project
    async fn save(&self, project: &Project) -> crate::Result<()>;

    /// Find a project by ID
    async fn find_by_id(&self, id: &ProjectId) -> crate::Result<Option<Project>>;

    /// Find a project by name
    async fn find_by_name(&self, name: &str) -> crate::Result<Option<Project>>;

    /// Find all projects
    async fn find_all(&self) -> crate::Result<Vec<Project>>;

    /// Update a project
    async fn update(&self, project: &Project) -> crate::Result<()>;

    /// Delete a project
    async fn delete(&self, id: &ProjectId) -> crate::Result<()>;

    /// Check if a project exists
    async fn exists(&self, id: &ProjectId) -> crate::Result<bool>;

    /// Check if a project name is taken
    async fn name_exists(&self, name: &str) -> crate::Result<bool>;
}
