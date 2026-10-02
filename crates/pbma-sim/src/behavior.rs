//! Population growth/decline behavior.

use pbma_core::ports::{BehaviorDelta, CellContext, PopulationBehavior};
use pbma_model::SpeciesId;

/// Logistic-style growth driven by per-species traits.
///
/// Each species' population grows toward the cell's carrying capacity
/// (derived from light). Growth rate decays linearly as the population
/// approaches capacity and is scaled by a temperature fitness factor in
/// [0, 1] computed from the species' own optimal temperature and tolerance.
/// Stale species ids (extinct lineages) contribute nothing. No randomness:
/// fully deterministic.
#[derive(Debug, Clone)]
pub struct LogisticBehavior;

/// Carrying capacity per cell at full light (vertical-slice constant).
const CAPACITY_AT_FULL_LIGHT: f64 = 100.0;

/// Temperature fitness in [0, 1]: 1.0 at the species' optimum, falling off
/// linearly to 0.0 one tolerance width away.
#[must_use]
pub fn temperature_fitness(temperature: f64, optimal: f64, tolerance: f64) -> f64 {
    if tolerance <= 0.0 {
        return if (temperature - optimal).abs() <= f64::EPSILON {
            1.0
        } else {
            0.0
        };
    }
    (1.0 - (temperature - optimal).abs() / tolerance).clamp(0.0, 1.0)
}

impl PopulationBehavior for LogisticBehavior {
    fn update(&self, species: SpeciesId, cell: &CellContext<'_>) -> BehaviorDelta {
        let Some((_, population)) = cell
            .populations
            .iter()
            .find(|(id, _)| id.raw == species.raw)
        else {
            return BehaviorDelta { delta: 0.0 };
        };
        let amount = population.amount.value();

        // Extinct lineage: population entries outlive their species until the
        // kernel prunes them; they must not grow.
        let Some(species_def) = cell.registry.get(species).ok() else {
            return BehaviorDelta { delta: 0.0 };
        };
        let traits = species_def.traits;

        let capacity = CAPACITY_AT_FULL_LIGHT * cell.env.light.max(0.0);
        let fitness = temperature_fitness(
            cell.env.temperature,
            traits.optimal_temperature,
            traits.temperature_tolerance,
        );

        if capacity <= 0.0 || amount <= 0.0 {
            return BehaviorDelta { delta: 0.0 };
        }
        let rate = traits.growth_rate * fitness * (1.0 - amount / capacity);
        BehaviorDelta {
            delta: amount * rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbma_core::SpeciesRegistry;
    use pbma_model::{Amount, EnvState, GlobalParams, Population, Species, SpeciesTraits};

    fn ctx<'a>(
        env: &'a EnvState,
        populations: &'a [(SpeciesId, Population)],
        params: &'a GlobalParams,
        registry: &'a SpeciesRegistry,
    ) -> CellContext<'a> {
        CellContext {
            x: 0,
            y: 0,
            env,
            neighbors: [env; 4],
            populations,
            params,
            registry,
        }
    }

    fn params() -> GlobalParams {
        GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
        }
    }

    fn species_with(growth_rate: f64, optimal: f64, tolerance: f64) -> Species {
        Species {
            name: format!("s{growth_rate}{optimal}{tolerance}"),
            traits: SpeciesTraits {
                optimal_temperature: optimal,
                temperature_tolerance: tolerance,
                growth_rate,
                crowding_threshold: 100.0,
            },
        }
    }

    fn env_at(temperature: f64, light: f64) -> EnvState {
        EnvState {
            gases: [Amount::ZERO; 4],
            temperature,
            light,
        }
    }

    #[test]
    fn zero_population_stays_zero() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let registry = SpeciesRegistry::new();
        let ctx = ctx(&env, &[], &p, &registry);
        assert_eq!(
            LogisticBehavior.update(SpeciesId { raw: 0 }, &ctx).delta,
            0.0
        );
    }

    #[test]
    fn absent_species_gets_no_delta() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.1, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let other = SpeciesId { raw: u64::MAX };
        let ctx = ctx(&env, &pops, &p, &registry);
        assert_eq!(LogisticBehavior.update(other, &ctx).delta, 0.0);
        assert!(LogisticBehavior.update(id, &ctx).delta > 0.0);
    }

    #[test]
    fn stale_species_gets_no_delta() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.1, 300.0, 10.0))
            .expect("fresh registry");
        registry.remove(id);
        let pops = [(
            id,
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &registry);
        assert_eq!(LogisticBehavior.update(id, &ctx).delta, 0.0);
    }

    #[test]
    fn traits_drive_growth() {
        // warm specialist vs cold specialist in the same cell
        let env = env_at(295.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let warm = registry
            .insert(species_with(0.5, 295.0, 10.0))
            .expect("fresh registry");
        let cold = registry
            .insert(species_with(0.5, 320.0, 10.0))
            .expect("fresh registry");
        let pops = [
            (
                warm,
                Population {
                    amount: Amount::new(10.0),
                },
            ),
            (
                cold,
                Population {
                    amount: Amount::new(10.0),
                },
            ),
        ];
        let ctx = ctx(&env, &pops, &p, &registry);
        let warm_delta = LogisticBehavior.update(warm, &ctx).delta;
        let cold_delta = LogisticBehavior.update(cold, &ctx).delta;
        assert!(warm_delta > 0.0);
        assert_eq!(cold_delta, 0.0);
    }

    #[test]
    fn growth_rate_trait_scales_delta() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let fast = registry
            .insert(species_with(0.8, 300.0, 10.0))
            .expect("fresh registry");
        let slow = registry
            .insert(species_with(0.2, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [
            (
                fast,
                Population {
                    amount: Amount::new(10.0),
                },
            ),
            (
                slow,
                Population {
                    amount: Amount::new(10.0),
                },
            ),
        ];
        let ctx = ctx(&env, &pops, &p, &registry);
        assert!(
            LogisticBehavior.update(fast, &ctx).delta > LogisticBehavior.update(slow, &ctx).delta
        );
    }

    #[test]
    fn growth_stops_at_capacity() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.1, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(100.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &registry);
        assert_eq!(LogisticBehavior.update(id, &ctx).delta, 0.0);
    }

    #[test]
    fn zero_light_stops_growth_but_not_population() {
        let env = env_at(300.0, 0.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.1, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &registry);
        assert_eq!(LogisticBehavior.update(id, &ctx).delta, 0.0);
    }
}
