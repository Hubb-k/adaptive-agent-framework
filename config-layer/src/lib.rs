use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub max_ticks: u64,
    pub snapshot_interval: u64,
    pub state_file: String,
    pub parallel: bool,
}

impl EngineConfig {
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: EngineConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn default_config() -> Self {
        Self {
            max_ticks: 10_000,
            snapshot_interval: 1000,
            state_file: "state.bin".to_string(),
            parallel: true,
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_ticks == 0 {
            return Err(ConfigError::ValidationError(
                "max_ticks must be > 0".to_string(),
            ));
        }
        if self.snapshot_interval == 0 {
            return Err(ConfigError::ValidationError(
                "snapshot_interval must be > 0".to_string(),
            ));
        }
        if self.snapshot_interval > self.max_ticks {
            return Err(ConfigError::ValidationError(
                "snapshot_interval cannot be greater than max_ticks".to_string(),
            ));
        }
        Ok(())
    }

    pub fn builder() -> EngineConfigBuilder {
        EngineConfigBuilder::new()
    }
}

pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl Default for EngineConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default_config(),
        }
    }

    pub fn max_ticks(mut self, value: u64) -> Self {
        self.config.max_ticks = value;
        self
    }

    pub fn snapshot_interval(mut self, value: u64) -> Self {
        self.config.snapshot_interval = value;
        self
    }

    pub fn state_file(mut self, value: String) -> Self {
        self.config.state_file = value;
        self
    }

    pub fn parallel(mut self, value: bool) -> Self {
        self.config.parallel = value;
        self
    }

    pub fn build(self) -> Result<EngineConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub initial_setpoint: f64,
    pub initial_inertia: f64,
    pub max_population: usize,
    pub homeostasis_threshold: usize,
    pub min_energy: f64,
    pub immunity_period: u32,
}

impl SimulationConfig {
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: SimulationConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn default_config() -> Self {
        Self {
            initial_setpoint: 0.5,
            initial_inertia: 0.1,
            max_population: 25,
            homeostasis_threshold: 10,
            min_energy: 0.01,
            immunity_period: 500,
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_population == 0 {
            return Err(ConfigError::ValidationError(
                "max_population must be > 0".to_string(),
            ));
        }
        if self.homeostasis_threshold > self.max_population {
            return Err(ConfigError::ValidationError(
                "homeostasis_threshold cannot be greater than max_population".to_string(),
            ));
        }
        if self.min_energy < 0.0 || self.min_energy > 1.0 {
            return Err(ConfigError::ValidationError(
                "min_energy must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(())
    }

    pub fn builder() -> SimulationConfigBuilder {
        SimulationConfigBuilder::new()
    }
}

pub struct SimulationConfigBuilder {
    config: SimulationConfig,
}

impl Default for SimulationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SimulationConfig::default_config(),
        }
    }

    pub fn initial_setpoint(mut self, value: f64) -> Self {
        self.config.initial_setpoint = value;
        self
    }

    pub fn initial_inertia(mut self, value: f64) -> Self {
        self.config.initial_inertia = value;
        self
    }

    pub fn max_population(mut self, value: usize) -> Self {
        self.config.max_population = value;
        self
    }

    pub fn homeostasis_threshold(mut self, value: usize) -> Self {
        self.config.homeostasis_threshold = value;
        self
    }

    pub fn min_energy(mut self, value: f64) -> Self {
        self.config.min_energy = value;
        self
    }

    pub fn immunity_period(mut self, value: u32) -> Self {
        self.config.immunity_period = value;
        self
    }

    pub fn build(self) -> Result<SimulationConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default_config();
        assert_eq!(config.max_ticks, 10_000);
        assert_eq!(config.snapshot_interval, 1000);
        assert!(config.parallel);
    }

    #[test]
    fn test_engine_config_builder() {
        let config = EngineConfig::builder()
            .max_ticks(5000)
            .snapshot_interval(500)
            .parallel(false)
            .build()
            .unwrap();

        assert_eq!(config.max_ticks, 5000);
        assert_eq!(config.snapshot_interval, 500);
        assert!(!config.parallel);
    }

    #[test]
    fn test_engine_config_validation() {
        let config = EngineConfig::builder().max_ticks(0).build();
        assert!(config.is_err());
    }

    #[test]
    fn test_simulation_config_default() {
        let config = SimulationConfig::default_config();
        assert!((config.initial_setpoint - 0.5).abs() < 1e-9);
        assert_eq!(config.max_population, 25);
    }

    #[test]
    fn test_simulation_config_builder() {
        let config = SimulationConfig::builder()
            .max_population(100)
            .immunity_period(1000)
            .build()
            .unwrap();

        assert_eq!(config.max_population, 100);
        assert_eq!(config.immunity_period, 1000);
    }

    #[test]
    fn test_engine_config_save_load() {
        let path = "test_engine_config.toml";
        let config = EngineConfig::builder()
            .max_ticks(2000)
            .snapshot_interval(200)
            .build()
            .unwrap();

        config.save(path).unwrap();
        let loaded = EngineConfig::load(path).unwrap();

        assert_eq!(loaded.max_ticks, 2000);
        assert_eq!(loaded.snapshot_interval, 200);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_simulation_config_validation() {
        let config = SimulationConfig::builder()
            .homeostasis_threshold(30)
            .max_population(25)
            .build();

        assert!(config.is_err());
    }
}
