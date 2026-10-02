//! Port traits: the injected behavior boundary of the kernel.

use pbma_model::{EnvState, GlobalParams, Population, SpeciesId};

/// Context handed to phases for one cell.
///
/// Read-only view of the previous tick plus the species populations present
/// in that cell. Iteration over `populations` is in ascending [`SpeciesId`]
/// order (determinism rule).
pub struct CellContext<'a> {
    /// Cell coordinates.
    pub x: u32,
    /// Cell coordinates.
    pub y: u32,
    /// Environment state from the previous tick (frozen snapshot).
    pub env: &'a EnvState,
    /// Neighboring environment states in fixed order:
    /// north, east, south, west (Von Neumann ring, missing edges skipped).
    pub neighbors: [&'a EnvState; 4],
    /// Populations present in this cell, ascending by species id.
    pub populations: &'a [(SpeciesId, Population)],
    /// Global parameters, constant for the run.
    pub params: &'a GlobalParams,
}

/// Updates the environment of one cell (cellular automaton step).
///
/// Implementations must be pure functions of the snapshot: no interior
/// mutation, no global state, no wall-clock dependence.
pub trait EnvironmentStep {
    /// Computes the next environment state for one cell.
    fn step(&self, cell: &CellContext<'_>) -> EnvState;
}

/// Outcome of the behavior phase for one species in one cell.
#[derive(Debug, Clone, Copy)]
pub struct BehaviorDelta {
    /// Net population change (births minus deaths) in this cell.
    pub delta: f64,
}

/// Updates populations of one species in one cell.
///
/// Later increments replace the default implementation with RL-derived
/// pre-compiled formulas; the kernel contract stays identical.
pub trait PopulationBehavior {
    /// Computes the net population delta for one species in one cell.
    fn update(&self, species: SpeciesId, cell: &CellContext<'_>) -> BehaviorDelta;
}

/// A migration flux: individuals moving from one cell to a neighbor.
#[derive(Debug, Clone, Copy)]
pub struct Flux {
    /// Source cell x.
    pub from_x: u32,
    /// Source cell y.
    pub from_y: u32,
    /// Direction of movement as (dx, dy), one of the four Von Neumann steps.
    pub direction: (i32, i32),
    /// Species that migrates.
    pub species: SpeciesId,
    /// Amount of individuals moving.
    pub amount: f64,
}

/// Computes migration fluxes for one cell.
///
/// Fluxes are emitted into a per-tick buffer and applied after all
/// computation; source and target never see each other's writes mid-tick.
pub trait MigrationPolicy {
    /// Emits fluxes leaving `cell` this tick, appended in deterministic order.
    fn fluxes(&self, cell: &CellContext<'_>, out: &mut Vec<Flux>);
}

/// Deterministic, seedable random number generator.
///
/// Randomness is always passed explicitly to keep runs reproducible.
pub trait Rng {
    /// Returns the next uniform float in [0, 1).
    fn next_f64(&mut self) -> f64;

    /// Derives an independent child generator (used for per-cell streams).
    fn child(&self, salt: u64) -> Box<dyn Rng>;
}

/// Receives per-tick metrics.
pub trait Recorder {
    /// Records one metric value for the given tick.
    fn record(&mut self, tick: u64, metric: &str, value: f64);
}
