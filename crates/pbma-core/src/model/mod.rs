//! Pure domain types for the PBMA simulation.
//!
//! These types define the vocabulary of the simulation (environment state of a
//! cell, extensible gas and trait registries, species definitions, population
//! amounts, planetary/orbital parameters, and physical helper functions).

/// Non-negative amounts with saturating arithmetic.
pub mod quantity;

/// Environment state held by a single grid cell and extensible gas registry.
pub mod env;

/// Species definition, extensible trait schema, and population bookkeeping.
pub mod species;

/// Global, planetary, and orbital simulation parameters.
pub mod params;

/// Pure planetary physics helpers (orbital insolation, ozone UV shielding,
/// radiative greenhouse balance, barometric scale height).
pub mod physics;

pub use env::{EnvState, GasAmounts, GasDef, GasId, GasRegistry, INLINE_GASES};
pub use params::{
    GRAVITATIONAL_CONSTANT, GlobalParams, IDEAL_GAS_CONSTANT, OrbitParams, PlanetParams,
    SOLAR_CONSTANT_WM2, STEFAN_BOLTZMANN,
};
pub use physics::{
    Insolation, OrbitalState, barometric_scale_height_m, cell_insolation, mean_molar_mass_kg,
    orbital_state, ozone_amount, ozone_net_delta, radiative_equilibrium_temperature,
    surface_uv_fraction,
};
pub use quantity::PositiveF64;
pub use species::{
    INLINE_TRAITS, Population, Species, SpeciesId, SpeciesLike, SpeciesTraits, TraitDef, TraitId,
    TraitSchema,
};
