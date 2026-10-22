//! Domain layer - Core business logic and entities
//!
//! This module contains the heart of the business logic following Domain-Driven Design principles.

pub mod entities;
pub mod events;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export commonly used types
pub use entities::{agent::Agent, build::Build, pipeline::Pipeline, project::Project, user::User};

pub use value_objects::{
    agent_id::AgentId, build_id::BuildId, build_status::BuildStatus,
    pipeline_config::PipelineConfig, pipeline_id::PipelineId, project_id::ProjectId,
    user_id::UserId,
};

pub use repositories::{
    agent::AgentRepository, build::BuildRepository, pipeline::PipelineRepository,
    project::ProjectRepository, user::UserRepository,
};

pub use services::{agent::AgentService, build::BuildService, pipeline::PipelineService};

pub use events::{DomainEvent, EventPublisher};
