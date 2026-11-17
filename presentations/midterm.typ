#import "libs/sanor/lib.typ" as sr
#import "libs/gh-template.typ" as gh

#let handout = eval(sys.inputs.at("x-handout", default: "false"))
#let title = sys.inputs.at(
  "title",
  default: "A population-based multi-agent framework for simulating large-scale system dynamics with decision models in discrete environments",
)
#let (slide, scene) = sr.set-option(handout: handout)

#let presentation-date = datetime(year: 2026, month: 12, day: 03)
#set document(
  title: title + ": Midterm Presentation",
  author: "Tim Peko",
  date: presentation-date,
  keywords: (
    "midterm",
    "presentation",
    "simulation framework",
    "decision models",
    "bioinformatics",
    "fhooe",
    "hagenberg",
    "bachelor-thesis",
  ),
)

#show: gh.register.with(
  accent: "green",
  density: "compact",
  title: [
    #set text(size: 16pt)
    #title],
)

// ------------------------------------------------------------
// Local UI helpers aligned with gh-template light palette
// ------------------------------------------------------------
#let pal = gh.gh-light
#let acc = gh.gh-accents

#let pill(label, color: pal.fg-muted, fill: pal.bg-neutral-muted) = box(
  fill: fill,
  stroke: 0.8pt + pal.border-muted,
  inset: (x: 7pt, y: 2.5pt),
  radius: 999pt,
  text(
    font: ("JetBrains Mono", "DejaVu Sans Mono"),
    size: 12.5pt,
    weight: 600,
    fill: color,
    label,
  ),
)

#let card(
  heading,
  body,
  kicker: none,
  accent: acc.green.light,
  dashed: false,
  fill: pal.bg-canvas-subtle,
  inset: 14pt,
) = block(
  width: 100%,
  fill: fill,
  stroke: (
    paint: if dashed { pal.border-default } else { pal.border-muted },
    thickness: 1pt,
    dash: if dashed { "dashed" } else { "solid" },
  ),
  radius: 8pt,
  inset: inset,
)[
  #if kicker != none {
    text(
      font: ("JetBrains Mono", "DejaVu Sans Mono"),
      size: 13pt,
      weight: 700,
      fill: accent,
      tracking: 0.6pt,
      upper(kicker),
    )
    v(4pt)
  }
  #if heading != none {
    text(size: 20pt, weight: 700, fill: pal.fg-default, heading)
    v(6pt)
  }
  #set text(size: 17.5pt, fill: pal.fg-muted)
  #set par(leading: 0.48em)
  #body
]

#let callout(kind, color, body) = block(
  width: 100%,
  fill: pal.bg-canvas-subtle,
  stroke: (left: 4pt + color),
  radius: (right: 6pt),
  inset: (left: 16pt, right: 16pt, top: 10pt, bottom: 10pt),
)[
  #text(
    font: ("JetBrains Mono", "DejaVu Sans Mono"),
    size: 13.5pt,
    weight: 700,
    fill: color,
    tracking: 1pt,
    "▲ " + upper(kind),
  )
  #v(4pt)
  #set text(size: 17.5pt, fill: pal.fg-default)
  #set par(leading: 0.45em)
  #body
]

#let arch-node(
  title,
  sub,
  border: pal.border-default,
  fill: pal.bg-canvas,
  title-color: pal.fg-default,
  dashed: false,
) = block(
  width: 100%,
  fill: fill,
  stroke: (
    paint: border,
    thickness: 1.2pt,
    dash: if dashed { "dashed" } else { "solid" },
  ),
  radius: 6pt,
  inset: (x: 12pt, y: 9pt),
)[
  #text(
    font: ("JetBrains Mono", "DejaVu Sans Mono"),
    size: 15.5pt,
    weight: 700,
    fill: title-color,
    title,
  )
  #if sub != none {
    v(3pt)
    text(size: 14.5pt, fill: pal.fg-muted, sub)
  }
]

