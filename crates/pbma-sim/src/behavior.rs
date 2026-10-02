//! Population growth/decline behavior.

use pbma_core::ports::{BehaviorDelta, CellContext, PopulationBehavior};
use pbma_model::SpeciesId;

/// Logistic-style growth modulated by temperature fit.
///
/// Each species' population grows toward the cell's carrying capacity
/// (derived from light). Growth rate decays linearly as the population
/// approaches capacity and is scaled by a temperature fitness factor in
/// [0, 1]. No randomness: fully deterministic.
#[derive(Debug, Clone)]
pub struct LogisticBehavior;

/// Carrying capacity per cell at full light (vertical-slice constant).
const CAPACITY_AT_FULL_LIGHT: f64 = 100.0;
/// Base growth rate per tick under ideal conditions.
const BASE_GROWTH_RATE: f64 = 0.1;
/// Optimal temperature in kelvin for the fitness curve.
const OPTIMAL_TEMPERATURE: f64 = 300.0;
/// Fitness stays 1.0 within this many kelvin of the optimum.
const TEMPERATURE_WINDOW: f64 = 10.0;

fn temperature_fitness(temperature: f64) -> f64 {
    (1.0 - (temperature - OPTIMAL_TEMPERATURE).abs() / TEMPERATURE_WINDOW).clamp(0.0, 1.0)
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

        let capacity = CAPACITY_AT_FULL_LIGHT * cell.env.light.max(0.0);
        let fitness = temperature_fitness(cell.env.temperature);

        if capacity <= 0.0 || amount <= 0.0 {
            return BehaviorDelta { delta: 0.0 };
        }
        let rate = BASE_GROWTH_RATE * fitness * (1.0 - amount / capacity);
        BehaviorDelta {
            delta: amount * rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbma_model::{Amount, EnvState, GlobalParams, Population};

    fn ctx<'a>(
        env: &'a EnvState,
        populations: &'a [(SpeciesId, Population)],
        params: &'a GlobalParams,
    ) -> CellContext<'a> {
        CellContext {
            x: 0,
            y: 0,
            env,
            neighbors: [env; 4],
            populations,
            params,
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

    #[test]
    fn zero_population_stays_zero() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
        let ctx = ctx(&env, &[], &p);
        assert_eq!(
            LogisticBehavior.update(SpeciesId { raw: 0 }, &ctx).delta,
            0.0
        );
    }

    #[test]
    fn absent_species_gets_no_delta() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
        let pops = [(
            SpeciesId { raw: 7 },
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p);
        assert_eq!(
            LogisticBehavior.update(SpeciesId { raw: 1 }, &ctx).delta,
            0.0
        );
        assert!(LogisticBehavior.update(SpeciesId { raw: 7 }, &ctx).delta > 0.0);
    }

    #[test]
    fn growth_stops_at_capacity() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let p = params();
        let pops = [(
            SpeciesId { raw: 7 },
            Population {
                amount: Amount::new(100.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p);
        assert_eq!(
            LogisticBehavior.update(SpeciesId { raw: 7 }, &ctx).delta,
            0.0
        );
    }

    #[test]
    fn zero_light_kills_growth_but_not_population() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 0.0,
        };
        let p = params();
        let pops = [(
            SpeciesId { raw: 7 },
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p);
        assert_eq!(
            LogisticBehavior.update(SpeciesId { raw: 7 }, &ctx).delta,
            0.0
        );
    }
}
