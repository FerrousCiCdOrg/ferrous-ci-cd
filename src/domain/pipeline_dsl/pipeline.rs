//! Typed builder for a whole [`PipelineConfig`].

use std::collections::HashMap;
use std::marker::PhantomData;

use super::stage::StageSpec;
use super::state::{BuilderState, Incomplete, Ready};
use crate::domain::value_objects::pipeline_config::{
    NotificationConfig, PipelineConfig, Stage, Trigger,
};

/// The pipeline definition format version this builder emits.
const DEFAULT_VERSION: &str = "1.0";

/// A pipeline under construction.
///
/// The `S` parameter records whether the pipeline has any stages yet. Only a
/// [`PipelineSpec<Ready>`] can be built, so an empty pipeline fails to compile.
#[derive(Debug, Clone)]
pub struct PipelineSpec<S: BuilderState = Incomplete> {
    version: String,
    stages: Vec<Stage>,
    triggers: Vec<Trigger>,
    environment: HashMap<String, String>,
    notifications: Option<NotificationConfig>,
    state: PhantomData<S>,
}

/// Start building a pipeline.
#[must_use]
pub fn pipeline() -> PipelineSpec<Incomplete> {
    PipelineSpec::new()
}

impl PipelineSpec<Incomplete> {
    /// Start building a pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: DEFAULT_VERSION.to_owned(),
            stages: Vec::new(),
            triggers: Vec::new(),
            environment: HashMap::new(),
            notifications: None,
            state: PhantomData,
        }
    }

    /// Add the pipeline's first stage, making it buildable.
    #[must_use]
    pub fn stage(self, stage: StageSpec<Ready>) -> PipelineSpec<Ready> {
        let mut ready: PipelineSpec<Ready> = self.transition();
        ready.stages.push(stage.into_stage());
        ready
    }
}

impl Default for PipelineSpec<Incomplete> {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineSpec<Ready> {
    /// Add another stage. Stages run in the order they are added.
    #[must_use]
    pub fn stage(mut self, stage: StageSpec<Ready>) -> Self {
        self.stages.push(stage.into_stage());
        self
    }

    /// Lower the builder into the intermediate representation and validate it.
    ///
    /// The type system has already ruled out empty stages, empty pipelines and
    /// unresolvable `needs` references. This runs the same
    /// [`PipelineConfig::validate`] every other front-end runs, so the
    /// remaining rules — a job needs a command or an image, the pipeline needs
    /// a trigger, names must be unique — are enforced identically no matter how
    /// the pipeline was written.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Validation`] if the resulting configuration does
    /// not pass [`PipelineConfig::validate`].
    pub fn build(self) -> crate::Result<PipelineConfig> {
        let config = self.into_config();
        config.validate()?;
        Ok(config)
    }

    /// Lower the builder into the intermediate representation without validating.
    ///
    /// Useful for tests that want to assert on an invalid configuration.
    #[must_use]
    pub fn into_config(self) -> PipelineConfig {
        PipelineConfig {
            version: self.version,
            stages: self.stages,
            triggers: self.triggers,
            environment: self.environment,
            notifications: self.notifications,
        }
    }
}

impl<S: BuilderState> PipelineSpec<S> {
    /// Override the pipeline definition format version.
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add a trigger that starts this pipeline.
    #[must_use]
    pub fn on(mut self, trigger: Trigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Set an environment variable shared by every job.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Configure where build results are announced.
    #[must_use]
    pub fn notifications(mut self, notifications: NotificationConfig) -> Self {
        self.notifications = Some(notifications);
        self
    }

    /// Move the builder to another state, keeping its contents.
    fn transition<T: BuilderState>(self) -> PipelineSpec<T> {
        PipelineSpec {
            version: self.version,
            stages: self.stages,
            triggers: self.triggers,
            environment: self.environment,
            notifications: self.notifications,
            state: PhantomData,
        }
    }
}
