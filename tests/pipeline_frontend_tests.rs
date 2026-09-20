//! The pipeline definition front-ends all converge on one intermediate
//! representation.
//!
//! ```text
//! YAML  ─┐
//! TOML  ─┼─→  PipelineConfig (IR)  ─→  engine / TUI / web UI
//! Rust  ─┘         ^ serde-serializable
//!  SDK
//! ```
//!
//! These tests are the architecture's assertion: the same pipeline written
//! three ways produces the same `PipelineConfig`, and every front-end enforces
//! the same rules on the way in.

use ferrous_ci_cd::domain::pipeline_dsl::{job, pipeline, stage};
use ferrous_ci_cd::domain::value_objects::pipeline_config::{PipelineConfig, Trigger};
use ferrous_ci_cd::infrastructure::pipeline_source::{self, PipelineFormat};

const YAML_PIPELINE: &str = r#"
version: "1.0"

environment:
  CARGO_TERM_COLOR: always

triggers:
  - type: Push
    branches:
      - main

stages:
  - name: build
    jobs:
      - name: compile
        image: "rust:1.75"
        commands:
          - cargo build --all-targets

  - name: verify
    parallel: true
    jobs:
      - name: unit-test
        commands:
          - cargo test
        needs:
          - compile

      - name: clippy
        commands:
          - cargo clippy -- -D warnings
        needs:
          - compile
"#;

const TOML_PIPELINE: &str = r#"
version = "1.0"

[environment]
CARGO_TERM_COLOR = "always"

[[triggers]]
type = "Push"
branches = ["main"]

[[stages]]
name = "build"

[[stages.jobs]]
name = "compile"
image = "rust:1.75"
commands = ["cargo build --all-targets"]

[[stages]]
name = "verify"
parallel = true

[[stages.jobs]]
name = "unit-test"
commands = ["cargo test"]
needs = ["compile"]

[[stages.jobs]]
name = "clippy"
commands = ["cargo clippy -- -D warnings"]
needs = ["compile"]
"#;

/// The same pipeline, written with the typed Rust front-end.
fn dsl_pipeline() -> ferrous_ci_cd::Result<PipelineConfig> {
    let compile = job("compile")
        .image("rust:1.75")
        .run("cargo build --all-targets");
    let unit = job("unit-test").run("cargo test").needs(&compile);
    let clippy = job("clippy")
        .run("cargo clippy -- -D warnings")
        .needs(&compile);

    pipeline()
        .env("CARGO_TERM_COLOR", "always")
        .on(Trigger::push(["main"]))
        .stage(stage("build").job(compile))
        .stage(stage("verify").parallel().job(unit).job(clippy))
        .build()
}

#[test]
fn yaml_toml_and_the_rust_dsl_produce_the_same_ir() {
    let from_yaml = pipeline_source::from_str(YAML_PIPELINE, PipelineFormat::Yaml)
        .expect("the YAML pipeline should load");
    let from_toml = pipeline_source::from_str(TOML_PIPELINE, PipelineFormat::Toml)
        .expect("the TOML pipeline should load");
    let from_dsl = dsl_pipeline().expect("the Rust pipeline should build");

    assert_eq!(from_yaml, from_toml, "YAML and TOML disagree");
    assert_eq!(
        from_yaml, from_dsl,
        "the text front-ends and the DSL disagree"
    );
}

#[test]
fn the_dsl_records_dependencies_taken_by_handle() {
    let config = dsl_pipeline().expect("the Rust pipeline should build");

    let verify = &config.stages[1];
    assert_eq!(verify.name, "verify");
    assert!(verify.parallel);
    assert_eq!(verify.jobs[0].needs, ["compile"]);
    assert_eq!(verify.jobs[1].needs, ["compile"]);
}

#[test]
fn repeating_a_dependency_does_not_repeat_it_in_the_ir() {
    let compile = job("compile").run("cargo build");
    let test = job("test")
        .run("cargo test")
        .needs(&compile)
        .needs(&compile);

    assert_eq!(test.dependencies(), ["compile".to_string()]);
}

#[test]
fn the_dsl_still_enforces_the_rules_the_types_do_not_cover() {
    let empty_command_job = pipeline()
        .on(Trigger::Manual)
        .stage(stage("build").job(job("compile")))
        .build();

    assert!(
        empty_command_job.is_err(),
        "a job with neither commands nor an image should be rejected"
    );

    let no_trigger = pipeline()
        .stage(stage("build").job(job("compile").run("cargo build")))
        .build();

    assert!(no_trigger.is_err(), "a pipeline needs at least one trigger");
}

