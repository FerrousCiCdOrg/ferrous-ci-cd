//! Domain layer - Core business logic and entities
//!
//! This module contains the heart of the business logic following Domain-Driven Design principles.

pub mod entities;
pub mod pipeline_dsl;
pub mod value_objects;
pub mod repositories;
pub mod services;
pub mod events;

// Re-export commonly used types
pub use entities::{
    pipeline::Pipeline,
    build::Build,
    project::Project,
    agent::Agent,
    user::User,
};

pub use value_objects::{
    build_id::BuildId,
    pipeline_id::PipelineId,
    project_id::ProjectId,
    agent_id::AgentId,
    user_id::UserId,
    build_status::BuildStatus,
    pipeline_config::PipelineConfig,
};

pub use repositories::{
    pipeline::PipelineRepository,
    build::BuildRepository,
    project::ProjectRepository,
    agent::AgentRepository,
    user::UserRepository,
};

pub use services::{
    pipeline::PipelineService,
    build::BuildService,
    agent::AgentService,
};

pub use events::{DomainEvent, EventPublisher};