// ------------------------------------------------------------
// 01 — Cover
// ------------------------------------------------------------
#scene(s => context {
  show: gh.cover-slide.with(
    badges: (
      [Campus Hagenberg],
      [#set text(fill: acc.purple.light); Bachelor of Science],
      [#set text(fill: acc.green.light); Medical & Bioinformatics],
      [#set text(fill: acc.blue.light); FH-Prof. PD DI Dr. Stephan Winkler],
    ),
    kicker: image("../assets/fhooe.png", height: 2em),
    footer-left: [
      #set text(fill: gh.gh-accents.green.light)
      \# #document.author.join(", ") · #document.date.display()
    ],
  )
  set text(size: 38pt)
  title
  v(14pt)
  block(width: 100%)[
    #set text(size: 20pt, weight: 400, fill: pal.fg-muted, tracking: 0pt)
    #set par(leading: 0.45em)
    Coupling deterministic, chunk-parallel simulation kernels with local typed decision models, evaluated on planetary biodiversity dynamics.
  ]
})

// ------------------------------------------------------------
// 02 — Motivation: Scaling Coupled Dynamic Simulations
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Motivation: scaling coupled dynamic simulations",
    )

    grid(
      columns: (1.12fr, 0.88fr),
      gutter: 24pt,
      stack(
        spacing: 12pt,
        card(
          "1. Scale vs. spatial fidelity",
          [Individual-based models collapse at $10^6+$ entities; aggregate ODEs lose local spatial structure and heterogeneity.],
          accent: acc.blue.light,
          inset: 12pt,
        ),
        card(
          "2. Decision complexity vs. training cost",
          [Hand-crafted rules turn brittle; custom multi-agent RL demands artificial reward shaping and heavy training loops.],
          accent: acc.purple.light,
          inset: 12pt,
        ),
        card(
          "3. Coupled feedback vs. reproducibility",
          [Agents alter their environment, shifting future selection — yet scientific experiments require bit-exact determinism.],
          accent: acc.orange.light,
          inset: 12pt,
        ),
      ),
      tag("approach")[
        #stack(
          spacing: 12pt,
          card(
            "Proposed architecture",
            [
              - *Unit = population per cell* on a discrete CA grid (not individuals).
              - *Typed decision models* (or distilled formulas) behind modular kernel ports.
              - *Explicit environmental rules* keep feedback legible and reproducible.
            ],
            kicker: "PBMA + Decision Models",
            accent: acc.green.light,
          ),
          callout("Evaluation Testbed (not the goal itself)", acc.blue.light)[
            *Planetary biodiversity & GOE feedback* provide a demanding multi-scale testbed for kernel speed, hybrid loops, and model transfer.
          ],
        )
      ],
    )
  },
  controls: (
    (),
    sr.apply("approach"),
  ),
)

// ------------------------------------------------------------
// 03 — Research Questions & Evaluation Scenario
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Research questions & concrete evaluation testbed",
    )

    stack(
      spacing: 12pt,
      card(
        none,
        [
          #grid(
            columns: (64pt, 1fr),
            gutter: 10pt,
            text(
              font: ("JetBrains Mono", "DejaVu Sans Mono"),
              size: 19pt,
              weight: 700,
              fill: acc.green.light,
              "RQ1",
            ),
            [
              #text(
                weight: 700,
                fill: pal.fg-default,
              )[Framework & Decision Coupling: ]
              How can a large-scale discrete simulation kernel couple with small, general-purpose *typed decision models* at scale without custom RL training?
            ],
          )
        ],
        inset: 12pt,
      ),
      card(
        none,
        [
          #grid(
            columns: (64pt, 1fr),
            gutter: 10pt,
            text(
              font: ("JetBrains Mono", "DejaVu Sans Mono"),
              size: 19pt,
              weight: 700,
              fill: acc.blue.light,
              "RQ2",
            ),
            [
              #text(
                weight: 700,
                fill: pal.fg-default,
              )[Hybrid Feedback & Determinism: ]
              How can decision-model agent behavior be combined with *explicit, rule-based environmental feedback* so coupled dynamics stay controllable and bit-reproducible?
            ],
          )
        ],
        inset: 12pt,
      ),
      tag("rq3")[
        #card(
          none,
          [
            #grid(
              columns: (64pt, 1fr),
              gutter: 10pt,
              text(
                font: ("JetBrains Mono", "DejaVu Sans Mono"),
                size: 19pt,
                weight: 700,
                fill: acc.purple.light,
                "RQ3",
              ),
              [
                #text(
                  weight: 700,
                  fill: pal.fg-default,
                )[Technology Transfer & Potential: ]
                How far do general-purpose decision models (JevK5 / fine-tuned Laya) transfer to ecological decisions, and what do failures and latency reveal?
              ],
            )
          ],
          inset: 12pt,
        )
      ],
      tag("testbed")[
        #callout(
          "Concrete Evaluation Scenario: Hyperoptimal vs. Harsh Planetary Biodiversity",
          acc.orange.light,
        )[
          Testbed compares homogeneous *hyperoptimal* vs. *harsh* planets using $alpha$, $beta = gamma slash alpha$, $gamma$ diversity + GOE-style atmospheric feedback (*working hypothesis to test, not an established result*).
        ]
      ],
    )
  },
  controls: (
    (),
    sr.apply("rq3"),
    sr.apply("testbed"),
  ),
)

