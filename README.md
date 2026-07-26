# Adaptive Agent Framework (AAF)

<p align="center">
  <strong>A high-performance, domain-agnostic co-evolutionary agent simulation engine built in Rust.</strong>
</p>

<p align="center">
  <a href="https://github.com/Hubb-k/adaptive-agent-framework/actions/workflows/ci.yml">
    <img src="https://github.com/Hubb-k/adaptive-agent-framework/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange" alt="Rust Version">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
</p>

---

## Overview

Adaptive Agent Framework is a parallel, extensible simulation engine for modeling self-organizing systems. Agents co-evolve through feedback loops, natural selection, and homeostatic regulation, all running on a domain-agnostic core.

Whether you are simulating smart power grids, viral content distribution, biological populations, or financial market regimes, AAF provides the mechanisms. You provide the policy.

## Key Features

- Domain-Agnostic Core: implement the Agent trait and plug in your domain logic. The engine handles lifecycle, evolution, and homeostasis.
- Parallel Processing: built-in rayon support scales to tens of thousands of computationally heavy agents (3-4x speedup on multi-core CPUs).
- Evolution and Homeostasis: mutation, natural selection, and automatic population replenishment when critical thresholds are breached.
- Binary Persistence: TLV-format serialization with forward compatibility for checkpoint/restore workflows.
- Real-Time Terminal Dashboard: in-place rendering without scroll, plus automatic CSV and JSON export.
- Production-Ready CI: GitHub Actions pipeline with fmt, clippy, and full test suite on every push.

## Architecture

    adaptive-agent-framework/
    ├── agent-core/        # Agent trait, PopulationManager, BaseAgentState, rayon integration
    ├── math-core/         # Alignment scoring, noise profiles
    ├── state-store/       # Binary TLV serialization with forward compatibility
    ├── config-layer/      # TOML-based engine configuration
    ├── examples/
    │   ├── grid_stability.rs      # Smart grid with cascading failures and black swans
    │   └── viral_campaign.rs      # Content strategy adaptation under algorithm changes
    ├── benches/
    │   └── population_benchmark.rs # Sequential vs parallel throughput benchmarks
    └── tools/
        └── visualize.py           # CSV log to HTML dashboard generator

## Quick Start

Prerequisites: Rust 1.75+

Run simulations:

    cargo run --example grid_stability
    cargo run --example viral_campaign

Benchmarks and tests:

    cargo bench
    cargo test --all

## Creating Your Own Domain

Implement the Agent trait and plug it into PopulationManager:

    use agent_core::{Agent, AgentConfig, BaseAgentState, PopulationManager};

    struct MyAgent {
        base: BaseAgentState,
    }

    impl Agent for MyAgent {
        fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
            let impact = /* your domain logic */;
            let error = self.base.apply_feedback(impact, setpoint, inertia);
            // adapt based on error...
            impact
        }
        // ... other trait methods
    }

## Analytics Output

After each simulation, a structured JSON report is generated in examples/analysis_report_*.json with MAE, RMSE, population statistics, and resonance hit rates.

## Docker

    docker build -t aaf .
    docker run -it aaf

## Contributing

1. Fork the repository
2. Create a feature branch (git checkout -b feature/my-feature)
3. Ensure all checks pass: cargo fmt && cargo clippy --all -- -D warnings && cargo test --all
4. Submit a Pull Request

## License

MIT License. See LICENSE for details.
