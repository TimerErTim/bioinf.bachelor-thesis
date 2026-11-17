# Midterm presentation outline (15 minutes)

**Working title:** *A Population-Based Multi-Agent Approach for Simulating Large-Scale System Dynamics in Discrete Environments*  
**Audience:** Bachelor-thesis midterm committee  
**Purpose:** Explain the problem and research direction, make the current prototype tangible, be candid about what is not yet implemented, and present a credible path to evaluation.

## Narrative through-line

> We want to study large-scale ecological dynamics without simulating every individual organism. We represent populations on a cellular-automaton grid, give species local decision-making, and model environmental change. The simulation framework and deterministic baseline now exist; Laya-based control, planetary feedback, scaling, and scientific evaluation remain work in progress.

## Timed slide plan

The slide times total **14:50**, leaving a 10-second buffer for transitions. Aim for roughly 1–2 central claims per slide; keep implementation detail in the spoken explanation or backup slides.

### 1. Title and one-sentence problem — 0:30 (0:00–0:30)

**On slide**
- Thesis title, name, supervisor, date.
- A simple visual: one planet-sized grid cell expanding into neighboring cells containing environment and population symbols.

**Say**
- “This thesis asks how to build a reproducible, population-based simulation framework for exploring large-scale ecosystem dynamics.”
- Set the expectation: today is a progress report, not a claim that a validated planetary model is already complete.

**Transition:** Why is this problem worth modeling?

### 2. Motivation: ecosystems are coupled, global, and slow — 1:20 (0:30–1:50)

**On slide**
- Three short ideas: evolution is too slow to observe directly; individual-based simulations become expensive at planetary scale; species both respond to and alter their environment.
- Great Oxidation Event as an intuitive example: oxygen-producing life changes the atmosphere, which changes the selective environment for life.

**Say**
- The scientific motivation is to explore how local population dynamics can produce large-scale changes and patterns in biodiversity.
- The computational motivation is to avoid representing every organism. A population can be the unit of simulation while retaining spatial structure.
- Frame the work as an **exploratory model/framework**, not a validated digital twin or a prediction of Earth’s history.

**Visual:** One feedback loop: environment → species survival/growth → environmental modification → new environment.

**Transition:** That motivation becomes a focused set of research questions and hypotheses.

### 3. Research questions and biodiversity hypothesis — 1:30 (1:50–3:20)

**On slide**
- **RQ1:** Can small, general-purpose typed decision models transfer to ecological species–species and environment–species decisions?
- **RQ2:** How can species-driven environmental change be represented in a controllable, reproducible way?
- **RQ3:** What do the models’ successes and failure patterns reveal about transfer to ecological decisions?
- Working hypothesis: compare hyperoptimal, spatially similar conditions with harsher/heterogeneous conditions using local (α), between-region (β), and global (γ) diversity.

**Say**
- Under hyperoptimal conditions, the hypothesis predicts weaker regional differentiation (α closer to γ; lower β) and potentially greater global γ-diversity. Under harsh conditions, stronger specialization should make local diversity lower relative to global diversity.
- Explain that this is a hypothesis to test, **not a result already established**.
- Note that metrics and scenario definitions must be operationalized before the experiment can support or reject the claim.

**Visual:** Two schematic maps: similar species everywhere vs. distinct regional communities; label α, β, γ without overloading the slide.

**Transition:** What kind of model could generate those patterns?

### 4. Approach: population agents on a cellular-automaton world — 1:40 (3:20–5:00)

**On slide**
- A 2D grid. A cell stores local environment (currently O₂, CO₂, CH₄, H₂O, temperature, light) and a list of species populations.
- One “agent” means **one species population in one cell**, not one individual organism.
- Neighboring-cell updates form the cellular automaton (CA); show the local neighborhood and movement of population between cells.

**Say**
- Each discrete tick advances the environment, changes population sizes, redistributes overcrowded populations to neighbors, then records outputs; evolution/species-registry work handles extinction and variants.
- The current environmental adapter diffuses gases between neighboring cells. The current population adapters provide trait-dependent growth and overcrowding migration.
- Emphasize the feedback direction: environment → population behavior is distinct from species → environment. The latter is intended to use explicit, deterministic rules (for example, a specified metabolic oxygen flux), but a full Great Oxidation Event scenario is not yet implemented.
- Explain the computational trade-off: aggregating organisms into population quantities makes large spatial simulations feasible, but omits individual-level variation and behavior.

**Visual:** Tick pipeline: frozen world snapshot → environmental CA → population update → migration/evolution → metrics → next tick.

**Transition:** The planned decision layer is where Laya may enter.

### 5. Species control: intended Laya pathway vs. implemented baseline — 1:40 (5:00–6:40)

**On slide**

