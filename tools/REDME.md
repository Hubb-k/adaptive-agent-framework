# Tools

Utilities for analyzing and visualizing simulation results.

## visualize.py

Generates an interactive HTML dashboard with charts based on simulation CSV logs.

### Usage

```bash
# Run the simulation
cargo run --example grid_stability

# Generate visualization
python tools/visualize.py

# Or specify a custom log file
python tools/visualize.py path/to/custom.log
```