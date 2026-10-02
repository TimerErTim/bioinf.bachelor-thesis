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
