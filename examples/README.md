
# Examples

This directory contains reference implementations demonstrating how to integrate domain-specific logic with the Adaptive Agent Framework. These examples are designed to be self-contained, reproducible, and free of hardcoded parameters, relying entirely on the `config.toml` configuration layer.

## Available Examples

### 1. Grid Stability (`grid_stability.rs`)
Simulates power grid substations attempting to maintain operational balance under dynamic loads and external disturbances. Each agent represents a substation that adjusts its phase angle to match a global target setpoint.

Key Mechanics:
- Dynamic Inertia Adaptation: System inertia increases when alignment drops, stabilizing the grid, and decreases when alignment is high, allowing for faster adjustments.
- Event-Driven Anomalies: The simulation injects disturbances, cascading failures, and black swan events at regular intervals to test system resilience.
- Homeostatic Replenishment: If substations fail or drop below the energy threshold, the population manager automatically spawns new units to maintain the homeostasis threshold.
- Real-Time Terminal Dashboard: In-place rendering of agent states, energy levels, and alignment metrics without terminal scrolling.

Execution:
cargo run --example grid_stability

### 2. Viral Campaign (`viral_campaign.rs`)
Models digital content channels adapting to shifting platform algorithms, viral trends, and competitor attacks. Each agent represents a content channel that adjusts its stylistic output to maximize engagement (reward) against a moving target.

Key Mechanics:
- Style Adaptation: Agents use a learning rate to gradually shift their internal style phase toward the current algorithmic target, with added stochastic noise to simulate creative exploration.
- Reward-Based Evolution: Agents that successfully match the target accumulate energy and rewards, making them eligible for mutation and long-term survival.
- Platform Dynamics: The target setpoint oscillates naturally but is periodically disrupted by algorithm changes or competitor attacks, forcing rapid adaptation.

Execution:
cargo run --example viral_campaign

### 3. Swarm Management (`swarm_management.rs`)
Demonstrates dynamic task distribution across a fleet of autonomous units experiencing hardware failures and scheduled fleet expansions. This example highlights how the framework handles population volatility.

Key Mechanics:
- Dynamic Load Balancing: The target setpoint for each unit is calculated dynamically as Total Demand divided by Current Population. If units are lost, the load per remaining unit increases proportionally.
- Population Volatility: Hardware failures randomly remove units from the manager, simulating real-world attrition.
- Resilience and Replenishment: The system tracks total lost and spawned units. When the population drops below the homeostasis threshold, new units are automatically deployed to restore operational capacity.

Execution:
cargo run --example swarm_management

## Creating Your Own Example

To implement a new domain scenario, follow these steps:

1. Create a new Rust file in this directory (e.g., `my_domain.rs`).
2. Register the example in the root `Cargo.toml` under the `[[example]]` section:
   ```toml
   [[example]]
   name = "my_domain"
   path = "examples/my_domain.rs"
   ```
3. Define a domain-specific struct and implement the `adaptive_agent_framework::Agent` trait. Ensure all required methods (`process_tick`, `get_energy`, `get_reward`, `get_label`, `get_birth_tick`, `mutate`, `save_state`) are implemented.
4. Initialize the simulation using `EngineConfig::load("config.toml")` and `SimulationConfig::load("config.toml")` to avoid hardcoded values.
5. Instantiate a `PopulationManager` using the builder pattern and inject your agents.
6. Execute the simulation loop, calling `manager.process_all_agents_parallel()` and `manager.run_evolution()` on each tick.
7. Write state metrics to a CSV file for post-simulation analysis.

## Visualization

All examples generate structured CSV logs in this directory. You can generate interactive HTML dashboards to analyze alignment, population dynamics, and reward accumulation over time.

To generate a visualization report, run the provided Python script:

```bash
python tools/visualize.py examples/<example_name>.log
```

This will create an HTML file in the `tools/` directory, which can be opened in any modern web browser to view Chart.js-based analytics.