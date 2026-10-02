# PBMA Roadmap

Progress tracker for the framework. Check off items and append a dated line to the progress log at the end of every increment.

## Milestones

### M0 — Docs & scaffolding

- [x] `docs/ARCHITECTURE.md` (crate map, tick pipeline, determinism rules, ADR log)
- [x] `docs/GUIDELINES.md` (software practices, jj workflow, thesis-sync rule, DoD)
- [x] `docs/ROADMAP.md` (this file)
- [x] `AGENTS.md` (agentic development contract)

### M1 — Vertical slice

- [ ] Workspace restructure: `crates/*` + `runnables/*` members, workspace deps, `[workspace.lints]`
- [ ] `pbma-model`: `EnvState` (O2, CO2, CH4, H2O, temperature), `Species`, `Population`, `GlobalParams`
- [ ] `pbma-grid`: `Grid<T>` chunked dense storage, coord math, Von Neumann/Moore neighborhoods, double buffer
- [ ] `pbma-core`: port traits (`EnvironmentStep`, `PopulationBehavior`, `MigrationPolicy`, `Rng`, `Recorder`), `SpeciesRegistry` (slotmap), `TickDriver`, errors
- [ ] `pbma-sim`: diffusion CA (mass-conserving), population growth/decline, overcrowding migration
- [ ] `pbma-cli`: config (grid size, seed, ticks) -> run -> CSV metrics
- [ ] Tests: golden-trace determinism, gas-mass conservation property, population bookkeeping
- [ ] Bench: criterion tick bench at ~1M cells
- [ ] Thesis: `implementation.typ` reflects kernel/grid/sim architecture

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
