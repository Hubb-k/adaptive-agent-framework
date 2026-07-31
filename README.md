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
  <img src="https://img.shields.io/badge/docker-ready-blue" alt="Docker Support">
  <img src="https://img.shields.io/badge/python-3.8+-blue" alt="Python Version">
</p>

> **Status: Work in Progress.** This project is under active development and is not yet suitable for production use. The core API is stabilizing, integrations are being developed, and breaking changes may occur.

## Overview

Adaptive Agent Framework (AAF) is a parallel, extensible simulation engine for modeling self-organizing systems. Agents co-evolve through feedback loops, natural selection, and homeostatic regulation, all running on a domain-agnostic core.

**Primary intended use case:** AAF is designed to serve as an adaptive execution layer (adapter) for a PUID (Perception-Understanding-Intention-Decision) layer. In this architecture, the PUID layer is responsible for collecting external data, analyzing it, and forming target setpoints. AAF accepts these targets and dynamically regulates system parameters through population management. Using feedback, mutation, and homeostatic regulation mechanisms, the engine achieves the specified objective under conditions of nonlinearity, noise, and environmental change — relieving the PUID layer from the need to algorithmically control each individual element of the system.

The framework is organized as a Cargo workspace with strict separation of concerns: a mathematically pure core, a strongly-typed configuration layer, a forward-compatible binary state store, and a parallel execution engine built on `rayon`.

## Key Features

- **Domain-Agnostic Core:** implement the `Agent` trait and plug in your domain logic. The engine handles lifecycle, evolution, and homeostasis.
- **Parallel Processing:** built-in `rayon` support scales to tens of thousands of computationally heavy agents with measured 4x speedup over sequential execution on multi-core CPUs.
- **Evolution and Homeostasis:** mutation, natural selection, and automatic population replenishment when critical thresholds are breached.
- **Binary Persistence:** TLV-format serialization with magic bytes and versioning for forward-compatible checkpoint/restore workflows.
- **Real-Time Terminal Dashboard:** in-place ANSI rendering without terminal scrolling, plus automatic CSV log export.
- **Strongly-Typed Configuration:** TOML-based engine and simulation configuration with validation and builder patterns.
- **Production-Ready CI:** GitHub Actions pipeline with `fmt`, `clippy`, and full test suite on every push.

*Note:* The examples in the repository (grid stability, viral campaign, swarm management) are technical illustrations of how to implement the `Agent` trait for different load distribution scenarios, and do not limit the scope of the framework.

## Architecture

```
adaptive-agent-framework/
── agent-core/        # Agent trait, PopulationManager, BaseAgentState, rayon integration
├── config-layer/      # TOML-based engine and simulation configuration with validation
├── math-core/         # Alignment scoring, noise profiles, domain-agnostic math
├── state-store/       # Binary TLV serialization with forward compatibility (magic: AAGF)
├── src/
│   ├── lib.rs         # Facade library re-exporting all public crates
│   └── main.rs        # Reference CLI simulation entry point
── examples/
│   ├── grid_stability.rs      # Substation balance under disturbances and black swans
│   ├── viral_campaign.rs      # Content adaptation under algorithm changes
│   └── swarm_management.rs    # Dynamic fleet task distribution with hardware failures
├── benches/
│   └── population_benchmark.rs # Sequential vs parallel throughput benchmarks (criterion)
── tools/
│   └── visualize.py           # CSV log to interactive HTML dashboard generator
├── Cargo.toml                 # Workspace manifest
├── config.toml                # Shared simulation configuration
└── docker-compose.yml         # Multi-service container orchestration
```

## Crate Descriptions

### `agent-core`
The execution engine. Defines the `Agent` trait (the only contract domain code must implement), `PopulationManager` for lifecycle management, and `BaseAgentState` for common agent fields. Parallel tick processing is implemented via `rayon::par_iter_mut`.

### `config-layer`
Strongly-typed TOML configuration. `EngineConfig` controls simulation length, snapshot intervals, and state file paths. `SimulationConfig` defines domain parameters such as `initial_setpoint`, `max_population`, and `homeostasis_threshold`. Both structs support validation, serialization, and builder patterns.

### `math-core`
Pure mathematical functions independent of agent state. Provides `calculate_alignment_score` (population-level metric of how well impacts match the setpoint) and `calculate_noise_profile` (interference-based noise function using golden ratio harmonics).

### `state-store`
Binary serialization with forward compatibility. Uses a 4-byte magic header (`AAGF`) and a 4-byte little-endian version tag, followed by `bincode`-encoded payload. Deserialization rejects mismatched magic or version, enabling safe migration across framework versions.

## Quick Start

### Prerequisites

- Rust 1.85 or later
- Python 3.8+ (for visualization only)
- Docker and Docker Compose (optional)

