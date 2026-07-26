# Contributing to Adaptive Agent Framework

Thank you for your interest in contributing to AAF. This document explains how to contribute effectively.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   git clone https://github.com/your-username/adaptive-agent-framework.git
   cd adaptive-agent-framework
3. Install Rust 1.75+ via rustup
4. Run the test suite:
   cargo test --all

## Development Workflow

### Code Style

Before submitting a PR, ensure:

    cargo fmt --all
    cargo clippy --all -- -D warnings
    cargo test --all

All checks must pass. CI will reject PRs that fail any of these.

### Branch Naming

- feature/add-crossover-operator
- fix/memory-leak-in-population-manager
- docs/improve-agent-trait-docs
- refactor/extract-base-agent-state

### Commit Messages

Follow conventional commits:

    feat(agent-core): add crossover operator for genetic evolution
    fix(math-core): correct alignment score calculation at boundary
    docs(readme): add docker deployment instructions
    refactor(state-store): simplify TLV parser error handling
    test(agent-core): add parallel processing benchmark

## What to Contribute

### Good First Issues

Look for issues labeled good first issue or help wanted.

### Domain Examples

The easiest way to contribute is adding a new domain example in examples/. Follow the pattern in grid_stability.rs or viral_campaign.rs:

1. Implement the Agent trait for your domain
2. Add a terminal dashboard with draw_dashboard
3. Export analytics to JSON
4. Document the domain mapping in the example file

### Core Engine Improvements

For changes to agent-core, math-core, or state-store:

1. Open an issue first to discuss the design
2. Ensure backward compatibility (especially for state-store binary format)
3. Add tests for new functionality
4. Update rustdoc comments for public APIs

### Documentation

Documentation improvements are always welcome: fix typos, add examples to rustdoc, improve README sections.

## Pull Request Process

1. Create a PR against the master branch
2. Fill out the PR template completely
3. Ensure CI passes (fmt, clippy, test, bench)
4. Request review from maintainers
5. Address feedback and update the PR

### PR Requirements

- All CI checks must pass
- New features must include tests
- Public API changes must include rustdoc updates
- Breaking changes must be documented in the PR description

## Code of Conduct

Be respectful and constructive. Focus on the code, not the person. Ask questions if something is unclear. Provide context for your suggestions. Accept feedback gracefully.

## Questions?

Open an issue with the question label. We will respond as soon as possible.

Thank you for helping make AAF better.
