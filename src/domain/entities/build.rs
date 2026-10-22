//! Build entity - Represents a pipeline execution instance

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::{
    agent_id::AgentId, build_id::BuildId, build_status::BuildStatus, pipeline_id::PipelineId,
    project_id::ProjectId,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Build entity representing an execution of a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    /// Unique identifier for the build
    id: BuildId,

    /// Pipeline this build belongs to
    pipeline_id: PipelineId,

    /// Project this build belongs to
    project_id: ProjectId,

    /// Build number (sequential per pipeline)
    number: u64,

    /// Current status of the build
    status: BuildStatus,

    /// Git commit SHA
    commit_sha: String,

    /// Git branch
    branch: String,

    /// Commit message
    commit_message: Option<String>,

    /// Commit author
    commit_author: Option<String>,

    /// Agent executing the build
    agent_id: Option<AgentId>,

    /// Build parameters
    parameters: HashMap<String, String>,

    /// Environment variables
    environment: HashMap<String, String>,

    /// Build start time
    started_at: Option<DateTime<Utc>>,

    /// Build completion time
    completed_at: Option<DateTime<Utc>>,

    /// Build trigger type
    trigger: BuildTrigger,

    /// Build logs URL
    logs_url: Option<String>,

    /// Artifacts produced by this build
    artifacts: Vec<String>,

    /// Error message if build failed
    error_message: Option<String>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Domain events
    events: Vec<DomainEvent>,
}

/// Build trigger types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum BuildTrigger {
    /// Manual trigger by a user
    Manual { user_id: String },
    /// Git push trigger
    Push,
    /// Pull request trigger
    PullRequest { pr_number: u32 },
    /// Scheduled trigger
    Schedule { cron: String },
    /// API trigger
    Api { token: String },
    /// Webhook trigger
    Webhook { source: String },
}

impl Build {
    /// Create a new build
    pub fn new(
        pipeline_id: PipelineId,
        project_id: ProjectId,
        number: u64,
        commit_sha: String,
        branch: String,
        trigger: BuildTrigger,
    ) -> Self {
        let now = Utc::now();
        let id = BuildId::new();

        let mut build = Self {
            id: id.clone(),
            pipeline_id: pipeline_id.clone(),
            project_id: project_id.clone(),
            number,
            status: BuildStatus::Pending,
            commit_sha: commit_sha.clone(),
            branch: branch.clone(),
            commit_message: None,
            commit_author: None,
            agent_id: None,
            parameters: HashMap::new(),
            environment: HashMap::new(),
            started_at: None,
            completed_at: None,
            trigger,
            logs_url: None,
            artifacts: Vec::new(),
            error_message: None,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        build.events.push(DomainEvent::BuildCreated {
            build_id: id,
            pipeline_id,
            project_id,
            number,
            created_at: now,
        });

        build
    }

    /// Get the build ID
    pub fn id(&self) -> &BuildId {
        &self.id
    }

    /// Get the pipeline ID
    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    /// Get the project ID
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    /// Get the build number
    pub fn number(&self) -> u64 {
        self.number
    }

    /// Get the build status
    pub fn status(&self) -> &BuildStatus {
        &self.status
    }

    /// Get the build duration
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Start the build
    pub fn start(&mut self, agent_id: AgentId) -> crate::Result<()> {
        if self.status != BuildStatus::Pending {
            return Err(crate::Error::build("Build is not in pending state"));
        }

        self.status = BuildStatus::Running;
        self.agent_id = Some(agent_id.clone());
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::BuildStarted {
            build_id: self.id.clone(),
            agent_id,
            started_at: self.started_at.unwrap(),
        });

        Ok(())
    }

    /// Complete the build successfully
    pub fn succeed(&mut self) -> crate::Result<()> {
        if self.status != BuildStatus::Running {
            return Err(crate::Error::build("Build is not running"));
        }

        self.status = BuildStatus::Success;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::BuildCompleted {
            build_id: self.id.clone(),
            status: BuildStatus::Success,
            completed_at: self.completed_at.unwrap(),
        });