#[test]
fn a_dependency_on_an_unknown_job_is_rejected() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: compile
        commands: ["cargo build"]
      - name: test
        commands: ["cargo test"]
        needs: ["biuld"]
"#;

    let error = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect_err("a typo in `needs` should be rejected");
    let message = error.to_string();

    assert!(
        message.contains("unknown job 'biuld'"),
        "unhelpful message: {message}"
    );
}

#[test]
fn a_dependency_cycle_is_rejected() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: a
        commands: ["true"]
        needs: ["c"]
      - name: b
        commands: ["true"]
        needs: ["a"]
      - name: c
        commands: ["true"]
        needs: ["b"]
"#;

    let error = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect_err("a dependency cycle should be rejected");

    assert!(
        error.to_string().contains("Circular job dependency"),
        "unhelpful message: {error}"
    );
}

#[test]
fn a_job_may_not_depend_on_itself() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: compile
        commands: ["cargo build"]
        needs: ["compile"]
"#;

    let error = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect_err("a self-dependency should be rejected");

    assert!(
        error.to_string().contains("depends on itself"),
        "unhelpful message: {error}"
    );
}

#[test]
fn a_dependency_on_a_later_stage_is_rejected() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: compile
        commands: ["cargo build"]
        needs: ["deploy"]
  - name: release
    jobs:
      - name: deploy
        commands: ["./deploy.sh"]
"#;

    let error = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect_err("depending on a later stage should be rejected");

    assert!(
        error.to_string().contains("later stage"),
        "unhelpful message: {error}"
    );
}

#[test]
fn a_duplicated_job_name_is_rejected() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: compile
        commands: ["cargo build"]
  - name: verify
    jobs:
      - name: compile
        commands: ["cargo test"]
"#;

    let error = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect_err("a duplicated job name should be rejected");

    assert!(
        error.to_string().contains("Duplicate job name"),
        "unhelpful message: {error}"
    );
}

#[test]
fn a_misspelled_key_is_rejected_rather_than_ignored() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: compile
        comands: ["cargo build"]
"#;

    let error = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect_err("an unknown key should be rejected");

    assert!(
        error.to_string().contains("comands"),
        "unhelpful message: {error}"
    );
}

#[test]
fn optional_fields_may_be_omitted() {
    let yaml = r#"
triggers:
  - type: Manual
stages:
  - name: build
    jobs:
      - name: compile
        commands: ["cargo build"]
"#;

    let config = pipeline_source::from_str(yaml, PipelineFormat::Yaml)
        .expect("a minimal pipeline should load");

    assert_eq!(config.version, "1.0");
    assert!(config.environment.is_empty());
    assert!(!config.stages[0].parallel);
    assert!(config.stages[0].jobs[0].needs.is_empty());
}

#[test]
fn the_ir_survives_a_round_trip_through_every_writable_format() {
    let original = dsl_pipeline().expect("the Rust pipeline should build");

    for format in [PipelineFormat::Json, PipelineFormat::Yaml] {
        let text = pipeline_source::to_string(&original, format)
            .unwrap_or_else(|error| panic!("{format} should serialize: {error}"));
        let parsed = pipeline_source::from_str(&text, format)
            .unwrap_or_else(|error| panic!("{format} should parse back: {error}"));

        assert_eq!(original, parsed, "{format} round-trip lost information");
    }
}

#[test]
fn toml_is_read_only() {
    let config = dsl_pipeline().expect("the Rust pipeline should build");

    assert!(!PipelineFormat::Toml.is_writable());
    assert!(pipeline_source::to_string(&config, PipelineFormat::Toml).is_err());
}

#[test]
fn the_format_is_taken_from_the_file_extension() {
    use std::path::Path;

    assert_eq!(
        PipelineFormat::from_path(Path::new("ferrous.yml")).unwrap(),
        PipelineFormat::Yaml
    );
    assert_eq!(
        PipelineFormat::from_path(Path::new("dir/pipeline.TOML")).unwrap(),
        PipelineFormat::Toml
    );
    assert!(PipelineFormat::from_path(Path::new("pipeline.ini")).is_err());
    assert!(PipelineFormat::from_path(Path::new("pipeline")).is_err());
}
