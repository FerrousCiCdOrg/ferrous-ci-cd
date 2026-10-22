//! Build repository interface

use crate::domain::entities::build::Build;
use crate::domain::value_objects::{
    build_id::BuildId, build_status::BuildStatus, pipeline_id::PipelineId, project_id::ProjectId,
};
use async_trait::async_trait;

/// Build query options
#[derive(Debug, Clone, Default)]
pub struct BuildQueryOptions {
    /// Filter by project ID
    pub project_id: Option<ProjectId>,

    /// Filter by pipeline ID
    pub pipeline_id: Option<PipelineId>,

    /// Filter by status
    pub status: Option<BuildStatus>,

    /// Filter by branch
    pub branch: Option<String>,

    /// Limit number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,

    /// Sort by field
    pub sort_by: Option<String>,

    /// Sort descending
    pub sort_desc: bool,
}

/// Build repository interface
#[async_trait]
pub trait BuildRepository: Send + Sync {
    /// Save a build
    async fn save(&self, build: &Build) -> crate::Result<()>;

    /// Find a build by ID
    async fn find_by_id(&self, id: &BuildId) -> crate::Result<Option<Build>>;

    /// Find builds by pipeline
    async fn find_by_pipeline(&self, pipeline_id: &PipelineId) -> crate::Result<Vec<Build>>;

    /// Find builds by project
    async fn find_by_project(&self, project_id: &ProjectId) -> crate::Result<Vec<Build>>;

    /// Find builds with query options
    async fn query(&self, options: BuildQueryOptions) -> crate::Result<Vec<Build>>;

    /// Find running builds
    async fn find_running(&self) -> crate::Result<Vec<Build>>;

    /// Get the next build number for a pipeline
    async fn next_build_number(&self, pipeline_id: &PipelineId) -> crate::Result<u64>;

    /// Update a build
    async fn update(&self, build: &Build) -> crate::Result<()>;

    /// Delete a build
    async fn delete(&self, id: &BuildId) -> crate::Result<()>;

    /// Count builds by status
    async fn count_by_status(&self, status: &BuildStatus) -> crate::Result<u64>;
}