        Ok(())
    }

    /// Fail the build
    pub fn fail(&mut self, error: String) -> crate::Result<()> {
        if self.status != BuildStatus::Running {
            return Err(crate::Error::build("Build is not running"));
        }

        self.status = BuildStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::BuildCompleted {
            build_id: self.id.clone(),
            status: BuildStatus::Failed,
            completed_at: self.completed_at.unwrap(),
        });

        Ok(())
    }

    /// Cancel the build
    pub fn cancel(&mut self) -> crate::Result<()> {
        if self.status == BuildStatus::Success || self.status == BuildStatus::Failed {
            return Err(crate::Error::build("Cannot cancel completed build"));
        }

        self.status = BuildStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::BuildCancelled {
            build_id: self.id.clone(),
            cancelled_at: self.completed_at.unwrap(),
        });

        Ok(())
    }

    /// Add a build parameter
    pub fn add_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Add an environment variable
    pub fn add_environment(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Add an artifact
    pub fn add_artifact(&mut self, path: String) {
        if !self.artifacts.contains(&path) {
            self.artifacts.push(path);
            self.updated_at = Utc::now();
        }
    }

    /// Set commit details
    pub fn set_commit_details(&mut self, message: String, author: String) {
        self.commit_message = Some(message);
        self.commit_author = Some(author);
        self.updated_at = Utc::now();
    }

    /// Get the domain events and clear them
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_build() -> Build {
        Build::new(
            PipelineId::new(),
            ProjectId::new(),
            1,
            "abc123".to_string(),
            "main".to_string(),
            BuildTrigger::Manual {
                user_id: "user123".to_string(),
            },
        )
    }

    #[test]
    fn test_new_build() {
        let build = create_test_build();

        assert_eq!(build.number(), 1);
        assert_eq!(build.status(), &BuildStatus::Pending);
        assert_eq!(build.branch, "main");
        assert_eq!(build.commit_sha, "abc123");
        assert_eq!(build.events.len(), 1);
    }

    #[test]
    fn test_build_lifecycle() {
        let mut build = create_test_build();
        let agent_id = AgentId::new();

        // Start build
        assert!(build.start(agent_id.clone()).is_ok());
        assert_eq!(build.status(), &BuildStatus::Running);
        assert_eq!(build.agent_id, Some(agent_id));
        assert!(build.started_at.is_some());

        // Complete build
        assert!(build.succeed().is_ok());
        assert_eq!(build.status(), &BuildStatus::Success);
        assert!(build.completed_at.is_some());
        assert!(build.duration().is_some());
    }

    #[test]
    fn test_build_failure() {
        let mut build = create_test_build();
        let agent_id = AgentId::new();

        assert!(build.start(agent_id).is_ok());
        assert!(build.fail("Test error".to_string()).is_ok());

        assert_eq!(build.status(), &BuildStatus::Failed);
        assert_eq!(build.error_message, Some("Test error".to_string()));
        assert!(build.completed_at.is_some());
    }

    #[test]
    fn test_build_cancellation() {
        let mut build = create_test_build();

        // Can cancel pending build
        assert!(build.cancel().is_ok());
        assert_eq!(build.status(), &BuildStatus::Cancelled);

        // Cannot cancel completed build
        let mut build = create_test_build();
        let agent_id = AgentId::new();
        assert!(build.start(agent_id).is_ok());
        assert!(build.succeed().is_ok());
        assert!(build.cancel().is_err());
    }

    #[test]
    fn test_build_parameters() {
        let mut build = create_test_build();

        build.add_parameter("VERSION".to_string(), "1.0.0".to_string());
        build.add_parameter("ENV".to_string(), "production".to_string());

        assert_eq!(build.parameters.len(), 2);
        assert_eq!(build.parameters.get("VERSION"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_build_artifacts() {
        let mut build = create_test_build();

        build.add_artifact("/path/to/artifact1".to_string());
        build.add_artifact("/path/to/artifact2".to_string());

        assert_eq!(build.artifacts.len(), 2);

        // Duplicate artifacts should not be added
        build.add_artifact("/path/to/artifact1".to_string());
        assert_eq!(build.artifacts.len(), 2);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut build = create_test_build();

        // Cannot succeed without starting
        assert!(build.succeed().is_err());

        // Cannot fail without starting
        assert!(build.fail("error".to_string()).is_err());

        // Cannot start twice
        let agent_id = AgentId::new();
        assert!(build.start(agent_id.clone()).is_ok());
        assert!(build.start(agent_id).is_err());
    }
}