| Intended decision pathway | Current implementation |
|---|---|
| Cell state + species traits → typed ecological questions → Laya probabilities → behavior updates | Trait-based logistic behavior and a closed-form formula surrogate; no Laya inference/fine-tuning integrated yet |

**Say**
- The proposed controller gives a decision model local context (environment, co-occurring species, population amounts, relevant traits) and asks constrained questions rather than requesting free-form text. Possible outputs include probabilities/scores that inform survival, growth, competition/predation, migration pressure, or trait changes.
- Laya is a candidate open-source non-autoregressive decision model to fine-tune for these ecological tasks. The fine-tuning dataset, task definitions, and ecological validation still need to be designed and tested.
- The thesis deliberately does **not** train a custom reinforcement-learning system: a defensible reward signal and the training infrastructure would distract from the system-dynamics focus.
- The current code instead has a logistic trait-based adapter and a deterministic closed-form surrogate. These are baselines/architecture seams, **not evidence that Laya has already controlled species**.
- The intended thesis path describes offline model training with a cheap explicit runtime formula. If neural inference is retained at runtime, measure batched inference latency and compare it against formula evaluation before choosing.
- Species → environment effects remain rule-based to keep that feedback legible and controllable.

**Visual:** Two-way interaction diagram with “Laya / behavior adapter (planned)” on environment→species and “explicit rules (planned)” on species→environment. Use dashed lines for unimplemented components.

**Transition:** The key progress so far is the framework that can host interchangeable adapters.

### 6. What exists now: a testable simulation kernel — 1:40 (6:40–8:20)

**On slide**
- **Implemented:** Rust workspace with a core/kernel and replaceable simulation adapters; dense chunk-aware grid storage; species registry; CLI runner and CSV recording.
- **Implemented behavior:** mass-conserving gas diffusion CA; logistic population growth using temperature/light and species traits; overcrowding-driven migration; deterministic evolution/extinction infrastructure; a closed-form behavior surrogate.
- **Not implemented:** rendered GUI (currently a placeholder), Laya integration, planetary-scale physical forcing, experiment scenarios, and parallel execution.

**Say**
- The kernel owns the world and tick orchestration; adapters define the particular rules. This separation allows behavior or environment models to change without rewriting the simulation core.
- Determinism is a design requirement: phases use snapshots and separate outputs, ordering is fixed, and random state is explicit/seedable.
- The implementation is a useful vertical slice, but this is **engineering progress**, not proof that the ecological assumptions are accurate.

**Visual:** Small architecture map: `pbma-core` ← `pbma-sim` ← CLI/GUI; mark GUI as placeholder. Alternatively show a short CLI/CSV screenshot if prepared.

**Transition:** What can we demonstrate today, and what does it not establish?

### 7. Evidence so far: invariants and a sequential baseline — 1:30 (8:20–9:50)

**On slide**
- Same configuration + seed → identical recorded trace (determinism test).
- Pure diffusion conserves total gas mass; migration bookkeeping preserves populations apart from behavior changes (tests).
- A one-million-cell sequential tick benchmark is recorded at approximately **0.21–0.23 s/tick** in project notes/text; show hardware, exact commit, and rerun timing if presenting a precise number.

**Say**
- These tests establish selected software invariants and reproducibility, not ecological realism.
- The benchmark is an initial sequential reference point. It does not demonstrate real-time, planetary-timescale, or hyper-real-time performance; the actual target rate needs a workload definition.
- There are not yet completed hypothesis experiments or a calibrated comparison against an objective ecosystem reference dataset.

**Visual:** A two-column “verified” vs. “not yet validated” checklist. Avoid presenting placeholder results plots.

**Transition:** The remaining work is partly scientific validation and partly performance/coverage.

### 8. Main challenges and limitations — 2:00 (9:50–11:50)

**On slide**
1. **No objective end-to-end reference yet:** no agreed dataset/benchmark and metric set for comparing complete simulated community dynamics to observed ecosystems.
2. **Laya adaptation:** define ecological tasks, curate training/evaluation cases, check calibration and failure modes, and avoid evaluating only on scenarios used for fine-tuning.
3. **Inference latency:** per-cell/per-species neural calls could dominate tick time. Non-autoregressive does not mean free; batching, inference frequency, model size, and offline formula/distillation choices need measurement.
4. **Scale:** current phases are sequential. Parallel execution, deterministic random streams/reductions, and scaling benchmarks are future work.
5. **Planet model is intentionally incomplete:** current global parameter structure is small (gravity, day length, axial tilt, diffusion coefficient, light intensity; some are not yet coupled to physical behavior). Planetary surface area/radius, mass, orbital distance/eccentricity, stellar forcing and richer climate cycles are not yet represented as a coherent model.
6. **Environmental feedback and scenario validation:** species-driven chemistry, including a GOE-style scenario, and biodiversity recorders/experiments remain to be completed.

