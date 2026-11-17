//! PBMA kernel: domain types, world state, tick orchestration, and ports.
//!
//! The kernel knows *when* phases run, adapters know *how* they run. All
//! simulation behavior is injected through the port traits in [`ports`];
//! the kernel never depends on concrete adapters. This is the API
//! downstream consumers (CLI, GUI, experiment tooling) build against.

#![deny(missing_docs)]

/// Domain types: cells, species, populations, global parameters, physics.
pub mod model;

/// Grid storage, boundary topologies, and neighborhood coordinates.
pub mod grid;

/// Error types shared across the kernel.
pub mod error;

/// Port traits: the hexagonal boundary of the kernel.
pub mod ports;

/// Species registry backed by a slotmap with generational keys.
pub mod registry;

/// World state and double-buffered, chunk-parallel tick orchestration.
pub mod world;

/// Fluent scenario builder for tests and experiments.
pub mod builder;

pub use builder::WorldBuilder;
pub use error::{Error, Result};
pub use model::{
    EnvState, GasAmounts, GasDef, GasId, GasRegistry, GlobalParams, Insolation, OrbitParams,
    OrbitalState, PlanetParams, Population, PositiveF64, Species, SpeciesId, SpeciesLike,
    SpeciesTraits, TraitDef, TraitId, TraitSchema,
};
pub use registry::{SpeciesRegistry, SpeciesRegistryIter};
pub use world::{TickDriver, World};
