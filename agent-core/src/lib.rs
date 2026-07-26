use rand::Rng;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct BaseAgentState {
    pub energy: f64,
    pub reward: f64,
    pub learning_rate: f64,
    pub birth_tick: u32,
}

impl BaseAgentState {
    pub fn new(birth_tick: u32, learning_rate: f64) -> Self {
        Self {
            energy: 1.0,
            reward: 0.0,
            learning_rate,
            birth_tick,
        }
    }

    pub fn apply_feedback(&mut self, impact: f64, setpoint: f64, inertia: f64) -> f64 {
        let error = (impact - setpoint).abs();
        self.energy = (self.energy - error.powi(2) * inertia * 0.01).max(0.0);
        if error < 0.3 {
            self.energy = (self.energy + 0.03).min(1.5);
            self.reward += 0.1;
        }
        error
    }

    pub fn mutate_energy_and_reward(&mut self) {
        self.energy *= 0.7;
        self.reward *= 0.5;
    }
}

pub trait Agent: Send {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64;
    fn get_energy(&self) -> f64;
    fn get_reward(&self) -> f64;
    fn get_label(&self) -> &str;
    fn get_birth_tick(&self) -> u32;
    fn mutate(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_population: usize,
    pub homeostasis_threshold: usize,
    pub emergency_threshold: usize,
    pub min_energy: f64,
    pub immunity_period: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_population: 25,
            homeostasis_threshold: 10,
            emergency_threshold: 5,
            min_energy: 0.01,
            immunity_period: 10_000,
        }
    }
}

pub struct PopulationManager {
    pub agents: Vec<Box<dyn Agent>>,
    pub config: AgentConfig,
}

impl PopulationManager {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            agents: Vec::new(),
            config,
        }
    }

    pub fn add_agent(&mut self, agent: Box<dyn Agent>) {
        self.agents.push(agent);
    }

    pub fn run_evolution(&mut self, current_tick: u32) {
        let mut rng = rand::thread_rng();
        let pop_size = self.agents.len();

        if pop_size < self.config.max_population {
            for agent in &mut self.agents {
                if agent.get_energy() > 0.9 && agent.get_reward() > 3.0 && rng.gen_bool(0.008) {
                    agent.mutate();
                }
            }
        }

        if pop_size < self.config.emergency_threshold {
            for agent in &mut self.agents {
                if agent.get_energy() > 0.5 && agent.get_reward() > 1.0 && rng.gen_bool(0.05) {
                    agent.mutate();
                }
            }
        }

        self.agents.retain(|a| {
            let age = current_tick.saturating_sub(a.get_birth_tick());
            a.get_energy() > self.config.min_energy || age < self.config.immunity_period
        });
    }
    pub fn process_all_agents_parallel(&mut self, setpoint: f64, inertia: f64) -> Vec<f64> {
        self.agents
            .par_iter_mut()
            .map(|agent| agent.process_tick(setpoint, inertia))
            .collect()
    }

    pub fn needs_homeostasis(&self) -> bool {
        self.agents.len() < self.config.homeostasis_threshold
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyAgent {
        base: BaseAgentState,
        label: String,
    }

    impl Agent for DummyAgent {
        fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
            self.base.apply_feedback(0.5, setpoint, inertia)
        }
        fn get_energy(&self) -> f64 {
            self.base.energy
        }
        fn get_reward(&self) -> f64 {
            self.base.reward
        }
        fn get_label(&self) -> &str {
            &self.label
        }
        fn get_birth_tick(&self) -> u32 {
            self.base.birth_tick
        }
        fn mutate(&mut self) {
            self.label = format!("{}_MUT", self.label);
            self.base.mutate_energy_and_reward();
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_evolution_removes_dead_agents() {
        let config = AgentConfig::default();
        let mut manager = PopulationManager::new(config);
        manager.add_agent(Box::new(DummyAgent {
            base: BaseAgentState {
                energy: 0.005,
                reward: 0.0,
                learning_rate: 1.0,
                birth_tick: 0,
            },
            label: "dead".to_string(),
        }));
        manager.add_agent(Box::new(DummyAgent {
            base: BaseAgentState {
                energy: 1.0,
                reward: 0.0,
                learning_rate: 1.0,
                birth_tick: 0,
            },
            label: "alive".to_string(),
        }));

        manager.run_evolution(10001);
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.agents[0].get_label(), "alive");
    }

    #[test]
    fn test_immunity_period() {
        let config = AgentConfig::default();
        let mut manager = PopulationManager::new(config);
        manager.add_agent(Box::new(DummyAgent {
            base: BaseAgentState {
                energy: 0.005,
                reward: 0.0,
                learning_rate: 1.0,
                birth_tick: 9999,
            },
            label: "baby".to_string(),
        }));

        manager.run_evolution(10000);
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_parallel_processing() {
        let config = AgentConfig::default();
        let mut manager = PopulationManager::new(config);

        for i in 0..100 {
            manager.add_agent(Box::new(DummyAgent {
                base: BaseAgentState::new(i, 1.0),
                label: format!("agent_{}", i),
            }));
        }

        let impacts = manager.process_all_agents_parallel(0.5, 0.1);
        assert_eq!(impacts.len(), 100);
    }
}
