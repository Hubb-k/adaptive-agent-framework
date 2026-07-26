use agent_core::{Agent, AgentConfig, BaseAgentState, PopulationManager};
use math_core::calculate_alignment_score;
use rand::Rng;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

#[derive(Serialize)]
struct SimulationReport {
    domain: String,
    total_ticks: u64,
    final_alignment: f64,
    resonance_hits: u64,
    hit_rate_percent: f64,
    metrics: AlignmentMetrics,
    population_stats: PopulationStats,
}

#[derive(Serialize)]
struct AlignmentMetrics {
    mae: f64,
    rmse: f64,
    min_alignment: f64,
    max_alignment: f64,
}

#[derive(Serialize)]
struct PopulationStats {
    final_count: usize,
    total_spawned: usize,
    avg_learning_rate: f64,
    min_learning_rate: f64,
    max_learning_rate: f64,
}

struct Substation {
    id: String,
    phase_angle: f64,
    base: BaseAgentState,
}

impl Substation {
    fn new(id: &str, tick: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: id.to_string(),
            phase_angle: rng.gen_range(0.0..std::f64::consts::TAU),
            base: BaseAgentState::new(tick, 0.8 + rng.gen_range(0.0..0.4)),
        }
    }
}

impl Agent for Substation {
    fn process_tick(&mut self, setpoint: f64, inertia: f64) -> f64 {
        let impact = (self.phase_angle.sin() + 1.0) / 2.0;
        let _error = self.base.apply_feedback(impact, setpoint, inertia);

        let phase_correction = (setpoint - impact) * self.base.learning_rate * 0.15;
        let noise = (rand::thread_rng().gen::<f64>() - 0.5) * 0.03;
        self.phase_angle += phase_correction + noise;
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
        &self.id
    }
    fn get_birth_tick(&self) -> u32 {
        self.base.birth_tick
    }

