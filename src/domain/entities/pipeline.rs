//! Pipeline entity - The core aggregate for CI/CD pipeline definitions

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::{
    pipeline_config::PipelineConfig, pipeline_id::PipelineId, project_id::ProjectId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline entity representing a CI/CD pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pipeline {
    /// Unique identifier for the pipeline
    id: PipelineId,

    /// Project this pipeline belongs to
    project_id: ProjectId,

    /// Pipeline name
    name: String,

    /// Pipeline description
    description: Option<String>,

    /// Pipeline configuration (stages, jobs, triggers, etc.)
    config: PipelineConfig,

    /// Whether the pipeline is enabled
    enabled: bool,

    /// Pipeline version
    version: u32,

    /// Tags for categorization
    tags: Vec<String>,

    /// Environment variables
    environment: HashMap<String, String>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Domain events
    events: Vec<DomainEvent>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(project_id: ProjectId, name: String, config: PipelineConfig) -> Self {
        let now = Utc::now();
        let id = PipelineId::new();

        let mut pipeline = Self {
            id: id.clone(),
            project_id: project_id.clone(),
            name: name.clone(),
            description: None,
            config,
            enabled: true,
            version: 1,
            tags: Vec::new(),
            environment: HashMap::new(),
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        pipeline.events.push(DomainEvent::PipelineCreated {
            pipeline_id: id,
            project_id,
            name,
            created_at: now,
        });

        pipeline
    }

    /// Get the pipeline ID
    pub fn id(&self) -> &PipelineId {
        &self.id
    }

    /// Get the project ID
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    /// Get the pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the pipeline configuration
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Check if the pipeline is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Update the pipeline configuration
    pub fn update_config(&mut self, config: PipelineConfig) -> crate::Result<()> {
        // Validate the new configuration
        config.validate()?;

        self.config = config;
        self.version += 1;
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::PipelineConfigUpdated {
            pipeline_id: self.id.clone(),
            old_version: self.version - 1,
            new_version: self.version,
            updated_at: self.updated_at,
        });

        Ok(())
    }

    /// Enable the pipeline
    pub fn enable(&mut self) {
        if !self.enabled {
            self.enabled = true;
            self.updated_at = Utc::now();

            self.events.push(DomainEvent::PipelineEnabled {
                pipeline_id: self.id.clone(),
                enabled_at: self.updated_at,
            });
        }
    }

    /// Disable the pipeline
    pub fn disable(&mut self) {
        if self.enabled {
            self.enabled = false;
            self.updated_at = Utc::now();

            self.events.push(DomainEvent::PipelineDisabled {
                pipeline_id: self.id.clone(),
                disabled_at: self.updated_at,
            });
        }
    }

    /// Add a tag to the pipeline
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag from the pipeline
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    /// Set an environment variable
    pub fn set_environment_variable(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Remove an environment variable
    pub fn remove_environment_variable(&mut self, key: &str) {
        self.environment.remove(key);
        self.updated_at = Utc::now();
    }

    /// Get the domain events and clear them
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }

    /// Validate the pipeline
    pub fn validate(&self) -> crate::Result<()> {
        // Validate name
        if self.name.is_empty() {
            return Err(crate::Error::validation("Pipeline name cannot be empty"));
        }

        if self.name.len() > 255 {
            return Err(crate::Error::validation("Pipeline name is too long"));
        }

        // Validate configuration
        self.config.validate()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::pipeline_config::{Job, Stage, Trigger};

    fn create_test_pipeline() -> Pipeline {
        let project_id = ProjectId::new();
        let config = PipelineConfig::new(
            vec![Stage::new(
                "build".to_string(),
                vec![
                    Job::new("compile".to_string()),
                    Job::new("test".to_string()),
                ],
            )],
            vec![Trigger::Push {
                branches: vec!["main".to_string()],
            }],
        );

        Pipeline::new(project_id, "test-pipeline".to_string(), config)
    }

    #[test]
    fn test_new_pipeline() {
        let pipeline = create_test_pipeline();

        assert_eq!(pipeline.name(), "test-pipeline");
        assert!(pipeline.is_enabled());
        assert_eq!(pipeline.version, 1);
        assert_eq!(pipeline.events.len(), 1);

        match &pipeline.events[0] {
            DomainEvent::PipelineCreated { name, .. } => {
                assert_eq!(name, "test-pipeline");
            }
            _ => panic!("Expected PipelineCreated event"),
        }
    }

    #[test]
    fn test_enable_disable() {
        let mut pipeline = create_test_pipeline();

        // Initially enabled
        assert!(pipeline.is_enabled());

        // Disable
        pipeline.disable();
        assert!(!pipeline.is_enabled());
        assert_eq!(pipeline.events.len(), 2); // Created + Disabled

        // Enable
        pipeline.enable();
        assert!(pipeline.is_enabled());
        assert_eq!(pipeline.events.len(), 3); // Created + Disabled + Enabled
    }

    #[test]
    fn test_tags() {
        let mut pipeline = create_test_pipeline();

        pipeline.add_tag("production".to_string());
        pipeline.add_tag("critical".to_string());
        assert_eq!(pipeline.tags.len(), 2);

        // Adding duplicate tag should not add it again
        pipeline.add_tag("production".to_string());
        assert_eq!(pipeline.tags.len(), 2);

        pipeline.remove_tag("critical");
        assert_eq!(pipeline.tags.len(), 1);
        assert_eq!(pipeline.tags[0], "production");
    }

    #[test]
    fn test_environment_variables() {
        let mut pipeline = create_test_pipeline();

        pipeline.set_environment_variable("NODE_ENV".to_string(), "production".to_string());
        pipeline.set_environment_variable("DEBUG".to_string(), "false".to_string());

        assert_eq!(pipeline.environment.len(), 2);
        assert_eq!(
            pipeline.environment.get("NODE_ENV"),
            Some(&"production".to_string())
        );

        pipeline.remove_environment_variable("DEBUG");
        assert_eq!(pipeline.environment.len(), 1);
    }

    #[test]
    fn test_validation() {
        let project_id = ProjectId::new();
        let config = PipelineConfig::new(vec![], vec![]);

        // Empty name should fail validation
        let pipeline = Pipeline::new(project_id.clone(), "".to_string(), config.clone());
        assert!(pipeline.validate().is_err());

        // Very long name should fail validation
        let long_name = "a".repeat(256);
        let pipeline = Pipeline::new(project_id, long_name, config);
        assert!(pipeline.validate().is_err());
    }

    #[test]
    fn test_take_events() {
        let mut pipeline = create_test_pipeline();

        assert_eq!(pipeline.events.len(), 1);

        let events = pipeline.take_events();
        assert_eq!(events.len(), 1);
        assert!(pipeline.events.is_empty());
    }
}