### Installation

```bash
git clone https://github.com/Hubb-k/adaptive-agent-framework.git
cd adaptive-agent-framework
cargo build --release
```

### Running Simulations

```bash
cargo run --example grid_stability
cargo run --example viral_campaign
cargo run --example swarm_management
```

Each example reads parameters from `config.toml` in the project root, generates a CSV log in `examples/`, and produces a binary checkpoint for resumability.

### Running the Main CLI

```bash
cargo run --release
```

The main binary runs a reference simulation loop with a real-time terminal dashboard and automatic checkpointing.

## Benchmarks

The project includes `criterion`-based benchmarks comparing sequential and parallel tick processing on a population of 10,000 agents with a realistic computational workload.

```bash
cargo bench
```

Typical results on a modern multi-core CPU:

```
sequential_10k_agents   time: [15.5 ms 15.9 ms 16.3 ms]
parallel_10k_agents     time: [3.6 ms 3.7 ms 3.8 ms]
```

## Creating Your Own Domain

The root `src/lib.rs` re-exports all public API from the workspace crates, so you only need to depend on `adaptive_agent_framework` and `config-layer`. Implement the `Agent` trait and plug your struct into `PopulationManager`:

```rust
use adaptive_agent_framework::{
    Agent, AgentConfig, BinarySerializer, PopulationManager, SimulationConfig,
    calculate_alignment_score,
};
use config_layer::EngineConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MyAgent {
    energy: f64,
    reward: f64,
    birth_tick: u32,
    learning_rate: f64,
}

impl Agent for MyAgent {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        let impact = 0.5;
        let error = (impact - setpoint).abs();
        self.energy = (self.energy - error.powi(2) * inertia * 0.01).max(0.0);
        if error < 0.3 {
            self.energy = (self.energy + 0.03).min(1.5);
            self.reward += 0.1;
        }
        impact
    }

    fn get_energy(&self) -> f64 { self.energy }
    fn get_reward(&self) -> f64 { self.reward }
    fn get_label(&self) -> &str { "my_agent" }
    fn get_birth_tick(&self) -> u32 { self.birth_tick }

    fn mutate(&mut self) {
        self.learning_rate = (self.learning_rate + 0.1).clamp(0.1, 3.0);
        self.energy *= 0.7;
        self.reward *= 0.5;
    }

    fn save_state(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}
```

Then register your example in `Cargo.toml`:

```toml
[[example]]
name = "my_domain"
path = "examples/my_domain.rs"
```

And run it:

```bash
cargo run --example my_domain
```

## Configuration

All examples and the main CLI share a single `config.toml` file in the project root. The `EngineConfig` and `SimulationConfig` structs read from the same file and ignore unknown fields, allowing a unified configuration surface.

```toml
# Engine parameters
max_ticks = 10000
snapshot_interval = 1000
state_file = "checkpoint.bin"
parallel = true

# Simulation parameters
initial_setpoint = 0.5
initial_inertia = 0.1
max_population = 25
homeostasis_threshold = 10
min_energy = 0.01
immunity_period = 500
```

Both configurations support validation (e.g., `homeostasis_threshold` cannot exceed `max_population`, `snapshot_interval` cannot exceed `max_ticks`) and can be constructed programmatically via their respective builders.

## Visualization

All examples generate structured CSV logs in the `examples/` directory. The provided Python script converts these logs into interactive HTML dashboards powered by Chart.js.

```bash
python tools/visualize.py examples/grid_stability.log
```

The generated HTML file is saved to `tools/<example_name>_analysis.html` and can be opened in any modern web browser. Each dashboard displays:

- Target vs. Alignment dynamics over time
- System inertia or load fluctuations
- Population size and cumulative reward progression
- Summary statistics (total ticks, resonance hits, final alignment, final population)

The script automatically detects the domain based on the log filename and adjusts labels accordingly.

## Docker

The project uses a multi-stage Docker build that caches dependencies independently of source code, reducing rebuild times significantly.

### Build

```bash
docker compose build
```

### Run

```bash
docker compose up grid-stability
docker compose up viral-campaign
docker compose up swarm-management
```

### Stop

```bash
docker compose down
```

All services share a named volume `aaf_data` mounted at `/app`, preserving checkpoints and logs across container restarts.

## Testing

```bash
cargo test --workspace
```

The test suite covers:

- Agent evolution and immunity period logic (`agent-core`)
- Binary serialization round-trips and magic byte validation (`state-store`)
- Configuration parsing, validation, and save/load cycles (`config-layer`)
- Alignment scoring bounds and edge cases (`math-core`)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Ensure all checks pass:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo bench --no-run
   ```
4. Submit a pull request

## License

MIT License. See `LICENSE` for details.