use agent_core::{Agent, AgentConfig, BaseAgentState, PopulationManager};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

struct BenchmarkAgent {
    phase_angle: f64,
    base: BaseAgentState,
}

impl BenchmarkAgent {
    fn new(tick: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            phase_angle: rng.gen_range(0.0..std::f64::consts::TAU),
            base: BaseAgentState::new(tick, 0.8 + rng.gen_range(0.0..0.4)),
        }
    }
}

impl Agent for BenchmarkAgent {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        let mut hidden_layer = vec![0.0; 100];
        for i in 0..100 {
            hidden_layer[i] = ((self.phase_angle * (i as f64 + 1.0)).sin()
                + (setpoint * (i as f64 * 0.1)).cos())
                * 0.5;
        }

        let output: f64 = hidden_layer.iter().sum::<f64>() / 100.0;
        let impact = (output.sin() + 1.0) / 2.0;

        self.base.apply_feedback(impact, setpoint, inertia);

        let momentum = hidden_layer.iter().take(10).sum::<f64>() / 10.0;
        self.phase_angle += (setpoint - impact) * self.base.learning_rate * 0.15 + momentum * 0.01;
        self.phase_angle = self.phase_angle.rem_euclid(std::f64::consts::TAU);

        impact
    }

    fn get_energy(&self) -> f64 {
        self.base.energy
    }
    fn get_reward(&self) -> f64 {
        self.base.reward
    }
    fn get_label(&self) -> &str {
        "bench"
    }
    fn get_birth_tick(&self) -> u32 {
        self.base.birth_tick
    }
    fn mutate(&mut self) {
        self.base.mutate_energy_and_reward();
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn benchmark_sequential(c: &mut Criterion) {
    let config = AgentConfig::default();
    let mut manager = PopulationManager::new(config);

    for i in 0..1000 {
        manager.add_agent(Box::new(BenchmarkAgent::new(i)));
    }

    c.bench_function("sequential_1000_agents", |b| {
        b.iter(|| {
            let mut impacts = Vec::with_capacity(manager.len());
            for agent in &mut manager.agents {
                impacts.push(agent.process_tick(black_box(0.5), black_box(0.1)));
            }
            impacts
        })
    });
}

fn benchmark_parallel(c: &mut Criterion) {
    let config = AgentConfig::default();
    let mut manager = PopulationManager::new(config);

    for i in 0..1000 {
        manager.add_agent(Box::new(BenchmarkAgent::new(i)));
    }

    c.bench_function("parallel_1000_agents", |b| {
        b.iter(|| manager.process_all_agents_parallel(black_box(0.5), black_box(0.1)))
    });
}

criterion_group!(benches, benchmark_sequential, benchmark_parallel);
criterion_main!(benches);
