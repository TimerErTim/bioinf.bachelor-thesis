//! PBMA kernel: world state, tick orchestration, and ports.
//!
//! The kernel knows *when* phases run, adapters know *how* they run. All
//! simulation behavior is injected through the port traits in [`ports`];
//! the kernel never depends on concrete adapters. This is the API
//! downstream consumers (CLI, GUI, experiment tooling) build against.

#![deny(missing_docs)]

/// Error types shared across the kernel.
pub mod error;

/// Port traits: the hexagonal boundary of the kernel.
pub mod ports;

/// Species registry backed by a slotmap with generational keys.
pub mod registry;

/// World state and tick orchestration.
pub mod world;

pub use error::{Error, Result};
pub use registry::{SpeciesRegistry, SpeciesRegistryIter};
pub use world::{TickDriver, World};
