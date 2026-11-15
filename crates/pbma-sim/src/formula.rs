//! RL-surrogate population behavior: a closed-form stand-in for a learned
//! policy.

use pbma_core::model::SpeciesId;
use pbma_core::ports::{BehaviorDelta, CellContext, PopulationBehavior};

/// Resource saturation rate `k` of the light response
/// `f_light = 1 - exp(-k * L)`.
const LIGHT_SATURATION_RATE: f64 = 2.0;

/// Numerical floor `eps` keeping the crowding denominator away from zero.
const CROWDING_EPSILON: f64 = 1e-9;

/// Baseline per-capita death rate `beta`, standing in for learned mortality
/// until a trained model supplies it.
const BASELINE_MORTALITY: f64 = 0.02;

/// Closed-form surrogate behavior: the first stand-in for a decision-model
/// or RL-derived pre-compiled formula.
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

        let Ok(species_def) = cell.registry.get(species) else {
            return BehaviorDelta { delta: 0.0 };
        };
        let traits = &species_def.traits;

        let light = cell.env.light.max(0.0);
        let f_light = 1.0 - (-LIGHT_SATURATION_RATE * light).exp();
        let f_temp = temperature_fitness(
            cell.env.temperature,
            traits.optimal_temperature(),
            traits.temperature_tolerance(),
        );
        let f_crowd =
            1.0 / (1.0 + (amount / (traits.crowding_threshold() + CROWDING_EPSILON)).powi(2));

        let alpha = traits.growth_rate() * f_temp * f_light * f_crowd;
        BehaviorDelta {
            delta: alpha * amount - BASELINE_MORTALITY * amount,
        }
    }
}

/// Gaussian temperature fitness in [0, 1]: 1.0 at the species' optimum,
/// decaying with the squared normalized distance from it.
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
    use pbma_core::model::{
        EnvState, GasRegistry, GlobalParams, Insolation, Population, PositiveF64, Species,
        SpeciesTraits, TraitSchema,
    };

    fn ctx<'a>(
        env: &'a EnvState,
        populations: &'a [(SpeciesId, Population)],
        params: &'a GlobalParams,
        gases: &'a GasRegistry,
        trait_schema: &'a TraitSchema,
        registry: &'a SpeciesRegistry,
    ) -> CellContext<'a> {
        CellContext {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            tick: 0,
            latitude_rad: 0.0,
            longitude_rad: 0.0,
            insolation: Insolation {
                cos_zenith: 1.0,
                toa_flux_wm2: 1361.0,
                normalized_light: env.light,
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

    fn species_with(growth_rate: f64, optimal: f64, tolerance: f64) -> Species {
        Species::new(
            format!("f{growth_rate}{optimal}{tolerance}"),
            SpeciesTraits::new(optimal, tolerance, growth_rate, 100.0),
        )
    }

    #[test]
    fn stale_species_gets_no_delta() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");
        registry.remove(id);
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        assert_eq!(FormulaBehavior.update(id, &ctx).delta, 0.0);
    }

    #[test]
    fn absent_species_gets_no_delta() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.1, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(10.0),
            },
        )];
        let other = SpeciesId { raw: u64::MAX };
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        assert_eq!(FormulaBehavior.update(other, &ctx).delta, 0.0);
        assert!(FormulaBehavior.update(id, &ctx).delta > 0.0);
    }

    #[test]
    fn equilibrium_root_bracketed_by_bisection() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");

        let delta_at = |amount: f64| {
            let pops = [(
                id,
                Population {
                    amount: PositiveF64::new(amount),
                },
            )];
            let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
            FormulaBehavior.update(id, &ctx).delta
        };

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

        let f_light = 1.0 - (-2.0_f64).exp();
        let expected = 100.0 * (0.5 * f_light / BASELINE_MORTALITY - 1.0).sqrt();
        assert!((root - expected).abs() < 1e-6);
    }

    #[test]
    fn growth_rate_monotonically_increases_delta() {
        let env = EnvState::new(300.0, 1.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
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
                    amount: PositiveF64::new(10.0),
                },
            ),
            (
                slow,
                Population {
                    amount: PositiveF64::new(10.0),
                },
            ),
        ];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        assert!(
            FormulaBehavior.update(fast, &ctx).delta > FormulaBehavior.update(slow, &ctx).delta
        );
    }

    #[test]
    fn temperature_mismatch_kills_growth() {
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(10.0),
            },
        )];

        let optimal_env = EnvState::new(300.0, 1.0);
        let mismatched_env = EnvState::new(350.0, 1.0);
        let optimal_ctx = ctx(&optimal_env, &pops, &p, &g, &ts, &registry);
        let mismatched_ctx = ctx(&mismatched_env, &pops, &p, &g, &ts, &registry);

        let optimal_delta = FormulaBehavior.update(id, &optimal_ctx).delta;
        let mismatched_delta = FormulaBehavior.update(id, &mismatched_ctx).delta;
        assert!(optimal_delta > 0.0);
        assert!(mismatched_delta < optimal_delta);
        assert!(mismatched_delta < 0.0);
    }

    #[test]
    fn zero_light_yields_pure_mortality() {
        let env = EnvState::new(300.0, 0.0);
        let p = GlobalParams::earth_like();
        let g = GasRegistry::with_defaults();
        let ts = TraitSchema::with_defaults();
        let mut registry = SpeciesRegistry::new();
        let id = registry
            .insert(species_with(0.5, 300.0, 10.0))
            .expect("fresh registry");
        let pops = [(
            id,
            Population {
                amount: PositiveF64::new(10.0),
            },
        )];
        let ctx = ctx(&env, &pops, &p, &g, &ts, &registry);
        let delta = FormulaBehavior.update(id, &ctx).delta;
        assert!(delta < 0.0);
        assert!((delta + BASELINE_MORTALITY * 10.0).abs() < 1e-12);
    }
}
