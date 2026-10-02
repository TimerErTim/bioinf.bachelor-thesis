# PBMA Development Guidelines

Software practices for this repo. These are enforced in review and in CI-style local checks.

## Rust code

- Library crates: no panics in library paths. Return `Result` with crate-local errors (`thiserror`). `unwrap`/`expect` allowed only in tests, benches, and provably-infallible internal spots with a comment.
- `#![deny(missing_docs)]` on all public items of lib crates. Public API = contract for downstream consumers; document it.
- Workspace lints live in the root `Cargo.toml` under `[workspace.lints]`; crates inherit via `[lints] workspace = true`.
- `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` must pass before every commit.
- Zero-allocation policy inside the tick loop. Buffers are allocated once at setup and reused via double buffering / scratch buffers. If a `Vec` grows during ticks, that is a design smell.
- Determinism rules from `docs/ARCHITECTURE.md` (fixed iteration order, snapshot reads, explicit RNG) are hard requirements, not style preferences.
- Prefer `u32`/`i64` over default int inference when the bound matters (counts, indices); document overflow behavior of population math (saturating vs checked) explicitly.

## Testing

- Unit tests per crate, colocated `#[cfg(test)]` modules.
- Property tests for invariants: mass conservation, population bookkeeping, coordinate round-trips.
- Golden-trace determinism test: fixed seed + config => identical aggregated metrics (hash) across runs.
- Criterion benches for hot paths (cell update, neighborhood iteration). Run benches when touching grid/sim kernels.
- Tests must not depend on wall-clock, thread scheduling, or HashMap iteration order.

## Version control (jj)

- One logical change per commit. Commit when it is done and verified, do not batch a whole increment into one mega-commit.
- Plain `jj commit -m "<message>"` with system-inferred author. Never set `JJ_USER`/`JJ_EMAIL`. Conventional commit style: `feat(grid): ...`, `fix(sim): ...`, `docs(thesis): ...`, `chore(deps): ...`.
- Multi-line commit descriptions are written directly (real newlines, never escaped `\n`).
- After a finished change: `jj new` to leave an empty working copy.
- Before describing a change: `jj st` / `jj diff` to know exactly what is being committed.

## Thesis sync (hard requirement)

- Every increment that changes the framework must update the thesis prose in the same commit(s): mainly `thesis/chapters/implementation.typ`, secondarily `foundation.typ` / `methodology.typ` when relevant.
- A commit with framework code and no corresponding thesis update is not allowed (pure chores like toolchain/CI files exempt).
- Thesis files are Typst; run `mise run fmt:typst` after editing thesis chapters.

## Documentation of progress

- `docs/ROADMAP.md` is the single progress tracker. Check off items and append a dated progress-log line at the end of every increment.
- `AGENTS.md` is the entry point for agentic development; keep its project map current when crate structure changes.

## Dependencies

- Keep the dependency tree small. Justify every new dependency in the commit message or ADR.
- Preferred stack: `slotmap` (registry), `rayon` (parallelism), `thiserror` (errors), `serde` + `serde_json`/`toml` (config), `criterion` (benches). GUI deps live only in `runnables/pbma-gui`.

## Definition of Done (per change)

1. Code complete, public items documented.
2. `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` green.
3. Benches run if hot paths touched.
4. Thesis chapters updated if framework behavior/structure changed.
5. `docs/ROADMAP.md` updated if an increment milestone moved.
6. Plain `jj commit` with conventional message, then `jj new`.
