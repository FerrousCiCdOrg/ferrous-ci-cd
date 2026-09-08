//! Typed builder for a single [`Job`].

use std::collections::HashMap;

use crate::domain::value_objects::pipeline_config::{
    ArtifactConfig, CacheConfig, Job, WhenCondition,
};

/// A job under construction, and the handle that other jobs depend on.
///
/// `JobSpec` is deliberately both the builder and the reference. [`JobSpec::needs`]
/// takes `&JobSpec` rather than a name, so a dependency can only be expressed by
/// naming a binding that already exists. A misspelled dependency becomes a
/// compile error instead of a pipeline that fails once it is already running:
///
/// ```
/// use ferrous_ci_cd::domain::pipeline_dsl::job;
///
/// let compile = job("compile").image("rust:1.75").run("cargo build");
/// let unit = job("unit-test").run("cargo test").needs(&compile);
/// // `job("unit-test").needs(&complie)` would not compile.
/// # assert_eq!(unit.dependencies().len(), 1);
/// # assert_eq!(unit.dependencies()[0], "compile");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct JobSpec {
    inner: Job,
}

/// Start building a job with the given name.
#[must_use]
pub fn job(name: impl Into<String>) -> JobSpec {
    JobSpec::new(name)
}

impl JobSpec {
    /// Start building a job with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Job::new(name.into()),
        }
    }

    /// The job's name, as other jobs will refer to it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// The names this job depends on, in declaration order.
    #[must_use]
    pub fn dependencies(&self) -> &[String] {
        &self.inner.needs
    }

    /// Run this job in the given container image.
    #[must_use]
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.inner.image = Some(image.into());
        self
    }

    /// Append a command to the job's script.
    #[must_use]
    pub fn run(mut self, command: impl Into<String>) -> Self {
        self.inner.commands.push(command.into());
        self
    }

    /// Set an environment variable scoped to this job.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.inner.environment.insert(key.into(), value.into());
        self
    }

    /// Run the job's commands from this directory.
    #[must_use]
    pub fn working_directory(mut self, directory: impl Into<String>) -> Self {
        self.inner.working_directory = Some(directory.into());
        self
    }

    /// Abort the job after this many seconds.
    #[must_use]
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.inner.timeout = Some(seconds);
        self
    }

    /// Retry the job this many times before failing the build.
    #[must_use]
    pub fn retry(mut self, attempts: u32) -> Self {
        self.inner.retry = Some(attempts);
        self
    }

    /// Collect artifacts produced by this job.
    #[must_use]
    pub fn artifacts(mut self, artifacts: ArtifactConfig) -> Self {
        self.inner.artifacts = Some(artifacts);
        self
    }

    /// Cache paths across runs of this job.
    #[must_use]
    pub fn cache(mut self, cache: CacheConfig) -> Self {
        self.inner.cache = Some(cache);
        self
    }

    /// Only run this job when the condition holds.
    #[must_use]
    pub fn when(mut self, condition: WhenCondition) -> Self {
        self.inner.when = Some(condition);
        self
    }

    /// Wait for another job before running this one.
    ///
    /// Taking the dependency by reference is the point: there is no way to name
    /// a job that was never declared. Repeating the same dependency is a no-op.
    #[must_use]
    pub fn needs(mut self, dependency: &Self) -> Self {
        let name = dependency.name();
        if !self.inner.needs.iter().any(|existing| existing == name) {
            self.inner.needs.push(name.to_owned());
        }
        self
    }

    /// Set several environment variables at once.
    #[must_use]
    pub fn envs<K, V>(mut self, variables: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        let entries: HashMap<String, String> = variables
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        self.inner.environment.extend(entries);
        self
    }

    /// Lower the builder into the intermediate representation.
    pub(super) fn into_job(self) -> Job {
        self.inner
    }
}
