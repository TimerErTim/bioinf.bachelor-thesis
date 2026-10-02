//! RL-surrogate population behavior: a closed-form stand-in for a learned
//! policy.
//!
//! The thesis models behavior as formulas pre-compiled from offline-trained
//! models. This adapter is the first such surrogate: an analytically defined
//! closed-form function standing in for a future learned policy. It fixes the
//! shape of the formula interface (frozen cell snapshot in, per-species delta
//! out) that generated formulas will reuse.

use pbma_core::ports::{BehaviorDelta, CellContext, PopulationBehavior};
use pbma_model::SpeciesId;

/// Resource saturation rate `k` of the light response
/// `f_light = 1 - exp(-k * L)`.
const LIGHT_SATURATION_RATE: f64 = 2.0;

/// Numerical floor `eps` keeping the crowding denominator away from zero.
const CROWDING_EPSILON: f64 = 1e-9;

/// Baseline per-capita death rate `beta`, standing in for learned mortality
/// until a trained model supplies it.
const BASELINE_MORTALITY: f64 = 0.02;

/// Closed-form surrogate behavior: the first stand-in for an RL-derived
/// pre-compiled formula.
///
/// For a species with amount `x` in a cell with light `L` and temperature
/// `T`, the per-tick delta is
///
/// ```text
/// delta = alpha(x, L, T) * x - beta * x,
/// alpha = r * f_temp(T) * f_light(L) * f_crowd(x)
/// ```
///
/// with the factors (each in [0, 1], so `alpha` lies in `[0, r]`, where `r`
/// is the [`SpeciesTraits`](pbma_model::SpeciesTraits) `growth_rate`):
///
/// ```text
/// f_temp(T)  = exp(-(T - T*)^2 / (2 * sigma^2))
/// f_light(L) = 1 - exp(-k * L)
/// f_crowd(x) = 1 / (1 + (x / (c + eps))^2)
/// ```
///
/// Here `T*` is the species' `optimal_temperature`, `sigma` its
/// `temperature_tolerance`, and `c` its `crowding_threshold`; `k = 2.0` is
/// [`LIGHT_SATURATION_RATE`], `eps = 1e-9` is [`CROWDING_EPSILON`], and
/// `beta = 0.02` is [`BASELINE_MORTALITY`]. A non-positive tolerance makes
/// `f_temp` a step: 1.0 exactly at the optimum, 0.0 elsewhere. Species
/// absent from the cell and stale (extinct) ids contribute a delta of zero.
/// Fully deterministic, no randomness.
#[derive(Debug, Clone)]
pub struct FormulaBehavior;

impl PopulationBehavior for FormulaBehavior {
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
        // kernel prunes them; extinct lineages contribute nothing.
        let Ok(species_def) = cell.registry.get(species) else {
            return BehaviorDelta { delta: 0.0 };
        };
        let traits = species_def.traits;

        let light = cell.env.light.max(0.0);
        let f_light = 1.0 - (-LIGHT_SATURATION_RATE * light).exp();
        let f_temp = temperature_fitness(
            cell.env.temperature,
            traits.optimal_temperature,
            traits.temperature_tolerance,
        );
        let f_crowd =
            1.0 / (1.0 + (amount / (traits.crowding_threshold + CROWDING_EPSILON)).powi(2));

        let alpha = traits.growth_rate * f_temp * f_light * f_crowd;
        BehaviorDelta {
            delta: alpha * amount - BASELINE_MORTALITY * amount,
        }
    }
}

/// Gaussian temperature fitness in [0, 1]: 1.0 at the species' optimum,
/// decaying with the squared normalized distance from it.
///
/// A non-positive tolerance degenerates to a step: 1.0 exactly at the
/// optimum, 0.0 anywhere else.
#[must_use]
fn temperature_fitness(temperature: f64, optimal: f64, tolerance: f64) -> f64 {
    if tolerance <= 0.0 {
        return if (temperature - optimal).abs() <= f64::EPSILON {
            1.0
        } else {
            0.0
        };
    }
    let normalized = (temperature - optimal) / tolerance;
    (-0.5 * normalized * normalized).exp()
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
            name: format!("f{growth_rate}{optimal}{tolerance}"),
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
    fn stale_species_gets_no_delta() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");
        registry.remove(id);
        let pops = [(
            id,
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &registry);
        assert_eq!(FormulaBehavior.update(id, &ctx).delta, 0.0);
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
        assert_eq!(FormulaBehavior.update(other, &ctx).delta, 0.0);
        assert!(FormulaBehavior.update(id, &ctx).delta > 0.0);
    }

    #[test]
    fn equilibrium_root_bracketed_by_bisection() {
        let env = env_at(300.0, 1.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");

        let delta_at = |amount: f64| {
            let pops = [(
                id,
                Population {
                    amount: Amount::new(amount),
                },
            )];
            let ctx = ctx(&env, &pops, &p, &registry);
            FormulaBehavior.update(id, &ctx).delta
        };

        // growth dominates below the equilibrium, mortality above it
        assert!(delta_at(10.0) > 0.0);
        assert!(delta_at(1000.0) < 0.0);

        let mut lo = 10.0;
        let mut hi = 1000.0;
        let mut root = 0.5 * (lo + hi);
        for _ in 0..60 {
            root = 0.5 * (lo + hi);
            if delta_at(root) > 0.0 {
                lo = root;
            } else {
                hi = root;
            }
        }
        assert!(root > 10.0 && root < 1000.0);
        assert!(delta_at(root).abs() < 1e-6);

        // cross-check: alpha * x = beta * x with x > 0 solves to
        // x* = c * sqrt(r * f_temp * f_light / beta - 1)
        let f_light = 1.0 - (-2.0_f64).exp();
        let expected = 100.0 * (0.5 * f_light / BASELINE_MORTALITY - 1.0).sqrt();
        assert!((root - expected).abs() < 1e-6);
    }

    #[test]
    fn growth_rate_monotonically_increases_delta() {
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
            FormulaBehavior.update(fast, &ctx).delta > FormulaBehavior.update(slow, &ctx).delta
        );
    }

    #[test]
    fn temperature_mismatch_kills_growth() {
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(10.0),
            },
        )];

        let optimal_env = env_at(300.0, 1.0);
        let mismatched_env = env_at(350.0, 1.0); // optimal + 5 * tolerance
        let optimal_ctx = ctx(&optimal_env, &pops, &p, &registry);
        let mismatched_ctx = ctx(&mismatched_env, &pops, &p, &registry);

        let optimal_delta = FormulaBehavior.update(id, &optimal_ctx).delta;
        let mismatched_delta = FormulaBehavior.update(id, &mismatched_ctx).delta;
        assert!(optimal_delta > 0.0);
        assert!(mismatched_delta < optimal_delta);
        assert!(mismatched_delta < 0.0);
    }

    #[test]
    fn zero_light_yields_pure_mortality() {
        let env = env_at(300.0, 0.0);
        let p = params();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: Amount::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &registry);
        let delta = FormulaBehavior.update(id, &ctx).delta;
        assert!(delta < 0.0);
        // f_light = 1 - exp(0) = 0, so alpha = 0 and delta = -beta * x exactly
        assert!((delta + BASELINE_MORTALITY * 10.0).abs() < 1e-12);
    }
}
