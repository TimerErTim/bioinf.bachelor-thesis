//! Mass-conserving gas diffusion cellular automaton.

use pbma_core::grid::coords::{MOORE_WEIGHT_SUM, MOORE_WEIGHTS};
use pbma_core::model::{EnvState, PositiveF64};
use pbma_core::ports::{CellContext, EnvironmentStep};

/// Diffuses every registered gas toward lower concentrations.
///
/// For each gas and each of the eight Moore neighbors, a weighted fraction
/// `alpha` of the difference flows to/from the neighbor: axis neighbors carry
/// weight `1.0`, diagonal neighbors weight `1/sqrt(2)` (they are farther away
/// by a factor of `sqrt(2)`). Inflow and outflow are computed from the same
/// frozen snapshot with symmetric weights, so per-cell totals cancel pairwise
/// and total gas mass is conserved across all supported topologies.
#[derive(Debug, Clone)]
pub struct DiffusionStep {
    /// Diffusion fraction per unit neighbor weight per tick, in
    /// `[0, 1 / (4 + 4/sqrt(2))]` (`~[0, 0.1464]`).
    pub alpha: f64,
}

impl DiffusionStep {
    /// Creates a diffusion step with the given alpha, clamped to the
    /// weighted stability limit.
    #[must_use]
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0 / MOORE_WEIGHT_SUM),
        }
    }
}

impl EnvironmentStep for DiffusionStep {
    fn step(&self, cell: &CellContext<'_>, out: &mut EnvState) {
        out.copy_from(cell.env);

        for (i, neighbor) in cell.neighbors.iter().enumerate() {
            let weight = MOORE_WEIGHTS[i];
            for gas in cell.gases.ids() {
                let own = cell.env.gas(gas).value();
                let other = neighbor.gas(gas).value();
                let flux = self.alpha * weight * (other - own);
                out.set_gas(gas, PositiveF64::new(out.gas(gas).value() + flux));
            }
        }
    }
}

/// Exposed for conservation tests: computes total gas across a grid slice.
#[must_use]
pub fn total_gas(envs: &[EnvState]) -> f64 {
    envs.iter().map(EnvState::total_mass).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbma_core::model::{GasId, GasRegistry, GlobalParams, Insolation, TraitSchema};

    fn ctx<'a>(
        env: &'a EnvState,
        neighbors: [&'a EnvState; 8],
        params: &'a GlobalParams,
        gases: &'a GasRegistry,
        trait_schema: &'a TraitSchema,
        registry: &'a pbma_core::SpeciesRegistry,
    ) -> CellContext<'a> {
        CellContext {
            x: 1,
            y: 1,
            width: 3,
            height: 3,
            tick: 0,
            latitude_rad: 0.0,
            longitude_rad: 0.0,
            insolation: Insolation {
                cos_zenith: 1.0,
                toa_flux_wm2: 1361.0,
                normalized_light: 0.5,
                toa_uv_wm2: 100.0,
            },
            env,
            neighbors,
            populations: &[],
            params,
            gases,
            trait_schema,
            registry,
        }
    }

    #[test]
    fn flat_field_is_stable() {
        let mut env = EnvState::new(288.0, 0.5);
        let gases = GasRegistry::with_defaults();
        for g in gases.ids() {
            env.set_gas(g, PositiveF64::new(10.0));
        }
        let params = GlobalParams::earth_like();
        let schema = TraitSchema::with_defaults();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.2);
        let mut next = EnvState::default();
        step.step(
            &ctx(&env, [&env; 8], &params, &gases, &schema, &registry),
            &mut next,
        );
        assert_eq!(next, env);
    }

    #[test]
    fn gradient_relaxes_conserving_mass() {
        let env_a = EnvState::new(288.0, 0.5);
        let mut env_b = env_a.clone();
        env_b.set_gas(GasId::O2, PositiveF64::new(8.0));

        let params = GlobalParams::earth_like();
        let gases = GasRegistry::with_defaults();
        let schema = TraitSchema::with_defaults();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.2);

        let mut na = EnvState::default();
        let mut nb = EnvState::default();
        step.step(
            &ctx(&env_a, [&env_b; 8], &params, &gases, &schema, &registry),
            &mut na,
        );
        step.step(
            &ctx(&env_b, [&env_a; 8], &params, &gases, &schema, &registry),
            &mut nb,
        );

        let before = env_a.gas(GasId::O2).value() * 5.0 + env_b.gas(GasId::O2).value() * 5.0;
        let after = na.gas(GasId::O2).value() * 5.0 + nb.gas(GasId::O2).value() * 5.0;
        assert!((before - after).abs() < 1e-9);
        assert!(na.gas(GasId::O2).value() > env_a.gas(GasId::O2).value());
        assert!(nb.gas(GasId::O2).value() < env_b.gas(GasId::O2).value());
    }

    #[test]
    fn diagonal_neighbor_carries_sqrt_half_impact() {
        let mut env = EnvState::new(288.0, 0.5);
        let gases = GasRegistry::with_defaults();
        for g in gases.ids() {
            env.set_gas(g, PositiveF64::new(10.0));
        }
        let low = EnvState::new(288.0, 0.5);
        let params = GlobalParams::earth_like();
        let schema = TraitSchema::with_defaults();
        let registry = pbma_core::SpeciesRegistry::new();
        let step = DiffusionStep::new(0.1);

        let mut axis_neighbors = [&env; 8];
        let mut diagonal_neighbors = [&env; 8];
        axis_neighbors[0] = &low;
        diagonal_neighbors[1] = &low;

        let mut axis_next = EnvState::default();
        let mut diag_next = EnvState::default();
        step.step(
            &ctx(&env, axis_neighbors, &params, &gases, &schema, &registry),
            &mut axis_next,
        );
        step.step(
            &ctx(
                &env,
                diagonal_neighbors,
                &params,
                &gases,
                &schema,
                &registry,
            ),
            &mut diag_next,
        );

        let axis_outflow = env.gas(GasId::O2).value() - axis_next.gas(GasId::O2).value();
        let diag_outflow = env.gas(GasId::O2).value() - diag_next.gas(GasId::O2).value();
        assert!((diag_outflow / axis_outflow - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
    }
}
