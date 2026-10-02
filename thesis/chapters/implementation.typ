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

== Population behavior and migration

The default behavior adapter implements logistic growth: each population
grows toward the carrying capacity of its cell, which scales with light
availability, and the growth rate is modulated by a temperature fitness
curve that falls off linearly beyond a tolerance window around the
species' own optimal temperature. Migration treats overcrowding as
emigration pressure: when a species' local density exceeds its own
crowding threshold, the excess fraction is split evenly over the four
cardinal directions and emitted as migration fluxes. The kernel applies
these fluxes only after all cells have been processed, so a population can
never read a half-migrated neighbor.

== Species registry as the behavior interface

The per-cell behavior and migration phases resolve trait values through
the species registry rather than through kernel constants: growth rate,
temperature optimum and tolerance, and the crowding threshold are all
per-lineage quantities. This closes a feedback loop that the vertical
slice could not express — two species in the same cell with different
traits experience different selection pressures, and trait changes
immediately change behavior without any code change.

The registry read also defines extinction semantics. When a lineage has
been removed, its population entries can temporarily survive in cells
until the next pruning step; such stale populations are detected by the
generational key check and contribute nothing to behavior or migration.
They are inert residuals, not hidden actors.

== Evolution as a kernel phase

Speciation and extinction are global operations on the species table, so
they sit outside the per-cell phases. A third port, the evolution engine,
runs once per tick after migration and has two responsibilities. First,
it reports every species that has no surviving population anywhere in the
world; the kernel removes those from the registry, which invalidates
their keys for every future use. Second, it spawns variant lineages of
surviving species: with a configurable probability, a variant copies the
parent's traits with a multiplicative drift applied to the growth rate
and a shift of the temperature optimum proportional to the parent's
tolerance width, so specialists drift less in absolute kelvin than
eurytherm species. Variant names are derived deterministically from the
parent name plus a batch index, and every random decision comes from an
explicitly injected, seedable random number generator (a splitmix64
implementation), keeping the whole evolutionary history reproducible.

Kernel-level integration tests prove the contract: a species that loses
its last populations is pruned in the same tick, stale keys are detected
instead of silently aliasing the next species in the table, and a spawned
variant appears in the registry with the derived trait values.

== Closed-form behavior surrogate

As the first stand-in for a behavior formula derived from offline
reinforcement learning, a second behavior adapter evaluates a closed-form
expression per species and cell: the effective growth rate is the product
of the species' base growth rate, a gaussian temperature fitness, a
saturating light response, and a quadratic crowding factor, minus a
constant baseline mortality. The gaussian fitness differs from the
linear ramp of the logistic adapter precisely where selection matters:
mismatched species are penalized smoothly instead of being cut off at a
hard tolerance boundary. A bisection test proves that the formula has an
interior equilibrium where growth and mortality cancel, and that this
equilibrium lies between the low-density and high-density regimes — the
qualitative property a learned policy must reproduce before it can
replace the surrogate.

== Verification and benchmarks

The simulation is verified on three levels. Unit tests cover the domain
rules of each crate, for example saturating arithmetic, registry staleness,
trait-driven growth differences between coexisting species, and chunk
bookkeeping. Integration tests assert the scientific invariants: a
golden-trace test proves that identical seeds and configurations reproduce
identical metric hashes, a conservation test proves that diffusion neither
creates nor destroys gas mass, and a bookkeeping test proves that migration
moves population between cells without creating or destroying it. Finally,
a benchmark simulates a 1000 × 1000 cell world (one million cells) and
measures the per-tick runtime — about 207 milliseconds per tick with the
trait-resolving behavior and migration phases — providing the baseline that
later parallel optimizations must improve upon.

== Extensibility contract

Downstream consumers extend the framework by implementing the kernel ports
rather than modifying it. A new environment model, a behavior formula
derived from offline reinforcement learning, or an alternative migration
policy each plug in as an adapter behind the same trait boundary; the kernel
and the existing binaries remain untouched. This is the property that makes
the framework a product in its own right rather than a one-off simulation
script.


== Diffusion cellular automaton

The default environment step implements gas diffusion as pressure
equalization: for each gas, a fraction α of the positive half-difference
flows from the higher to the lower concentration across each of the four
neighbor pairs. Inflow and outflow are computed from the same frozen
snapshot, so exchanges cancel pairwise and total gas mass is conserved
exactly. α is clamped to 0.25, the stability limit of the four-neighbor
scheme. Boundary cells treat the missing neighbor as a mirror of themselves,
so no mass crosses the world edge.