**Say**
- Be explicit that “no objective data” means no selected, end-to-end validation target for the full system at this stage—not that no relevant observations exist. Historical geochemical records may support narrower validation of an oxygen/GOE submodel once implemented.
- Don’t solve every planetary variable at once. Prioritize only parameters needed for the thesis hypotheses, state the assumptions, and bound claims to that scope.
- For Laya, separate model-transfer evaluation from overall ecosystem plausibility: they are different questions and need different tests.

**Visual:** Risk matrix with “scientific validity”, “decision model”, and “runtime/scaling”; indicate mitigation rather than implying completion.

**Transition:** These risks determine the most useful path from prototype to defensible thesis results.

### 9. Current state → target state: evaluation and next steps — 2:00 (11:50–13:50)

**On slide**

| Now | Target by thesis completion |
|---|---|
| Core tick loop, diffusion, simple population behavior, migration/evolution support; reproducibility/invariant tests | Clearly scoped ecological scenarios with explicit assumptions and chosen validation targets |
| Logistic/formula surrogate; Laya not integrated | Defined/fine-tuned typed Laya tasks, evaluated on held-out ecological cases; documented runtime approach |
| Sequential 1M-cell reference; limited global parameters | Parallel, deterministic phases with measured scaling; only the planetary parameters required for the chosen experiments |
| No completed diversity experiments/GOE scenario | Replicated hyperoptimal-vs-harsh runs, α/β/γ and extinction/environment metrics, limitations reported |

**Suggested work sequence**
1. Lock the minimum thesis scope and define measurable scenarios/metrics; add α/β/γ diversity recording.
2. Complete one vertical ecological scenario, including explicit species→environment rules; keep model assumptions traceable.
3. Define Laya’s typed tasks and held-out evaluations; fine-tune only after evaluation cases are fixed. Benchmark batched inference versus the formula baseline and decide whether to run or distill the model.
4. Add deterministic chunk-parallelism and benchmark scaling against the sequential baseline.
5. Run repeated controlled experiments, report uncertainty/variation, compare only to available relevant references, and present unsupported claims as limitations.

**Say**
- The success criterion is not “simulate all of Earth.” It is a reproducible, computationally useful framework and a transparent, bounded test of the selected hypotheses and decision-model transfer question.
- Prioritize evaluation before adding realism: an unvalidated list of more planetary parameters will not by itself make the model more scientifically reliable.

**Visual:** A three-step roadmap: **complete vertical scenario → validate model components → run controlled experiments**. Add parallelization as a cross-cutting engineering task.

**Transition:** Close by restating what is already credible and what the next milestone will prove.

### 10. Take-home message and discussion — 1:00 (13:50–14:50)

**On slide**
- **Built:** a deterministic, modular population-based simulation baseline.
- **Still to prove:** that Laya-style control and the ecological rules yield useful, validated system-level behavior at the required scale.
- **Next milestone:** one instrumented, reproducible scenario with biodiversity metrics and a measured runtime.

**Say**
- “The main result at midterm is a working, testable framework—not yet a validated planetary simulation. The remaining thesis work is to make one ecological scenario measurable, evaluate the decision layer, and quantify scaling.”
- Invite questions. Use the 10-second buffer if earlier slides run long.

## Suggested backup slides (not part of the 15 minutes)

1. **Tick pipeline and determinism:** snapshot/update/swap, fixed neighbor order, explicit RNG; which properties current tests cover.
2. **Current vs. planned parameter coverage:** implemented fields versus absent planetary inputs; indicate which fields are currently used by adapters.
3. **Laya evaluation protocol:** typed question examples, train/validation/test split by scenario, probability calibration, baseline comparison, and runtime timing plan.
4. **Performance protocol:** machine/compiler/build mode, grid sizes, warm-up/repetitions, per-phase timings, single-thread vs. parallel scaling; state the intended meaning of “hyper-real-time.”
5. **Diversity definitions:** α, β = γ/α as used in the thesis, and γ, with an illustrative two-region example.

## Presenter preparation checklist

- Rerun the million-cell benchmark and record hardware, build mode, exact timing, and commit. Project notes currently show approximately 207 ms/tick while thesis prose says about 225 ms/tick; present a rounded/reverified number or omit the exact figure.
- If showing a demo, use the CLI/CSV output; the GUI is currently a placeholder. Prepare a static screenshot in case a live run is slow.
- Label Laya, species→environment rules, GUI rendering, parallelism, GOE scenario, and biodiversity experiments clearly as planned/not yet integrated where appropriate.
- Avoid implying that the tests validate biological accuracy, that Laya is already running the populations, or that the hypothesis has been confirmed.
- Define “hyper-real-time” operationally before making performance claims (e.g., simulated-time advancement relative to wall-clock under a specified grid and tick workload).
