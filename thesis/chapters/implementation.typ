= Rust Implementation

The framework is developed as a set of Rust workspace crates. Rust was chosen
for its performance profile (zero-cost abstractions, no garbage collector),
its expressive type system for encoding invariants, and first-class tooling
for testing and benchmarking. The implementation follows a hexagonal
architecture: a small kernel defines *when* simulation phases run, while
injected adapters define *how* they run. This keeps the framework consumable
by downstream applications (a headless CLI runner and a graphical
visualization) that never need to modify kernel code.

== Crate structure

The workspace separates the framework into libraries and runnable binaries:

- `pbma-model`: pure domain types with no framework dependencies. The
  environment state of a cell (gas amounts for O₂, CO₂, CH₄ and H₂O,
  temperature, light), the species definition with its trait values, the
  population quantity type with saturating non-negative arithmetic, and the
  global parameters (gravity, day length, axial tilt, diffusion coefficient).
- `pbma-core`: the kernel. It owns the world state, drives the tick loop, and
  declares the *ports* (traits) through which all simulation behavior is
  injected. It depends only on `pbma-model`.
- `pbma-grid`: grid cell management. Coordinates with canonical neighborhood
  orders, dense row-major storage with 64×64 chunk bookkeeping for later
  parallel work splitting, and double-buffered grids for snapshot-and-swap
  phases.
- `pbma-sim`: default adapters implementing the kernel ports: a
  mass-conserving diffusion cellular automaton, a logistic population
  behavior, an overcrowding-driven migration policy, and a metrics recorder.
- `runnables/pbma-cli` and `runnables/pbma-gui`: binaries that wire concrete
  adapters to the kernel. Only binaries may depend on concrete adapters; the
  libraries depend on ports exclusively.

The dependency direction is strictly inward: `model` ← `core` ← `grid`/`sim`,
with binaries at the outside. This realizes dependency inversion at the
compiler level: the kernel cannot reference adapter code even by accident.

== Species registry

Species have dynamic lifetimes (speciation and extinction happen during
runs), while cells are static. The species table is therefore a slotmap with
generational keys: removing a species invalidates its key, and a later slot
reuse bumps the generation so stale identifiers are detected instead of
silently addressing the wrong lineage. Cell storage stays a plain dense
vector, which is faster and simpler for a fixed grid. Populations within a
cell are kept sorted by species identifier, which makes iteration order
deterministic and merges cheap.

== Tick pipeline

One simulation tick runs four phases in fixed order: environment step,
population behavior, migration, and metrics recording. Each phase receives a
read-only context for one cell: the frozen environment snapshot of the
previous tick, the neighbor environments in canonical order (north, east,
south, west), and the sorted populations of the cell. The kernel collects
behavior deltas and migration fluxes into fresh buffers and applies them only
after all computation has finished, then publishes the new environment and
swaps buffers.

== Determinism rules

Reproducibility is a hard requirement for the experiments of this thesis, and
it is also what makes parallel execution safe. Four rules are enforced:

1. Phases read an immutable previous-tick snapshot and write to private
  buffers; no phase observes another phase's partial writes.
2. Neighbor aggregation always iterates in the canonical offset order.
3. Simulation logic never iterates hash maps; population lists are sorted
  vectors.
4. All randomness is passed explicitly through a seedable port, never
  generated from global state.

These rules are backed by tests: a golden-trace test asserts that the same
seed and configuration produce identical metric hashes, and a property test
asserts gas mass conservation under diffusion.

== Diffusion cellular automaton

The default environment step implements gas diffusion as pressure
equalization: for each gas, a fraction α of the positive half-difference
flows from the higher to the lower concentration across each of the four
neighbor pairs. Inflow and outflow are computed from the same frozen
snapshot, so exchanges cancel pairwise and total gas mass is conserved
exactly. α is clamped to 0.25, the stability limit of the four-neighbor
scheme. Boundary cells treat the missing neighbor as a mirror of themselves,
so no mass crosses the world edge.
