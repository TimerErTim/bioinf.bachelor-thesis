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

The workspace separates the framework into a kernel library, an adapter
library, and runnable binaries:

- `pbma-core`: the kernel. It contains the domain types (the environment
  state of a cell: inline-first `GasAmounts` indexed by an extensible
  `GasRegistry` pre-populated with O₂, CO₂, CH₄, H₂O, O₃, and N₂,
  temperature, normalized light, and surface UV flux; the `Species` definition
  implementing `SpeciesLike` with derived taxon strings, lineage metadata, and
  an extensible `TraitSchema`; the `PositiveF64` quantity type with saturating
  non-negative arithmetic; planetary and orbital `GlobalParams` deriving
  surface gravity $g = G M / R^2$ from mass and diameter; and pure physics
  helpers for Keplerian orbital insolation, Beer--Lambert ozone UV shielding,
  greenhouse equilibrium, and barometric scale height), the grid storage
  (coordinates with canonical neighborhood orders, planetary/toroidal/clamped
  boundary topologies, chunk-first dense storage with 64×64 contiguous chunk
  slices for parallel work splitting, double-buffered grids for snapshot-and-swap
  phases), the `World` state, `WorldBuilder`, the `TickDriver`, and the *ports*
  (traits) through which all simulation behavior is injected. The domain types
  and grid storage are kept as modules of the kernel rather than as separate
  crates: downstream implementors depend on the ports of the kernel anyway, and
  a single kernel crate keeps the public surface coherent.
- `pbma-sim`: default adapters implementing the kernel ports: a
  mass-conserving diffusion cellular automaton, a logistic population
  behavior, an overcrowding-driven migration policy, a deterministic
  evolution engine, and a metrics recorder.
- `runnables/pbma-cli` and `runnables/pbma-gui`: binaries that wire concrete
  adapters to the kernel. Only binaries wire concrete adapters to the
  kernel; the kernel never references adapter code.

The dependency direction is strictly inward: `core` ← `sim`, with binaries
at the outside. This realizes dependency inversion at the compiler level:
the kernel cannot reference adapter code even by accident.

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
previous tick, the neighbor environments in canonical order (north,
north-east, east, south-east, south, south-west, west, north-west — the
Moore ring, clockwise from north, resolved under the configured boundary
topology where diagonal neighbors carry weight $1 / sqrt(2)$ in exchange
computations since they are farther away by a factor of $sqrt(2)$), geographic
coordinates and instantaneous orbital insolation, and the sorted populations
of the cell. The `World` holds a `DoubleBuffer<Cell>` organized in chunk-first
order: phases read the immutable front buffer and write directly into
pre-allocated back-buffer cells and per-chunk flux buffers across `rayon`
workers without cloning the grid each tick. Migration fluxes are then merged
in canonical chunk order before swapping buffers, preserving bit-identical
results between single-threaded and parallel execution. The metrics phase is
consumer-defined: the kernel hands the recorder the complete final world state
of the tick, and the recorder decides what to extract — a few aggregates,
detailed rows, or a verbatim snapshot of every cell on every tick. The kernel
itself knows no metrics at all.

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

The current variant model modifies trait values by numeric drift, but the
direction of travel is a more expressive, *implicit* divergence model.
Later integration of Jev-like decision models will express species
modifications not as direct growth-rate add/subtract operations, but as
decisions over named effects ("growth-rate-add _noun_: yes/no",
"growth-rate-down _noun_: yes/no"). A lineage that accumulates both
directions for the same effect is pushed toward a divergent species split:
the modification structure itself decides when speciation happens, without
a hand-coded threshold. This is planned work (marked as TODO in the kernel
and tracked in the roadmap) and does not affect the current experiments.

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
measures the per-tick runtime — about 113 milliseconds per tick for the
full eight-neighbor environment phase across six gases with orbital insolation,
trait-resolving behavior, and migration phases — down from 225 milliseconds in
the initial single-threaded snapshot-cloning baseline.

== Extensibility contract

Downstream consumers extend the framework by implementing the kernel ports
rather than modifying it. A new environment model, a decision-model or
formula-driven behavior adapter, or an alternative migration policy each plug in
as an adapter behind the same trait boundary; the kernel and the existing
binaries remain untouched. Furthermore, `GasRegistry` and `TraitSchema` allow
consumers to register additional atmospheric gases and species traits at world
setup via `WorldBuilder`, while `SpeciesLike` exposes derived taxon descriptors
and lineage ancestry without patching the kernel. This is the property that
makes the framework a reusable experimental platform rather than a one-off
simulation script.


== Diffusion cellular automaton

The default environment step implements gas diffusion as pressure
equalization: for each gas, a fraction of the half-difference flows from
the higher to the lower concentration across each of the eight neighbor
pairs (the Moore ring). The exchange between a cell and one neighbor is
$α w_i (c_("other") - c_("own"))$, where $w_i = 1.0$ for the four axis
neighbors and $w_i = 1 / sqrt(2)$ for the four diagonals, which are farther
away by a factor of $sqrt(2)$ and therefore influence the exchange less.
The weights are symmetric per neighbor pair, and inflow and outflow are
computed from the same frozen snapshot, so exchanges cancel pairwise and
total gas mass is conserved exactly. α is clamped to $1 / (4 + 4 / sqrt(2))
approx 0.146$, the stability limit of the weighted scheme: total outflow
can then never exceed the cell's own content. Boundary cells treat the
missing neighbor as a mirror of themselves, so no mass crosses the world
edge.

== Data layout and cache behavior

How the fields of a cell are arranged in memory matters for a simulation
that touches millions of cells per tick. Two layouts compete. In an Array
of Structures (AoS), one array stores complete records: cell 0 with all
its fields, then cell 1 with all of its fields, and so on. In a Structure
of Arrays (SoA), every field gets its own array: all gas amounts in one
array, all temperatures in another, all light values in a third. The
difference shows up in the CPU cache. A cache does not load single values
but whole cache lines (typically 64 bytes) at once, and a cache miss — a
round trip to main memory because the needed line is not cached — costs
orders of magnitude more than a cached access. If one update touches all
fields of a cell, AoS pulls exactly what is needed into the cache and
uses every byte of the loaded line. If instead one update touches only
one field across many cells, AoS drags the unused fields along, wastes
most of each loaded line, and cache misses multiply.

The framework deliberately uses AoS: the environment state bundles the
four gas amounts, temperature, and light in one struct, and every
grid-cell update of the environment phase reads and writes all fields
together. With all fields in use per update, AoS is at least as efficient
as SoA and considerably simpler to pass through the port interfaces. If
profiling ever shows phases that touch only a subset of fields across many
cells, migrating to SoA would trade interface simplicity for fewer cache
misses, but would require redesigning the update passes and context
types. The kernel code records this trade-off at the type definition.

// TODO(thesis): add a small figure contrasting AoS and SoA memory layout
// (two memory bars with field coloring plus a cache-line overlay).
// Optional; the prose above is self-contained.