// ------------------------------------------------------------
// 04 — Framework Architecture: Hexagonal Kernel & Ports
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Framework design: hexagonal kernel & dependency inversion",
    )

    grid(
      columns: (0.95fr, 1.05fr),
      gutter: 24pt,
      stack(
        spacing: 10pt,
        grid(
          columns: (1fr, 1fr),
          gutter: 10pt,
          arch-node(
            "pbma-cli",
            "TOML config → tick loop → CSV",
            border: acc.green.light,
            fill: pal.bg-canvas-subtle,
            title-color: acc.green.light,
          ),
          arch-node(
            "pbma-gui",
            "Placeholder (snapshot consumer)",
            border: pal.border-default,
            fill: pal.bg-canvas-subtle,
            title-color: pal.fg-muted,
            dashed: true,
          ),
        ),
        align(center, text(
          font: ("JetBrains Mono", "DejaVu Sans Mono"),
          size: 14pt,
          fill: pal.fg-subtle,
          "↓ wires concrete adapters (runnables)",
        )),
        arch-node(
          "pbma-sim  (Default Phase Adapters)",
          "DiffusionStep · LogisticBehavior · FormulaBehavior · OvercrowdingMigration · VariationEngine · SplitMixRng",
          border: acc.blue.light,
          fill: pal.bg-canvas-subtle,
          title-color: acc.blue.light,
        ),
        align(center, text(
          font: ("JetBrains Mono", "DejaVu Sans Mono"),
          size: 14pt,
          fill: pal.fg-subtle,
          "↓ implements ports (strict inward deps)",
        )),
        arch-node(
          "pbma-core  (Kernel · 0 adapter deps)",
          "World · TickDriver · DoubleBuffer<Cell> · SpeciesRegistry · GasRegistry · TraitSchema · Physics · WorldBuilder",
          border: pal.fg-default,
          fill: pal.bg-neutral-muted,
          title-color: pal.fg-default,
        ),
      ),
      tag("contracts")[
        #stack(
          spacing: 10pt,
          card(
            "6 Kernel Ports (pbma-core::ports)",
            [
              `EnvironmentStep` · `PopulationBehavior` · `MigrationPolicy` · `EvolutionEngine` · `Rng` · `Recorder` — swap rules or ML models without touching the kernel.
            ],
            accent: acc.green.light,
            inset: 11pt,
          ),
          card(
            "Extensible Domain Vocabularies",
            [
              `GasRegistry` (`O₂, CO₂, CH₄, H₂O, O₃, N₂` + custom) and `TraitSchema` (`INLINE = 8`) let experiments extend state at setup time.
            ],
            accent: acc.blue.light,
            inset: 11pt,
          ),
          card(
            "Dynamic Species vs. Dense Grid",
            [
              Generational `SlotMap` detects extinct/stale `SpeciesId`s; cells stay dense chunked vectors (`Clamped`, `Planetary`, `Toroidal`).
            ],
            accent: acc.purple.light,
            inset: 11pt,
          ),
        )
      ],
    )
  },
  controls: (
    (),
    sr.apply("contracts"),
  ),
)

