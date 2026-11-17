//! Core kernel invariants: topology mass conservation, parallel vs sequential
//! bit-exact determinism, planetary physics coupling, and extensible registries.

use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use pbma_core::grid::coords::{MOORE_WEIGHT_SUM, MOORE_WEIGHTS, Topology};
use pbma_core::model::{
    EnvState, GasDef, GasId, PositiveF64, Species, SpeciesId, SpeciesTraits, TraitDef, TraitId,
    ozone_net_delta, surface_uv_fraction,
};
use pbma_core::ports::{
    BehaviorDelta, CellContext, EnvironmentStep, Flux, MigrationPolicy, PopulationBehavior,
    Recorder,
};
use pbma_core::{TickDriver, World, WorldBuilder};

struct TestDiffusion {
    alpha: f64,
    couple_insolation_and_ozone: bool,
}

impl EnvironmentStep for TestDiffusion {
    fn step(&self, cell: &CellContext<'_>, out: &mut EnvState) {
        out.copy_from(cell.env);
        let alpha = self.alpha.clamp(0.0, 1.0 / MOORE_WEIGHT_SUM);

        for (i, neighbor) in cell.neighbors.iter().enumerate() {
            let weight = MOORE_WEIGHTS[i];
            for gas in cell.gases.ids() {
                let own = cell.env.gas(gas).value();
                let other = neighbor.gas(gas).value();
                let flux = alpha * weight * (other - own);
                out.set_gas(gas, PositiveF64::new(out.gas(gas).value() + flux));
            }
        }

        if self.couple_insolation_and_ozone {
            out.light = cell.insolation.normalized_light;
            let o2 = out.gas(GasId::O2).value();
            let o3 = out.gas(GasId::O3).value();
            let uv_frac = surface_uv_fraction(out, cell.gases);
            out.uv = cell.insolation.toa_uv_wm2 * uv_frac;
            let do3 = ozone_net_delta(
                o2,
                o3,
                cell.insolation.toa_uv_wm2,
                cell.params.planet.magnetic_field_strength,
                0.1,
            );
            out.set_gas(GasId::O3, PositiveF64::new(o3 + do3));
        }
    }
}

struct SimpleGrowth;

impl PopulationBehavior for SimpleGrowth {
    fn update(&self, species: SpeciesId, cell: &CellContext<'_>) -> BehaviorDelta {
        let Some((_, pop)) = cell
            .populations
            .iter()
            .find(|(id, _)| id.raw == species.raw)
        else {
            return BehaviorDelta { delta: 0.0 };
        };
        let Ok(def) = cell.registry.get(species) else {
            return BehaviorDelta { delta: 0.0 };
        };
        let amount = pop.amount.value();
        let rate = def.traits.growth_rate() * cell.env.light * (1.0 - amount / 200.0);
        BehaviorDelta {
            delta: amount * rate,
        }
    }
}

struct CardinalMigration;

impl MigrationPolicy for CardinalMigration {
    fn fluxes(&self, cell: &CellContext<'_>, out: &mut Vec<Flux>) {
        for (id, pop) in cell.populations {
            let Ok(def) = cell.registry.get(*id) else {
                continue;
            };
            let thresh = def.traits.crowding_threshold();
            if pop.amount.value() > thresh {
                let excess = (pop.amount.value() - thresh) * 0.25;
                for dir in pbma_core::grid::coords::VON_NEUMANN {
                    out.push(Flux {
                        from_x: cell.x,
                        from_y: cell.y,
                        direction: dir,
                        species: *id,
                        amount: excess,
                    });
                }
            }
        }
    }
}

struct HashRecorder {
    hasher: Rc<RefCell<std::collections::hash_map::DefaultHasher>>,
}

impl Recorder for HashRecorder {
    fn record(&mut self, tick: u64, world: &World) {
        let mut h = self.hasher.borrow_mut();
        tick.hash(&mut *h);
        for (x, y, cell) in world.grid().iter_row_major() {
            x.hash(&mut *h);
            y.hash(&mut *h);
            cell.env.temperature.to_bits().hash(&mut *h);
            cell.env.light.to_bits().hash(&mut *h);
            cell.env.uv.to_bits().hash(&mut *h);
            for (_, amt) in cell.env.gases.iter() {
                amt.value().to_bits().hash(&mut *h);
            }
            for (sid, pop) in &cell.populations {
                sid.raw.hash(&mut *h);
                pop.amount.value().to_bits().hash(&mut *h);
            }
        }
    }
}

#[test]
fn diffusion_conserves_gas_mass_across_all_topologies() {
    for topology in [Topology::Clamped, Topology::Planetary, Topology::Toroidal] {
        let mut world = WorldBuilder::new(17, 13)
            .topology(topology)
            .init_env(|x, y, _, _, _, _, env| {
                env.set_gas(GasId::O2, PositiveF64::new(10.0 + f64::from(x * 3 + y)));
                env.set_gas(GasId::CO2, PositiveF64::new(0.5 + f64::from(y)));
            })
            .build()
            .unwrap();

        let initial_mass: f64 = world.cells().iter().map(|c| c.env.total_mass()).sum();
        let hasher = Rc::new(RefCell::new(
            std::collections::hash_map::DefaultHasher::new(),
        ));
        let mut driver = TickDriver::new(
            Box::new(TestDiffusion {
                alpha: 0.14,
                couple_insolation_and_ozone: false,
            }),
            Box::new(SimpleGrowth),
            Box::new(CardinalMigration),
            Box::new(HashRecorder {
                hasher: Rc::clone(&hasher),
            }),
        );

        for t in 0..25 {
            driver.tick(&mut world, t);
        }

        let final_mass: f64 = world.cells().iter().map(|c| c.env.total_mass()).sum();
        assert!(
            (initial_mass - final_mass).abs() < 1e-8,
            "mass drift on {topology:?}: {initial_mass} vs {final_mass}"
        );
    }
}

