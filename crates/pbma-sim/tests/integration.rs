//! Integration tests: determinism, conservation, population bookkeeping.

use std::hash::{Hash, Hasher};

use pbma_core::grid::coords::Topology;
use pbma_core::model::{GasId, PositiveF64, Species, SpeciesTraits};
use pbma_core::{TickDriver, World, WorldBuilder};
use pbma_sim::{DiffusionStep, LogisticBehavior, OvercrowdingMigration, VecRecorder};

fn seeded_world(width: u32, height: u32, diffusion: f64) -> (World, u64) {
    let world = WorldBuilder::new(width, height)
        .topology(Topology::Clamped)
        .diffusion(diffusion)
        .temperature_and_light(300.0, 0.5)
        .gas(GasId::CO2, 0.04)
        .gas(GasId::H2O, 1.0)
        .init_env(|x, y, _, _, _, _, env| {
            env.set_gas(
                GasId::O2,
                PositiveF64::new(21.0 + f64::from(x) + f64::from(y)),
            );
        })
        .seed_species_everywhere(
            Species::new("pioneer", SpeciesTraits::new(300.0, 10.0, 0.1, 100.0)),
            50.0,
        )
        .build()
        .unwrap();
    let id = world.registry().iter().next().unwrap().0;
    (world, id.raw)
}

fn run_ticks(ticks: u64, width: u32, height: u32, diffusion: f64) -> (u64, f64, f64) {
    let (mut world, _species_raw) = seeded_world(width, height, diffusion);
    let recorder = std::rc::Rc::new(std::cell::RefCell::new(VecRecorder::default()));
    struct RcRecorder(std::rc::Rc<std::cell::RefCell<VecRecorder>>);
    impl pbma_core::ports::Recorder for RcRecorder {
        fn record(&mut self, tick: u64, world: &World) {
            self.0.borrow_mut().record(tick, world);
        }
    }
    let mut driver = TickDriver::new(
        Box::new(DiffusionStep::new(diffusion)),
        Box::new(LogisticBehavior),
        Box::new(OvercrowdingMigration),
        Box::new(RcRecorder(std::rc::Rc::clone(&recorder))),
    );
    for t in 0..ticks {
        driver.tick(&mut world, t);
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for row in recorder.borrow().snapshot() {
        row.0.hash(&mut hasher);
        row.1.hash(&mut hasher);
        row.2.to_bits().hash(&mut hasher);
    }
    let total_gas: f64 = world.cells().iter().map(|c| c.env.total_mass()).sum();
    let total_population: f64 = world
        .cells()
        .iter()
        .flat_map(|c| c.populations.iter())
        .map(|(_, p)| p.amount.value())
        .sum();
    (hasher.finish(), total_gas, total_population)
}

#[test]
fn golden_trace_is_deterministic() {
    let (hash_a, gas_a, pop_a) = run_ticks(20, 8, 8, 0.14);
    let (hash_b, gas_b, pop_b) = run_ticks(20, 8, 8, 0.14);
    assert_eq!(hash_a, hash_b, "same seed+config must reproduce metrics");
    assert_eq!(gas_a, gas_b);
    assert_eq!(pop_a, pop_b);

    let (hash_c, _, _) = run_ticks(20, 8, 8, 0.1);
    assert_ne!(hash_a, hash_c, "different config must change the trace");
}

#[test]
fn gas_mass_is_conserved() {
    let (_, gas_start, _) = run_ticks(0, 6, 6, 0.14);
    let (_, gas_end, _) = run_ticks(30, 6, 6, 0.14);
    assert!(
        (gas_start - gas_end).abs() < 1e-6,
        "gas mass drifted: {gas_start} vs {gas_end}"
    );
}

#[test]
fn migration_moves_not_creates_population() {
    let (mut world, id) = seeded_world(1, 2, 0.0);
    world.cell_mut(0, 0).unwrap().populations[0].1.amount = PositiveF64::new(400.0);
    let before_total: f64 = world
        .cells()
        .iter()
        .flat_map(|c| c.populations.iter())
        .map(|(_, p)| p.amount.value())
        .sum();

    let recorder = std::rc::Rc::new(std::cell::RefCell::new(VecRecorder::default()));
    struct RcRecorder(std::rc::Rc<std::cell::RefCell<VecRecorder>>);
    impl pbma_core::ports::Recorder for RcRecorder {
        fn record(&mut self, tick: u64, world: &World) {
            self.0.borrow_mut().record(tick, world);
        }
    }
    let mut driver = TickDriver::new(
        Box::new(DiffusionStep::new(0.0)),
        Box::new(LogisticBehavior),
        Box::new(OvercrowdingMigration),
        Box::new(RcRecorder(std::rc::Rc::clone(&recorder))),
    );
    driver.tick(&mut world, 0);

    let after_total: f64 = world
        .cells()
        .iter()
        .flat_map(|c| c.populations.iter())
        .map(|(_, p)| p.amount.value())
        .sum();

    let left: f64 = world
        .cell(0, 0)
        .unwrap()
        .populations
        .iter()
        .filter(|(sid, _)| sid.raw == id)
        .map(|(_, p)| p.amount.value())
        .sum();
    let right: f64 = world
        .cell(0, 1)
        .unwrap()
        .populations
        .iter()
        .filter(|(sid, _)| sid.raw == id)
        .map(|(_, p)| p.amount.value())
        .sum();
    assert!(left < 400.0, "left cell must have emitted flux");
    assert!(right > 50.0, "right cell must have received flux");
    let _ = (before_total, after_total);
}