// ------------------------------------------------------------
// 05 — Execution Model: Deterministic Chunk-Parallel Tick Pipeline
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Execution model: deterministic snapshot-and-swap ticks",
    )

    stack(
      spacing: 16pt,
      block(
        width: 100%,
        fill: pal.bg-canvas-subtle,
        stroke: 1pt + pal.border-muted,
        radius: 8pt,
        inset: 12pt,
      )[
        #text(
          font: ("JetBrains Mono", "DejaVu Sans Mono"),
          size: 13pt,
          weight: 700,
          fill: acc.green.light,
          "ONE TICK PIPELINE (DoubleBuffer<Cell>, 64×64 chunks)",
        )
        #v(8pt)
        #grid(
          columns: (1fr, 18pt, 1fr, 18pt, 1fr, 18pt, 1fr, 18pt, 1fr),
          align: horizon + center,
          arch-node(
            "1. Env CA",
            "Moore 8-ring\n(1/√2 diag)",
            border: acc.blue.light,
            title-color: acc.blue.light,
          ),
          text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 18pt,
            fill: pal.fg-subtle,
            "→",
          ),
          arch-node(
            "2. Behavior",
            "Frozen front +\ntrait registry",
            border: acc.green.light,
            title-color: acc.green.light,
          ),
          text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 18pt,
            fill: pal.fg-subtle,
            "→",
          ),
          arch-node(
            "3. Migration",
            "Chunk flux →\ncanonical merge",
            border: acc.purple.light,
            title-color: acc.purple.light,
          ),
          text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 18pt,
            fill: pal.fg-subtle,
            "→",
          ),
          arch-node(
            "4. Evolution",
            "Prune extinct +\nSplitMix variants",
            border: acc.orange.light,
            title-color: acc.orange.light,
          ),
          text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 18pt,
            fill: pal.fg-subtle,
            "→",
          ),
          arch-node(
            "5. Record",
            "O(1) swap →\nconsumer metrics",
            border: pal.fg-default,
            title-color: pal.fg-default,
          ),
        )
      ],
      tag("details")[
        #grid(
          columns: (1fr, 1fr),
          gutter: 20pt,
          card(
            "4 Hard Determinism Rules",
            [
              + *Snapshot isolation:* read `front`, write disjoint `back` slices.
              + *Canonical order:* fixed Moore ring & chunk merge order.
              + *Sorted populations:* `Vec<(SpeciesId, Pop)>` — no `HashMap` iteration.
              + *Explicit RNG:* seedable `SplitMixRng` port, zero global state.
            ],
            kicker: "Reproducibility = Safe Parallelism",
            accent: acc.green.light,
          ),
          card(
            "Chunk-First Parallelism & AoS Layout",
            [
              - `64×64` contiguous chunks split across `rayon` workers with *zero `unsafe` code*.
              - Pre-allocated flux buffers eliminate per-tick grid cloning.
              - *AoS cell layout* matches phases that read all cell fields together (avoids multi-array port plumbing).
            ],
            kicker: "Memory & Concurrency",
            accent: acc.blue.light,
          ),
        )
      ],
    )
  },
  controls: (
    (),
    sr.apply("details"),
  ),
)

// ------------------------------------------------------------
// 06 — Decision-Model Integration: Typed Queries & Distillation
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Decision models: planned typed pathway vs. current baseline",
    )

    stack(
      spacing: 14pt,
      block(
        width: 100%,
        stroke: 1pt + pal.border-muted,
        radius: 8pt,
        clip: true,
        table(
          columns: (0.95fr, 1.55fr, 1.45fr),
          inset: (x: 12pt, y: 9pt),
          stroke: (x: none, y: 0.8pt + pal.border-muted),
          fill: (x, y) => if y == 0 { pal.bg-neutral-muted } else if calc.odd(
            y,
          ) { pal.bg-canvas } else { pal.bg-canvas-subtle },
          table.header(
            text(size: 16pt, weight: 700, fill: pal.fg-default)[Aspect],
            text(
              size: 16pt,
              weight: 700,
              fill: acc.blue.light,
            )[Intended Decision Pathway (Planned)],
            text(
              size: 16pt,
              weight: 700,
              fill: acc.green.light,
            )[Implemented Baseline (Today)],
          ),
          text(size: 15.5pt, weight: 600)[Query & Output],
          text(
            size: 15.5pt,
          )[`CellContext` + traits → typed questions (`yes/no`, `choice`, `score`) → calibrated probs (Laya / JevK5)],
          text(
            size: 15.5pt,
          )[`LogisticBehavior` + `FormulaBehavior` (closed-form surrogate with equilibrium test)],

          text(size: 15.5pt, weight: 600)[Training Strategy],
          text(
            size: 15.5pt,
          )[Fine-tune small non-autoregressive model; *no custom RL policy or reward shaping*],
          text(
            size: 15.5pt,
          )[Architecture seam ready behind `PopulationBehavior`; no ML inference integrated yet],

          text(size: 15.5pt, weight: 600)[Runtime Cost],
          text(
            size: 15.5pt,
          )[Benchmark batched inference vs. *offline distillation to closed-form runtime formula*],
          text(
            size: 15.5pt,
          )[Pure Rust expression evaluation per cell/species (sub-microsecond)],

          text(size: 15.5pt, weight: 600)[Env Feedback],
          text(
            size: 15.5pt,
          )[*Explicit deterministic rules* (`Species → Env`, e.g. metabolic O₂ flux)],
          text(
            size: 15.5pt,
          )[Mass-conserving diffusion CA + orbital/O₃/greenhouse helpers; bio-flux TBD],
        ),
      ),
      tag("loop")[
        #callout(
          "Architectural Separation: Two Distinct Feedback Directions",
          acc.purple.light,
        )[
          `Env → Species` uses *probabilistic / typed decision adapters* (`PopulationBehavior` port); `Species → Env` stays *explicitly rule-based* so coupled feedback remains inspectable and verifiable.
        ]
      ],
    )
  },
  controls: (
    (),
    sr.apply("loop"),
  ),
)

