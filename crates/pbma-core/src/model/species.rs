//! Species definitions, extensible trait schema, and population bookkeeping.

use std::fmt::Write;

use serde::{Deserialize, Serialize};

use super::quantity::PositiveF64;
use crate::{Error, Result};

/// Inline trait slots before spilling to heap storage.
pub const INLINE_TRAITS: usize = 8;

/// Stable identifier for a species within one simulation run.
///
/// Issued by the species registry in `pbma-core`; generations make stale
/// ids detectable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Identifier for a species trait in a [`TraitSchema`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TraitId(pub u16);

impl TraitId {
    /// Preferred temperature in kelvin (`"ot"`).
    pub const OPTIMAL_TEMPERATURE: Self = Self(0);
    /// Temperature tolerance width in kelvin (`"tt"`).
    pub const TEMPERATURE_TOLERANCE: Self = Self(1);
    /// Per-tick reproduction rate (`"gr"`).
    pub const GROWTH_RATE: Self = Self(2);
    /// Overcrowding migration threshold (`"ct"`).
    pub const CROWDING_THRESHOLD: Self = Self(3);
    /// Tolerance to surface UV radiation (`"uv"`).
    pub const UV_TOLERANCE: Self = Self(4);
    /// Per-capita $\mathrm{O_2}$ production coefficient (`"o2p"`).
    pub const O2_PRODUCTION: Self = Self(5);
    /// Sensitivity to ambient $\mathrm{O_2}$ toxicity (`"o2t"`).
    pub const O2_TOXICITY: Self = Self(6);
    /// Baseline metabolic maintenance rate (`"mr"`).
    pub const METABOLIC_RATE: Self = Self(7);

    /// Index of this trait in [`SpeciesTraits`].
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Metadata describing one species trait dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDef {
    /// Short deterministic key used in derived taxon identifiers (e.g. `"ot"`).
    pub key: String,
    /// Human-readable description of the trait.
    pub description: String,
    /// Default value assigned to species that do not override this trait.
    pub default_value: f64,
}

/// Runtime-extensible schema of species traits.
///
/// Pre-populates the eight core traits (`ot`, `tt`, `gr`, `ct`, `uv`, `o2p`,
/// `o2t`, `mr`) and lets downstream consumers register additional trait
/// dimensions without patching `pbma-core`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSchema {
    traits: Vec<TraitDef>,
}

impl TraitSchema {
    /// Creates a schema pre-populated with the eight built-in traits.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            traits: vec![
                TraitDef {
                    key: "ot".into(),
                    description: "Optimal temperature (K)".into(),
                    default_value: 288.0,
                },
                TraitDef {
                    key: "tt".into(),
                    description: "Temperature tolerance width (K)".into(),
                    default_value: 10.0,
                },
                TraitDef {
                    key: "gr".into(),
                    description: "Intrinsic per-tick growth rate".into(),
                    default_value: 0.1,
                },
                TraitDef {
                    key: "ct".into(),
                    description: "Overcrowding migration threshold".into(),
                    default_value: 100.0,
                },
                TraitDef {
                    key: "uv".into(),
                    description: "UV radiation tolerance".into(),
                    default_value: 1.0,
                },
                TraitDef {
                    key: "o2p".into(),
                    description: "Per-capita O2 production rate".into(),
                    default_value: 0.0,
                },
                TraitDef {
                    key: "o2t".into(),
                    description: "Ambient O2 toxicity sensitivity".into(),
                    default_value: 0.0,
                },
                TraitDef {
                    key: "mr".into(),
                    description: "Baseline metabolic maintenance rate".into(),
                    default_value: 0.02,
                },
            ],
        }
    }

    /// Registers a custom trait dimension and returns its [`TraitId`].
    ///
    /// # Errors
    /// Returns [`Error::DuplicateTrait`] if `def.key` already exists.
    pub fn register(&mut self, def: TraitDef) -> Result<TraitId> {
        if self.traits.iter().any(|t| t.key == def.key) {
            return Err(Error::DuplicateTrait(def.key));
        }
        let id = u16::try_from(self.traits.len())
            .map_err(|_| Error::InvalidParams("trait schema capacity exceeded".into()))?;
        self.traits.push(def);
        Ok(TraitId(id))
    }

    /// Looks up trait metadata by id.
    ///
    /// # Errors
    /// Returns [`Error::UnknownTrait`] if `id` is out of bounds.
    pub fn get(&self, id: TraitId) -> Result<&TraitDef> {
        self.traits.get(id.index()).ok_or(Error::UnknownTrait(id))
    }

    /// Finds a trait id by its short key.
    #[must_use]
    pub fn by_key(&self, key: &str) -> Option<TraitId> {
        self.traits
            .iter()
            .position(|t| t.key == key)
            .map(|i| TraitId(i as u16))
    }

    /// Number of registered traits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.traits.len()
    }

    /// True when empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.traits.is_empty()
    }

    /// Iterates `(TraitId, &TraitDef)` in canonical order.
    pub fn iter(&self) -> impl Iterator<Item = (TraitId, &TraitDef)> {
        self.traits
            .iter()
            .enumerate()
            .map(|(i, def)| (TraitId(i as u16), def))
    }

    /// Builds default [`SpeciesTraits`] matching all entries in this schema.
    #[must_use]
    pub fn default_traits(&self) -> SpeciesTraits {
        let mut out = SpeciesTraits::default();
        for (id, def) in self.iter() {
            out.set(id, def.default_value);
        }
        out
    }
}

