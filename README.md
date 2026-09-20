# Ferrous CI/CD 🦀🚀

[![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/ferrous-ci-cd/ci.yml?branch=main)](https://github.com/yourusername/ferrous-ci-cd/actions)
[![Crates.io](https://img.shields.io/crates/v/ferrous-ci-cd.svg)](https://crates.io/crates/ferrous-ci-cd)
[![Documentation](https://docs.rs/ferrous-ci-cd/badge.svg)](https://docs.rs/ferrous-ci-cd)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![codecov](https://codecov.io/gh/yourusername/ferrous-ci-cd/branch/main/graph/badge.svg)](https://codecov.io/gh/yourusername/ferrous-ci-cd)

A modern, high-performance CI/CD system built with Rust, inspired by Jenkins but designed for the cloud-native era.

## 🌟 Features

- **🚀 High Performance**: Built with Rust for maximum performance and reliability
- **📦 Container-Native**: First-class Docker and Kubernetes support
- **🔄 Pipeline as Code**: Write pipelines in YAML, TOML, or typed Rust — every notation lowers to one validated representation
- **🎯 Domain-Driven Design**: Clean architecture following DDD principles
- **🔌 Extensible**: Plugin system for custom integrations
- **🔐 Secure**: Built-in authentication and authorization
- **📊 Observability**: Comprehensive metrics and tracing
- **🌍 Distributed**: Support for distributed builds across multiple agents
- **💾 Multiple Storage Backends**: PostgreSQL, SQLite support
- **🔔 Notifications**: Webhook, email, and Slack notifications

## 📋 Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [Installation](#-installation)
- [Configuration](#-configuration)
- [Usage](#-usage)
- [Architecture](#-architecture)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Docker (optional, for containerized builds)
- PostgreSQL or SQLite

### Installation from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/ferrous-ci-cd.git
cd ferrous-ci-cd

# Build the project
cargo build --release

# Run tests
cargo test

# Start the server
cargo run --release
```

### Docker Installation

```bash
# Using Docker
docker run -d \
  -p 8080:8080 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v ferrous-data:/data \
  ferrous/ferrous-ci-cd:latest

# Using Docker Compose
docker-compose up -d
```

## ⚙️ Configuration

Create a `config.yaml` file:

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

database:
  type: "postgres" # or "sqlite"
  url: "postgresql://user:password@localhost/ferrous_ci"
  max_connections: 10

storage:
  artifacts_path: "./artifacts"
  workspace_path: "./workspace"

security:
  jwt_secret: "your-secret-key"
  session_timeout: 3600

git:
  ssh_key_path: "~/.ssh/id_rsa"
  
agents:
  max_concurrent_builds: 5
  heartbeat_interval: 30
```

## 📖 Usage

### Define a Pipeline

A pipeline definition is lowered into one intermediate representation before
anything acts on it. The notation you write it in is a front-end:

```
YAML  ─┐
TOML  ─┼─→  PipelineConfig (IR)  ─→  engine / TUI / web UI
Rust  ─┘         ^ serde-serializable
 SDK
```

Whichever you pick, the same validation runs: `needs` must name a job that
exists, job names must be unique, dependencies may not form a cycle or point at
a stage that has not run yet, and a misspelled key is rejected rather than
ignored.

#### YAML

Create a `.ferrous-ci.yaml` file in your repository:

```yaml
version: "1.0"

environment:
  CARGO_TERM_COLOR: always

triggers:
  - type: Push
    branches: ["main", "develop"]
  - type: PullRequest
    branches: ["main"]
  - type: Schedule
    cron: "0 0 * * *"

stages:
  - name: build
    jobs:
      - name: compile
        image: rust:1.75
        commands:
          - cargo build --release --locked
        artifacts:
          paths:
            - "target/release/*"

  - name: verify
    parallel: true
    jobs:
      - name: unit-test
        image: rust:1.75
        commands:
          - cargo test --all-targets
        needs: ["compile"]

      - name: clippy
        image: rust:1.75
        commands:
          - cargo clippy --all-targets -- -D warnings
        needs: ["compile"]

  - name: deploy
    when:
      branch: main
      event: push
    jobs:
      - name: deploy-production
        commands:
          - ./scripts/deploy.sh production
        needs: ["unit-test", "clippy"]
```

TOML works the same way; see [`examples/pipeline.toml`](examples/pipeline.toml).

#### Typed Rust

The IR stores dependencies as strings, because that is what a text file can
express. The Rust front-end never asks you for one — `needs` takes the job
itself, so a misspelled dependency is a compile error rather than a build that
fails halfway through:

```rust
use ferrous_ci_cd::domain::pipeline_dsl::{job, pipeline, stage};
use ferrous_ci_cd::domain::value_objects::pipeline_config::Trigger;

let compile = job("compile").image("rust:1.75").run("cargo build --locked");
let unit = job("unit-test").run("cargo test").needs(&compile);
let clippy = job("clippy").run("cargo clippy -- -D warnings").needs(&compile);

let config = pipeline()
    .env("CARGO_TERM_COLOR", "always")
    .on(Trigger::push(["main"]))
    .stage(stage("build").job(compile))
    .stage(stage("verify").parallel().job(unit).job(clippy))
    .build()?;
```

A stage with no jobs and a pipeline with no stages do not compile either. The
rules the types cannot express — a job needs a command or an image, names must
be unique, the dependency graph must hold together — run in `build()`, the same
check the text front-ends run.

The server never builds or runs this. Emit the representation and commit it:

```bash
cargo run --example pipeline_sdk > .ferrous/pipeline.json
ferrous-ci-cd pipeline validate .ferrous/pipeline.json
```

#### Checking a definition

```bash
# Parse, validate, and report what the pipeline contains
ferrous-ci-cd pipeline validate examples/pipeline.yaml

# Print the canonical representation (also converts between notations)
ferrous-ci-cd pipeline emit examples/pipeline.toml --format yaml
```

See [the pipeline definition specification](spec/07-pipeline-definition.md) for
the full schema and validation rules.

### CLI Usage

```bash
# Login to the server
ferrous-ci login --server http://localhost:8080

# Create a new project
ferrous-ci project create --name my-app --repo https://github.com/user/repo

# Trigger a build
ferrous-ci build trigger --project my-app --branch main

# View build logs
ferrous-ci build logs --project my-app --build-id 123

# List running builds
ferrous-ci build list --status running

# Manage agents
ferrous-ci agent list
ferrous-ci agent register --name agent-01 --labels "os=linux,arch=x86_64"
```

### API Usage

```bash
# Trigger a build via API
curl -X POST http://localhost:8080/api/v1/projects/my-app/builds \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"branch": "main", "commit": "abc123"}'

# Get build status
curl http://localhost:8080/api/v1/builds/123 \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## 🏗️ Architecture

Ferrous CI/CD follows Domain-Driven Design (DDD) principles:

```
┌─────────────────────────────────────────────────────┐
│                   Presentation Layer                 │
│         (REST API, GraphQL, CLI, Web UI)            │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                 Application Layer                    │
│     (Use Cases, DTOs, Application Services)         │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                    Domain Layer                      │
│   (Entities, Value Objects, Domain Services,        │
│    Aggregates, Repository Interfaces)               │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│                Infrastructure Layer                  │
│  (Database, File System, External APIs, Message     │
│   Queues, Repository Implementations)               │
└─────────────────────────────────────────────────────┘
```

### Core Domain Concepts

- **Pipeline**: The core aggregate representing a CI/CD pipeline
- **Build**: An execution instance of a pipeline
- **Stage**: A phase in the pipeline execution
- **Job**: A unit of work within a stage
- **Agent**: A worker that executes jobs
- **Artifact**: Build outputs stored for later use
- **Workspace**: The working directory for build execution

## 🧪 Development

### Running Tests

```bash
# Run all tests (125+ tests)
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --tests

# Run specific test file
cargo test --test pipeline_tests

# Run all tests including stress tests
cargo test --tests --ignored

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Run benchmarks
cargo bench
```

### Test Suite Structure

Ferrous CI/CD achieves over 90% test coverage.

- **Unit Tests**: 90+ unit tests (domain logic, services, repositories)
- **Integration Tests**: 24+ integration tests
  - `integration_test.rs`: E2E workflows (4 tests)
  - `pipeline_tests.rs`: Pipeline management (5 tests)
  - `build_tests.rs`: Build execution flows (5 tests)
  - `agent_tests.rs`: Agent management (6 tests)
  - `event_tests.rs`: Event system (4 tests)
  - `stress_tests.rs`: Load tests (4 ignored tests)

See [tests/README.md](tests/README.md) for details.

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Security audit
cargo audit

# Check dependencies
cargo outdated
```

### Building Documentation

```bash
# Generate documentation
cargo doc --no-deps --open

# Generate mdBook documentation
mdbook build docs/
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated

# Run in development mode with auto-reload
cargo watch -x run

# Run tests on file change
cargo watch -x test
```

## 📚 Documentation

### Specifications

Detailed design specifications are available in the `/spec` directory:

- [Specifications Index](spec/README.md)
- [Architecture Specification](spec/01-architecture.md)
- [Domain Model Specification](spec/02-domain-model.md)
- [API Specification](spec/03-api-specification.md)
- [Configuration Specification](spec/04-configuration.md)
- [Events Specification](spec/05-events.md)
- [Packages Specification](spec/06-packages.md) - Detailed descriptions of Rust packages used
- [Pipeline Definition Specification](spec/07-pipeline-definition.md) - The pipeline representation, its front-ends, and validation rules

## 🗺️ Roadmap

- [ ] Web UI Dashboard
- [ ] GraphQL API
- [ ] Kubernetes Operator
- [ ] Multi-cloud support (AWS, Azure, GCP)
- [ ] Advanced caching strategies
- [ ] Machine learning-based build optimization
- [ ] GitOps integration
- [ ] Terraform provider

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- Inspired by [Jenkins](https://www.jenkins.io/), [GitLab CI](https://docs.gitlab.com/ee/ci/), and [GitHub Actions](https://github.com/features/actions)
- Built with amazing Rust ecosystem crates
- Thanks to all contributors!

## 📧 Contact

- **Issue Tracker**: [GitHub Issues](https://github.com/yourusername/ferrous-ci-cd/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/ferrous-ci-cd/discussions)
- **Discord**: [Join our community](https://discord.gg/ferrous-ci-cd)

---

<div align="center">
Made with ❤️ and 🦀 by the Ferrous CI/CD Team
</div>