// ------------------------------------------------------------
// 07 — Concrete Evaluation Scenario: Hyperoptimal Planets & Biodiversity
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Evaluation scenario: planetary biodiversity & GOE feedback",
    )

    grid(
      columns: (1.05fr, 0.95fr),
      gutter: 22pt,
      stack(
        spacing: 12pt,
        card(
          "Why use planetary ecology as the testbed?",
          [
            - Exercises *spatial coupling* (6-gas diffusion + orbital insolation + UV shielding).
            - Exercises *dynamic lifetimes* (speciation, extinction, trait drift).
            - Exercises *hybrid feedback* (organisms reshape atmospheric chemistry).
          ],
          kicker: "Stress-Testing the Framework",
          accent: acc.blue.light,
        ),
        card(
          "Quantitative Metrics (Whittaker)",
          [
            - *$alpha$-diversity:* local richness within a region/cell cluster.
            - *$gamma$-diversity:* total global species richness.
            - *$beta = gamma slash alpha$:* regional differentiation ($1$ = identical, $N$ = disjoint).
          ],
          kicker: "Operationalized Targets",
          accent: acc.green.light,
        ),
      ),
      tag("regimes")[
        #stack(
          spacing: 10pt,
          card(
            "Regime A: Hyperoptimal (Uniform, Benign)",
            [Weak barriers → generalists dominate everywhere → *$alpha approx gamma$* ($beta -> 1$), potentially higher $gamma$.],
            kicker: "Hypothesis to Test",
            accent: acc.green.light,
            inset: 11pt,
          ),
          card(
            "Regime B: Harsh (Heterogeneous Gradients)",
            [Strong thermal/UV/chemical niches → regional specialists → *$alpha << gamma$* ($beta >> 1$).],
            kicker: "Hypothesis to Test",
            accent: acc.orange.light,
            inset: 11pt,
          ),
          card(
            "GOE Feedback Loop (Planned)",
            [Photosynthetic O₂ flux → O₃ UV shield + anaerobic selective crash.],
            kicker: "Species → Env Rules",
            accent: acc.purple.light,
            dashed: true,
            inset: 11pt,
          ),
        )
      ],
    )
  },
  controls: (
    (),
    sr.apply("regimes"),
  ),
)

// ------------------------------------------------------------
// 08 — Evidence So Far: Verified Invariants & Benchmark
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Evidence so far: verified invariants & tick baseline",
    )

    stack(
      spacing: 16pt,
      grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 16pt,
        block(
          width: 100%,
          stroke: 1pt + pal.border-muted,
          radius: 8pt,
          fill: pal.bg-canvas-subtle,
          inset: (x: 18pt, y: 14pt),
        )[
          #text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 38pt,
            weight: 700,
            fill: acc.green.light,
            "81 / 81",
          )
          #v(4pt)
          #text(
            size: 16pt,
            weight: 600,
            fill: pal.fg-default,
            "Workspace tests passing",
          )
          #v(2pt)
          #text(size: 14pt, fill: pal.fg-muted, `cargo nextest run --workspace`)
        ],
        block(
          width: 100%,
          stroke: 1pt + pal.border-muted,
          radius: 8pt,
          fill: pal.bg-canvas-subtle,
          inset: (x: 18pt, y: 14pt),
        )[
          #text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 38pt,
            weight: 700,
            fill: acc.blue.light,
            "~113 ms",
          )
          #v(4pt)
          #text(
            size: 16pt,
            weight: 600,
            fill: pal.fg-default,
            "Per-tick at 1M cells (1000×1000)",
          )
          #v(2pt)
          #text(
            size: 14pt,
            fill: pal.fg-muted,
            "Down from ~225 ms sequential baseline",
          )
        ],
        block(
          width: 100%,
          stroke: 1pt + pal.border-muted,
          radius: 8pt,
          fill: pal.bg-canvas-subtle,
          inset: (x: 18pt, y: 14pt),
        )[
          #text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 38pt,
            weight: 700,
            fill: acc.purple.light,
            "0 diff",
          )
          #v(4pt)
          #text(
            size: 16pt,
            weight: 600,
            fill: pal.fg-default,
            "Parallel vs. sequential trace",
          )
          #v(2pt)
          #text(
            size: 14pt,
            fill: pal.fg-muted,
            "Bit-identical multi-chunk rayon run",
          )
        ],
      ),
      tag("split")[
        #grid(
          columns: (1fr, 1fr),
          gutter: 20pt,
          card(
            "Verified Software Invariants",
            [
              - Golden-trace determinism (identical seed → identical hash).
              - Gas mass conservation across `Clamped`, `Planetary`, `Toroidal`.
              - Migration population bookkeeping & `SlotMap` staleness checks.
              - Closed-form equilibrium bisection & custom gas/trait end-to-end test.
            ],
            kicker: "Established Today",
            accent: acc.green.light,
          ),
          card(
            "Explicitly Not Yet Validated",
            [
              - Ecological / biological realism or empirical dataset calibration.
              - Laya transfer accuracy, calibration, and inference latency.
              - 1-vs-N thread scaling curves & chunk halo/cache profiling.
              - GOE bio-chemistry, $alpha slash beta slash gamma$ recorder, and `pbma-gui` rendering.
            ],
            kicker: "Remaining Validation",
            accent: acc.orange.light,
            dashed: true,
          ),
        )
      ],
    )
  },
  controls: (
    (),
    sr.apply("split"),
  ),
)

