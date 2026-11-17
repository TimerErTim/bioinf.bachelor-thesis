= Introduction

== Motivation

Complex adaptive systems---from urban traffic and epidemiology to planetary ecosystems---share a fundamental computational challenge: macroscopic behavior emerges from millions of local interactions between agents and a spatially heterogeneous environment over long time horizons. Designing a simulation framework for such systems requires reconciling three competing demands:

+ *Computational scale and reproducibility:* Individual-based simulations become prohibitively expensive once millions of entities must be tracked across large spatial domains, while purely aggregate differential equations discard spatial structure and local feedback. At the same time, scientific experiments demand bit-reproducible execution across parallel hardware.
+ *Expressive agent decision-making without bespoke RL overhead:* Hand-crafted heuristic rules quickly become brittle as state spaces grow, whereas training a custom multi-agent reinforcement learning (MARL) policy requires designing a defensible reward signal and investing heavily in training infrastructure rather than system-dynamics modeling.
+ *Architectural modularity for hybrid feedback:* Agents do not merely react to their surroundings; they actively reshape the environment, which in turn alters future selection pressures. Separating the simulation kernel from concrete physical rules, behavior models, and telemetry allows individual components to evolve and be compared fairly.

This thesis addresses that engineering and modeling problem by designing a *population-based multi-agent (PBMA) simulation framework* in Rust. Instead of simulating every individual entity, the framework represents aggregated *populations per cell* on a discrete cellular-automaton grid. A small, deterministic, chunk-parallel kernel (`pbma-core`) owns the spatial storage, double-buffered tick orchestration, and extensible domain registries, while all domain dynamics---environmental physics, agent behavior, migration, evolution, and metrics---are injected through strictly typed ports.

A central architectural hypothesis of this work is that local agent interaction decisions can be decoupled from bespoke RL training by delegating them to small, open-source, non-autoregressive *typed decision models* (e.g., JevK5 or a fine-tuned Laya checkpoint). Rather than generating free-form text, these models receive a structured cell context and answer constrained typed questions (yes/no, choice, or scalar score) in a single forward pass---either directly at runtime or distilled offline into cheap closed-form runtime formulas behind the same behavior port. Conversely, agent-to-environment effects are deliberately kept explicit and rule-based so that coupled feedback remains transparent, controllable, and verifiable.

To evaluate the framework and the potential of this decision-model technology under demanding conditions, this thesis uses *planetary macroevolution and biodiversity* as its concrete evaluation scenario. Ecological systems exhibit strong spatial coupling, long timescales that cannot be observed directly in a laboratory, and dramatic global feedback loops such as the Great Oxidation Event (GOE), where early oxygenic organisms reshaped the atmosphere and forced widespread adaptation or extinction. Within this testbed, we examine a concrete macroecological question: what happens to regional versus global biodiversity when living conditions become "hyperoptimal" everywhere? Under benign, spatially uniform conditions, generalist species may outcompete local specialists and homogenize communities across regions---a hypothesis that requires both large spatial grids and rich local interaction dynamics to test. Crucially, the framework is designed as an *exploratory experimental platform*, not a calibrated digital twin of Earth's history; the ecological scenarios serve to stress-test kernel scaling, hybrid feedback, and decision-model transferability.

== Ecosystem Diversities <chp:ecosystem-diversities>

To quantify biodiversity outcomes in the concrete evaluation scenario across geographical regions, three established measures are used:

- *#math.alpha\-Diversity* describes the amount of species diversity within a particular area or ecosystem. It is usually expressed as the number of species. @src:whittaker-alpha-beta-diversity

- *#math.beta\-Diversity* describes the amount of species diversity between ecosystems. It is usually expressed as the number of species that are unique to each area. @src:whittaker-alpha-beta-diversity

- *#math.gamma\-Diversity* describes the amount of species diversity across all ecosystems of a large region. This number is typically derived by counting the total number of species, similar to the #math.alpha\-Diversity. @src:whittaker-alpha-beta-diversity

