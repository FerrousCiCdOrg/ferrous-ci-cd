//! Typed builder for a [`Stage`].

use std::marker::PhantomData;

use super::job::JobSpec;
use super::state::{BuilderState, Incomplete, Ready};
use crate::domain::value_objects::pipeline_config::{Job, Stage, WhenCondition};

/// A stage under construction.
///
/// The `S` parameter records whether the stage has any jobs yet. Only a
/// [`StageSpec<Ready>`] can be added to a pipeline, so a stage with no jobs
/// fails to compile rather than failing validation later.
#[derive(Debug, Clone)]
pub struct StageSpec<S: BuilderState = Incomplete> {
    name: String,
    jobs: Vec<Job>,
    parallel: bool,
    when: Option<WhenCondition>,
    state: PhantomData<S>,
}

/// Start building a stage with the given name.
#[must_use]
pub fn stage(name: impl Into<String>) -> StageSpec<Incomplete> {
    StageSpec::new(name)
}

impl StageSpec<Incomplete> {
    /// Start building a stage with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            jobs: Vec::new(),
            parallel: false,
            when: None,
            state: PhantomData,
        }
    }

    /// Add the stage's first job, making it usable in a pipeline.
    #[must_use]
    pub fn job(self, job: JobSpec) -> StageSpec<Ready> {
        let mut ready: StageSpec<Ready> = self.transition();
        ready.jobs.push(job.into_job());
        ready
    }
}

impl StageSpec<Ready> {
    /// Add another job to the stage.
    #[must_use]
    pub fn job(mut self, job: JobSpec) -> Self {
        self.jobs.push(job.into_job());
        self
    }

    /// Lower the builder into the intermediate representation.
    pub(super) fn into_stage(self) -> Stage {
        Stage {
            name: self.name,
            jobs: self.jobs,
            parallel: self.parallel,
            when: self.when,
        }
    }
}

impl<S: BuilderState> StageSpec<S> {
    /// The stage's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Run the stage's jobs concurrently instead of one after another.
    #[must_use]
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Only run this stage when the condition holds.
    #[must_use]
    pub fn when(mut self, condition: WhenCondition) -> Self {
        self.when = Some(condition);
        self
    }

    /// Move the builder to another state, keeping its contents.
    fn transition<T: BuilderState>(self) -> StageSpec<T> {
        StageSpec {
            name: self.name,
            jobs: self.jobs,
            parallel: self.parallel,
            when: self.when,
            state: PhantomData,
        }
    }
}