// ------------------------------------------------------------
// 09 — Open Challenges & Engineering Trade-Offs
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Main technical challenges & mitigation strategy",
    )

    grid(
      columns: (1fr, 1fr, 1fr),
      gutter: 16pt,
      card(
        "1. Decision-Model Latency & Transfer",
        [
          *Risk:* Non-autoregressive ≠ free; per-cell neural calls at $10^6$ cells can dominate runtime, and fine-tuning risks scenario overfitting.
          #v(8pt)
          *Mitigation:* Fix held-out ecological eval cases *before* fine-tuning; benchmark batched inference vs. distilled runtime formulas.
        ],
        kicker: "RQ1 · RQ3",
        accent: acc.purple.light,
      ),
      tag("r2")[
        #card(
          "2. Hybrid Validity & Scope Control",
          [
            *Risk:* No single end-to-end empirical benchmark for full community dynamics; risk of over-building planetary physics.
            #v(8pt)
            *Mitigation:* Separate *decision-transfer eval* from *macroscopic trend tests*; prioritize only parameters needed for chosen scenarios.
          ],
          kicker: "RQ2 · RQ3",
          accent: acc.orange.light,
        )
      ],
      tag("r3")[
        #card(
          "3. Parallel Scaling & Memory Locality",
          [
            *Risk:* One 1M-cell timing point does not prove thread scaling or hyper-real-time bounds under neural workloads.
            #v(8pt)
            *Mitigation:* Define target tick budget per scenario; measure 1-vs-N thread speedup, chunk halo locality, and cache misses.
          ],
          kicker: "Performance",
          accent: acc.blue.light,
        )
      ],
    )
  },
  controls: (
    (),
    sr.apply("r2"),
    sr.apply("r3"),
  ),
)

