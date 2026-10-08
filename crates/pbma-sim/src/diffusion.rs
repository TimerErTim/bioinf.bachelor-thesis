//! Mass-conserving gas diffusion cellular automaton.

use pbma_core::grid::coords::{MOORE_WEIGHT_SUM, MOORE_WEIGHTS};
use pbma_core::model::{EnvState, GASES};
use pbma_core::ports::{CellContext, EnvironmentStep};

/// Diffuses every gas toward lower concentrations.
///
/// For each gas and each of the eight Moore neighbors, a weighted fraction
/// `alpha` of the positive half-difference flows to the neighbor: axis
/// neighbors carry weight 1.0, diagonal neighbors weight 1/sqrt(2) (they
/// are farther away by a factor of sqrt(2)). Inflow and outflow are
/// computed from the same frozen snapshot with symmetric weights, so
/// per-cell totals cancel pairwise and total gas mass is conserved.
/// Boundary cells get their own state mirrored as the missing neighbor
/// (clamped boundary, no flux across the edge).
#[derive(Debug, Clone)]
pub struct DiffusionStep {
    /// Diffusion fraction per unit neighbor weight per tick, in
    /// [0, 1 / (4 + 4/sqrt(2))] (~[0, 0.1464]).
    pub alpha: f64,
}

impl DiffusionStep {
    /// Creates a diffusion step with the given alpha, clamped to the
    /// weighted stability limit.
    ///
    /// 1 / sum(MOORE_WEIGHTS) ~ 0.1464 is the stability limit for the
    /// weighted eight-neighbor averaging: total outflow can then never
    /// exceed the cell's own content.
    #[must_use]
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0 / MOORE_WEIGHT_SUM),
        }
    }
}

impl EnvironmentStep for DiffusionStep {
    fn step(&self, cell: &CellContext<'_>) -> EnvState {
        let mut next = *cell.env;

        // mirror self for missing neighbors: no flux across world edges.
        for (i, neighbor) in cell.neighbors.iter().enumerate() {
            let weight = MOORE_WEIGHTS[i];
            for gas in GASES {
                let own = cell.env.gas(gas).value();
                let other = neighbor.gas(gas).value();
                let flux = self.alpha * weight * (other - own);
                next.set_gas(
                    gas,
                    pbma_core::model::PositiveF64::new(next.gas(gas).value() + flux),
                );
            }
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
    use pbma_core::model::{EnvState, Gas, PositiveF64};

    // Helper building a CellContext manually.
    fn ctx<'a>(
        env: &'a EnvState,
        neighbors: [&'a EnvState; 8],
        params: &'a pbma_core::model::GlobalParams,
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
            gases: [PositiveF64::new(10.0); 4],
            temperature: 288.0,
            light: 0.5,
        };
        let params = test_params();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.2);
        let next = step.step(&ctx(&env, [&env; 8], &params, &registry));
        assert_eq!(next, env);
    }

    #[test]
    fn gradient_relaxes_conserving_mass() {
        let env_a = EnvState {
            gases: [PositiveF64::ZERO; 4],
            temperature: 288.0,
            light: 0.5,
        };
        let mut env_b = env_a;
        env_b.set_gas(Gas::O2, PositiveF64::new(8.0));

        let params = test_params();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.2);

        // two-cell pair, symmetric exchange
        let na = step.step(&ctx(&env_a, [&env_b; 8], &params, &registry));
        let nb = step.step(&ctx(&env_b, [&env_a; 8], &params, &registry));

        let before = env_a.gas(Gas::O2).value() * 5.0 + env_b.gas(Gas::O2).value() * 5.0;
        let after = na.gas(Gas::O2).value() * 5.0 + nb.gas(Gas::O2).value() * 5.0;
        assert!((before - after).abs() < 1e-9);
        assert!(na.gas(Gas::O2).value() > env_a.gas(Gas::O2).value());
        assert!(nb.gas(Gas::O2).value() < env_b.gas(Gas::O2).value());
    }

    #[test]
    fn diagonal_neighbor_carries_sqrt_half_impact() {
        let env = EnvState {
            gases: [PositiveF64::new(10.0); 4],
            temperature: 288.0,
            light: 0.5,
        };
        let low = EnvState {
            gases: [PositiveF64::ZERO; 4],
            temperature: 288.0,
            light: 0.5,
        };
        let params = test_params();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.1);

        // identical setups except that the low neighbor sits in one axis
        // slot (N, weight 1.0) vs. one diagonal slot (NE, weight 1/sqrt(2)).
        let mut axis_neighbors = [&env; 8];
        let mut diagonal_neighbors = [&env; 8];
        axis_neighbors[0] = &low;
        diagonal_neighbors[1] = &low;

        let axis_next = step.step(&ctx(&env, axis_neighbors, &params, &registry));
        let diag_next = step.step(&ctx(&env, diagonal_neighbors, &params, &registry));

        let axis_outflow = env.gas(Gas::O2).value() - axis_next.gas(Gas::O2).value();
        let diag_outflow = env.gas(Gas::O2).value() - diag_next.gas(Gas::O2).value();
        assert!((diag_outflow / axis_outflow - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
    }

    fn test_params() -> pbma_core::model::GlobalParams {
        pbma_core::model::GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
            light_intensity: 1.0,
        }
    }
}
