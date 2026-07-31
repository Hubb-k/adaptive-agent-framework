// Adaptive Agent Framework (API Facade)

pub use agent_core::{
    Agent, AgentConfig, AgentConfigBuilder, BaseAgentState, PopulationManager,
    PopulationManagerBuilder,
};

pub use config_layer::{
    ConfigError, EngineConfig, EngineConfigBuilder, SimulationConfig, SimulationConfigBuilder,
};

pub use math_core::{calculate_alignment_score, calculate_noise_profile};

pub use state_store::{BinarySerializer, StateError};
