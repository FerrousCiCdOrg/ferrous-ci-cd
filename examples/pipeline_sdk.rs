//! Author a pipeline in typed Rust and emit the intermediate representation.
//!
//! This is the front-end that gets you compile-time checking of the definition's
//! shape. The output is an ordinary static file:
//!
//! ```sh
//! cargo run --example pipeline_sdk > .ferrous/pipeline.json
//! ferrous-ci-cd pipeline validate .ferrous/pipeline.json
//! ```
//!
//! Committing that file is the point. The server reads plain data the way it
//! reads a YAML definition, so nothing has to build or run this program on a
//! machine that received it from a repository.

use ferrous_ci_cd::domain::pipeline_dsl::{job, pipeline, stage, JobSpec};
use ferrous_ci_cd::domain::value_objects::pipeline_config::Trigger;
use ferrous_ci_cd::infrastructure::pipeline_source::{self, PipelineFormat};
use ferrous_ci_cd::Result;

/// The checks every crate in the workspace runs, sharing one compiled artifact.
///
/// A helper like this is the other reason to define pipelines in Rust: it is a
/// function, so there is no template language and no anchor syntax involved.
fn check(name: &str, command: &str, after: &JobSpec) -> JobSpec {
    job(name)
        .image("rust:1.75")
        .run(command)
        .needs(after)
        .timeout(600)
}

fn main() -> Result<()> {
    let compile = job("compile")
        .image("rust:1.75")
        .run("cargo build --all-targets --locked")
        .timeout(1_800);

    let unit = check("unit-test", "cargo test --all-targets", &compile);
    let clippy = check(
        "clippy",
        "cargo clippy --all-targets -- -D warnings",
        &compile,
    );
    let format = check("format", "cargo fmt --all -- --check", &compile);

    // `needs(&unit)` cannot name a job that was never declared: the dependency
    // is the binding itself, so a misspelling is a compile error.
    let release = job("release")
        .image("rust:1.75")
        .run("cargo build --release --locked")
        .needs(&unit)
        .needs(&clippy)
        .needs(&format);

    let config = pipeline()
        .env("CARGO_TERM_COLOR", "always")
        .env("RUST_BACKTRACE", "1")
        .on(Trigger::push(["main"]))
        .on(Trigger::pull_request(["main"]))
        .stage(stage("build").job(compile))
        .stage(stage("verify").parallel().job(unit).job(clippy).job(format))
        .stage(stage("release").job(release))
        .build()?;

    println!(
        "{}",
        pipeline_source::to_string(&config, PipelineFormat::Json)?
    );

    Ok(())
}
