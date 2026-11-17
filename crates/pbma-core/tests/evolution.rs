//! Kernel-level evolution integration: pruning, variant spawning, stale-key cleanup.

use std::cell::RefCell;
use std::rc::Rc;

use pbma_core::model::{EnvState, Population, PositiveF64, Species, SpeciesId, SpeciesTraits};
use pbma_core::ports::{EvolutionEngine, Recorder, Rng};
use pbma_core::{TickDriver, World, WorldBuilder};

fn species(name: &str, growth_rate: f64) -> Species {
    Species::new(name, SpeciesTraits::new(300.0, 10.0, growth_rate, 100.0))
}

/// Fixed-sequence rng for reproducible engine tests.
struct SeqRng {
    values: Rc<RefCell<Vec<f64>>>,
}

impl SeqRng {
    fn shared(values: Vec<f64>) -> (Box<Self>, Rc<RefCell<Vec<f64>>>) {
        let rc = Rc::new(RefCell::new(values));
        (
            Box::new(Self {
                values: Rc::clone(&rc),
            }),
            rc,
        )
    }
}

impl Rng for SeqRng {
    fn next_f64(&mut self) -> f64 {
        self.values.borrow_mut().pop().unwrap_or(0.0)
    }

    fn child(&self, _salt: u64) -> Box<dyn Rng> {
        Box::new(Self {
            values: Rc::clone(&self.values),
        })
    }
}

/// Discard-all recorder for kernel tests.
struct SinkRecorder;

impl Recorder for SinkRecorder {
    fn record(&mut self, _tick: u64, _world: &World) {}
}

fn driver_with(evolution: Option<(Box<dyn EvolutionEngine>, Vec<f64>)>) -> TickDriver {
    struct NullEnvironment;
    impl pbma_core::ports::EnvironmentStep for NullEnvironment {
        fn step(&self, cell: &pbma_core::ports::CellContext<'_>, out: &mut EnvState) {
            out.copy_from(cell.env);
        }
    }
    struct NullBehavior;
    impl pbma_core::ports::PopulationBehavior for NullBehavior {
        fn update(
            &self,
            _species: SpeciesId,
            _cell: &pbma_core::ports::CellContext<'_>,
        ) -> pbma_core::ports::BehaviorDelta {
            pbma_core::ports::BehaviorDelta { delta: 0.0 }
        }
    }
    struct NullMigration;
    impl pbma_core::ports::MigrationPolicy for NullMigration {
        fn fluxes(
            &self,
            _cell: &pbma_core::ports::CellContext<'_>,
            _out: &mut Vec<pbma_core::ports::Flux>,
        ) {
        }
    }

    let mut driver = TickDriver::new(
        Box::new(NullEnvironment),
        Box::new(NullBehavior),
        Box::new(NullMigration),
        Box::new(SinkRecorder),
    );
    if let Some((engine, rng_values)) = evolution {
        let (rng, _shared) = SeqRng::shared(rng_values);
        driver.with_evolution(engine, rng);
    }
    driver
}

fn count_species(world: &World) -> usize {
    world.registry().len()
}

fn live_populations(world: &World) -> usize {
    world
        .cells()
        .iter()
        .flat_map(|c| c.populations.iter())
        .count()
}

#[test]
fn extinct_species_is_pruned_and_populations_cleared() {
    let mut world = WorldBuilder::new(2, 1)
        .temperature_and_light(300.0, 0.5)
        .build()
        .unwrap();
    let id = world
        .registry_mut()
        .insert(species("doomed", 0.0))
        .expect("fresh registry");
    assert_eq!(count_species(&world), 1);

    let mut driver = driver_with(None);
    driver.with_evolution(Box::new(PruneOnly), SeqRng::shared(vec![]).0);
    assert_eq!(live_populations(&world), 0);

    driver.tick(&mut world, 1);
    assert_eq!(count_species(&world), 0, "extinct species must be pruned");
    assert!(world.registry().get(id).is_err(), "stale key detected");
}

struct PruneOnly;

impl EvolutionEngine for PruneOnly {
    fn spawn_variants(
        &mut self,
        _tick: u64,
        _world: &World,
        _rng: &mut dyn Rng,
        _out: &mut Vec<(SpeciesId, Species)>,
    ) {
    }

    fn prune_extinct(&mut self, world: &World, out: &mut Vec<SpeciesId>) {
        for (id, _species) in world.registry().iter() {
            let alive = world
                .cells()
                .iter()
                .any(|cell| cell.populations.iter().any(|(pid, _)| pid.raw == id.raw));
            if !alive {
                out.push(id);
            }
        }
    }
}

#[test]
fn population_loss_triggers_extinction_next_tick() {
    let mut world = WorldBuilder::new(2, 1)
        .temperature_and_light(300.0, 0.0)
        .build()
        .unwrap();
    let id = world
        .registry_mut()
        .insert(species("dying", 0.0))
        .expect("fresh registry");
    world.cell_mut(0, 0).unwrap().populations.push((
        id,
        Population {
            amount: PositiveF64::new(0.0),
        },
    ));
    world.cell_mut(1, 0).unwrap().populations.push((
        id,
        Population {
            amount: PositiveF64::new(0.0),
        },
    ));

    let mut driver = driver_with(None);
    driver.with_evolution(Box::new(PruneOnly), SeqRng::shared(vec![]).0);
    driver.tick(&mut world, 0);

    assert_eq!(live_populations(&world), 0);
    assert_eq!(count_species(&world), 0);
}

#[test]
fn spawned_variant_is_registered_and_descended() {
    let mut world = WorldBuilder::new(2, 1)
        .seed_species_at(species("root", 0.1), 0, 0, 10.0)
        .build()
        .unwrap();
    let parent = world.registry().iter().next().unwrap().0;

    struct SpawnOnce {
        fired: bool,
        parent_raw: u64,
    }
    impl EvolutionEngine for SpawnOnce {
        fn spawn_variants(
            &mut self,
            tick: u64,
            world: &World,
            _rng: &mut dyn Rng,
            out: &mut Vec<(SpeciesId, Species)>,
        ) {
            if self.fired {
                return;
            }
            self.fired = true;
            let parent_id = SpeciesId {
                raw: self.parent_raw,
            };
            let Some((_, base)) = world
                .registry()
                .iter()
                .find(|(sid, _)| sid.raw == self.parent_raw)
            else {
                return;
            };
            let mut traits = base.traits.clone();
            let rate = traits.growth_rate() * 1.1;
            traits.set(pbma_core::TraitId::GROWTH_RATE, rate);
            let variant = Species::variant(format!("{}'", base.name), traits, parent_id, tick);
            out.push((parent_id, variant));
        }

        fn prune_extinct(&mut self, _world: &World, _out: &mut Vec<SpeciesId>) {}
    }

    let mut driver = driver_with(None);
    driver.with_evolution(
        Box::new(SpawnOnce {
            fired: false,
            parent_raw: parent.raw,
        }),
        SeqRng::shared(vec![]).0,
    );
    driver.tick(&mut world, 3);

    assert_eq!(count_species(&world), 2, "variant joined the registry");
    let child = world
        .registry()
        .iter()
        .find(|(_, s)| s.name == "root'")
        .expect("variant present")
        .1;
    assert!((child.traits.growth_rate() - 0.11).abs() < 1e-9);
    assert_eq!(child.parent, Some(parent));
    assert_eq!(child.spawn_tick, 3);
}
