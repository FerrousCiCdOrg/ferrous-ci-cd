# Pipeline Definition Specification

## Overview

A pipeline definition describes what a build does. Ferrous CI/CD accepts several
notations for writing one, but nothing in the system executes a notation
directly: every definition is first lowered into a single intermediate
representation, and the execution engine, the API and the terminal UI only ever
see that.

```
YAML  ─┐
TOML  ─┼─→  PipelineConfig (IR)  ─→  engine / TUI / web UI
Rust  ─┘         ^ serde-serializable
 SDK
```

The intermediate representation is the `PipelineConfig` value object
(`src/domain/value_objects/pipeline_config.rs`). It is plain, serde-serializable
data with no behaviour of its own beyond validation.

## Why an intermediate representation

Writing pipelines in typed Rust catches mistakes a text format cannot. Writing
them in YAML costs nothing to parse and requires no toolchain. Separating the
notation from the representation keeps both.

| | Text formats | Rust SDK |
|---|---|---|
| Definition errors caught | at load time | at compile time for the definition's shape |
| Toolchain required to author | none | a Rust toolchain |
| Cost to read a definition | milliseconds | a `cargo build` |
| Server executes repository-supplied code | no | no, if the SDK's output is committed |

Reading a definition must stay cheap and safe, because a server does it on every
webhook, for every repository, including forks that opened a pull request.
Lowering the Rust SDK's output to the IR and committing the result keeps that
property: the SDK runs on the author's machine, and the server reads static data
either way. This is the same arrangement as Pulumi's or AWS CDK's `synth` step.

The type system covers the definition's *shape*. It does not cover what the
commands inside a job do; `commands` is a list of shell strings, and no front-end
can make a wrong image tag or a flaky test into a compile error.

## Intermediate representation

```rust
pub struct PipelineConfig {
    pub version: String,
    pub stages: Vec<Stage>,
    pub triggers: Vec<Trigger>,
    pub environment: HashMap<String, String>,
    pub notifications: Option<NotificationConfig>,
}
```

Stages run in the order they are declared. A stage's jobs run in order unless the
stage sets `parallel`, and a job may declare `needs` to wait for other jobs.

Every field except a stage's and a job's `name` may be omitted; omitted fields
take their default. Unknown fields are rejected rather than ignored, so a
misspelled key is reported instead of silently doing nothing.

### Job names

Job names form one namespace covering the whole pipeline, because `needs` refers
to jobs by name and a job may depend on a job declared in an earlier stage.

## Validation

`PipelineConfig::validate` is the single set of rules. Every front-end runs it
before handing a configuration to anything else, so a definition is accepted or
rejected identically no matter how it was written.

Structural rules:

- the version is not empty
- the pipeline has at least one stage and at least one trigger
- every stage has a name and at least one job
- every job has a name, and either a command or an image

Dependency graph rules, checked across all stages at once:

- stage names are unique
- job names are unique pipeline-wide
- every `needs` entry names a job that exists
- no job depends on itself
- no job depends on a job in a later stage, which could never have run
- the `needs` relation contains no cycle

## Front-ends

### Text formats

`infrastructure::pipeline_source` parses YAML, TOML and JSON, dispatching on the
file extension. Loading validates.

```rust
let config = pipeline_source::from_path("ferrous.yaml")?;
```

YAML and JSON can also be written back out. TOML is read-only: the IR nests
scalars after arrays of tables, which TOML cannot represent in the order a
serializer emits them, so a TOML round-trip would not be faithful.

### Rust SDK

`domain::pipeline_dsl` builds the same IR from typed Rust.

```rust
let compile = job("compile").image("rust:1.75").run("cargo build");
let unit = job("unit-test").run("cargo test").needs(&compile);

let config = pipeline()
    .on(Trigger::push(["main"]))
    .stage(stage("build").job(compile))
    .stage(stage("verify").job(unit))
    .build()?;
```

Three mistakes become compile errors rather than validation failures:

- `needs` takes `&JobSpec`, not a name, so a dependency can only be written by
  naming a binding that exists — a misspelled dependency does not compile
- a stage with no jobs cannot be added to a pipeline
- a pipeline with no stages cannot be built

Everything else stays in `PipelineConfig::validate`, shared with the text
front-ends. `build()` runs it.

To emit a definition for a server to read, serialize the result:

```sh
cargo run --example pipeline_sdk > .ferrous/pipeline.json
```

### CLI

```sh
ferrous-ci-cd pipeline validate examples/pipeline.yaml
ferrous-ci-cd pipeline emit examples/pipeline.toml --format yaml
```

`emit` prints the canonical IR, so it also converts between notations.

## Adding a front-end

A new notation implements exactly one thing: text (or anything else) in,
`PipelineConfig` out. It must call `PipelineConfig::validate` before returning,
and it must not extend the IR with anything the other front-ends cannot express.
`tests/pipeline_frontend_tests.rs` asserts that the existing front-ends agree;
a new one belongs in that comparison.

## Version Information

- **Document Version**: 1.0.0
- **Last Updated**: 2026-09-08
- **Target System Version**: 0.1.0