impl Default for TraitSchema {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Extensible trait vector of a species.
///
/// Stores the eight core traits inline without heap indirection and spills
/// custom traits registered in [`TraitSchema`] to `extra`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesTraits {
    inline: [f64; INLINE_TRAITS],
    len: usize,
    extra: Vec<f64>,
}

impl SpeciesTraits {
    /// Creates core traits with sensible defaults for extended attributes
    /// (`uv_tolerance = 1.0`, `o2_production = 0.0`, `o2_toxicity = 0.0`,
    /// `metabolic_rate = 0.02`).
    #[must_use]
    pub const fn new(
        optimal_temperature: f64,
        temperature_tolerance: f64,
        growth_rate: f64,
        crowding_threshold: f64,
    ) -> Self {
        Self {
            inline: [
                optimal_temperature,
                temperature_tolerance,
                growth_rate,
                crowding_threshold,
                1.0,
                0.0,
                0.0,
                0.02,
            ],
            len: INLINE_TRAITS,
            extra: Vec::new(),
        }
    }

    /// Reads trait `id`, returning `0.0` if out of bounds.
    #[must_use]
    #[inline]
    pub fn get(&self, id: TraitId) -> f64 {
        let idx = id.index();
        if idx >= self.len {
            0.0
        } else if idx < INLINE_TRAITS {
            self.inline[idx]
        } else {
            self.extra[idx - INLINE_TRAITS]
        }
    }

    /// Sets trait `id`, extending storage if needed.
    #[inline]
    pub fn set(&mut self, id: TraitId, value: f64) {
        let idx = id.index();
        if idx >= self.len {
            if idx >= INLINE_TRAITS {
                self.extra.resize(idx + 1 - INLINE_TRAITS, 0.0);
            }
            self.len = idx + 1;
        }
        if idx < INLINE_TRAITS {
            self.inline[idx] = value;
        } else {
            self.extra[idx - INLINE_TRAITS] = value;
        }
    }

    /// Fluent helper setting trait `id` and returning `self`.
    #[must_use]
    pub fn with(mut self, id: TraitId, value: f64) -> Self {
        self.set(id, value);
        self
    }

    /// Preferred temperature in kelvin.
    #[must_use]
    #[inline]
    pub fn optimal_temperature(&self) -> f64 {
        self.inline[TraitId::OPTIMAL_TEMPERATURE.index()]
    }

    /// Temperature tolerance width in kelvin.
    #[must_use]
    #[inline]
    pub fn temperature_tolerance(&self) -> f64 {
        self.inline[TraitId::TEMPERATURE_TOLERANCE.index()]
    }

    /// Per-tick reproduction rate under ideal conditions.
    #[must_use]
    #[inline]
    pub fn growth_rate(&self) -> f64 {
        self.inline[TraitId::GROWTH_RATE.index()]
    }

    /// Density above which emigration pressure is triggered.
    #[must_use]
    #[inline]
    pub fn crowding_threshold(&self) -> f64 {
        self.inline[TraitId::CROWDING_THRESHOLD.index()]
    }

    /// Tolerance to surface UV flux.
    #[must_use]
    #[inline]
    pub fn uv_tolerance(&self) -> f64 {
        self.inline[TraitId::UV_TOLERANCE.index()]
    }

    /// Per-capita $\mathrm{O_2}$ production coefficient.
    #[must_use]
    #[inline]
    pub fn o2_production(&self) -> f64 {
        self.inline[TraitId::O2_PRODUCTION.index()]
    }

