//! Job entity - Represents a unit of work within a stage

use crate::domain::value_objects::{agent_id::AgentId, build_id::BuildId, job_id::JobId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Job entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier
    id: JobId,

    /// Build this job belongs to
    build_id: BuildId,

    /// Job name
    name: String,

    /// Stage name
    stage: String,

    /// Job status
    status: JobStatus,

    /// Agent executing the job
    agent_id: Option<AgentId>,

    /// Docker image to use
    image: Option<String>,

    /// Commands to execute
    commands: Vec<String>,

    /// Environment variables
    environment: HashMap<String, String>,

    /// Working directory
    working_directory: Option<String>,

    /// Job timeout in seconds
    timeout: u64,

    /// Retry count
    retry: u32,

    /// Current attempt number
    attempt: u32,

    /// Job dependencies (other job names)
    dependencies: Vec<String>,

    /// Job logs
    logs: String,

    /// Job start time
    started_at: Option<DateTime<Utc>>,

    /// Job completion time
    completed_at: Option<DateTime<Utc>>,

    /// Exit code
    exit_code: Option<i32>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    /// Job is waiting for dependencies
    Pending,
    /// Job is queued for execution
    Queued,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Success,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job was skipped
    Skipped,
}

impl Job {
    /// Create a new job
    pub fn new(build_id: BuildId, name: String, stage: String, commands: Vec<String>) -> Self {
        let now = Utc::now();

        Self {
            id: JobId::new(),
            build_id,
            name,
            stage,
            status: JobStatus::Pending,
            agent_id: None,
            image: None,
            commands,
            environment: HashMap::new(),
            working_directory: None,
            timeout: 3600, // Default 1 hour
            retry: 0,
            attempt: 0,
            dependencies: Vec::new(),
            logs: String::new(),
            started_at: None,
            completed_at: None,
            exit_code: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the job ID
    pub fn id(&self) -> &JobId {
        &self.id
    }

    /// Get the job name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the job status
    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    /// Get the job duration
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Set the Docker image
    pub fn set_image(&mut self, image: String) {
        self.image = Some(image);
        self.updated_at = Utc::now();
    }

    /// Add an environment variable
    pub fn add_environment(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dependency: String) {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
            self.updated_at = Utc::now();
        }
    }

    /// Mark job as queued
    pub fn queue(&mut self) -> crate::Result<()> {
        if self.status != JobStatus::Pending {
            return Err(crate::Error::build("Job is not in pending state"));
        }

        self.status = JobStatus::Queued;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Start the job
    pub fn start(&mut self, agent_id: AgentId) -> crate::Result<()> {
        if self.status != JobStatus::Queued {
            return Err(crate::Error::build("Job is not queued"));
        }

        self.status = JobStatus::Running;
        self.agent_id = Some(agent_id);
        self.started_at = Some(Utc::now());
        self.attempt += 1;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Complete the job successfully
    pub fn succeed(&mut self, exit_code: i32) -> crate::Result<()> {
        if self.status != JobStatus::Running {
            return Err(crate::Error::build("Job is not running"));
        }

        self.status = JobStatus::Success;
        self.exit_code = Some(exit_code);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Fail the job
    pub fn fail(&mut self, exit_code: i32) -> crate::Result<()> {
        if self.status != JobStatus::Running {
            return Err(crate::Error::build("Job is not running"));
        }

        self.status = JobStatus::Failed;
        self.exit_code = Some(exit_code);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Cancel the job
    pub fn cancel(&mut self) -> crate::Result<()> {
        if self.status == JobStatus::Success || self.status == JobStatus::Failed {
            return Err(crate::Error::build("Cannot cancel completed job"));
        }

        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Skip the job
    pub fn skip(&mut self) -> crate::Result<()> {
        if self.status != JobStatus::Pending && self.status != JobStatus::Queued {
            return Err(crate::Error::build("Cannot skip job in current state"));
        }

        self.status = JobStatus::Skipped;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Append logs
    pub fn append_logs(&mut self, logs: String) {
        self.logs.push_str(&logs);
        self.updated_at = Utc::now();
    }

    /// Check if the job can be retried
    pub fn can_retry(&self) -> bool {
        self.status == JobStatus::Failed && self.attempt < self.retry + 1
    }

    /// Retry the job
    pub fn retry_job(&mut self) -> crate::Result<()> {
        if !self.can_retry() {
            return Err(crate::Error::build("Job cannot be retried"));
        }

        self.status = JobStatus::Queued;
        self.started_at = None;
        self.completed_at = None;
        self.exit_code = None;
        self.logs.clear();
        self.updated_at = Utc::now();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_job() -> Job {
        Job::new(
            BuildId::new(),
            "test-job".to_string(),
            "build".to_string(),
            vec!["echo 'Hello'".to_string()],
        )
    }

    #[test]
    fn test_new_job() {
        let job = create_test_job();

        assert_eq!(job.name(), "test-job");
        assert_eq!(job.status(), &JobStatus::Pending);
        assert_eq!(job.commands.len(), 1);
    }

    #[test]
    fn test_job_lifecycle() {
        let mut job = create_test_job();

        // Queue the job
        assert!(job.queue().is_ok());
        assert_eq!(job.status(), &JobStatus::Queued);

        // Start the job
        let agent_id = AgentId::new();
        assert!(job.start(agent_id).is_ok());
        assert_eq!(job.status(), &JobStatus::Running);
        assert!(job.started_at.is_some());

        // Complete the job
        assert!(job.succeed(0).is_ok());
        assert_eq!(job.status(), &JobStatus::Success);
        assert!(job.completed_at.is_some());
        assert_eq!(job.exit_code, Some(0));
    }

    #[test]
    fn test_job_failure() {
        let mut job = create_test_job();

        job.queue().unwrap();
        job.start(AgentId::new()).unwrap();

        assert!(job.fail(1).is_ok());
        assert_eq!(job.status(), &JobStatus::Failed);
        assert_eq!(job.exit_code, Some(1));
    }

    #[test]
    fn test_job_cancellation() {
        let mut job = create_test_job();

        job.queue().unwrap();

        assert!(job.cancel().is_ok());
        assert_eq!(job.status(), &JobStatus::Cancelled);
    }

    #[test]
    fn test_job_retry() {
        let mut job = create_test_job();
        job.retry = 2;

        job.queue().unwrap();
        job.start(AgentId::new()).unwrap();
        job.fail(1).unwrap();

        // Can retry
        assert!(job.can_retry());
        assert!(job.retry_job().is_ok());
        assert_eq!(job.status(), &JobStatus::Queued);
    }

    #[test]
    fn test_job_logs() {
        let mut job = create_test_job();

        job.append_logs("Line 1\n".to_string());
        job.append_logs("Line 2\n".to_string());

        assert!(job.logs.contains("Line 1"));
        assert!(job.logs.contains("Line 2"));
    }

    #[test]
    fn test_job_environment() {
        let mut job = create_test_job();

        job.add_environment("NODE_ENV".to_string(), "production".to_string());
        job.add_environment("DEBUG".to_string(), "false".to_string());

        assert_eq!(job.environment.len(), 2);
        assert_eq!(
            job.environment.get("NODE_ENV"),
            Some(&"production".to_string())
        );
    }
}