#[test]
fn parallel_and_sequential_execution_produce_bit_identical_traces() {
    let build_world = || {
        // Use grid larger than DEFAULT_CHUNK_SIZE (64) to exercise multi-chunk parallel splitting.
        WorldBuilder::new(70, 68)
            .topology(Topology::Planetary)
            .latitudinal_temperature(305.0, 260.0)
            .init_env(|x, y, _, _, _, _, env| {
                env.set_gas(GasId::O2, PositiveF64::new(15.0 + f64::from(x % 7)));
                env.set_gas(GasId::O3, PositiveF64::new(0.05 * f64::from(y % 5)));
            })
            .seed_species(
                Species::new("alpha", SpeciesTraits::new(295.0, 15.0, 0.2, 60.0)),
                |x, y, _, _| if (x + y) % 5 == 0 { 80.0 } else { 15.0 },
            )
            .build()
            .unwrap()
    };

    let run = |parallel: bool| -> u64 {
        let mut world = build_world();
        let hasher = Rc::new(RefCell::new(
            std::collections::hash_map::DefaultHasher::new(),
        ));
        let mut driver = TickDriver::new(
            Box::new(TestDiffusion {
                alpha: 0.12,
                couple_insolation_and_ozone: true,
            }),
            Box::new(SimpleGrowth),
            Box::new(CardinalMigration),
            Box::new(HashRecorder {
                hasher: Rc::clone(&hasher),
            }),
        )
        .with_parallel(parallel);

        for t in 0..10 {
            driver.tick(&mut world, t);
        }
        hasher.borrow().finish()
    };

    assert_eq!(run(false), run(true));
}

#[test]
fn planetary_migration_wraps_east_west() {
    let mut world = WorldBuilder::new(6, 3)
        .topology(Topology::Planetary)
        .seed_species_at(
            Species::new("migrant", SpeciesTraits::new(288.0, 10.0, 0.0, 50.0)),
            0,
            1,
            250.0,
        )
        .build()
        .unwrap();

    let hasher = Rc::new(RefCell::new(
        std::collections::hash_map::DefaultHasher::new(),
    ));
    let mut driver = TickDriver::new(
        Box::new(TestDiffusion {
            alpha: 0.0,
            couple_insolation_and_ozone: false,
        }),
        Box::new(SimpleGrowth),
        Box::new(CardinalMigration),
        Box::new(HashRecorder { hasher }),
    );
    driver.tick(&mut world, 0);

    // Westward flux from x=0 must wrap to x=5 at y=1.
    let wrapped = world.cell(5, 1).unwrap().populations[0].1.amount.value();
    assert!(
        (wrapped - 50.0).abs() < 1e-9,
        "expected 50.0 wrapped, got {wrapped}"
    );
}

#[test]
fn extensible_gases_and_traits_work_end_to_end() {
    let mut neon = GasId(0);
    let mut salinity_tol = TraitId(0);

    let mut world = WorldBuilder::new(4, 4)
        .register_gas(
            GasDef {
                symbol: "Ne".into(),
                name: "Neon".into(),
                molar_mass_kg: 0.020_179,
                greenhouse_factor: 0.0,
                uv_absorption: 0.0,
                relative_diffusivity: 1.3,
            },
            Some(&mut neon),
        )
        .register_trait(
            TraitDef {
                key: "sal".into(),
                description: "Salinity tolerance".into(),
                default_value: 0.5,
            },
            Some(&mut salinity_tol),
        )
        .init_env(move |x, _, _, _, _, _, env| {
            if x == 0 {
                env.set_gas(neon, PositiveF64::new(16.0));
            }
        })
        .seed_species_everywhere(
            Species::new(
                "halophile",
                SpeciesTraits::default().with(salinity_tol, 0.95),
            ),
            10.0,
        )
        .build()
        .unwrap();

    let mut driver = TickDriver::new(
        Box::new(TestDiffusion {
            alpha: 0.14,
            couple_insolation_and_ozone: false,
        }),
        Box::new(SimpleGrowth),
        Box::new(CardinalMigration),
        Box::new(HashRecorder {
            hasher: Rc::new(RefCell::new(
                std::collections::hash_map::DefaultHasher::new(),
            )),
        }),
    );

    driver.tick(&mut world, 0);

    // Neon must have diffused into (1, 0) while preserving total Neon mass.
    assert!(world.cell(1, 0).unwrap().env.gas(neon).value() > 0.0);
    let total_ne: f64 = world.cells().iter().map(|c| c.env.gas(neon).value()).sum();
    assert!((total_ne - 64.0).abs() < 1e-9);

    // Taxon descriptor includes the runtime-registered trait.
    let species_def = world.registry().iter().next().unwrap().1;
    assert!(
        species_def
            .traits
            .taxon(world.trait_schema())
            .contains("sal:0.95")
    );
}
