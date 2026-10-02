//! Species definitions and population bookkeeping.

use serde::{Deserialize, Serialize};

use crate::quantity::Amount;

/// Stable identifier for a species within one simulation run.
///
/// Issued by the species registry in `pbma-core`; generations make stale
/// ids detectable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesId {
    /// Underlying slotmap key raw representation.
    pub raw: u64,
}

impl SpeciesId {
    /// Wraps a slotmap key into a species id.
    #[must_use]
    pub fn from_key(key: slotmap::DefaultKey) -> Self {
        Self {
            raw: slotmap::Key::data(&key).as_ffi(),
        }
    }

    /// Rebuilds the slotmap key from the raw representation.
    #[must_use]
    pub fn to_key(self) -> slotmap::DefaultKey {
        slotmap::DefaultKey::from(slotmap::KeyData::from_ffi(self.raw))
    }
}

impl From<slotmap::DefaultKey> for SpeciesId {
    fn from(key: slotmap::DefaultKey) -> Self {
        Self::from_key(key)
    }
}

impl From<SpeciesId> for slotmap::DefaultKey {
    fn from(id: SpeciesId) -> Self {
        id.to_key()
    }
}

/// A species: an evolutionary lineage whose members are similar enough to
/// be treated as one unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    /// Human-readable name, unique per run.
    pub name: String,
    /// Trait values driving behavior formulas.
    pub traits: SpeciesTraits,
}

/// Trait values of a species. Kept small and flat; behavior phases read
/// these directly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpeciesTraits {
    /// Preferred temperature in kelvin; growth declines with distance.
    pub optimal_temperature: f64,
    /// Tolerance width around [`SpeciesTraits::optimal_temperature`].
    pub temperature_tolerance: f64,
    /// Per-tile reproduction rate under ideal conditions.
    pub growth_rate: f64,
    /// Overcrowding threshold: density above this triggers emigration pressure.
    pub crowding_threshold: f64,
}

/// Population of one species in one cell.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Population {
    /// Amount of individuals (population units) in the cell.
    pub amount: Amount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_serializes_roundtrip() {
        let species = Species {
            name: "cyano".into(),
            traits: SpeciesTraits {
                optimal_temperature: 300.0,
                temperature_tolerance: 10.0,
                growth_rate: 0.1,
                crowding_threshold: 100.0,
            },
        };
        let json = serde_json::to_string(&species).unwrap();
        let back: Species = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "cyano");
    }
}
