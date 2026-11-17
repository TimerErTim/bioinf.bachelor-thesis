= Methodology

== Environment Simulation

Eucledian simulation (discrete timesteps)
Grid-based, every cell has environment state
- Gases amounts
  - $0_2$
  - $C O_2$
  - $C H_4$
  - $H_2 O$
  - ...
- Temperature
- Light
- ...

There are global environment parameters such as:
- Gravity
- Length of day and night
- Axis tilt
- ...

=== Cellular Automata

Every cell's environment state is updated based on the state of the neighboring cells. Pressure equalization is simulated by diffusion of the gases.

== Agent Representation

Every agent represents a single species/population. It has different attributes:
- ...

A cell holds a list of species and their amounts present in the cell.

=== Definition of a single species

Evolutionary lineage with similar enough traits to meaningfully differ from other species.

=== Calculating species diversity

As seen in @tbl:example-beta-gamma-alpha-calculation of @chp:ecosystem-diversities, the number of different $beta$-Diversities escalates with the amount of $n$ local Ecosystems as $vec(n, 2)$. Instead, @eq:beta-gamma-alpha-relation defines the general relationship between $alpha$-, $beta$-, and $gamma$-Diversities. This enables the usage of a single derived $beta$-Diversity value to evaluate the diversity across all ecosystems as average or individually. @src:whittaker-alpha-beta-diversity @src:whittaker-vegetation-1960

This is very suitable for evaluating the correctness of the first hypothesis in @chp:hypothesis, as this $beta$-Diversity exhibits two extreme cases:
- *Minimum Differentiation ($beta = 1$)*: If every local ecosystem contains similar species as the global ecosystem, local diversity equals global diversity ($alpha = gamma$), resulting in a $beta$ of $1$. Therfore, the simulation should reach this value if the first hypothesis is correct.

- *Maximum Differentiation ($beta = N$)*: If $N$ different local ecosystems share zero species in common, global diversity equals the sum of all local diversities ($gamma = alpha times N$), resulting in a $beta$ equal to the number of local ecosystems ($N$). Therefore, the simulation should reach this value if the first hypothesis is incorrect.

$
  beta = gamma slash alpha
$ <eq:beta-gamma-alpha-relation>

== Approaches to Agent Control <approaches-agent-control>

How to update the population's attributes, specifically amount present and evolution over time?

=== Skipped: Custom Reinforcement Learning Model

Training a custom reinforcement learning model for predicting species <-> species and environment -> species interactions was considered, but is deliberately *skipped* in this work:

- Training a custom RL model requires a reward signal that does not naturally exist for ecological interactions.
- Curricula and simulation loops for training would dominate the implementation effort, while the actual research focus lies on the large-scale system dynamics.
- Pretrained open-source models offer a functional alternative at a fraction of the cost.

=== Used Instead: Open-Source Typesafe Decision Models ("Jev"-Style)

Species <-> species and environment -> species interactions are instead answered by small, open-source, typesafe "System 1" decision models, e.g.:

- JevK5: open-weight model (Apache-2.0), answers typed yes/no, choice, and score questions with calibrated probabilities in a single forward pass, without generating text.
- Laya: open-source non-autoregressive decision model; the base checkpoint is intended as a foundation to be fine-tuned for the ecological interaction decisions in this work.

The models receive the state of a cell (species present, population amounts, environmental conditions) and a set of typed questions and return a probability distribution over the possible options. These probabilities then parameterize the population update step (reproduction, predation, migration pressure, trait adaptation), which keeps the runtime decisions cheap, deterministic in shape, and inspectable.

== Species #sym.arrow.l.r Environment: Hard Rule-Based

The reverse direction, in which species actively reshape their environment (e.g. oxygen production during the Great Oxidation Event), is deliberately *not* delegated to a learned model. It is modeled as hard, deterministic rule sets (to be defined), because:

- Environment modification is a direct physiological consequence of metabolism and population size, making it well-suited to explicit rules.
- Deterministic rules keep the environment side reproducible and prevent uncontrolled feedback loops between the learned and rule-based directions.
- Rule sets can be validated directly against geochemical records (e.g. atmospheric oxygen curves).

=== RQ3: Transfer and Potential of Generalistic Decision Models

Since these models are trained on general-purpose decision data rather than ecological domain data, their transferability and runtime viability within a large-scale simulation framework is an open question (RQ3). Using the ecological testbed, this work examines how far generalistic typed decision models transfer to domain-specific interaction decisions, and what can be inferred from their behavior---agreement with domain expectations, characteristic failure patterns, calibration drift, and inference-versus-formula latency---about both the biological/systems understanding these models encode and the practical potential of coupling them with discrete simulation kernels.

== Self-Supervised MARL

*Note:* This direction was considered as the basis for a custom reinforcement learning model, but a custom RL model is no longer trained in this work (see @approaches-agent-control). The intrinsic-motivation concepts below remain relevant as inspiration for the typed questions posed to the decision models, and for the secondary research question on their high-level biological and systems understanding (@chp:research-questions).

Wenn biologische Organismen in eine völlig neue Umgebung geworfen werden, haben sie keinen zentralen "Score", den sie optimieren. Sie werden durch intrinsische Reize gesteuert. In dezentralen MARL-Systemen lässt sich das abbilden, indem man Agenten nicht für ein externes Ziel belohnt, sondern für die Informationsverarbeitung.

- Curiosity (Neugier) und Vorhersagefehler: Der Agent erhält eine Belohnung (Forward Prediction Loss), wenn er Zustände mit hoher Unsicherheit erkundet oder wenn er lernt, die Konsequenzen seiner Aktionen besser vorherzusagen. Er lernt also kontinuierlich, wie die Umwelt funktioniert, wodurch sich flexible Überlebensstrategien von ganz allein entwickeln.
- Sozialer Einfluss (Social Influence): Agenten werden dafür belohnt, dass ihre Aktionen das Verhalten oder den Zustand anderer Spezies vorhersehbar beeinflussen. Dies fördert das spontane Entstehen von realistischen inter-spezifischen Rollen (wie Prädator, Beute oder Symbiont), da die Spezies lernen, aufeinander zu reagieren, um ein soziales beziehungsweise ökologisches Gleichgewicht zu finden.

== Vom trainierten Modell zur Laufzeitformel

Die in dieser Arbeit entwickelte Simulationsarchitektur trennt bewusst zwischen Training und Laufzeit. Verhaltensmodelle werden offline trainiert; zur Laufzeit liegt ihr Ergebnis nicht als neuronales Netz, sondern als explizite, evaluierbare Formel vor, die die Verhaltensphase pro Zelle auswertet. Die Softwarearchitektur unterstützt diesen Ansatz direkt: das Verhalten wird über ein Port-Interface injiziert, sodass ein trainiertes Modell und eine handgeschriebene Näherungsformel dieselbe Schnittstelle bedienen und sich gegenseitig ersetzen können, ohne dass der Simulationskern geändert werden muss.
