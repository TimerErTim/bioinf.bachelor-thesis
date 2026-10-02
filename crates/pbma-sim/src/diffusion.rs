//! Mass-conserving gas diffusion cellular automaton.

use pbma_core::ports::{CellContext, EnvironmentStep};
use pbma_model::{EnvState, GASES};

/// Diffuses every gas toward lower concentrations.
///
/// For each gas and each of the four neighbors, a fraction `alpha` of the
/// positive half-difference flows to the neighbor. Inflow and outflow are
/// computed from the same frozen snapshot, so per-cell totals cancel
/// pairwise and total gas mass is conserved. Boundary cells get their own
/// state mirrored as the missing neighbor (clamped boundary, no flux across
/// the edge).
#[derive(Debug, Clone)]
pub struct DiffusionStep {
    /// Diffusion fraction per neighbor pair per tick, in [0, 0.25].
    pub alpha: f64,
}

impl DiffusionStep {
    /// Creates a diffusion step with the given alpha, clamped to [0, 0.25].
    ///
    /// 0.25 is the stability limit for four-neighbor averaging.
    #[must_use]
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 0.25),
        }
    }
}

impl EnvironmentStep for DiffusionStep {
    fn step(&self, cell: &CellContext<'_>) -> EnvState {
        let mut next = *cell.env;

        // mirror self for missing neighbors: no flux across world edges.
        let neighbors = [
            cell.neighbors[0],
            cell.neighbors[1],
            cell.neighbors[2],
            cell.neighbors[3],
        ];

        for gas in GASES {
            let own = cell.env.gas(gas).value();
            let mut net_inflow = 0.0;
            for neighbor in neighbors {
                let other = neighbor.gas(gas).value();
                if own > other {
                    // outflow to this neighbor
                    net_inflow -= (own - other) * self.alpha;
                } else {
                    // inflow from this neighbor
                    net_inflow += (other - own) * self.alpha;
                }
            }
            next.set_gas(gas, pbma_model::Amount::new(own + net_inflow));
        }
        next
    }
}

/// Exposed for conservation tests: computes total gas across a grid slice.
#[must_use]
pub fn total_gas(envs: &[EnvState]) -> f64 {
    envs.iter()
        .map(|e| GASES.iter().map(|g| e.gas(*g).value()).sum::<f64>())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbma_model::{Amount, EnvState, Gas};

    // Helper building a CellContext manually.
    fn ctx<'a>(
        env: &'a EnvState,
        neighbors: [&'a EnvState; 4],
        params: &'a pbma_model::GlobalParams,
        registry: &'a pbma_core::SpeciesRegistry,
    ) -> CellContext<'a> {
        CellContext {
            x: 1,
            y: 1,
            env,
            neighbors,
            populations: &[],
            params,
            registry,
        }
    }

    #[test]
    fn flat_field_is_stable() {
        let env = EnvState {
            gases: [Amount::new(10.0); 4],
            temperature: 288.0,
            light: 0.5,
        };
        let params = test_params();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.2);
        let next = step.step(&ctx(&env, [&env; 4], &params, &registry));
        assert_eq!(next, env);
    }

    #[test]
    fn gradient_relaxes_conserving_mass() {
        let env_a = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 288.0,
            light: 0.5,
        };
        let mut env_b = env_a;
        env_b.set_gas(Gas::O2, Amount::new(8.0));

        let params = test_params();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.2);

        // two-cell pair, symmetric exchange
        let na = step.step(&ctx(
            &env_a,
            [&env_b, &env_b, &env_b, &env_b],
            &params,
            &registry,
        ));
        let nb = step.step(&ctx(
            &env_b,
            [&env_a, &env_a, &env_a, &env_a],
            &params,
            &registry,
        ));

        let before = env_a.gas(Gas::O2).value() * 5.0 + env_b.gas(Gas::O2).value() * 5.0;
        let after = na.gas(Gas::O2).value() * 5.0 + nb.gas(Gas::O2).value() * 5.0;
        assert!((before - after).abs() < 1e-9);
        assert!(na.gas(Gas::O2).value() > env_a.gas(Gas::O2).value());
        assert!(nb.gas(Gas::O2).value() < env_b.gas(Gas::O2).value());
    }

    fn test_params() -> pbma_model::GlobalParams {
        pbma_model::GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
        }
    }
}
