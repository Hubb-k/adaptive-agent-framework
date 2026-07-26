use agent_core::{Agent, AgentConfig, PopulationManager};
use config_layer::EngineConfig;
use math_core::calculate_alignment_score;
use state_store::{AgentRecord, BinarySerializer, GlobalState};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

type AgentFactory = Box<dyn Fn(u32) -> Box<dyn Agent>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EngineConfig::load("config.toml").unwrap_or_else(|_| {
        println!("[!] config.toml не найден, использую дефолтный конфиг.");
        EngineConfig::default_config()
    });

    let pop_config = AgentConfig::default();
    let mut manager = PopulationManager::new(pop_config);
    let factory: AgentFactory = Box::new(|tick: u32| Box::new(SimpleAgent::new(tick)));

    let mut state = GlobalState {
        tick: 0,
        global_setpoint: config.initial_setpoint,
        system_inertia: config.initial_inertia,
        ..Default::default()
    };

    for i in 0..5 {
        manager.add_agent(factory(i));
    }

    let log_path = "core_simulation.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Не удалось создать файл лога");
    let _ = writeln!(log_file, "tick,alignment,inertia,population,hits,reward");

    println!("[!] Adaptive Agent Framework запущен.");
    println!(
        "Setpoint: {:.2} | Inertia: {:.2} | Max ticks: {}",
        state.global_setpoint, state.system_inertia, config.max_ticks
    );
    println!("----------------------------------------------------------------------");

    print!("\x1B[?25l");
    io::stdout().flush().unwrap();

    let mut last_lines = 0;

    loop {
        if state.tick >= config.max_ticks {
            break;
        }
        state.tick += 1;

        let clamped_alignment = state.alignment_score.clamp(0.0, 1.0);
        let target_inertia = (1.05 - clamped_alignment).powi(2);
        state.system_inertia += (target_inertia - state.system_inertia) * 0.1;

        let mut impacts = Vec::with_capacity(manager.len());
        for agent in &mut manager.agents {
            impacts.push(agent.process_tick(state.global_setpoint, state.system_inertia));
        }
        state.alignment_score = calculate_alignment_score(&impacts, state.global_setpoint);

        let delay = ((state.system_inertia + 0.1) * (state.tick as f64).ln_1p())
            / (state.global_setpoint + 1e-9);
        thread::sleep(Duration::from_micros((delay * 50.0).min(1000.0) as u64));

        if state.alignment_score > 0.95 {
            state.success_count += 1;
            state.accumulated_reward += 0.001;
            state.complexity_factor += 0.005;
        }

        manager.run_evolution(state.tick as u32);
        if manager.needs_homeostasis() {
            manager.add_agent(factory(state.tick as u32));
        }

        if state.tick.is_multiple_of(config.snapshot_interval) {
            let serialized_agents: Vec<AgentRecord> = manager
                .agents
                .iter()
                .map(|a| AgentRecord {
                    global_setpoint: state.global_setpoint,
                    learning_rate: 1.0,
                    energy: a.get_energy(),
                    accumulated_reward: a.get_reward(),
                    birth_tick: a.get_birth_tick(),
                    label: a.get_label().to_string(),
                })
                .collect();

            let _ = BinarySerializer::save(&config.state_file, &state, &serialized_agents);

            if last_lines > 0 {
                print!("\x1B[{}A", last_lines);
            }
            print!("\x1B[J");

            println!("=== CORE SIMULATION MONITOR ===");
            println!("Tick: {:<6} | Align: {:.3} | Inertia: {:.3} | Pop: {:>2} | Hits: {:>4} | Reward: {:.3}",
                state.tick, state.alignment_score, state.system_inertia, manager.len(), state.success_count, state.accumulated_reward);
            println!("----------------------------------------------------------------------");
            io::stdout().flush().unwrap();

            let _ = writeln!(
                log_file,
                "{},{:.4},{:.4},{},{},{:.4}",
                state.tick,
                state.alignment_score,
                state.system_inertia,
                manager.len(),
                state.success_count,
                state.accumulated_reward
            );

            last_lines = 3;
        }
    }

    print!("\x1B[?25h");
    println!("\n[✓] Симуляция завершена. Лог: {}", log_path);
    println!(
        "Итого тиков: {} | Успешных: {} | Финальный alignment: {:.3}",
        state.tick, state.success_count, state.alignment_score
    );
    Ok(())
}

struct SimpleAgent {
    energy: f64,
    reward: f64,
    birth_tick: u32,
    label: String,
    learning_rate: f64,
    internal_phase: f64,
}

impl SimpleAgent {
    fn new(tick: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            energy: 1.0,
            reward: 0.0,
            birth_tick: tick,
            label: format!("agent_{}", tick),
            learning_rate: 0.5 + rng.gen_range(0.0..1.0),
            internal_phase: rng.gen_range(0.0..std::f64::consts::TAU),
        }
    }
}

impl Agent for SimpleAgent {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        use rand::Rng;
        let drift = (setpoint - self.internal_phase) * self.learning_rate * 0.1;
        self.internal_phase += drift + (rand::thread_rng().gen::<f64>() - 0.5) * 0.05;
        let impact = (self.internal_phase.sin() + 1.0) / 2.0;
        let error = (impact - setpoint).abs();
        self.energy = (self.energy - error.powi(2) * inertia * 0.01).max(0.0);
        if error < 0.2 {
            self.energy = (self.energy + 0.02).min(1.5);
            self.reward += 0.05;
        }
        impact
    }
    fn get_energy(&self) -> f64 {
        self.energy
    }
    fn get_reward(&self) -> f64 {
        self.reward
    }
    fn get_label(&self) -> &str {
        &self.label
    }
    fn get_birth_tick(&self) -> u32 {
        self.birth_tick
    }
    fn mutate(&mut self) {
        use rand::Rng;
        self.learning_rate =
            (self.learning_rate + rand::thread_rng().gen_range(-0.2..0.2)).clamp(0.1, 2.0);
        self.energy *= 0.7;
        self.reward *= 0.5;
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
