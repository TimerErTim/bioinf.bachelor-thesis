//! Full-tick benchmark at ~1M cells (1000x1000 world).

use std::hint::black_box;

use pbma_core::grid::coords::Topology;
use pbma_core::model::{GasId, Species, SpeciesTraits};
use pbma_core::{TickDriver, World, WorldBuilder};
use pbma_sim::{DiffusionStep, LogisticBehavior, OvercrowdingMigration, VecRecorder};

/// Builds a 1000x1000 world with one seeded species.
fn world_1m() -> World {
    WorldBuilder::new(1000, 1000)
        .topology(Topology::Planetary)
        .diffusion(0.14)
        .temperature_and_light(300.0, 0.5)
        .gas(GasId::O2, 21.0)
        .gas(GasId::CO2, 0.04)
        .gas(GasId::H2O, 1.0)
        .seed_species_everywhere(
            Species::new("bench", SpeciesTraits::new(300.0, 10.0, 0.1, 100.0)),
            10.0,
        )
        .build()
        .expect("valid benchmark world")
}

fn main() {
    let mut world = black_box(world_1m());
    let recorder = std::rc::Rc::new(std::cell::RefCell::new(VecRecorder::default()));
    struct RcRecorder(std::rc::Rc<std::cell::RefCell<VecRecorder>>);
    impl pbma_core::ports::Recorder for RcRecorder {
        fn record(&mut self, tick: u64, world: &World) {
            self.0.borrow_mut().record(tick, world);
        }
    }
    let mut driver = TickDriver::new(
        Box::new(DiffusionStep::new(0.14)),
        Box::new(LogisticBehavior),
        Box::new(OvercrowdingMigration),
        Box::new(RcRecorder(std::rc::Rc::clone(&recorder))),
    );

    // warmup
    driver.tick(&mut world, 0);

    let start = std::time::Instant::now();
    let ticks = 10u32;
    for t in 1..=ticks {
        driver.tick(&mut world, u64::from(t));
    }
    let per_tick = start.elapsed() / ticks;
    println!("per-tick time over 1M cells: {per_tick:?}");
}
