# Repository instructions

## Project overview

- This is a Rust 2021 project that implements a CI/CD system using Tokio, Axum,
  and a Domain-Driven Design (DDD) style.
- Follow the clean-architecture boundaries documented in
  `spec/01-architecture.md`:
  - `src/domain` contains business rules, entities, value objects, domain
    services, events, and repository interfaces. It must not depend on the
    other application layers.
  - `src/application` coordinates use cases and DTOs and depends on domain
    abstractions rather than infrastructure implementations.
  - `src/infrastructure` implements persistence and external-system adapters.
  - `src/presentation` owns API and CLI entry points and delegates application
    behavior to the application layer.
- Consult the relevant document under `spec/` before changing documented
  architecture, domain behavior, configuration, events, APIs, or packaging.

## Implementation guidelines

- Keep changes focused and follow the existing module layout and naming
  conventions.
- Use Rust's `Result` and `?` for error propagation. Add useful context at
  infrastructure or application boundaries; do not silently discard errors.
- Preserve asynchronous behavior for I/O and other Tokio-based workflows. Do
  not block an async runtime thread with synchronous network or filesystem work.
- Add or update tests for behavior changes. Keep unit tests beside the code and
  integration or workflow tests under `tests/`.
- Add `///` documentation for new public items and update `README.md`,
  `CONTRIBUTING.md`, or `spec/` when public behavior or developer workflows
  change.
- Never commit credentials, private keys, `.env` files, build output, coverage
  output, or other generated artifacts.

## Validation

- During development, run the smallest relevant test target, for example
  `cargo test --test pipeline_tests` or a filtered `cargo test` invocation.
- Before handing off a code change, run the same core checks as CI:

  ```bash
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  cargo test --doc --all-features
  ```

- For dependency changes, also run `cargo audit` when it is installed.
- Do not run ignored, stress, or external-infrastructure tests unless the task
  requires them and their prerequisites are available.

## Code review rules

- Flag dependency-direction violations between domain, application,
  infrastructure, and presentation layers.
- Flag behavior changes that lack focused tests or contradict the specifications.
- Flag swallowed errors, blocking work on async runtime threads, hard-coded
  secrets, unsafe credential logging, and unbounded resource use.
- Treat formatting and lint output as automated-check results rather than
  duplicating style-only review comments.
