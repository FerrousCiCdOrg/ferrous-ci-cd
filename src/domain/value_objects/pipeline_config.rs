//! Pipeline Configuration value object

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline Configuration value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Pipeline version
    pub version: String,

    /// Pipeline stages
    pub stages: Vec<Stage>,

    /// Pipeline triggers
    pub triggers: Vec<Trigger>,

    /// Global environment variables
    pub environment: HashMap<String, String>,

    /// Notification settings
    pub notifications: Option<NotificationConfig>,
}

/// Pipeline stage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    /// Stage name
    pub name: String,

    /// Jobs in this stage
    pub jobs: Vec<Job>,

    /// Whether jobs in this stage can run in parallel
    pub parallel: bool,

    /// Conditions for running this stage
    pub when: Option<WhenCondition>,
}

/// Job configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    /// Job name
    pub name: String,

    /// Docker image to use
    pub image: Option<String>,

    /// Commands to execute
    pub commands: Vec<String>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Working directory
    pub working_directory: Option<String>,

    /// Job timeout in seconds
    pub timeout: Option<u64>,

    /// Number of retry attempts
    pub retry: Option<u32>,

    /// Artifacts to save
    pub artifacts: Option<ArtifactConfig>,

    /// Cache configuration
    pub cache: Option<CacheConfig>,

    /// Dependencies on other jobs
    pub needs: Vec<String>,

    /// Conditions for running this job
    pub when: Option<WhenCondition>,
}

/// Pipeline trigger
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Trigger {
    /// Trigger on push
    Push {
        /// Branches to trigger on
        branches: Vec<String>,
    },
    /// Trigger on pull request
    PullRequest {
        /// Branches to trigger on
        branches: Vec<String>,
    },
    /// Scheduled trigger
    Schedule {
        /// Cron expression
        cron: String,
    },
    /// Manual trigger
    Manual,
    /// Tag trigger
    Tag {
        /// Tag patterns
        patterns: Vec<String>,
    },
}

/// Condition for running a stage or job
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhenCondition {
    /// Branch condition
    pub branch: Option<String>,

    /// Event type condition
    pub event: Option<String>,

    /// Status condition
    pub status: Option<String>,
}

/// Artifact configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactConfig {
    /// Paths to include
    pub paths: Vec<String>,

    /// Paths to exclude
    pub exclude: Vec<String>,

    /// Artifact name
    pub name: Option<String>,

    /// Expiration time in days
    pub expire_in: Option<u32>,
}

/// Cache configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache key
    pub key: String,

    /// Paths to cache
    pub paths: Vec<String>,

    /// Cache policy (pull, push, pull-push)
    pub policy: Option<String>,
}

/// Notification configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Email notifications
    pub email: Option<Vec<String>>,

    /// Slack notifications
    pub slack: Option<SlackNotification>,

    /// Webhook notifications
    pub webhooks: Vec<String>,
}

/// Slack notification configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlackNotification {
    /// Slack channel
    pub channel: String,

    /// Notify on success
    pub on_success: bool,

    /// Notify on failure
    pub on_failure: bool,
}

impl PipelineConfig {
    /// Create a new pipeline configuration
    pub fn new(stages: Vec<Stage>, triggers: Vec<Trigger>) -> Self {
        Self {
            version: "1.0".to_string(),
            stages,
            triggers,
            environment: HashMap::new(),
            notifications: None,
        }
    }

    /// Validate the pipeline configuration
    pub fn validate(&self) -> crate::Result<()> {
        // Validate version
        if self.version.is_empty() {
            return Err(crate::Error::validation("Pipeline version cannot be empty"));
        }

        // Validate stages
        if self.stages.is_empty() {
            return Err(crate::Error::validation(
                "Pipeline must have at least one stage",
            ));
        }

        for stage in &self.stages {
            stage.validate()?;
        }

        // Validate triggers
        if self.triggers.is_empty() {
            return Err(crate::Error::validation(
                "Pipeline must have at least one trigger",
            ));
        }

        Ok(())
    }

    /// Add a stage
    pub fn add_stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }

    /// Add a trigger
    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.push(trigger);
    }

    /// Set environment variable
    pub fn set_environment(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }
}

impl Stage {
    /// Create a new stage
    pub fn new(name: String, jobs: Vec<Job>) -> Self {
        Self {
            name,
            jobs,
            parallel: false,
            when: None,
        }
    }

    /// Validate the stage
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::Error::validation("Stage name cannot be empty"));
        }

        if self.jobs.is_empty() {
            return Err(crate::Error::validation("Stage must have at least one job"));
        }

        for job in &self.jobs {
            job.validate()?;
        }

        Ok(())
    }
}

impl Job {
    /// Create a new job
    pub fn new(name: String) -> Self {
        Self {
            name,
            image: None,
            commands: Vec::new(),
            environment: HashMap::new(),
            working_directory: None,
            timeout: None,
            retry: None,
            artifacts: None,
            cache: None,
            needs: Vec::new(),
            when: None,
        }
    }

    /// Validate the job
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::Error::validation("Job name cannot be empty"));
        }

        if self.commands.is_empty() && self.image.is_none() {
            return Err(crate::Error::validation(
                "Job must have at least one command or an image",
            ));
        }

        Ok(())
    }

    /// Add a command
    pub fn add_command(&mut self, command: String) {
        self.commands.push(command);
    }

    /// Set the image
    pub fn set_image(&mut self, image: String) {
        self.image = Some(image);
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            stages: Vec::new(),
            triggers: Vec::new(),
            environment: HashMap::new(),
            notifications: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_creation() {
        let config = PipelineConfig::new(
            vec![Stage::new(
                "build".to_string(),
                vec![Job::new("compile".to_string())],
            )],
            vec![Trigger::Push {
                branches: vec!["main".to_string()],
            }],
        );

        assert_eq!(config.stages.len(), 1);
        assert_eq!(config.triggers.len(), 1);
    }

    #[test]
    fn test_pipeline_config_validation() {
        let config = PipelineConfig::new(vec![], vec![]);
        assert!(config.validate().is_err());

        let mut config = PipelineConfig::new(
            vec![Stage::new(
                "build".to_string(),
                vec![Job::new("compile".to_string())],
            )],
            vec![Trigger::Manual],
        );

        // Job needs commands
        assert!(config.validate().is_err());

        config.stages[0].jobs[0].add_command("echo 'Hello'".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_job_validation() {
        let mut job = Job::new("test".to_string());

        // Job without commands or image should fail
        assert!(job.validate().is_err());

        // Job with commands should pass
        job.add_command("echo 'test'".to_string());
        assert!(job.validate().is_ok());

        // Job with image should pass
        let mut job2 = Job::new("docker-job".to_string());
        job2.set_image("rust:latest".to_string());
        assert!(job2.validate().is_ok());
    }

    #[test]
    fn test_stage_validation() {
        let mut stage = Stage::new("build".to_string(), vec![]);

        // Stage without jobs should fail
        assert!(stage.validate().is_err());

        // Add a job with commands
        let mut job = Job::new("compile".to_string());
        job.add_command("cargo build".to_string());
        stage.jobs.push(job);

        assert!(stage.validate().is_ok());
    }
}
