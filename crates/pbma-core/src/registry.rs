//! Species registry with generational keys.

use slotmap::{DefaultKey, SlotMap};

use pbma_model::{Species, SpeciesId};

use crate::{Error, Result};

/// Global table of live species.
///
/// Backed by a slotmap: removing a species invalidates its key; reusing the
/// slot bumps the generation so stale ids are detected instead of silently
/// pointing at a different species.
pub struct SpeciesRegistry {
    map: SlotMap<DefaultKey, Species>,
}

impl SpeciesRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: SlotMap::default(),
        }
    }

    /// Registers a new species and returns its id.
    ///
    /// # Errors
    /// Returns [`Error::DuplicateSpecies`] if the name is taken.
    pub fn insert(&mut self, species: Species) -> Result<SpeciesId> {
        if self.map.values().any(|s| s.name == species.name) {
            return Err(Error::DuplicateSpecies(species.name));
        }
        let key = self.map.insert(species);
        Ok(SpeciesId::from_key(key))
    }

    /// Resolves an id to the live species.
    ///
    /// # Errors
    /// Returns [`Error::UnknownSpecies`] for stale or foreign ids.
    pub fn get(&self, id: SpeciesId) -> Result<&Species> {
        let key = DefaultKey::from(id);
        self.map.get(key).ok_or(Error::UnknownSpecies(id))
    }

    /// Mutable reference to a live species.
    ///
    /// # Errors
    /// Returns [`Error::UnknownSpecies`] for stale or foreign ids.
    pub fn get_mut(&mut self, id: SpeciesId) -> Result<&mut Species> {
        let key = DefaultKey::from(id);
        self.map.get_mut(key).ok_or(Error::UnknownSpecies(id))
    }

    /// True when the id resolves to a live species.
    #[must_use]
    pub fn contains(&self, id: SpeciesId) -> bool {
        let key = DefaultKey::from(id);
        self.map.get(key).is_some()
    }

    /// Removes a species. Later use of its id returns [`Error::UnknownSpecies`].
    pub fn remove(&mut self, id: SpeciesId) {
        let key = DefaultKey::from(id);
        self.map.remove(key);
    }

    /// Number of live species.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when no species is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterates live species in slotmap order (insertion order for live slots).
    pub fn iter(&self) -> SpeciesRegistryIter<'_> {
        SpeciesRegistryIter {
            inner: self.map.iter(),
        }
    }

    /// Spawns a mutated variant of `parent`.
    ///
    /// The variant copies the parent's traits with `mutate` applied and gets
    /// a name derived from the parent's. Insertion errors propagate.
    ///
    /// # Errors
    /// Returns [`Error::UnknownSpecies`] when the parent id is stale, or
    /// [`Error::DuplicateSpecies`] when the generated name is taken.
    pub fn spawn_variant<F>(&mut self, parent: SpeciesId, mutate: F) -> Result<SpeciesId>
    where
        F: FnOnce(pbma_model::SpeciesTraits) -> pbma_model::SpeciesTraits,
    {
        let base = self.get(parent)?;
        let name = format!("{}'", base.name);
        let variant = Species {
            name,
            traits: mutate(base.traits),
        };
        self.insert(variant)
    }
}

impl Default for SpeciesRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over live species with their ids.
pub struct SpeciesRegistryIter<'a> {
    inner: slotmap::basic::Iter<'a, DefaultKey, Species>,
}

impl<'a> Iterator for SpeciesRegistryIter<'a> {
    type Item = (SpeciesId, &'a Species);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (SpeciesId::from_key(k), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbma_model::SpeciesTraits;

    fn species(name: &str) -> Species {
        Species {
            name: name.into(),
            traits: SpeciesTraits {
                optimal_temperature: 300.0,
                temperature_tolerance: 5.0,
                growth_rate: 0.1,
                crowding_threshold: 100.0,
            },
        }
    }

    #[test]
    fn insert_and_get_roundtrip() {
        let mut reg = SpeciesRegistry::new();
        let id = reg.insert(species("a")).unwrap();
        assert_eq!(reg.get(id).unwrap().name, "a");
        assert!(reg.contains(id));
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut reg = SpeciesRegistry::new();
        reg.insert(species("a")).unwrap();
        assert!(reg.insert(species("a")).is_err());
    }

    #[test]
    fn removed_key_becomes_stale() {
        let mut reg = SpeciesRegistry::new();
        let id = reg.insert(species("a")).unwrap();
        reg.remove(id);
        assert!(!reg.contains(id));
        assert!(reg.get(id).is_err());
        // slot reuse must not resurrect the old id
        let _ = reg.insert(species("b")).unwrap();
        assert!(reg.get(id).is_err());
    }

    #[test]
    fn spawn_variant_copies_and_mutates() {
        let mut reg = SpeciesRegistry::new();
        let parent = reg.insert(species("a")).unwrap();
        let child = reg
            .spawn_variant(parent, |mut t| {
                t.growth_rate += 0.01;
                t
            })
            .unwrap();
        assert_eq!(reg.get(child).unwrap().name, "a'");
        assert!(
            reg.get(child).unwrap().traits.growth_rate
                > reg.get(parent).unwrap().traits.growth_rate
        );
    }

    #[test]
    fn variant_of_stale_parent_fails() {
        let mut reg = SpeciesRegistry::new();
        let parent = reg.insert(species("a")).unwrap();
        reg.remove(parent);
        assert!(reg.spawn_variant(parent, |t| t).is_err());
    }

    #[test]
    fn duplicate_variant_name_rejected() {
        let mut reg = SpeciesRegistry::new();
        let parent = reg.insert(species("a")).unwrap();
        reg.spawn_variant(parent, |t| t).unwrap();
        assert!(reg.spawn_variant(parent, |t| t).is_err());
    }

    #[test]
    fn get_mut_allows_trait_updates() {
        let mut reg = SpeciesRegistry::new();
        let id = reg.insert(species("a")).unwrap();
        reg.get_mut(id).unwrap().traits.growth_rate = 0.5;
        assert_eq!(reg.get(id).unwrap().traits.growth_rate, 0.5);
    }
}