#{
  show: it => [#it <tbl:example-beta-gamma-alpha-calculation>]
  show: figure.with(
    caption: [Illustration of the calculation of $alpha$-, $beta$-, and $gamma$-Diversity],
  )

  let ec1_species = ("A", "B", "C", "D", "E", "F")
  let ec2_species = ("F", "G", "H", "I")
  let ec3_species = ("H", "I", "J")
  let all_ecosystems = (ec1_species, ec2_species, ec3_species)
  table(
    columns: (auto,) + (1fr,) * 3,
    fill: (x, y) => {
      if y == 0 {
        return gray.lighten(50%)
      }

      if calc.rem(y, 2) == 0 {
        return gray.lighten(80%)
      }
    },
    table.header[*Hypothetical Species*][*Ecosystem 1*][*Ecosystem 2*][*Ecosystem 3*],
    ..for species in all_ecosystems.join().dedup().sorted() {
      (
        [#species],
        ..all_ecosystems.map(ec => if ec.contains(
          species,
        ) [#sym.crossmark] else []),
      )
    },
    [$alpha$-Diversity], ..all_ecosystems.map(ec => [#ec.dedup().len()]),
    table.cell(rowspan: 2, align: horizon)[$beta$-Diversity],
    ..(
      ..for i in range(all_ecosystems.len() - 1) {
        for j in range(all_ecosystems.len()) {
          ((j, range(all_ecosystems.len()).filter(k => k != j).at(i)),)
        }
      },
    ).map(((i, j)) => [
      #let this_ecosystem = all_ecosystems.at(i)
      #let target_ecosystem = all_ecosystems.at(j)
      #[#set text(size: 0.8em)
        vs. Ecosystem #(j + 1):]
      #(
        (this_ecosystem + target_ecosystem).dedup().len()
          - this_ecosystem
            .filter(species => target_ecosystem.contains(species))
            .len()
      )
    ]),
    [$gamma$-Diversity],
    table.cell(colspan: all_ecosystems.len())[#(
      all_ecosystems.join().dedup().len()
    )],
  )
}

An illustrative example of their calculation is given in @tbl:example-beta-gamma-alpha-calculation.


== Research Questions <chp:research-questions>

#grid(
  columns: (auto, 1fr),
  inset: (y: 0.65em),
  gutter: 0.5em,
  align: (left, left),
  [*RQ1*],
  [*How can a large-scale discrete simulation framework be designed to couple a deterministic, parallelizable kernel with small, general-purpose typed decision models at scale without training a custom reinforcement learning policy?*

    Rather than hard-coding all agent heuristics or building bespoke RL training loops, the kernel exposes typed ports that supply cell context and species traits to interchangeable adapters. Population-level decisions can be answered by small, open-source, non-autoregressive "System 1" decision models (e.g., JevK5 or a fine-tuned Laya checkpoint) returning calibrated probabilities for typed queries in a single forward pass, or by offline-distilled closed-form runtime formulas behind the same interface.],

  [*RQ2*],
  [*How can decision-model-driven agent behavior be cleanly combined with explicit, rule-based environmental feedback so that coupled system dynamics remain controllable, extensible, and bit-reproducible?*

    In coupled spatial systems, agents actively modify their surroundings (e.g., metabolic oxygen production during a Great Oxidation Event). Keeping the agent-to-environment direction strictly deterministic and rule-based---while enforcing double-buffered snapshots, canonical iteration orders, and explicit seedable randomness in the kernel---prevents uncontrolled feedback instabilities and preserves bit-identical parallel execution.],

  [*RQ3*\ Research Question 3],
  [*How far do general-purpose typed decision models transfer to domain-specific ecological decisions in a concrete evaluation scenario, and what do their accuracy, failure patterns, and runtime trade-offs reveal about this technology?*

    Evaluating the framework on planetary biodiversity and atmospheric feedback tests both the software architecture and the decision-model paradigm itself. Because base decision models are trained on general-purpose decision data rather than ecological domain data, measuring where they succeed, where calibration drifts, and how inference latency compares against closed-form surrogates illuminates the practical potential and limits of this technology.],
)

== Goals

- Design and implement a modular, bit-reproducible, chunk-parallel simulation framework in Rust (`pbma-core`, `pbma-sim`) based on hexagonal ports, double-buffered grid storage, and extensible gas/trait registries.
- Establish an interchangeable behavior-adapter boundary that supports closed-form baselines, offline-distilled runtime formulas, and small typed decision models (JevK5, fine-tuned Laya) without custom RL training.
- Combine learned or formula-driven agent behavior with explicit, rule-based environmental physics and agent-to-environment feedback.
- Validate the framework's scalability, determinism, and decision-model transferability on a concrete planetary biodiversity evaluation scenario (hyperoptimal versus harsh conditions measured via #math.alpha\-/#math.beta\-/#math.gamma\-diversity and a Great Oxidation Event feedback loop).


== Evaluation Scenario Hypotheses <chp:hypothesis>

To instantiate the framework on a concrete scientific testbed and address the research questions in @chp:research-questions, the planetary biodiversity evaluation scenario tests the following working expectations:

+ Under hyperoptimal living conditions, the #math.alpha\-Diversity is close to equal to the #math.gamma\-Diversity ($beta = gamma slash alpha -> 1$).
+ Under harsh, heterogeneous living conditions, the #math.alpha\-Diversity is significantly lower than the #math.gamma\-Diversity, exhibiting strong regional specialization ($beta >> 1$).
+ Hyperoptimal living conditions result in a higher #math.gamma\-Diversity than harsh living conditions.

Additionally, tied directly to RQ3 on technology transfer: general-purpose typed decision models are expected to transfer to ecological interaction decisions only partially out of the box---exposing characteristic failure modes, calibration gaps, and latency-versus-accuracy trade-offs that characterize what systems-level understanding these models encode and where fine-tuning or formula distillation is required.

