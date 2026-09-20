//! Typed Rust front-end for authoring pipelines.
//!
//! # Where this sits
//!
//! [`PipelineConfig`] is the intermediate representation every part of the
//! system agrees on. Nothing executes a pipeline definition directly; a
//! definition is first lowered into the IR, and the engine, the API and the
//! terminal UI only ever see the IR:
//!
//! ```text
//! YAML  ─┐
//! TOML  ─┼─→  PipelineConfig (IR)  ─→  engine / TUI / web UI
//! Rust  ─┘         ^ serde-serializable
//!  SDK
//! ```
//!
//! This module is the third front-end. It produces exactly the same IR as
//! [`crate::infrastructure::pipeline_source`] produces from text, and runs the
//! same [`PipelineConfig::validate`] before handing it back.
//!
//! # What the types buy you
//!
//! The IR stores dependencies as `Vec<String>`, because that is what a YAML file
//! can express. The builder never asks you for that string:
//! [`JobSpec::needs`] takes `&JobSpec`, so a dependency can only be written by
//! naming a binding that exists. Two further mistakes are ruled out by
//! [`StageSpec`] and [`PipelineSpec`] carrying a type-level marker: a stage with
//! no jobs and a pipeline with no stages do not compile.
//!
//! Everything else — a job needing a command or an image, a pipeline needing a
//! trigger, names being unique — stays a run-time check in
//! [`PipelineConfig::validate`], shared with the text front-ends. The type
//! system covers the definition's shape, not what the commands inside it do.
//!
//! # Example
//!
//! ```
//! use ferrous_ci_cd::domain::pipeline_dsl::{job, pipeline, stage};
//! use ferrous_ci_cd::domain::value_objects::pipeline_config::Trigger;
//!
//! let compile = job("compile").image("rust:1.75").run("cargo build --all-targets");
//! let unit = job("unit-test").run("cargo test").needs(&compile);
//! let clippy = job("clippy").run("cargo clippy -- -D warnings").needs(&compile);
//!
//! let config = pipeline()
//!     .env("CARGO_TERM_COLOR", "always")
//!     .on(Trigger::push(["main"]))
//!     .stage(stage("build").job(compile))
//!     .stage(stage("verify").parallel().job(unit).job(clippy))
//!     .build()?;
//!
//! assert_eq!(config.stages.len(), 2);
//! assert_eq!(config.stages[1].jobs[0].needs, ["compile"]);
//! # Ok::<(), ferrous_ci_cd::Error>(())
//! ```
//!
//! To keep the server away from user-supplied Rust, emit the IR and commit it:
//! serialize the returned [`PipelineConfig`] with
//! [`crate::infrastructure::pipeline_source::to_string`] and let the server read
//! the resulting static file like any other pipeline definition.

pub mod job;
pub mod pipeline;
pub mod stage;
pub mod state;

pub use job::{job, JobSpec};
pub use pipeline::{pipeline, PipelineSpec};
pub use stage::{stage, StageSpec};
pub use state::{BuilderState, Incomplete, Ready};

// Brought into scope so the intra-doc links above resolve.
#[allow(unused_imports)]
use crate::domain::value_objects::pipeline_config::PipelineConfig;
