use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    pub initial_setpoint: f64,
    pub initial_inertia: f64,
    pub max_ticks: u64,
    pub snapshot_interval: u64,
    pub state_file: String,
}

impl EngineConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: EngineConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_config() -> Self {
        Self {
            initial_setpoint: 0.5,
            initial_inertia: 0.1,
            max_ticks: 10_000,
            snapshot_interval: 1000,
            state_file: "state.bin".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default_config();
        assert!((config.initial_setpoint - 0.5).abs() < 1e-9);
        assert_eq!(config.max_ticks, 10_000);
    }

    #[test]
    fn test_load_from_file() {
        let path = "test_config.toml";
        fs::write(
            path,
            "initial_setpoint = 0.7\ninitial_inertia = 0.2\nmax_ticks = 5000\nsnapshot_interval = 500\nstate_file = \"test.bin\"",
        )
        .unwrap();

        let config = EngineConfig::load(path).unwrap();
        assert!((config.initial_setpoint - 0.7).abs() < 1e-9);
        assert_eq!(config.max_ticks, 5000);

        fs::remove_file(path).ok();
    }
}
