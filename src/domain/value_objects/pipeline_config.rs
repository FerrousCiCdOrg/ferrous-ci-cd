//! Pipeline Configuration value object

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// The pipeline definition format version assumed when a definition omits one
pub const DEFAULT_VERSION: &str = "1.0";

/// serde default for [`PipelineConfig::version`]
fn default_version() -> String {
    DEFAULT_VERSION.to_string()
}

/// Pipeline Configuration value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    /// Pipeline version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Pipeline stages
    #[serde(default)]
    pub stages: Vec<Stage>,
    
    /// Pipeline triggers
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    
    /// Global environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
    
    /// Notification settings
    #[serde(default)]
    pub notifications: Option<NotificationConfig>,
}

/// Pipeline stage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// Stage name
    pub name: String,
    
    /// Jobs in this stage
    #[serde(default)]
    pub jobs: Vec<Job>,
    
    /// Whether jobs in this stage can run in parallel
    #[serde(default)]
    pub parallel: bool,
    
    /// Conditions for running this stage
    #[serde(default)]
    pub when: Option<WhenCondition>,
}

/// Job configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    /// Job name
    pub name: String,
    
    /// Docker image to use
    #[serde(default)]
    pub image: Option<String>,
    
    /// Commands to execute
    #[serde(default)]
    pub commands: Vec<String>,
    
    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
    
    /// Working directory
    #[serde(default)]
    pub working_directory: Option<String>,
    
    /// Job timeout in seconds
    #[serde(default)]
    pub timeout: Option<u64>,
    
    /// Number of retry attempts
    #[serde(default)]
    pub retry: Option<u32>,
    
    /// Artifacts to save
    #[serde(default)]
    pub artifacts: Option<ArtifactConfig>,
    
    /// Cache configuration
    #[serde(default)]
    pub cache: Option<CacheConfig>,
    
    /// Dependencies on other jobs, referenced by job name
    ///
    /// Job names form a single pipeline-wide namespace, so a job may depend on
    /// a job declared in an earlier stage. [`PipelineConfig::validate`] rejects
    /// references that cannot be resolved.
    #[serde(default)]
    pub needs: Vec<String>,
    
    /// Conditions for running this job
    #[serde(default)]
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
#[serde(deny_unknown_fields)]
pub struct WhenCondition {
    /// Branch condition
    #[serde(default)]
    pub branch: Option<String>,
    
    /// Event type condition
    #[serde(default)]
    pub event: Option<String>,
    
    /// Status condition
    #[serde(default)]
    pub status: Option<String>,
}

/// Artifact configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactConfig {
    /// Paths to include
    pub paths: Vec<String>,
    
    /// Paths to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
    
    /// Artifact name
    #[serde(default)]
    pub name: Option<String>,
    
    /// Expiration time in days
    #[serde(default)]
    pub expire_in: Option<u32>,
}

/// Cache configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheConfig {
    /// Cache key
    pub key: String,
    
    /// Paths to cache
    pub paths: Vec<String>,
    
    /// Cache policy (pull, push, pull-push)
    #[serde(default)]
    pub policy: Option<String>,
}

/// Notification configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationConfig {
    /// Email notifications
    #[serde(default)]
    pub email: Option<Vec<String>>,
    
    /// Slack notifications
    #[serde(default)]
    pub slack: Option<SlackNotification>,
    
    /// Webhook notifications
    #[serde(default)]
    pub webhooks: Vec<String>,
}

/// Slack notification configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlackNotification {
    /// Slack channel
    pub channel: String,
    
    /// Notify on success
    #[serde(default)]
    pub on_success: bool,
    
    /// Notify on failure
    #[serde(default)]
    pub on_failure: bool,
}

impl PipelineConfig {
    /// Create a new pipeline configuration
    pub fn new(stages: Vec<Stage>, triggers: Vec<Trigger>) -> Self {
        Self {
            version: DEFAULT_VERSION.to_string(),
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
            return Err(crate::Error::validation("Pipeline must have at least one stage"));
        }
        
        for stage in &self.stages {
            stage.validate()?;
        }
        
        // Validate triggers
        if self.triggers.is_empty() {
            return Err(crate::Error::validation("Pipeline must have at least one trigger"));
        }
        
        // Validate the dependency graph spanning every stage
        self.validate_job_graph()?;
        
        Ok(())
    }
    
