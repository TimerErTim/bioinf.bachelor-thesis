# AGENTS.md — Agent Development Contract

Entry point for agentic development in this repo. Read this before making changes.

## What this project is

Bachelor thesis framework "pbma": population-based multi-agent simulation of large-scale ecosystem dynamics on a discrete grid. One agent = one species population in one cell. Built as a downstream-consumable Rust framework (kernel + adapters), not a one-off simulation script.

## Project map

```
crates/
  pbma-model   pure domain types (Species, Population, EnvState, GlobalParams)
  pbma-core    kernel: TickDriver, port traits, SpeciesRegistry, errors
  pbma-grid    grid storage: coords, chunked Grid<T>, neighborhoods, double buffer
  pbma-sim     default adapters: diffusion CA, behavior, migration
runnables/
  pbma-cli     headless runner (config -> ticks -> CSV)
  pbma-gui     visualization, consumes kernel APIs only
docs/
  ARCHITECTURE.md  crate map, tick pipeline, determinism rules, ADR log
  GUIDELINES.md    coding practices, jj workflow, thesis-sync rule, DoD
  ROADMAP.md       milestones + progress log (update every increment)
thesis/
  chapters/        Typst thesis; implementation.typ must stay in sync with code
```

## Where code goes

- New domain type -> `pbma-model`.
- New orchestration/port/registry/error -> `pbma-core`.
- Grid math or storage -> `pbma-grid`.
- New simulation rule/phase implementation -> `pbma-sim` (as an adapter implementing a `pbma-core` port).
- New binary -> `runnables/`. Binaries wire concrete adapters; libs never depend on concrete adapters.
- Docs/progress -> `docs/`; thesis prose -> `thesis/chapters/`.

## Hard rules

1. **Thesis sync**: every framework-code change ships with its thesis update (mainly `thesis/chapters/implementation.typ`) in the same commit. Code-only commits are not allowed (pure chores exempt).
2. **Determinism**: follow the rules in `docs/ARCHITECTURE.md` (snapshot reads, fixed iteration order, explicit RNG). Same seed + config must reproduce identical results.
3. **No panics in lib paths**: return `Result`. `missing_docs` is denied.
4. **Incremental commits**: one logical change per commit, commit as soon as it is verified. Plain `jj commit -m "..."` with system-inferred author — never set `JJ_USER`/`JJ_EMAIL`. Conventional commit style. Real newlines in descriptions, never escaped. `jj new` after a finished change.
5. **Checks before commit**: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`. Benches if hot paths changed. `mise run fmt:typst` after thesis edits.
6. **Progress**: update `docs/ROADMAP.md` (checkboxes + dated progress-log line) at the end of every increment.

## Workflow per increment

1. Pick the next unchecked roadmap item.
2. Implement in small logical steps; verify each step; commit each step plainly (conventional message).
3. Update thesis chapters + ROADMAP in the matching commit(s).
4. Finish increment with green checks and a dated progress-log entry.
