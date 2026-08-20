= Foundational works

== Tierra <tierra-introduction>

https://en.wikipedia.org/wiki/Tierra_(computer_simulation)

== Avida

Inspired by Tierra (@tierra-introduction)

https://de.wikipedia.org/wiki/Avida

== Räumlich erweitertes TaNa Modell

== Multi-Level Makroevolutions-Frameworks

State of the Art, Bottom-Up Ansätze. 2025 von Latorre et al. in PLoS ONE. @src-macroevolutionary-trends-2025

== klassische ABMs

Oft nicht skalierbar

CoMSES Net (OpenABM)

== Homeostatic Reinforcement Learning

https://www.researchgate.net/publication/354597646_Continuous_Homeostatic_Reinforcement_Learning_for_Self-Regulated_Autonomous_Agents @src-homeostatic-rl

== Lotka-Volterra-Dynamiken

== Das "EcoTwin"-Projekt

#quote(block: true)[
  Twin models of natural ecosystems hold great promise for informing real-world decisions on sustainable land use and biodiversity conservation. However, existing simulations of animal behavior often rely on manually crafted rules, limiting their scalability and practical utility. Here, we present a flexible and scalable agent-based modeling approach that uses reinforcement learning---instead of hand-coded rules---to simulate animal behavior. We validate this approach across ten alpine ecosystems featuring wolves, chamois, and vegetation.By comparing model outputs with empirical data, we show that the simulations reproduce realistic ecological and behavioral patterns, including population dynamics, life history traits, and social interactions. We then use the model to assess ecosystem resilience under scenarios of habitat degradation, game hunting, and heat stress. Our framework paves the way for realistic simulations advancing our ability to predict ecosystem responses to disturbance and tipping points leading to biodiversity loss, in order to support conservation planning and guide the sustainable use of natural resources.
] @src-ecotwin-strannegard-2025

== "PredPreyGrass" via MADRL (CoMSES Net Repositorium)

Exploring learned cooperation, coevolution and free-riding. Learning is achieved through Multi-Agent Deep Reinforcement Learning (MADRL) in an ecological environment. The environment emits no other than sparse reproduction rewards. No reward shaping, no explicit cooperation signal. @src-predpreyggrass-madrl

== Multi-Agent Inverse Reinforcement Learning

Recent comparative studies between classical population-based modeling (PBM)—which typically uses differential equations—and agent-based modeling (ABM) have increasingly applied Multi-Agent Inverse Reinforcement Learning (MIRL). Instead of prescribing a reward function to the agents, MIRL uses empirical macroscopic data to infer, in reverse, the underlying reward functions that drive real-world collective or dispersal behaviors.

These studies provide mathematical evidence that ABMs, when applied to very large and homogeneous populations, can accurately approximate the same macroscopic dynamics as traditional PBM approaches. This finding directly supports the design choice of this thesis: modeling agents as aggregated population units, rather than simulating millions of individual entities, is both justified and efficient for capturing large-scale ecological dynamics.

== Agent-Based Modeling vs. Population-Based Modeling

#quote(block: true)[
  This paper addresses comparative evaluation of population-based simulation in comparison to agent-based simulation for different numbers of agents. Population-based simulation, such as for example in the classical approaches to predator-prey modelling and modelling of epidemics, has computational advantages over agent-based modelling with large numbers of agents. Therefore the latter approaches can be considered useful only when the results are expected to deviate from the results of population-based simulation, and are considered more realistic. However, there is sometimes also a silent assumption that for larger numbers of agents, agent-based simulations approximate population-based simulations, which would indicate that agent-based simulation just can be replaced by population-based simulation. The paper evaluates such assumptions by two detailed comparative case studies: one in epidemics, and one in economical context. The former case study addresses the spread of an infectious disease over a population. The latter case study addresses the interplay between individual greed as a psychological concept and global economical concepts. It is shown that under certain conditions agent-based and population-based simulations may show similar results, but not always. 1 Parts of this paper have been presented at the conferences ECMS'08 [20] and IEA/AIE'10 [7]. The current paper extends these conference papers by providing additional simulation experiments, simulation results relating to empirical data, more detailed analyses of simulation results, and a more extensive discussion of related work and of the differences between agent-based and population-based modelling.
] @src-agentbased-populationbased-2012

Grundlage für den Ansatz, dass Populationbasierte Modelle für sehr große und homogene Populationen exakt das gleiche makroskopische Verhalten wie traditionelle PBMs approximieren können.