    /// Validate the job dependency graph across the whole pipeline
    ///
    /// Individual [`Stage::validate`] and [`Job::validate`] calls only see one
    /// node at a time. This pass is what catches the mistakes that actually
    /// break pipelines written by hand: a `needs` entry with a typo in it, two
    /// jobs sharing a name, a dependency on a stage that has not run yet, and
    /// dependency cycles.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Validation`] if a stage or job name is
    /// duplicated, if a `needs` entry names a job that does not exist, if a job
    /// depends on itself or on a job in a later stage, or if the dependencies
    /// form a cycle.
    fn validate_job_graph(&self) -> crate::Result<()> {
        let mut stage_names: HashSet<&str> = HashSet::new();
        let mut job_stage: HashMap<&str, usize> = HashMap::new();
        
        for (index, stage) in self.stages.iter().enumerate() {
            if !stage_names.insert(stage.name.as_str()) {
                return Err(crate::Error::validation(format!(
                    "Duplicate stage name: '{}'",
                    stage.name
                )));
            }
            
            for job in &stage.jobs {
                if job_stage.insert(job.name.as_str(), index).is_some() {
                    return Err(crate::Error::validation(format!(
                        "Duplicate job name: '{}'. Job names must be unique across the \
                         whole pipeline because `needs` references them by name",
                        job.name
                    )));
                }
            }
        }
        
        for (index, stage) in self.stages.iter().enumerate() {
            for job in &stage.jobs {
                for need in &job.needs {
                    if need == &job.name {
                        return Err(crate::Error::validation(format!(
                            "Job '{}' depends on itself",
                            job.name
                        )));
                    }
                    
                    let Some(&dependency_stage) = job_stage.get(need.as_str()) else {
                        return Err(crate::Error::validation(format!(
                            "Job '{}' depends on unknown job '{}'",
                            job.name, need
                        )));
                    };
                    
                    if dependency_stage > index {
                        return Err(crate::Error::validation(format!(
                            "Job '{}' in stage '{}' depends on job '{}' in the later stage \
                             '{}', which can never have run",
                            job.name, stage.name, need, self.stages[dependency_stage].name
                        )));
                    }
                }
            }
        }
        
        self.detect_dependency_cycle()
    }
    
    /// Walk the dependency graph depth-first and report the first cycle found
    fn detect_dependency_cycle(&self) -> crate::Result<()> {
        let mut dependencies: HashMap<&str, &Vec<String>> = HashMap::new();
        for stage in &self.stages {
            for job in &stage.jobs {
                dependencies.insert(job.name.as_str(), &job.needs);
            }
        }
        
        let mut visits: HashMap<&str, VisitState> = HashMap::new();
        let mut path: Vec<&str> = Vec::new();
        
        for stage in &self.stages {
            for job in &stage.jobs {
                visit_job(job.name.as_str(), &dependencies, &mut visits, &mut path)?;
            }
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

impl Trigger {
    /// Trigger on pushes to any of the given branches
    #[must_use]
    pub fn push<I, S>(branches: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Trigger::Push {
            branches: collect_strings(branches),
        }
    }
    
    /// Trigger on pull requests targeting any of the given branches
    #[must_use]
    pub fn pull_request<I, S>(branches: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Trigger::PullRequest {
            branches: collect_strings(branches),
        }
    }
    
    /// Trigger on the given cron schedule
    #[must_use]
    pub fn schedule(cron: impl Into<String>) -> Self {
        Trigger::Schedule { cron: cron.into() }
    }
    
    /// Trigger on tags matching any of the given patterns
    #[must_use]
    pub fn tag<I, S>(patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Trigger::Tag {
            patterns: collect_strings(patterns),
        }
    }
}

/// Collect anything string-like into owned strings
fn collect_strings<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    values.into_iter().map(Into::into).collect()
}

/// Depth-first search state for [`PipelineConfig::detect_dependency_cycle`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    /// The node is on the current search path
    InProgress,
    /// The node and everything it reaches are known to be cycle-free
    Done,
}

/// Visit one job and everything it depends on
fn visit_job<'a>(
    name: &'a str,
    dependencies: &HashMap<&'a str, &'a Vec<String>>,
    visits: &mut HashMap<&'a str, VisitState>,
    path: &mut Vec<&'a str>,
) -> crate::Result<()> {
    match visits.get(name) {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::InProgress) => {
            let start = path.iter().position(|node| *node == name).unwrap_or(0);
            let mut cycle: Vec<&str> = path[start..].to_vec();
            cycle.push(name);
            return Err(crate::Error::validation(format!(
                "Circular job dependency detected: {}",
                cycle.join(" -> ")
            )));
        }
        None => {}
    }
    
    visits.insert(name, VisitState::InProgress);
    path.push(name);
    
    if let Some(needs) = dependencies.get(name) {
        for need in needs.iter() {
            visit_job(need.as_str(), dependencies, visits, path)?;
        }
    }
    
    path.pop();
    visits.insert(name, VisitState::Done);
    
    Ok(())
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
            version: DEFAULT_VERSION.to_string(),
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

