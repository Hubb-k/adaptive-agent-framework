# Tools

Utilities for analyzing and visualizing simulation results.

## visualize.py

Generates an interactive HTML dashboard with Chart.js visualizations based on the structured CSV logs produced by the simulation examples. It automatically detects the domain (Grid Stability, Viral Campaign, or Swarm Management) based on the log file name and adjusts the chart labels accordingly.

### Usage

1. Run a simulation example to generate a CSV log file:
   ```bash
   cargo run --example grid_stability
   ```

2. Generate the visualization dashboard. By default, the script processes `examples/grid_stability.log`:
   ```bash
   python tools/visualize.py
   ```

3. To visualize a different simulation log, pass the specific file path as an argument:
   ```bash
   python tools/visualize.py examples/viral_campaign.log
   python tools/visualize.py examples/swarm_management.log
   ```

### Output

The script generates a standalone HTML file in the `tools/` directory (e.g., `tools/grid_stability_analysis.html`). This file can be opened in any modern web browser to inspect:
- Target vs. Alignment dynamics over time.
- System inertia or load fluctuations.
- Population size and cumulative reward progression.
- Key summary statistics (total ticks, resonance hits, final alignment).