# PBMA Roadmap

Progress tracker for the framework. Check off items and append a dated line to the progress log at the end of every increment.

## Milestones

### M0 — Docs & scaffolding

- [x] `docs/ARCHITECTURE.md` (crate map, tick pipeline, determinism rules, ADR log)
- [x] `docs/GUIDELINES.md` (software practices, jj workflow, thesis-sync rule, DoD)
- [x] `docs/ROADMAP.md` (this file)
- [x] ~~`AGENTS.md` (agentic development contract)~~ removed again: no agentic development on this project

### M1 — Vertical slice

- [x] Workspace restructure: `crates/*` + `runnables/*` members, workspace deps, `[workspace.lints]`
- [x] `pbma-model`: `EnvState` (O2, CO2, CH4, H2O, temperature), `Species`, `Population`, `GlobalParams`
- [x] `pbma-grid`: `Grid<T>` chunked dense storage, coord math, Von Neumann/Moore neighborhoods, double buffer
- [x] `pbma-core`: port traits (`EnvironmentStep`, `PopulationBehavior`, `MigrationPolicy`, `Rng`, `Recorder`), `SpeciesRegistry` (slotmap), `TickDriver`, errors
- [x] `pbma-sim`: diffusion CA (mass-conserving), population growth/decline, overcrowding migration
- [x] `pbma-cli`: config (grid size, seed, ticks) -> run -> CSV metrics
- [x] Tests: golden-trace determinism, gas-mass conservation property, population bookkeeping
- [x] Bench: criterion tick bench at ~1M cells
- [x] Thesis: `implementation.typ` reflects kernel/grid/sim architecture

### M2 — Behavior depth

- [x] Species traits/parameters influencing behavior (growth rate, temperature optimum/tolerance, crowding threshold read from registry)
- [x] Extinction/speciation via registry (`EvolutionEngine` port, `VariationEngine` adapter, stale-key handling proven in kernel tests)
- [x] First RL-derived "pre-compiled" behavior adapter behind `PopulationBehavior` (`FormulaBehavior` closed-form surrogate with analytic equilibrium test)
- [x] Deterministic `SplitMixRng` adapter for seeded evolution

### M3 — Design items (review follow-ups)

- [x] Extensible gas set: core ships predefined gases (`O2`, `CO2`, `CH4`, `H2O`, `O3`, `N2`), downstream consumers can register more at setup time via `GasRegistry` (`GasId` + inline-first `GasAmounts`)
- [x] Species-as-trait redesign: `SpeciesLike` interface with `taxon()`, `traits()`, `parent()`, `spawn_tick()`; taxon derived from trait values (e.g. `ot:300.00;tt:10.00`)
- [x] Runtime-extensible species traits via `TraitSchema` (`TraitId` + inline-first `SpeciesTraits`) so downstream consumers can configure agents without patching core
- [ ] Implicit species divergence: Jev-like decision models express modifications as "growth-rate-add/down noun: yes/no"; a lineage carrying both triggers a divergent species split
- [x] Global parameter extension: planetary mass & diameter (derived gravity & area), orbital distance, eccentricity, year/day length, axial tilt, solar luminosity, magnetic field strength, geothermal flux, and orbital/ozone/greenhouse physics helpers
- [ ] Audit pass over `pbma-sim` adapters (internal review: not yet independently verified)

### M3 — Parallel scaling

- [x] Chunk-parallel phases with `rayon`, verified bit-identical to sequential runs
- [ ] Halo exchange / ghost cells for chunk locality
- [ ] Scaling benches (1 vs N threads), memory profile

### M4 — GUI

- [ ] `pbma-gui` renders grid snapshots + telemetry from kernel APIs (no sim logic in GUI)

### M5 — Hypothesis experiments

- [ ] Great Oxidation Event scenario config
- [ ] Biodiversity metrics (alpha/gamma diversity) in recorder
- [ ] Experiment runs + thesis results chapter

## Progress log

- 2026-10-02: M0 complete. Architecture, guidelines, roadmap, agent contract written.
- 2026-10-02: M1 complete. Multi-crate workspace (model, core, grid, sim + cli/gui runnables). Kernel ports, slotmap registry, tick driver; chunked grid with double buffer; diffusion CA, logistic behavior, overcrowding migration; headless CLI with CSV metrics. Integration tests: golden-trace determinism, gas-mass conservation, migration bookkeeping. Bench: 1M-cell tick ~224 ms (sequential baseline). Thesis implementation chapter written in sync; methodology chapter extended with offline-training/runtime-formula section.
- 2026-10-02: M2 complete. `CellContext` exposes the registry; behavior/migration fully traits-driven. New `EvolutionEngine` port + `VariationEngine` adapter (extinction pruning, drifted variants), `SplitMixRng`, `FormulaBehavior` closed-form RL surrogate with analytic equilibrium test. Kernel evolution integration tests prove pruning, staleness, variant registration. 36 unit + 3 integration tests green; bench improved to ~207 ms/tick. Thesis implementation chapter extended in sync.
- 2026-10-08: Review batch. `pbma-model`/`pbma-grid` merged into `pbma-core` (modules `model`, `grid`); `Amount` renamed to `PositiveF64`; neighbor context extended to the Moore ring with 1/sqrt(2) diagonal weights (stability cap 1/(4+4/sqrt(2))); metric recording made consumer-defined (`Recorder::record(tick, &World)`); `light_intensity` added to `GlobalParams`. Design follow-ups tracked in M3 design items.
- 2026-11-16: Core sophistication & optimization batch. Fixed chunk-first `Grid<T>` storage and wired `DoubleBuffer<Cell>` into `World` and `TickDriver` with zero-allocation back-buffer writes, pre-allocated per-chunk flux buffers, and `rayon` chunk parallelism (verified bit-identical to sequential runs). Added boundary `Topology` (`Clamped`, `Planetary`, `Toroidal`), `GasRegistry` (`O2`, `CO2`, `CH4`, `H2O`, `O3`, `N2` + runtime custom gases), `TraitSchema` + `SpeciesLike` (`taxon`, parent/spawn tracking), planetary/orbital physics (`PlanetParams`, `OrbitParams`, Keplerian insolation, ozone UV shielding, radiative equilibrium, scale height), and `WorldBuilder` scenario harness.
