//! Stage entity - Represents a pipeline stage

use crate::domain::value_objects::{build_id::BuildId, stage_id::StageId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Stage entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    /// Unique identifier
    id: StageId,

    /// Build this stage belongs to
    build_id: BuildId,

    /// Stage name
    name: String,

    /// Stage status
    status: StageStatus,

    /// Jobs in this stage
    job_ids: Vec<String>,

    /// Stage start time
    started_at: Option<DateTime<Utc>>,

    /// Stage completion time
    completed_at: Option<DateTime<Utc>>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,
}

/// Stage status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StageStatus {
    /// Stage is waiting to start
    Pending,
    /// Stage is currently running
    Running,
    /// Stage completed successfully
    Success,
    /// Stage failed
    Failed,
    /// Stage was cancelled
    Cancelled,
    /// Stage was skipped
    Skipped,
}

impl Stage {
    /// Create a new stage
    pub fn new(build_id: BuildId, name: String) -> Self {
        let now = Utc::now();

        Self {
            id: StageId::new(),
            build_id,
            name,
            status: StageStatus::Pending,
            job_ids: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the stage ID
    pub fn id(&self) -> &StageId {
        &self.id
    }

    /// Get the stage name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the stage status
    pub fn status(&self) -> &StageStatus {
        &self.status
    }

    /// Get the stage duration
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Add a job to the stage
    pub fn add_job(&mut self, job_id: String) {
        if !self.job_ids.contains(&job_id) {
            self.job_ids.push(job_id);
            self.updated_at = Utc::now();
        }
    }

    /// Start the stage
    pub fn start(&mut self) -> crate::Result<()> {
        if self.status != StageStatus::Pending {
            return Err(crate::Error::build("Stage is not in pending state"));
        }

        self.status = StageStatus::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Complete the stage successfully
    pub fn succeed(&mut self) -> crate::Result<()> {
        if self.status != StageStatus::Running {
            return Err(crate::Error::build("Stage is not running"));
        }

        self.status = StageStatus::Success;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Fail the stage
    pub fn fail(&mut self) -> crate::Result<()> {
        if self.status != StageStatus::Running {
            return Err(crate::Error::build("Stage is not running"));
        }

        self.status = StageStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Cancel the stage
    pub fn cancel(&mut self) -> crate::Result<()> {
        if self.status == StageStatus::Success || self.status == StageStatus::Failed {
            return Err(crate::Error::build("Cannot cancel completed stage"));
        }

        self.status = StageStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Skip the stage
    pub fn skip(&mut self) -> crate::Result<()> {
        if self.status != StageStatus::Pending {
            return Err(crate::Error::build("Cannot skip stage in current state"));
        }

        self.status = StageStatus::Skipped;
        self.updated_at = Utc::now();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stage() {
        let build_id = BuildId::new();
        let stage = Stage::new(build_id, "build".to_string());

        assert_eq!(stage.name(), "build");
        assert_eq!(stage.status(), &StageStatus::Pending);
    }

    #[test]
    fn test_stage_lifecycle() {
        let build_id = BuildId::new();
        let mut stage = Stage::new(build_id, "test".to_string());

        // Start stage
        assert!(stage.start().is_ok());
        assert_eq!(stage.status(), &StageStatus::Running);
        assert!(stage.started_at.is_some());

        // Complete stage
        assert!(stage.succeed().is_ok());
        assert_eq!(stage.status(), &StageStatus::Success);
        assert!(stage.completed_at.is_some());
        assert!(stage.duration().is_some());
    }

    #[test]
    fn test_stage_failure() {
        let build_id = BuildId::new();
        let mut stage = Stage::new(build_id, "deploy".to_string());

        stage.start().unwrap();
        assert!(stage.fail().is_ok());

        assert_eq!(stage.status(), &StageStatus::Failed);
    }

    #[test]
    fn test_add_job() {
        let build_id = BuildId::new();
        let mut stage = Stage::new(build_id, "build".to_string());

        stage.add_job("job1".to_string());
        stage.add_job("job2".to_string());

        assert_eq!(stage.job_ids.len(), 2);

        // Duplicate job should not be added
        stage.add_job("job1".to_string());
        assert_eq!(stage.job_ids.len(), 2);
    }
}
