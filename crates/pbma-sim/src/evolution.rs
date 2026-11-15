//! Deterministic evolution adapter: trait-drift variants and extinction pruning.

use pbma_core::World;
use pbma_core::model::{Species, SpeciesId, SpeciesTraits, TraitId};
use pbma_core::ports::{EvolutionEngine, Rng};

/// Drives speciation and extinction from the post-migration world state.
#[derive(Debug, Clone)]
pub struct VariationEngine {
    /// Probability that a surviving lineage spawns a variant per tick, in [0, 1].
    pub spawn_probability: f64,
    /// Upper bound of the multiplicative drift applied to mutable traits.
    pub drift_max: f64,
}

impl VariationEngine {
    /// Engine with the given per-tick spawn probability and drift bound.
    #[must_use]
    pub fn new(spawn_probability: f64, drift_max: f64) -> Self {
        Self {
            spawn_probability: spawn_probability.clamp(0.0, 1.0),
            drift_max: drift_max.clamp(1.0, 2.0),
        }
    }

    /// Derives variant traits from parent traits with a drift factor.
    #[must_use]
    fn variant_traits(parent: &SpeciesTraits, factor: f64) -> SpeciesTraits {
        let mut next = parent.clone();
        next.set(
            TraitId::OPTIMAL_TEMPERATURE,
            parent.optimal_temperature() + (factor - 1.0) * parent.temperature_tolerance(),
        );
        next.set(TraitId::GROWTH_RATE, parent.growth_rate() * factor);
        next
    }

    /// Deterministic unique variant name: parent name plus a batch index.
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
        tick: u64,
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
            let u = rng.next_f64();
            let factor = (2.0 - drift_max) + u * (2.0 * drift_max - 2.0);
            let name = Self::variant_name(&species.name, batch_index);
            batch_index += 1;
            out.push((
                id,
                Species::variant(
                    name,
                    Self::variant_traits(&species.traits, factor),
                    id,
                    tick,
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SplitMixRng;
    use pbma_core::WorldBuilder;

    fn species(name: &str) -> Species {
        Species::new(name, SpeciesTraits::new(300.0, 10.0, 0.1, 100.0))
    }

    fn world_with_living_species() -> World {
        WorldBuilder::new(2, 1)
            .temperature_and_light(300.0, 0.5)
            .seed_species_at(species("root"), 0, 0, 10.0)
            .build()
            .unwrap()
    }

    fn world_with_extinct_species() -> World {
        let mut world = WorldBuilder::new(2, 1)
            .temperature_and_light(300.0, 0.5)
            .build()
            .unwrap();
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
        engine.spawn_variants(0, &world, &mut rng, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn probability_one_spawns_variant_with_drift() {
        let world = world_with_living_species();
        let mut engine = VariationEngine::new(1.0, 1.2);
        let mut rng = SplitMixRng::new(42);
        let mut out = Vec::new();
        engine.spawn_variants(5, &world, &mut rng, &mut out);
        assert_eq!(out.len(), 1);
        let (_parent, variant) = &out[0];
        assert_eq!(variant.name, "root'0");
        assert_eq!(variant.spawn_tick, 5);
        let parent_rate = 0.1;
        assert!(variant.traits.growth_rate() > 0.0);
        assert!(variant.traits.growth_rate() <= parent_rate * 1.2 + f64::EPSILON);
        assert!((variant.traits.optimal_temperature() - 300.0).abs() <= 10.0 * 0.2 + f64::EPSILON);
    }

    #[test]
    fn same_seed_same_variants() {
        let world = world_with_living_species();
        let mut engine = VariationEngine::new(0.5, 1.2);
        let mut run = |world: &World| {
            let mut rng = SplitMixRng::new(7);
            let mut out = Vec::new();
            engine.spawn_variants(0, world, &mut rng, &mut out);
            out.into_iter()
                .map(|(_, s)| (s.name.clone(), s.traits.growth_rate()))
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
