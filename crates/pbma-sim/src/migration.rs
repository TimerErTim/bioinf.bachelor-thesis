//! Overcrowding-driven migration.

use pbma_core::ports::{CellContext, Flux, MigrationPolicy};

/// Emits fluxes when local density exceeds the crowding threshold.
///
/// For each species above threshold in the cell, the excess fraction moves
/// out, split evenly over the four Von Neumann directions (deterministic
/// fixed order; edges reduce the number of targets but not the order).
#[derive(Debug, Clone)]
pub struct OvercrowdingMigration;

impl OvercrowdingMigration {
    /// Von Neumann directions in canonical order.
    const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
}

impl MigrationPolicy for OvercrowdingMigration {
    fn fluxes(&self, cell: &CellContext<'_>, out: &mut Vec<Flux>) {
        let total: f64 = cell.populations.iter().map(|(_, p)| p.amount.value()).sum();
        let threshold = 100.0; // vertical-slice constant; per-species traits in M2
        if total <= threshold {
            return;
        }
        let excess = (total - threshold) / total;
        let share = 0.25;
        for (species, population) in cell.populations {
            let moving = population.amount.value() * excess * share;
            if moving <= 0.0 {
                continue;
            }
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
    use pbma_model::{Amount, EnvState, GlobalParams, Population, SpeciesId};

    #[test]
    fn below_threshold_emits_nothing() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let params = GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
        };
        let pops = [(
            SpeciesId { raw: 1 },
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
            params: &params,
        };
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn above_threshold_emits_four_fluxes_per_species() {
        let env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 1.0,
        };
        let params = GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
        };
        let pops = [(
            SpeciesId { raw: 1 },
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
            params: &params,
        };
        let mut out = Vec::new();
        OvercrowdingMigration.fluxes(&ctx, &mut out);
        assert_eq!(out.len(), 4);
        // excess = (200-100)/200 = 0.5, share 0.25 -> 25 per direction
        assert!((out[0].amount - 25.0).abs() < 1e-9);
    }
}
