//! Deterministic evolution adapter: trait-drift variants and extinction pruning.

use pbma_core::World;
use pbma_core::ports::{EvolutionEngine, Rng};
use pbma_model::{Species, SpeciesId};

/// Drives speciation and extinction from the post-migration world state.
///
/// Two responsibilities, both deterministic:
///
/// - **Pruning**: a species with zero surviving populations anywhere is
///   reported extinct; the kernel removes it from the registry.
/// - **Variation**: each surviving lineage has a chance to spawn a variant
///   whose traits drift by a multiplicative factor sampled from the rng.
///   Drift applies to `growth_rate` and `optimal_temperature` only; the
///   crowding threshold is inherited unchanged so migration pressure stays
///   comparable between parent and child.
///
/// Iteration order is registry insertion order and every random decision
/// comes from the injected rng, so runs with the same seed reproduce the
/// same evolutionary history.
#[derive(Debug, Clone)]
pub struct VariationEngine {
    /// Probability that a surviving lineage spawns a variant per tick, in [0, 1].
    pub spawn_probability: f64,
    /// Upper bound of the multiplicative drift applied to mutable traits.
    /// The factor is sampled uniform in `[2 - drift_max, drift_max]`, so a
    /// value of 1.2 yields drift factors in [0.8, 1.2].
    pub drift_max: f64,
}

impl VariationEngine {
    /// Engine with the given per-tick spawn probability and drift bound.
    ///
    /// Values are clamped to sane ranges: probability to [0, 1], drift to
    /// [1.0, 2.0] (a drift below 1 would only shrink traits).
    #[must_use]
    pub fn new(spawn_probability: f64, drift_max: f64) -> Self {
        Self {
            spawn_probability: spawn_probability.clamp(0.0, 1.0),
            drift_max: drift_max.clamp(1.0, 2.0),
        }
    }

    /// Derives variant traits from parent traits with a drift factor.
    ///
    /// Growth rate scales multiplicatively; the temperature optimum shifts
    /// proportionally to the parent's tolerance width so specialists drift
    /// less in absolute kelvin than eurytherm species.
    #[must_use]
    fn variant_traits(parent: pbma_model::SpeciesTraits, factor: f64) -> pbma_model::SpeciesTraits {
        pbma_model::SpeciesTraits {
            optimal_temperature: parent.optimal_temperature
                + (factor - 1.0) * parent.temperature_tolerance,
            temperature_tolerance: parent.temperature_tolerance,
            growth_rate: parent.growth_rate * factor,
            crowding_threshold: parent.crowding_threshold,
        }
    }

    /// Deterministic unique variant name: parent name plus a batch index.
    ///
    /// The index disambiguates multiple variants of one parent within a
    /// single tick; across ticks names keep growing distinct suffixes.
    #[must_use]
    fn variant_name(parent_name: &str, batch_index: usize) -> String {
        format!("{parent_name}'{batch_index}")
    }
}

impl EvolutionEngine for VariationEngine {
    fn prune_extinct(&mut self, world: &World, out: &mut Vec<SpeciesId>) {
        for (id, _species) in world.registry().iter() {
            let alive = world
                .cells()
                .iter()
                .any(|cell| cell.populations.iter().any(|(pid, _)| pid.raw == id.raw));
            if !alive {
                out.push(id);
            }
        }
    }

