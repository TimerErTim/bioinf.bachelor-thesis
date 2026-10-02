# PBMA Roadmap

Progress tracker for the framework. Check off items and append a dated line to the progress log at the end of every increment.

## Milestones

### M0 — Docs & scaffolding

- [x] `docs/ARCHITECTURE.md` (crate map, tick pipeline, determinism rules, ADR log)
- [x] `docs/GUIDELINES.md` (software practices, jj workflow, thesis-sync rule, DoD)
- [x] `docs/ROADMAP.md` (this file)
- [x] `AGENTS.md` (agentic development contract)

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

- [ ] Species traits/parameters influencing behavior
- [ ] Extinction/speciation via registry (stale-key handling proven in tests)
- [ ] First RL-derived "pre-compiled" behavior adapter behind `PopulationBehavior`

### M3 — Parallel scaling

- [ ] Chunk-parallel phases with rayon, verified deterministic
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
