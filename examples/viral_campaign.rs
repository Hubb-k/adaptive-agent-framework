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
struct ViralState {
    tick: u64,
    base_target: f64,
    inertia: f64,
    alignment: f64,
    hits: u64,
    total_reward: f64,
    total_spawned: usize,
}

impl Default for ViralState {
    fn default() -> Self {
        Self {
            tick: 0,
            base_target: 0.5,
            inertia: 0.1,
            alignment: 0.0,
            hits: 0,
            total_reward: 0.0,
            total_spawned: 5,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ContentChannel {
    id: String,
    style: f64,
    energy: f64,
    reward: f64,
    birth_tick: u32,
    learning_rate: f64,
}

impl ContentChannel {
    fn new(id: &str, tick: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: id.to_string(),
            style: rng.gen_range(0.0..std::f64::consts::TAU),
            energy: 1.0,
            reward: 0.0,
            birth_tick: tick,
            learning_rate: 0.8 + rng.gen_range(0.0..0.4),
        }
    }
}

impl Agent for ContentChannel {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        let impact = (self.style.sin() + 1.0) / 2.0;
        let error = (impact - setpoint).abs();
        self.energy = (self.energy - error.powi(2) * inertia * 0.01).max(0.0);
        if error < 0.3 {
            self.energy = (self.energy + 0.03).min(1.5);
            self.reward += 0.1;
        }
        let style_correction = (setpoint - impact) * self.learning_rate * 0.15;
        let noise = (rand::thread_rng().gen::<f64>() - 0.5) * 0.03;
        self.style += style_correction + noise;
        self.style = self.style.rem_euclid(std::f64::consts::TAU);
        impact
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
    target: f64,
    alignment: f64,
    inertia: f64,
    pop: usize,
    hits: u64,
    reward: f64,
    agents: &[Box<dyn Agent>],
    log_file: &mut std::fs::File,
    last_lines: &mut usize,
    event: &str,
) {
    if *last_lines > 0 {
        print!("\x1B[{}A", *last_lines);
    }
    print!("\x1B[J");
    println!("=== VIRAL CAMPAIGN MONITOR ===");
    println!(
        "Tick: {:<6} | Target: {:.3} | Align: {:.3} | Inertia: {:.3} | Pop: {:>2} | Hits: {:>4} | {}",
        tick, target, alignment, inertia, pop, hits, event
    );
    println!("----------------------------------------------------------------------");
    for agent in agents {
        let energy = agent.get_energy();
        let status = if energy > 0.8 {
            "\x1B[32mTRENDING\x1B[0m"
        } else if energy > 0.3 {
            "\x1B[33mACTIVE  \x1B[0m"
        } else {
            "\x1B[31mDYING   \x1B[0m"
        };
        let bar_width = ((energy * 20.0) as usize).min(20);
        let bar: String = "█".repeat(bar_width) + &"░".repeat(20 - bar_width);
        println!(
            "[{:10}] Energy: {:4.2} | Reward: {:5.2} | {} {}",
            agent.get_label(),
            energy,
            agent.get_reward(),
            status,
            bar
        );
    }
    let _ = writeln!(
        log_file,
        "{},{:.4},{:.4},{:.4},{},{},{:.4},{}",
        tick, target, alignment, inertia, pop, hits, reward, event
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
        .homeostasis_threshold(sim_config.homeostasis_threshold)
        .min_energy(sim_config.min_energy)
        .immunity_period(sim_config.immunity_period)
        .build();

    let mut manager = PopulationManager::builder().config(pop_config).build();

    let mut state = ViralState {
        base_target: sim_config.initial_setpoint,
        inertia: sim_config.initial_inertia,
        ..Default::default()
    };

    let checkpoint_file = "examples/viral_checkpoint.bin";
    if let Ok(loaded) = BinarySerializer::load::<ViralState>(checkpoint_file) {
        println!("[+] Checkpoint loaded (tick {})", loaded.tick);
        state = loaded;
    } else {
        println!("[!] Checkpoint not found, starting from scratch.");
        let channel_names = ["TikTok", "Instagram", "Twitter", "YouTube", "LinkedIn"];
        for (i, name) in channel_names.iter().enumerate() {
            manager.add_agent(Box::new(ContentChannel::new(name, i as u32)));
        }
    }

    let log_path = "examples/viral_campaign.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to create log file");
    let _ = writeln!(
        log_file,
        "tick,target,alignment,inertia,population,hits,reward,event"
    );

    let mut last_lines = 0;

    println!("[!] Viral Campaign Domain started.");
    println!("Log: {}", log_path);
    println!("----------------------------------------------------------------------");
    print!("\x1B[?25l");
    io::stdout().flush().unwrap();

    loop {
        if state.tick >= engine_config.max_ticks {
            break;
        }
        state.tick += 1;

        let daily_cycle = 0.15 * (state.tick as f64 * 0.026).sin();
        let mut current_target = state.base_target + daily_cycle;
        let mut event = "NORMAL";

        if state.tick > 0 && state.tick.is_multiple_of(250) {
            let severity = rand::thread_rng().gen::<f64>();
            if severity < 0.6 {
                current_target = 0.85 + rand::thread_rng().gen::<f64>() * 0.1;
                event = "VIRAL TREND";
            } else if severity < 0.9 {
                current_target = 0.15 + rand::thread_rng().gen::<f64>() * 0.15;
                event = "ALGORITHM CHANGE";
            } else {
                current_target = rand::thread_rng().gen_range(0.0..1.0);
                event = "COMPETITOR ATTACK";
            }
        }

        let noise = (rand::thread_rng().gen::<f64>() - 0.5) * 0.25;
        current_target = (current_target + noise).clamp(0.05, 0.95);

        let clamped = state.alignment.clamp(0.0, 1.0);
        state.inertia += ((1.05 - clamped).powi(2) - state.inertia) * 0.5;
        state.inertia = state.inertia.clamp(0.01, 2.0);

        let impacts = manager.process_all_agents_parallel(current_target, state.inertia);
        state.alignment = calculate_alignment_score(&impacts, current_target);

        thread::sleep(Duration::from_millis(10));

        if state.alignment > 0.85 {
            state.hits += 1;
            state.total_reward += 0.001;
        }

        manager.run_evolution(state.tick as u32);

        while manager.needs_homeostasis() && manager.len() < sim_config.max_population {
            state.total_spawned += 1;
            manager.add_agent(Box::new(ContentChannel::new(
                &format!("Channel-{}", state.total_spawned),
                state.tick as u32,
            )));
        }

        if state.tick.is_multiple_of(engine_config.snapshot_interval) {
            let _ = BinarySerializer::save(checkpoint_file, &state);
            draw_dashboard(
                state.tick,
                current_target,
                state.alignment,
                state.inertia,
                manager.len(),
                state.hits,
                state.total_reward,
                &manager.agents,
                &mut log_file,
                &mut last_lines,
                event,
            );
        }
    }

    let _ = BinarySerializer::save(checkpoint_file, &state);
    print!("\x1B[?25h");
    println!("\n[✓] Campaign completed.");
    println!(
        "Ticks: {} | Hits: {} | Final alignment: {:.3}",
        state.tick, state.hits, state.alignment
    );
    println!("\nTo generate charts, run:");
    println!("python tools/visualize.py {}", log_path);

    Ok(())
}