//! Full-tick benchmark at ~1M cells (1000x1000 world).

use std::hint::black_box;

use pbma_core::{TickDriver, World};
use pbma_model::{Amount, EnvState, GlobalParams, Population, Species, SpeciesTraits};
use pbma_sim::{DiffusionStep, LogisticBehavior, OvercrowdingMigration, VecRecorder};

/// Builds a 1000x1000 world with one seeded species.
fn world_1m() -> World {
    let params = GlobalParams {
        gravity: 9.81,
        day_length_ticks: 240,
        axial_tilt: 0.0,
        diffusion_coefficient: 0.2,
    };
    let initial = EnvState {
        gases: [
            Amount::new(21.0),
            Amount::new(0.04),
            Amount::ZERO,
            Amount::new(1.0),
        ],
        temperature: 300.0,
        light: 0.5,
    };
    let mut world = World::new(1000, 1000, initial, params);
    let id = world
        .registry_mut()
        .insert(Species {
            name: "bench".into(),
            traits: SpeciesTraits {
                optimal_temperature: 300.0,
                temperature_tolerance: 10.0,
                growth_rate: 0.1,
                crowding_threshold: 100.0,
            },
        })
        .expect("fresh registry");
    for cell in world.cells_mut() {
        cell.populations.push((
            id,
            Population {
                amount: Amount::new(10.0),
            },
        ));
        cell.populations.sort_by_key(|(sid, _)| sid.raw);
    }
    world
}

fn main() {
    let mut world = black_box(world_1m());
    let recorder = std::rc::Rc::new(std::cell::RefCell::new(VecRecorder::default()));
    struct RcRecorder(std::rc::Rc<std::cell::RefCell<VecRecorder>>);
    impl pbma_core::ports::Recorder for RcRecorder {
        fn record(&mut self, tick: u64, metric: &str, value: f64) {
            self.0.borrow_mut().record(tick, metric, value);
        }
    }
    let mut driver = TickDriver::new(
        Box::new(DiffusionStep::new(0.2)),
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