    /// Sensitivity to ambient $\mathrm{O_2}$ toxicity.
    #[must_use]
    #[inline]
    pub fn o2_toxicity(&self) -> f64 {
        self.inline[TraitId::O2_TOXICITY.index()]
    }

    /// Baseline metabolic maintenance rate.
    #[must_use]
    #[inline]
    pub fn metabolic_rate(&self) -> f64 {
        self.inline[TraitId::METABOLIC_RATE.index()]
    }

    /// Derives a canonical taxon descriptor string from trait values,
    /// e.g. `"ot:300.00;tt:10.00;gr:0.10;ct:100.00"`.
    #[must_use]
    pub fn taxon(&self, schema: &TraitSchema) -> String {
        let mut out = String::new();
        for (idx, (id, def)) in schema.iter().enumerate() {
            if idx > 0 {
                out.push(';');
            }
            let val = if id.index() < self.len {
                self.get(id)
            } else {
                def.default_value
            };
            let _ = write!(out, "{}:{val:.2}", def.key);
        }
        out
    }
}

impl Default for SpeciesTraits {
    fn default() -> Self {
        Self::new(288.0, 10.0, 0.1, 100.0)
    }
}

/// Interface implemented by species definitions so downstream consumers can
/// inspect lineages polymorphically.
pub trait SpeciesLike {
    /// Human-readable unique lineage name.
    fn name(&self) -> &str;
    /// Derived canonical taxon descriptor from trait values.
    fn taxon(&self, schema: &TraitSchema) -> String;
    /// Trait values driving behavior phases.
    fn traits(&self) -> &SpeciesTraits;
    /// Mutable trait values.
    fn traits_mut(&mut self) -> &mut SpeciesTraits;
    /// Parent species id if spawned via evolution.
    fn parent(&self) -> Option<SpeciesId>;
    /// Simulation tick on which this species originated.
    fn spawn_tick(&self) -> u64;
}

/// A species: an evolutionary lineage whose members are similar enough to
/// be treated as one unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Species {
    /// Human-readable name, unique per run.
    pub name: String,
    /// Trait values driving behavior formulas.
    pub traits: SpeciesTraits,
    /// Parent lineage if spawned by evolution.
    pub parent: Option<SpeciesId>,
    /// Tick on which this lineage appeared (`0` for initial species).
    pub spawn_tick: u64,
}

impl Species {
    /// Creates a root species originating at tick `0`.
    #[must_use]
    pub fn new(name: impl Into<String>, traits: SpeciesTraits) -> Self {
        Self {
            name: name.into(),
            traits,
            parent: None,
            spawn_tick: 0,
        }
    }

    /// Creates a variant species descended from `parent` at `spawn_tick`.
    #[must_use]
    pub fn variant(
        name: impl Into<String>,
        traits: SpeciesTraits,
        parent: SpeciesId,
        spawn_tick: u64,
    ) -> Self {
        Self {
            name: name.into(),
            traits,
            parent: Some(parent),
            spawn_tick,
        }
    }
}

impl SpeciesLike for Species {
    fn name(&self) -> &str {
        &self.name
    }

    fn taxon(&self, schema: &TraitSchema) -> String {
        self.traits.taxon(schema)
    }

    fn traits(&self) -> &SpeciesTraits {
        &self.traits
    }

    fn traits_mut(&mut self) -> &mut SpeciesTraits {
        &mut self.traits
    }

    fn parent(&self) -> Option<SpeciesId> {
        self.parent
    }

    fn spawn_tick(&self) -> u64 {
        self.spawn_tick
    }
}

/// Population of one species in one cell.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Population {
    /// PositiveF64 of individuals (population units) in the cell.
    pub amount: PositiveF64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_serializes_roundtrip() {
        let species = Species::new("cyano", SpeciesTraits::new(300.0, 10.0, 0.1, 100.0));
        let json = serde_json::to_string(&species).unwrap();
        let back: Species = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "cyano");
        assert!((back.traits.optimal_temperature() - 300.0).abs() < 1e-12);
    }

    #[test]
    fn custom_trait_extends_schema_and_taxon() {
        let mut schema = TraitSchema::with_defaults();
        let motility = schema
            .register(TraitDef {
                key: "mot".into(),
                description: "Motility speed".into(),
                default_value: 0.5,
            })
            .unwrap();
        let mut traits = schema.default_traits();
        traits.set(motility, 1.75);
        assert!((traits.get(motility) - 1.75).abs() < 1e-12);
        let taxon = traits.taxon(&schema);
        assert!(taxon.contains("ot:288.00"));
        assert!(taxon.contains("mot:1.75"));
    }
}
