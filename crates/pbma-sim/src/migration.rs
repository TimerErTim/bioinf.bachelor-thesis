//! Overcrowding-driven migration.

use pbma_core::ports::{CellContext, Flux, MigrationPolicy};

/// Emits fluxes when a species' local density exceeds its own crowding
/// threshold (read from the registry).
///
/// The excess fraction moves out, split evenly over the four Von Neumann
/// directions (deterministic fixed order; edges reduce the number of valid
/// targets but not the order). Stale species ids emit nothing.
#[derive(Debug, Clone)]
pub struct OvercrowdingMigration;

impl OvercrowdingMigration {
    /// Von Neumann directions in canonical order.
    const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
}

impl MigrationPolicy for OvercrowdingMigration {
    fn fluxes(&self, cell: &CellContext<'_>, out: &mut Vec<Flux>) {
        for (species, population) in cell.populations {
            let Some(definition) = cell.registry.get(*species).ok() else {
                continue;
            };
            let threshold = definition.traits.crowding_threshold;
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
    use pbma_model::{Amount, EnvState, GlobalParams, Population, Species, SpeciesTraits};

    fn params() -> GlobalParams {
        GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
        }
    }

    fn species_with_threshold(threshold: f64) -> Species {
        Species {
            name: format!("t{threshold}"),
            traits: SpeciesTraits {
                optimal_temperature: 300.0,
                temperature_tolerance: 10.0,
                growth_rate: 0.1,
                crowding_threshold: threshold,
            },
        }
    }

    #[test]
    fn below_threshold_emits_nothing() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with_threshold(100.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(50.0),
            },
        )];
        let ctx = CellContext {
            x: 2,
            y: 2,
            env: &env,
            neighbors: [&env; 4],
            populations: &pops,
            params: &p,
            registry: &registry,
        };
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn above_threshold_emits_four_fluxes() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with_threshold(100.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(200.0),
            },
        )];
        let ctx = CellContext {
            x: 2,
            y: 2,
            env: &env,
            neighbors: [&env; 4],
            populations: &pops,
            params: &p,
            registry: &registry,
        };
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert_eq!(out.len(), 4);
        // excess = (200-100)/200 = 0.5, split 0.25 -> 25 per direction
        assert!((out[0].amount - 25.0).abs() < 1e-9);
    }

    #[test]
    fn per_species_thresholds_differ() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
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
                    amount: Amount::new(200.0),
                },
            ),
            (
                sensitive,
                Population {
                    amount: Amount::new(200.0),
                },
            ),
        ];
        let ctx = CellContext {
            x: 2,
            y: 2,
            env: &env,
            neighbors: [&env; 4],
            populations: &pops,
            params: &p,
            registry: &registry,
        };
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert_eq!(out.len(), 4, "only the sensitive species migrates");
        assert!(out.iter().all(|f| f.species.raw == sensitive.raw));
    }

    #[test]
    fn stale_species_emits_nothing() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with_threshold(10.0))
            .expect("fresh registry");
        registry.remove(id);
        let pops = [(
            id,
            Population {
                amount: Amount::new(500.0),
            },
        )];
        let ctx = CellContext {
            x: 2,
            y: 2,
            env: &env,
            neighbors: [&env; 4],
            populations: &pops,
            params: &p,
            registry: &registry,
        };
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert!(out.is_empty());
    }
}