    fn mutate(&mut self) {
        use rand::Rng;
        self.base.learning_rate =
            (self.base.learning_rate + rand::thread_rng().gen_range(-0.2..0.2)).clamp(0.1, 3.0);
        self.base.mutate_energy_and_reward();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn draw_dashboard(
    tick: u64,
    target: f64,
    alignment: f64,
    inertia: f64,
    pop: usize,
    hits: u64,
    reward: f64,
    substations: &[&Substation],
    log_file: &mut std::fs::File,
    last_lines: &mut usize,
    event: &str,
) {
    if *last_lines > 0 {
        print!("\x1B[{}A", *last_lines);
    }
    print!("\x1B[J");

    println!("=== GRID STABILITY MONITOR (REALISTIC LOAD) ===");
    println!(
        "Tick: {:<6} | Target: {:.3} | Align: {:.3} | Inertia: {:.3} | Pop: {:>2} | Hits: {:>4} | {}",
        tick, target, alignment, inertia, pop, hits, event
    );
    println!("----------------------------------------------------------------------");

    for sub in substations {
        let status = if sub.base.energy > 0.8 {
            "\x1B[32mONLINE  \x1B[0m"
        } else if sub.base.energy > 0.3 {
            "\x1B[33mSTRESS  \x1B[0m"
        } else {
            "\x1B[31mCRITICAL\x1B[0m"
        };

        let bar_width = ((sub.base.energy * 20.0) as usize).min(20);
        let bar: String = "█".repeat(bar_width) + &"░".repeat(20 - bar_width);

        println!(
            "[{:8}] Energy: {:4.2} | LR: {:4.2} | Phase: {:6.2} | {} {}",
            sub.id, sub.base.energy, sub.base.learning_rate, sub.phase_angle, status, bar
        );
    }

    let _ = writeln!(
        log_file,
        "{},{:.4},{:.4},{:.4},{},{},{:.4},{}",
        tick, target, alignment, inertia, pop, hits, reward, event
    );

    *last_lines = 3 + substations.len();
    io::stdout().flush().unwrap();
}

fn main() {
    let log_path = "examples/grid_stability.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Не удалось создать лог");
    let _ = writeln!(
        log_file,
        "tick,target,alignment,inertia,population,hits,reward,event"
    );

    let pop_config = AgentConfig::default();
    let mut manager = PopulationManager::new(pop_config);
    for i in 0..5 {
        manager.add_agent(Box::new(Substation::new(
            &format!("SUB-{}", i + 1),
            i as u32,
        )));
    }

    let mut tick: u64 = 0;
    let base_setpoint: f64 = 0.5;
    let mut inertia: f64 = 0.1;
    let mut alignment: f64 = 0.0;
    let mut hits: u64 = 0;
    let mut total_reward: f64 = 0.0;
    let max_ticks: u64 = 10000;
    let mut last_lines = 0;

    let mut min_align = 1.0;
    let mut max_align = 0.0;
    let mut total_spawned: usize = 5;

    println!("[!] Grid Stability Domain запущен (REALISTIC LOAD MODE).");
    println!("Лог: {}", log_path);
    println!("----------------------------------------------------------------------");

    print!("\x1B[?25l");
    io::stdout().flush().unwrap();

    loop {
        if tick >= max_ticks {
            break;
        }
        tick += 1;

        let daily_cycle = 0.15 * (tick as f64 * 0.026).sin();
        let mut current_setpoint = base_setpoint + daily_cycle;
        let mut event = "NORMAL";

        if tick > 0 && tick % 250 == 0 {
            let severity = rand::thread_rng().gen::<f64>();
            if severity < 0.7 {
                current_setpoint += (rand::thread_rng().gen::<f64>() - 0.5) * 0.6;
                event = "DISTURBANCE";
            } else if severity < 0.95 {
                current_setpoint += (rand::thread_rng().gen::<f64>() - 0.5) * 1.0;
                event = "CASCADE FAILURE";
            } else {
                current_setpoint = rand::thread_rng().gen_range(0.0..1.0);
                event = "BLACK SWAN ⚡";
            }
        }

        let noise = (rand::thread_rng().gen::<f64>() - 0.5) * 0.25;
        current_setpoint = (current_setpoint + noise).clamp(0.05, 0.95);

        let clamped = alignment.clamp(0.0, 1.0);
        inertia += ((1.05 - clamped).powi(2) - inertia) * 0.5;
        inertia = inertia.clamp(0.01, 2.0);

        let impacts = manager.process_all_agents_parallel(current_setpoint, inertia);
        alignment = calculate_alignment_score(&impacts, current_setpoint);

        if alignment < min_align {
            min_align = alignment;
        }
        if alignment > max_align {
            max_align = alignment;
        }

        thread::sleep(Duration::from_millis(10));

        if alignment > 0.85 {
            hits += 1;
            total_reward += 0.001;
        }

        manager.run_evolution(tick as u32);
        if manager.needs_homeostasis() {
            manager.add_agent(Box::new(Substation::new(
                &format!("SUB-{}", tick),
                tick as u32,
            )));
            total_spawned += 1;
        }

        if tick % 100 == 0 {
            let subs: Vec<&Substation> = manager
                .agents
                .iter()
                .map(|a| a.as_any().downcast_ref::<Substation>().unwrap())
                .collect();
            draw_dashboard(
                tick,
                current_setpoint,
                alignment,
                inertia,
                manager.len(),
                hits,
                total_reward,
                &subs,
                &mut log_file,
                &mut last_lines,
                event,
            );
        }
    }

    let mae = (1.0 - alignment).abs();
    let rmse = ((1.0 - alignment).powi(2)).sqrt();

    let mut total_lr = 0.0;
    let mut min_lr = f64::MAX;
    let mut max_lr = f64::MIN;

    for agent in &manager.agents {
        if let Some(sub) = agent.as_any().downcast_ref::<Substation>() {
            let lr = sub.base.learning_rate;
            total_lr += lr;
            if lr < min_lr {
                min_lr = lr;
            }
            if lr > max_lr {
                max_lr = lr;
            }
        }
    }

    let pop_count = manager.len();
    let avg_lr = if pop_count > 0 {
        total_lr / pop_count as f64
    } else {
        0.0
    };

    let report = SimulationReport {
        domain: "Grid Stability".to_string(),
        total_ticks: tick,
        final_alignment: alignment,
        resonance_hits: hits,
        hit_rate_percent: (hits as f64 / tick as f64) * 100.0,
        metrics: AlignmentMetrics {
            mae,
            rmse,
            min_alignment: min_align,
            max_alignment: max_align,
        },
        population_stats: PopulationStats {
            final_count: pop_count,
            total_spawned,
            avg_learning_rate: avg_lr,
            min_learning_rate: min_lr,
            max_learning_rate: max_lr,
        },
    };

    let report_path = "examples/analysis_report_grid.json";
    let file = File::create(report_path).expect("Не удалось создать файл отчета");
    serde_json::to_writer_pretty(file, &report).expect("Не удалось записать JSON");

    print!("\x1B[?25h");
    println!("\n[✓] Симуляция завершена.");
    println!(
        "Тиков: {} | Hits: {} | Финальный alignment: {:.3}",
        tick, hits, alignment
    );
    println!("[+] Аналитический отчет сохранен: {}", report_path);
}
