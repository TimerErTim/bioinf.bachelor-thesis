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

== Approaches to Agent Control

How to update the population's attributes, specifically amount present and evolution over time? Good question... further research needed.

== Self-Supervised MARL

Wenn biologische Organismen in eine völlig neue Umgebung geworfen werden, haben sie keinen zentralen "Score", den sie optimieren. Sie werden durch intrinsische Reize gesteuert. In dezentralen MARL-Systemen lässt sich das abbilden, indem man Agenten nicht für ein externes Ziel belohnt, sondern für die Informationsverarbeitung.

- Curiosity (Neugier) und Vorhersagefehler: Der Agent erhält eine Belohnung (Forward Prediction Loss), wenn er Zustände mit hoher Unsicherheit erkundet oder wenn er lernt, die Konsequenzen seiner Aktionen besser vorherzusagen. Er lernt also kontinuierlich, wie die Umwelt funktioniert, wodurch sich flexible Überlebensstrategien von ganz allein entwickeln.
- Sozialer Einfluss (Social Influence): Agenten werden dafür belohnt, dass ihre Aktionen das Verhalten oder den Zustand anderer Spezies vorhersehbar beeinflussen. Dies fördert das spontane Entstehen von realistischen inter-spezifischen Rollen (wie Prädator, Beute oder Symbiont), da die Spezies lernen, aufeinander zu reagieren, um ein soziales beziehungsweise ökologisches Gleichgewicht zu finden.
