use adaptive_agent_framework::{
    calculate_alignment_score, Agent, AgentConfig, BinarySerializer, PopulationManager,
    SimulationConfig,
};
use config_layer::EngineConfig;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct SwarmState {
    tick: u64,
    total_demand: f64,
    alignment: f64,
    hits: u64,
    total_reward: f64,
    total_spawned: usize,
    total_lost: usize,
}

impl Default for SwarmState {
    fn default() -> Self {
        Self {
            tick: 0,
            total_demand: 5.0,
            alignment: 0.0,
            hits: 0,
            total_reward: 0.0,
            total_spawned: 3,
            total_lost: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SwarmUnit {
    id: String,
    target_load: f64,
    energy: f64,
    reward: f64,
    birth_tick: u32,
    learning_rate: f64,
}

impl SwarmUnit {
    fn new(id: &str, tick: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: id.to_string(),
            target_load: 0.0,
            energy: 1.0,
            reward: 0.0,
            birth_tick: tick,
            learning_rate: 0.8 + rng.gen_range(0.0..0.4),
        }
    }
}

impl Agent for SwarmUnit {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        let error = (self.target_load - setpoint).abs();
        self.energy = (self.energy - error.powi(2) * inertia * 0.01).max(0.0);
        if error < 0.3 {
            self.energy = (self.energy + 0.03).min(1.5);
            self.reward += 0.1;
        }
        let correction = (setpoint - self.target_load) * self.learning_rate * 0.15;
        let noise = (rand::thread_rng().gen::<f64>() - 0.5) * 0.03;
        self.target_load += correction + noise;
        self.target_load = self.target_load.max(0.0);
        self.target_load
    }
    fn get_energy(&self) -> f64 {
        self.energy
    }
    fn get_reward(&self) -> f64 {
        self.reward
    }
    fn get_label(&self) -> &str {
        &self.id
    }
    fn get_birth_tick(&self) -> u32 {
        self.birth_tick
    }
    fn mutate(&mut self) {
        use rand::Rng;
        self.learning_rate =
            (self.learning_rate + rand::thread_rng().gen_range(-0.2..0.2)).clamp(0.1, 3.0);
        self.energy *= 0.7;
        self.reward *= 0.5;
    }
    fn save_state(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_dashboard(
    tick: u64,
    demand: f64,
    setpoint: f64,
    alignment: f64,
    pop: usize,
    hits: u64,
    reward: f64,
    lost: usize,
    agents: &[Box<dyn Agent>],
    log_file: &mut std::fs::File,
    last_lines: &mut usize,
    event: &str,
) {
    if *last_lines > 0 {
        print!("\x1B[{}A", *last_lines);
    }
    print!("\x1B[J");
    println!("=== SWARM FLEET MANAGEMENT ===");
    println!(
        "Tick: {:<6} | Demand: {:.2} | Units: {:>2} | Load/Unit: {:.3} | Align: {:.3} | {}",
        tick, demand, pop, setpoint, alignment, event
    );
    println!("----------------------------------------------------------------------");
    for agent in agents {
        let energy = agent.get_energy();
        let status = if energy > 0.8 {
            "\x1B[32mOPTIMAL \x1B[0m"
        } else if energy > 0.3 {
            "\x1B[33mSTRESSED\x1B[0m"
        } else {
            "\x1B[31mCRITICAL\x1B[0m"
        };
        let bar_width = ((energy * 20.0) as usize).min(20);
        let bar: String = "█".repeat(bar_width) + &"░".repeat(20 - bar_width);
        println!(
            "[{:8}] Energy: {:4.2} | Reward: {:5.2} | {} {}",
            agent.get_label(),
            energy,
            agent.get_reward(),
            status,
            bar
        );
    }
    let _ = writeln!(
        log_file,
        "{},{:.4},{},{:.4},{},{},{:.4},{},{}",
        tick, demand, pop, setpoint, alignment, hits, reward, lost, event
    );
    *last_lines = 3 + agents.len();
    io::stdout().flush().unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine_config = EngineConfig::load("config.toml").unwrap_or_else(|_| {
        println!("[!] config.toml not found, using default config.");
        EngineConfig::default_config()
    });

    let sim_config = SimulationConfig::load("config.toml")
        .unwrap_or_else(|_| SimulationConfig::default_config());

    let pop_config = AgentConfig::builder()
        .max_population(sim_config.max_population)
        .homeostasis_threshold(2)
        .min_energy(sim_config.min_energy)
        .immunity_period(sim_config.immunity_period)
        .build();

    let mut manager = PopulationManager::builder().config(pop_config).build();

    let mut state = SwarmState {
        total_demand: 5.0,
        ..Default::default()
    };

    let checkpoint_file = "examples/swarm_checkpoint.bin";
    if let Ok(loaded) = BinarySerializer::load::<SwarmState>(checkpoint_file) {
        println!("[+] Checkpoint loaded (tick {})", loaded.tick);
        state = loaded;
    } else {
        println!("[!] Checkpoint not found, starting from scratch.");
        for i in 0..3 {
            manager.add_agent(Box::new(SwarmUnit::new(
                &format!("UNIT-{}", i + 1),
                i as u32,
            )));
        }
    }

    let log_path = "examples/swarm_management.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to create log file");
    let _ = writeln!(
        log_file,
        "tick,demand,population,setpoint,alignment,hits,reward,lost,event"
    );

    let mut last_lines = 0;

    println!("[!] Swarm Fleet Management started.");
    println!("Log: {}", log_path);
    println!("----------------------------------------------------------------------");
    print!("\x1B[?25l");
    io::stdout().flush().unwrap();

    loop {
        if state.tick >= engine_config.max_ticks {
            break;
        }
        state.tick += 1;

        let demand_cycle = 5.0 + 3.0 * (state.tick as f64 * 0.01).sin();
        state.total_demand = (demand_cycle + (rand::thread_rng().gen::<f64>() - 0.5) * 1.5).max(1.0);

        let pop = manager.len().max(1);
        let current_setpoint = state.total_demand / pop as f64;
        let mut event = "NORMAL";

        if state.tick > 0 && state.tick.is_multiple_of(700) && manager.len() > 2 {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..manager.len());
            manager.agents.remove(idx);
            state.total_lost += 1;
            event = "HARDWARE FAILURE";
        }

        if state.tick > 0 && state.tick.is_multiple_of(1000) && manager.len() < sim_config.max_population {
            let new_units = 2;
            for _ in 0..new_units {
                state.total_spawned += 1;
                manager.add_agent(Box::new(SwarmUnit::new(
                    &format!("UNIT-{}", state.total_spawned),
                    state.tick as u32,
                )));
            }
            event = "FLEET EXPANSION";
        }

        let inertia = 0.1;
        let impacts = manager.process_all_agents_parallel(current_setpoint, inertia);
        state.alignment = calculate_alignment_score(&impacts, current_setpoint);

        thread::sleep(Duration::from_millis(10));

        if state.alignment > 0.85 {
            state.hits += 1;
            state.total_reward += 0.001;
        }

        manager.run_evolution(state.tick as u32);

        if state.tick.is_multiple_of(engine_config.snapshot_interval) {
            let _ = BinarySerializer::save(checkpoint_file, &state);
            draw_dashboard(
                state.tick,
                state.total_demand,
                current_setpoint,
                state.alignment,
                manager.len(),
                state.hits,
                state.total_reward,
                state.total_lost,
                &manager.agents,
                &mut log_file,
                &mut last_lines,
                event,
            );
        }
    }

    let _ = BinarySerializer::save(checkpoint_file, &state);
    print!("\x1B[?25h");
    println!("\n[✓] Swarm simulation completed.");
    println!(
        "Ticks: {} | Hits: {} | Final alignment: {:.3}",
        state.tick, state.hits, state.alignment
    );
    println!("Total units spawned: {}, Total lost: {}", state.total_spawned, state.total_lost);
    println!("\nTo generate charts, run:");
    println!("python tools/visualize.py {}", log_path);

    Ok(())
}