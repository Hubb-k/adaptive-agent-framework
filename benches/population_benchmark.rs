use agent_core::{Agent, AgentConfig, PopulationManager};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BenchmarkAgent {
    phase_angle: f64,
    energy: f64,
    reward: f64,
    birth_tick: u32,
    learning_rate: f64,
}

impl BenchmarkAgent {
    fn new(tick: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            phase_angle: rng.gen_range(0.0..std::f64::consts::TAU),
            energy: 1.0,
            reward: 0.0,
            birth_tick: tick,
            learning_rate: 0.8 + rng.gen_range(0.0..0.4),
        }
    }
}

impl Agent for BenchmarkAgent {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        let mut hidden_layer = vec![0.0; 100];
        for (i, val) in hidden_layer.iter_mut().enumerate() {
            *val = ((self.phase_angle * (i as f64 + 1.0)).sin()
                + (setpoint * (i as f64 * 0.1)).cos())
                * 0.5;
        }
        let output: f64 = hidden_layer.iter().sum::<f64>() / 100.0;
        let impact = (output.sin() + 1.0) / 2.0;
        let error = (impact - setpoint).abs();
        self.energy = (self.energy - error.powi(2) * inertia * 0.01).max(0.0);
        if error < 0.3 {
            self.energy = (self.energy + 0.03).min(1.5);
            self.reward += 0.1;
        }
        let momentum = hidden_layer.iter().take(10).sum::<f64>() / 10.0;
        self.phase_angle += (setpoint - impact) * self.learning_rate * 0.15 + momentum * 0.01;
        self.phase_angle = self.phase_angle.rem_euclid(std::f64::consts::TAU);
        impact
    }
    fn get_energy(&self) -> f64 {
        self.energy
    }
    fn get_reward(&self) -> f64 {
        self.reward
    }
    fn get_label(&self) -> &str {
        "bench"
    }
    fn get_birth_tick(&self) -> u32 {
        self.birth_tick
    }
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        self.learning_rate = (self.learning_rate + rng.gen_range(-0.2..0.2)).clamp(0.1, 3.0);
        self.energy *= 0.7;
        self.reward *= 0.5;
    }
    fn save_state(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

fn setup_population(size: usize) -> PopulationManager {
    let config = AgentConfig::builder()
        .max_population(size)
        .homeostasis_threshold(size / 2)
        .min_energy(0.01)
        .immunity_period(100)
        .build();

    let mut manager = PopulationManager::builder().config(config).build();
    for i in 0..size {
        manager.add_agent(Box::new(BenchmarkAgent::new(i as u32)));
    }
    manager
}

fn benchmark_sequential(c: &mut Criterion) {
    let size = 10_000;
    let mut manager = setup_population(size);
    c.bench_function(&format!("sequential_{}k_agents", size / 1000), |b| {
        b.iter(|| {
            let mut impacts = Vec::with_capacity(size);
            for agent in &mut manager.agents {
                impacts.push(agent.process_tick(black_box(0.5), black_box(0.1)));
            }
            impacts
        })
    });
}

fn benchmark_parallel(c: &mut Criterion) {
    let size = 10_000;
    let mut manager = setup_population(size);
    c.bench_function(&format!("parallel_{}k_agents", size / 1000), |b| {
        b.iter(|| manager.process_all_agents_parallel(black_box(0.5), black_box(0.1)))
    });
}

criterion_group!(benches, benchmark_sequential, benchmark_parallel);
criterion_main!(benches);