    fn spawn_variants(
        &mut self,
        world: &World,
        rng: &mut dyn Rng,
        out: &mut Vec<(SpeciesId, Species)>,
    ) {
        let spawn_probability = self.spawn_probability;
        let drift_max = self.drift_max;
        let mut batch_index = 0usize;
        for (id, species) in world.registry().iter() {
            let alive = world
                .cells()
                .iter()
                .any(|cell| cell.populations.iter().any(|(pid, _)| pid.raw == id.raw));
            if !alive {
                continue;
            }
            if rng.next_f64() >= spawn_probability {
                continue;
            }
            // drift factor uniform in [2 - drift_max, drift_max]
            let u = rng.next_f64();
            let factor = (2.0 - drift_max) + u * (2.0 * drift_max - 2.0);
            let name = Self::variant_name(&species.name, batch_index);
            batch_index += 1;
            out.push((
                id,
                Species {
                    name,
                    traits: Self::variant_traits(species.traits, factor),
                },
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SplitMixRng;
    use pbma_model::{Amount, EnvState, GlobalParams, Population, Species, SpeciesTraits};

    fn params() -> GlobalParams {
        GlobalParams {
            gravity: 9.81,
            day_length_ticks: 240,
            axial_tilt: 0.0,
            diffusion_coefficient: 0.2,
        }
    }

    fn env() -> EnvState {
        EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 300.0,
            light: 0.5,
        }
    }

    fn species(name: &str) -> Species {
        Species {
            name: name.into(),
            traits: SpeciesTraits {
                optimal_temperature: 300.0,
                temperature_tolerance: 10.0,
                growth_rate: 0.1,
                crowding_threshold: 100.0,
            },
        }
    }

    fn world_with_living_species() -> World {
        let mut world = World::new(2, 1, env(), params());
        let id = world
            .registry_mut()
            .insert(species("root"))
            .expect("fresh registry");
        world.cells_mut()[0].populations.push((
            id,
            Population {
                amount: Amount::new(10.0),
            },
        ));
        world
    }

    fn world_with_extinct_species() -> World {
        let mut world = World::new(2, 1, env(), params());
        let _ = world
            .registry_mut()
            .insert(species("ghost"))
            .expect("fresh registry");
        world
    }

    #[test]
    fn extinct_species_reported() {
        let world = world_with_extinct_species();
        let mut engine = VariationEngine::new(0.0, 1.0);
        let mut out = Vec::new();
        engine.prune_extinct(&world, &mut out);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn living_species_not_reported() {
        let world = world_with_living_species();
        let mut engine = VariationEngine::new(0.0, 1.0);
        let mut out = Vec::new();
        engine.prune_extinct(&world, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn zero_probability_spawns_nothing() {
        let world = world_with_living_species();
        let mut engine = VariationEngine::new(0.0, 1.2);
        let mut rng = SplitMixRng::new(42);
        let mut out = Vec::new();
        engine.spawn_variants(&world, &mut rng, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn probability_one_spawns_variant_with_drift() {
        let world = world_with_living_species();
        let mut engine = VariationEngine::new(1.0, 1.2);
        let mut rng = SplitMixRng::new(42);
        let mut out = Vec::new();
        engine.spawn_variants(&world, &mut rng, &mut out);
        assert_eq!(out.len(), 1);
        let (_parent, variant) = &out[0];
        assert_eq!(variant.name, "root'0");
        // growth rate must have drifted but stay within bounds
        let parent_rate = 0.1;
        assert!(variant.traits.growth_rate > 0.0);
        assert!(variant.traits.growth_rate <= parent_rate * 1.2 + f64::EPSILON);
        // temperature optimum shifted within one tolerance width
        assert!((variant.traits.optimal_temperature - 300.0).abs() <= 10.0 * 0.2 + f64::EPSILON);
    }

    #[test]
    fn same_seed_same_variants() {
        let world = world_with_living_species();
        let mut engine = VariationEngine::new(0.5, 1.2);
        let mut run = |world: &World| {
            let mut rng = SplitMixRng::new(7);
            let mut out = Vec::new();
            engine.spawn_variants(world, &mut rng, &mut out);
            out.into_iter()
                .map(|(_, s)| (s.name.clone(), s.traits.growth_rate))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(&world), run(&world));
    }

    #[test]
    fn variant_name_batch_index_unique() {
        assert_eq!(VariationEngine::variant_name("a", 0), "a'0");
        assert_eq!(VariationEngine::variant_name("a", 1), "a'1");
        assert_ne!(
            VariationEngine::variant_name("a", 0),
            VariationEngine::variant_name("a", 1)
        );
    }
}
