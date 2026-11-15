//! Headless PBMA runner: config -> tick loop -> CSV metrics.

use std::io::Write;
use std::path::PathBuf;

use pbma_core::grid::coords::Topology;
use pbma_core::model::{GasId, Species, SpeciesTraits};
use pbma_core::{TickDriver, World, WorldBuilder};
use pbma_sim::{DiffusionStep, LogisticBehavior, OvercrowdingMigration, VecRecorder};
use serde::Deserialize;

/// Simulation configuration, loaded from a TOML file or built from CLI args.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// World width in cells.
    pub width: u32,
    /// World height in cells.
    pub height: u32,
    /// Number of ticks to simulate.
    pub ticks: u64,
    /// Seed for deterministic runs.
    pub seed: u64,
    /// Diffusion fraction per neighbor pair.
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
            diffusion: 0.14,
            initial_temperature: 300.0,
            initial_light: 0.5,
            output: None,
        }
    }
}

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
    let mut world = WorldBuilder::new(config.width, config.height)
        .topology(Topology::Planetary)
        .diffusion(config.diffusion)
        .temperature_and_light(config.initial_temperature, config.initial_light)
        .gas(GasId::O2, 21.0)
        .gas(GasId::CO2, 0.04)
        .gas(GasId::H2O, 1.0)
        .seed_species_everywhere(
            Species::new(
                "pioneer",
                SpeciesTraits::new(config.initial_temperature, 10.0, 0.1, 100.0),
            ),
            10.0,
        )
        .build()
        .unwrap_or_else(|e| die(&format!("build world: {e}")));

    struct RcRecorder(std::rc::Rc<std::cell::RefCell<VecRecorder>>);
    impl pbma_core::ports::Recorder for RcRecorder {
        fn record(&mut self, tick: u64, world: &World) {
            self.0.borrow_mut().record(tick, world);
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

fn print_help() {
    println!("pbma-cli --config <config.toml>");
    println!("Runs the population-based multi-agent simulation and emits CSV metrics.");
}

fn die(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(2);
}
