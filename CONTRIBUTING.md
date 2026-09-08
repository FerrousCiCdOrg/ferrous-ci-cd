# Contributing to Ferrous CI/CD

Thank you for your interest in contributing to Ferrous CI/CD! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Set up the development environment
4. Create a feature branch
5. Make your changes
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.75 or later (install via [rustup](https://rustup.rs/))
- Docker (optional, for integration tests)
- PostgreSQL or SQLite
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/FerrousCiCdOrg/ferrous-ci-cd.git
cd ferrous-ci-cd

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated cargo-nextest

# Build the project
cargo build

# Run tests
cargo test

# Run with auto-reload
cargo watch -x run
```

### Database Setup

For PostgreSQL:
```bash
# Create database
createdb ferrous_ci

# Run migrations
diesel migration run
```

For SQLite:
```bash
# Migrations will run automatically
cargo run
```

## Project Structure

```
ferrous-ci-cd/
├── src/
│   ├── domain/          # Domain layer (DDD)
│   │   ├── entities/    # Domain entities
│   │   ├── value_objects/  # Value objects
│   │   ├── repositories/   # Repository interfaces
│   │   ├── services/    # Domain services
│   │   └── events.rs    # Domain events
│   ├── application/     # Application layer
│   │   ├── use_cases/   # Use cases
│   │   └── dto/         # Data Transfer Objects
│   ├── infrastructure/  # Infrastructure layer
│   │   ├── repositories/  # Repository implementations
│   │   ├── database/    # Database connection
│   │   ├── storage/     # File storage
│   │   └── git/         # Git operations
│   ├── presentation/    # Presentation layer
│   │   ├── api/         # REST API
│   │   └── cli/         # CLI handlers
│   ├── config.rs        # Configuration
│   ├── error.rs         # Error handling
│   └── lib.rs           # Library entry point
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── docs/                # Documentation
└── migrations/          # Database migrations
```

## Development Workflow

### Feature Development

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write code following the coding standards
   - Add tests for new functionality
   - Update documentation as needed

3. **Run tests**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add your feature"
   ```

5. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a pull request**

### Bug Fixes

1. Create a branch named `fix/bug-description`
2. Fix the bug and add a regression test
3. Follow the same process as feature development

## Coding Standards

### Rust Style Guide

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Maximum line length: 100 characters

### Naming Conventions

- **Structs**: PascalCase (e.g., `BuildService`)
- **Functions**: snake_case (e.g., `create_build`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`)
- **Files**: snake_case (e.g., `build_service.rs`)

### Documentation

- Add documentation comments (`///`) for public items
- Include examples in documentation when helpful
- Keep documentation up to date with code changes

Example:
```rust
/// Create a new build for the given pipeline.
///
/// # Arguments
///
/// * `pipeline_id` - The ID of the pipeline
/// * `commit_sha` - The git commit SHA
///
/// # Returns
///
/// Returns a `Result` containing the created `Build`
///
/// # Example
///
/// ```
/// let build = service.create_build(pipeline_id, "abc123").await?;
/// ```
pub async fn create_build(
    &self,
    pipeline_id: PipelineId,
    commit_sha: String,
) -> Result<Build> {
    // ...
}
```

### Error Handling

- Use the custom `Error` type defined in `error.rs`
- Provide context for errors using the `context()` method
- Use `?` operator for error propagation
- Handle errors at appropriate levels

```rust
// Good
self.repository
    .save(&build)
    .await
    .context("Failed to save build to repository")?;

// Bad - swallowing errors
let _ = self.repository.save(&build).await;
```

## Testing Guidelines

### Unit Tests

- Place unit tests in the same file as the code being tested
- Use the `#[cfg(test)]` module
- Test public interfaces and edge cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_creation() {
        let build = Build::new(/* ... */);
        assert_eq!(build.status(), BuildStatus::Pending);
    }
}
```

### Integration Tests

- Place integration tests in the `tests/` directory
- Test complete workflows
- Use test fixtures and helpers

```rust
#[tokio::test]
async fn test_complete_build_workflow() {
    let app = setup_test_app().await;
    // Test the complete workflow
}
```

### Test Coverage

- Aim for 90%+ code coverage
- Run coverage with:
  ```bash
  cargo tarpaulin --out Html
  ```
- Check coverage report in `tarpaulin-report.html`

### Property-Based Testing

- Use `proptest` for property-based tests
- Test invariants and properties

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_build_number_always_increases(
        count in 1..100usize
    ) {
        // Test that build numbers always increase
    }
}
```

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### Examples

```
feat(build): add support for parallel job execution

Implement parallel job execution within stages to improve
build performance. Jobs can now run concurrently based on
their dependencies.

Closes #123
```

```
fix(agent): prevent memory leak in job executor

Fixed a memory leak that occurred when jobs were cancelled
before completion. Resources are now properly cleaned up.

Fixes #456
```

## Pull Request Process

### Before Submitting

1. **Update tests**: Add tests for new functionality
2. **Update documentation**: Update README, docs, and inline comments
3. **Run quality checks**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo audit
   ```
4. **Rebase on main**: Ensure your branch is up to date
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

### PR Description Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe the testing you've done

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Code follows style guidelines
- [ ] All tests passing
- [ ] No new warnings
```

### Review Process

1. At least one maintainer approval required
2. All tests must pass
3. Code coverage must not decrease
4. Documentation must be updated

### After Approval

- Maintainers will merge using squash merge
- Your contribution will be included in the next release

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release notes
4. Tag the release
5. Publish to crates.io

## Getting Help

### Resources

- **Documentation**: Check the [docs/](docs/) directory
- **Examples**: See [examples/](examples/) directory
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions

### Communication Channels

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: General questions and discussions
- Discord: Real-time chat with the community

## Recognition

Contributors will be acknowledged in:
- README.md Contributors section
- Release notes
- Project documentation

Thank you for contributing to Ferrous CI/CD! 🦀🚀

