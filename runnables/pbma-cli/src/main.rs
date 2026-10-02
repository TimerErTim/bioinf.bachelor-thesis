//! Headless PBMA runner: config -> tick loop -> CSV metrics.

use std::io::Write;
use std::path::PathBuf;

use pbma_core::{TickDriver, World};
use pbma_sim::{DiffusionStep, LogisticBehavior, OvercrowdingMigration, VecRecorder};
use serde::Deserialize;

/// Simulation configuration, loaded from a TOML file or built from CLI args.
/// Missing fields fall back to [`Config::default`] values via `#[serde(default)]`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// World width in cells.
    pub width: u32,
    /// World height in cells.
    pub height: u32,
    /// Number of ticks to simulate.
    pub ticks: u64,
    /// Seed for deterministic runs (reserved for stochastic phases).
    pub seed: u64,
    /// Diffusion fraction per neighbor pair, clamped to [0, 0.25].
    pub diffusion: f64,
    /// Initial temperature in kelvin for all cells.
    pub initial_temperature: f64,
    /// Initial light intensity for all cells, in [0, 1].
    pub initial_light: f64,
    /// Optional CSV output path; stdout when absent.
    pub output: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            ticks: 100,
            seed: 42,
            diffusion: 0.2,
            initial_temperature: 300.0,
            initial_light: 0.5,
            output: None,
        }
    }
}

/// Entry: parse `--config path.toml` or defaults, run ticks, print CSV.
fn main() {
    let mut config = Config::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--config" {
            let path = args.next().unwrap_or_else(|| die("--config needs a path"));
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| die(&format!("read {path:?}: {e}")));
            config = toml::from_str(&raw).unwrap_or_else(|e| die(&format!("parse {path:?}: {e}")));
        } else if arg == "--help" || arg == "-h" {
            print_help();
            return;
        } else {
            die(&format!("unknown argument {arg:?} (try --help)"));
        }
    }

    run(config);
}

fn run(config: Config) {
    let params = pbma_model::GlobalParams {
        gravity: 9.81,
        day_length_ticks: 240,
        axial_tilt: 0.0,
        diffusion_coefficient: config.diffusion,
    };
    let initial = pbma_model::EnvState {
        gases: [
            pbma_model::Amount::new(21.0), // O2
            pbma_model::Amount::new(0.04), // CO2
            pbma_model::Amount::ZERO,      // CH4
            pbma_model::Amount::new(1.0),  // H2O
        ],
        temperature: config.initial_temperature,
        light: config.initial_light,
    };
    let mut world = World::new(config.width, config.height, initial, params);

    // Seed one species evenly across the world.
    let species = pbma_model::Species {
        name: "pioneer".into(),
        traits: pbma_model::SpeciesTraits {
            optimal_temperature: config.initial_temperature,
            temperature_tolerance: 10.0,
            growth_rate: 0.1,
            crowding_threshold: 100.0,
        },
    };
    let species_id = match world.registry_mut().insert(species) {
        Ok(id) => id,
        Err(e) => die(&format!("register species: {e}")),
    };
    let seed_population = pbma_model::Population {
        amount: pbma_model::Amount::new(10.0),
    };
    for cell in world.cells_mut() {
        cell.populations.push((species_id, seed_population));
    }
    cell_sort(&mut world);

    struct RcRecorder(std::rc::Rc<std::cell::RefCell<VecRecorder>>);
    impl pbma_core::ports::Recorder for RcRecorder {
        fn record(&mut self, tick: u64, metric: &str, value: f64) {
            self.0.borrow_mut().record(tick, metric, value);
        }
    }
    let shared = std::rc::Rc::new(std::cell::RefCell::new(VecRecorder::default()));
    let mut driver = TickDriver::new(
        Box::new(DiffusionStep::new(config.diffusion)),
        Box::new(LogisticBehavior),
        Box::new(OvercrowdingMigration),
        Box::new(RcRecorder(std::rc::Rc::clone(&shared))),
    );

    for tick in 0..config.ticks {
        driver.tick(&mut world, tick);
    }

    // Final aggregated metrics row set + per-tick rows -> CSV.
    let mut out: Box<dyn Write> = match &config.output {
        Some(path) => Box::new(
            std::fs::File::create(path).unwrap_or_else(|e| die(&format!("create {path:?}: {e}"))),
        ),
        None => Box::new(std::io::stdout().lock()),
    };
    writeln!(out, "tick,metric,value").unwrap_or_else(|e| die(&format!("write csv: {e}")));
    for (tick, metric, value) in shared.borrow_mut().drain() {
        writeln!(out, "{tick},{metric},{value}")
            .unwrap_or_else(|e| die(&format!("write csv: {e}")));
    }
}

/// Keeps the per-cell population ordering invariant after bulk seeding.
fn cell_sort(world: &mut World) {
    for cell in world.cells_mut() {
        cell.populations.sort_by_key(|(id, _)| id.raw);
    }
}

fn print_help() {
    println!("pbma-cli --config <config.toml>");
    println!("Runs the population-based multi-agent simulation and emits CSV metrics.");
}

fn die(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(2);
}
