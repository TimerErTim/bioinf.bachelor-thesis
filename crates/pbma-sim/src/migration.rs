//! Overcrowding-driven migration.

use pbma_core::ports::{CellContext, Flux, MigrationPolicy};

/// Emits fluxes when a species' local density exceeds its own crowding
/// threshold (read from the registry).
#[derive(Debug, Clone)]
pub struct OvercrowdingMigration;

impl OvercrowdingMigration {
    /// Von Neumann directions in canonical order.
    const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
}

impl MigrationPolicy for OvercrowdingMigration {
    fn fluxes(&self, cell: &CellContext<'_>, out: &mut Vec<Flux>) {
        for (species, population) in cell.populations {
            let Ok(definition) = cell.registry.get(*species) else {
                continue;
            };
            let threshold = definition.traits.crowding_threshold();
            let amount = population.amount.value();
            if amount <= threshold {
                continue;
            }
            let excess = (amount - threshold) / amount;
            let moving = amount * excess * 0.25;
            for (dx, dy) in Self::DIRECTIONS {
                out.push(Flux {
                    from_x: cell.x,
                    from_y: cell.y,
                    direction: (dx, dy),
                    species: *species,
                    amount: moving,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbma_core::SpeciesRegistry;
    use pbma_core::model::{
        EnvState, GasRegistry, GlobalParams, Insolation, Population, PositiveF64, Species,
        SpeciesTraits, TraitSchema,
    };

    fn ctx<'a>(
        env: &'a EnvState,
        populations: &'a [(pbma_core::SpeciesId, Population)],
        params: &'a GlobalParams,
        gases: &'a GasRegistry,
        trait_schema: &'a TraitSchema,
        registry: &'a SpeciesRegistry,
    ) -> CellContext<'a> {
        CellContext {
            x: 2,
            y: 2,
            width: 5,
            height: 5,
            tick: 0,
            latitude_rad: 0.0,
            longitude_rad: 0.0,
            insolation: Insolation {
                cos_zenith: 1.0,
                toa_flux_wm2: 1361.0,
                normalized_light: 1.0,
                toa_uv_wm2: 100.0,
            },
            env,
            neighbors: [env; 8],
            populations,
            params,
            gases,
            trait_schema,
            registry,
        }
    }

    fn species_with_threshold(threshold: f64) -> Species {
        Species::new(
            format!("t{threshold}"),
            SpeciesTraits::new(300.0, 10.0, 0.1, threshold),
        )
    }

    #[test]
    fn below_threshold_emits_nothing() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with_threshold(100.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(50.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn above_threshold_emits_four_fluxes() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with_threshold(100.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(200.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert_eq!(out.len(), 4);
        assert!((out[0].amount - 25.0).abs() < 1e-9);
    }

    #[test]
    fn per_species_thresholds_differ() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let tolerant = registry
            .insert(species_with_threshold(1000.0))
            .expect("fresh registry");
        let sensitive = registry
            .insert(species_with_threshold(10.0))
            .expect("fresh registry");
        let pops = [
            (
                tolerant,
                Population {
                    amount: PositiveF64::new(200.0),
                },
            ),
            (
                sensitive,
                Population {
                    amount: PositiveF64::new(200.0),
                },
            ),
        ];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert_eq!(out.len(), 4, "only the sensitive species migrates");
        assert!(out.iter().all(|f| f.species.raw == sensitive.raw));
    }

    #[test]
    fn stale_species_emits_nothing() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with_threshold(10.0))
            .expect("fresh registry");
        registry.remove(id);
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(500.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert!(out.is_empty());
    }
}