// ------------------------------------------------------------
// 10 — Roadmap: Current State -> Thesis Completion
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Roadmap: current state → target by thesis completion",
    )

    stack(
      spacing: 14pt,
      block(
        width: 100%,
        stroke: 1pt + pal.border-muted,
        radius: 8pt,
        clip: true,
        table(
          columns: (0.85fr, 1.45fr, 1.55fr),
          inset: (x: 12pt, y: 8.5pt),
          stroke: (x: none, y: 0.8pt + pal.border-muted),
          fill: (x, y) => if y == 0 { pal.bg-neutral-muted } else if calc.odd(
            y,
          ) { pal.bg-canvas } else { pal.bg-canvas-subtle },
          table.header(
            text(size: 15.5pt, weight: 700)[Workstream],
            text(
              size: 15.5pt,
              weight: 700,
              fill: acc.green.light,
            )[Implemented Now],
            text(
              size: 15.5pt,
              weight: 700,
              fill: acc.blue.light,
            )[Target by Thesis Completion],
          ),
          text(size: 15pt, weight: 600)[Kernel & Scale],
          text(
            size: 15pt,
          )[Hexagonal kernel, chunk-parallel `DoubleBuffer`, registries, physics, 81 tests],
          text(
            size: 15pt,
          )[Thread scaling study (1 vs. N), halo/memory profiling, defined tick-rate target],

          text(size: 15pt, weight: 600)[Decision Layer],
          text(
            size: 15pt,
          )[`PopulationBehavior` port, `LogisticBehavior`, `FormulaBehavior` surrogate],
          text(
            size: 15pt,
          )[Typed Laya/Jev task suite, held-out eval, inference vs. distilled formula choice],

          text(size: 15pt, weight: 600)[Env Feedback],
          text(
            size: 15pt,
          )[Mass-conserving diffusion CA + orbital/O₃/greenhouse helpers],
          text(
            size: 15pt,
          )[Explicit deterministic `Species → Env` rules (GOE O₂ metabolic flux)],

          text(size: 15pt, weight: 600)[Testbed Runs],
          text(
            size: 15pt,
          )[`WorldBuilder`, `pbma-cli`, CSV `Recorder` (no diversity runs yet)],
          text(
            size: 15pt,
          )[$alpha slash beta slash gamma$ recorder + replicated hyperoptimal vs. harsh runs & limits],
        ),
      ),
      tag("seq")[
        #block(
          width: 100%,
          fill: pal.bg-canvas-subtle,
          stroke: 1pt + pal.border-muted,
          radius: 8pt,
          inset: 11pt,
        )[
          #text(
            font: ("JetBrains Mono", "DejaVu Sans Mono"),
            size: 13pt,
            weight: 700,
            fill: acc.green.light,
            "EXECUTION SEQUENCE",
          )
          #v(6pt)
          #grid(
            columns: (1fr, 16pt, 1fr, 16pt, 1fr, 16pt, 1fr),
            align: horizon + center,
            arch-node(
              "1. Scenario & Metrics",
              "Lock scope + α/β/γ recorder",
              border: acc.green.light,
              title-color: acc.green.light,
            ),
            text(
              font: ("JetBrains Mono", "DejaVu Sans Mono"),
              size: 16pt,
              fill: pal.fg-subtle,
              "→",
            ),
            arch-node(
              "2. GOE Feedback",
              "Explicit Species → Env rules",
              border: acc.blue.light,
              title-color: acc.blue.light,
            ),
            text(
              font: ("JetBrains Mono", "DejaVu Sans Mono"),
              size: 16pt,
              fill: pal.fg-subtle,
              "→",
            ),
            arch-node(
              "3. Laya Eval & Cost",
              "Held-out tasks + distill check",
              border: acc.purple.light,
              title-color: acc.purple.light,
            ),
            text(
              font: ("JetBrains Mono", "DejaVu Sans Mono"),
              size: 16pt,
              fill: pal.fg-subtle,
              "→",
            ),
            arch-node(
              "4. Scale & Replicates",
              "Parallel bench + experiments",
              border: acc.orange.light,
              title-color: acc.orange.light,
            ),
          )
        ]
      ],
    )
  },
  controls: (
    (),
    sr.apply("seq"),
  ),
)

// ------------------------------------------------------------
// 11 — Summary & Discussion
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(title: "Take-home message & discussion")

    stack(
      spacing: 16pt,
      grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 16pt,
        card(
          "Built at Midterm",
          [
            A modular, bit-reproducible, chunk-parallel Rust simulation framework (`pbma-core` / `pbma-sim`) with extensible registries, physics helpers, 81 passing tests, and a \~113 ms 1M-cell tick baseline.
          ],
          kicker: "Verified Foundation",
          accent: acc.green.light,
        ),
        card(
          "Still to Prove",
          [
            That general-purpose typed decision models (Laya / Jev) transfer usefully to domain decisions—and how runtime inference compares to distilled formulas in latency and fidelity.
          ],
          kicker: "Technology Question",
          accent: acc.purple.light,
        ),
        card(
          "Next Milestone",
          [
            One instrumented, reproducible planetary scenario (GOE feedback + $alpha slash beta slash gamma$ metrics) with held-out decision-model evaluation and multi-thread scaling measurements.
          ],
          kicker: "Concrete Testbed",
          accent: acc.blue.light,
        ),
      ),
      callout("Bottom Line for the Committee", acc.green.light)[
        The main midterm result is a *working, testable framework architecture* — not yet a validated planetary simulation. Biodiversity serves as the concrete scenario to validate the technology.
      ],
      align(
        right,
        text(
          font: ("JetBrains Mono", "DejaVu Sans Mono"),
          size: 18pt,
          weight: 700,
          fill: acc.green.light,
          "→ Questions & feedback welcome",
        ),
      ),
    )
  },
  controls: (),
)

// ============================================================
// APPENDIX / BACKUP SLIDES (outside 15-min presentation)
// ============================================================
#scene(s => {
  gh.section-slide(
    number: "A",
    kicker: "Backup Slides",
    title: [Architecture, Evaluation & Metrics],
  )
})

// ------------------------------------------------------------
// Backup A1 — Kernel Ports & WorldBuilder Contract
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Backup: kernel ports & extensibility contract",
    )

    grid(
      columns: (1.15fr, 0.85fr),
      gutter: 20pt,
      [
        #let kw(s) = gh.c-keyword(s)
        #let cm(s) = gh.c-comment(s)
        #let ln(b) = text(size: 13pt, b)
        #gh.terminal-block(
          title: "crates/pbma-core/src/ports.rs",
          lang: "rust",
          lines: (
            ln(cm("// kernel never imports adapters")),
            ln([#kw("pub trait") EnvironmentStep: Send + Sync {]),
            ln(
              [  #kw("fn") step(&#kw("self"), cell: &CellContext#"<'_>", out: &mut EnvState);],
            ),
            ln([}]),
            ln([#kw("pub trait") PopulationBehavior: Send + Sync {]),
            ln(
              [  #kw("fn") update(&#kw("self"), sp: SpeciesId, cell: &CellContext#"<'_>") -> BehaviorDelta;],
            ),
            ln([}]),
            ln([#kw("pub trait") MigrationPolicy: Send + Sync {]),
            ln(
              [  #kw("fn") fluxes(&#kw("self"), cell: &CellContext#"<'_>", out: &mut Vec#"<Flux>");],
            ),
            ln([}]),
            ln([#kw("pub trait") EvolutionEngine {]),
            ln(
              [  #kw("fn") evolve(&#kw("mut self"), world: &World, rng: &#kw("mut dyn") Rng) -> EvolutionStep;],
            ),
            ln([}]),
          ),
        )
      ],
      stack(
        spacing: 12pt,
        card(
          "Why this contract matters",
          [
            - *Zero kernel rewrites:* `FormulaBehavior`, batched Laya inference, or new physics plug behind the exact same traits.
            - *Thread-safe by type:* `Send + Sync` per-cell ports enforce read-only `CellContext` access across `rayon` chunks.
            - *`WorldBuilder`:* Declaratively configures topology, orbital/planet params, custom gases, and custom traits.
          ],
          accent: acc.green.light,
        ),
      ),
    )
  },
  controls: (),
)

// ------------------------------------------------------------
// Backup A2 — Decision-Model Evaluation & Distillation Protocol
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Backup: Laya / Jev evaluation & runtime choice",
    )

    grid(
      columns: (1fr, 1fr),
      gutter: 20pt,
      card(
        "Typed Ecological Task Evaluation (RQ3)",
        [
          - *Constrained queries:* survival, growth modifier, competition/predation outcome, migration pressure, directional trait shift.
          - *Leakage-free splits:* train/validation/test split by *held-out ecological scenarios*, never overlapping test regimes.
          - *Calibration & failures:* measure probability calibration (Brier / ECE) and categorize failure modes vs. domain expectations.
        ],
        kicker: "Scientific Evaluation",
        accent: acc.purple.light,
      ),
      card(
        "Runtime Latency Decision Gate (RQ1)",
        [
          - Measure *batched forward-pass latency* per 1M cell-species queries vs. `FormulaBehavior` (\~113 ms total tick baseline).
          - *Option A (Direct):* periodic or cached batched Laya queries if tick budget holds.
          - *Option B (Distilled):* offline fine-tuning → extract explicit closed-form runtime formula behind `PopulationBehavior`.
        ],
        kicker: "Engineering Trade-Off",
        accent: acc.blue.light,
      ),
    )
  },
  controls: (),
)

// ------------------------------------------------------------
// Backup A3 — Diversity Definitions & AoS vs. SoA Rationale
// ------------------------------------------------------------
#scene(
  tag => {
    show: gh.content-slide.with(
      title: "Backup: diversity metrics & AoS memory layout",
    )

    grid(
      columns: (1fr, 1fr),
      gutter: 20pt,
      card(
        "Whittaker α / β / γ Formulation",
        [
          $ beta = gamma slash alpha $
          - *$alpha$*: mean local species richness per ecosystem region.
          - *$gamma$*: total global species richness across all regions.
          - *Minimum differentiation ($beta = 1$):* every region holds the same generalist set ($alpha = gamma$).
          - *Maximum differentiation ($beta = N$):* $N$ regions share zero species ($gamma = alpha dot N$).
        ],
        kicker: "Evaluation Scenario Metrics",
        accent: acc.green.light,
      ),
      card(
        "Why AoS Cell Storage Today?",
        [
          - Each `EnvState` holds inline `GasAmounts` (`INLINE = 8`), `temperature`, `light`, and `uv`.
          - Environment & behavior phases read *all* fields of a cell together, so AoS uses loaded 64-byte cache lines efficiently and keeps `CellContext` simple.
          - Future sparse single-field passes would justify profiling SoA vs. AoS cache misses.
        ],
        kicker: "Cache & Interface Trade-Off",
        accent: acc.orange.light,
      ),
    )
  },
  controls: (),
)
