= Introduction

== Motivation

When we look at nature, we see an incredibly complex interplay of countless animal and plant species. They prey on one another, compete for space, adapt to temperatures, and change the world around them simply by existing. This process, known as evolution, has shaped our Earth over billions of years. However, modern science faces a fundamental problem: we cannot put an entire planet into a laboratory. If we want to understand how a lifeless rock turned into a green, oxygen-rich world full of life, we run into an issue of time. Real evolution is far too slow to be observed in real-time.

This is exactly where modern bioinformatics comes into play. The overarching goal of this research field is to move evolution into the computer. If we can program realistic virtual worlds, we can simulate millions of years of Earth's history in a single afternoon. This is not just a theoretical game; it has enormous practical value for the real world. By understanding the fundamental rules of how entire ecosystems build up or collapse, we can better predict how our modern environment will react to dramatic changes—whether caused by global climate change, invasive species, or pollution. We are essentially creating a "digital twin" of ecological networks to test the breaking points of life.

This thesis focuses on developing exactly this kind of planetary simulation. The goal is to build a computational framework that can simulate the development of species on a global scale. However, instead of trying to calculate every single animal or every leaf on a tree—which would immediately overload even the most powerful supercomputers—this work introduces a clever, alternative approach. The virtual world is divided into a massive spatial grid. The cells in this grid do not contain individual animals, but rather entire populations of species acting as agents. If a species thrives, its "influence" in that region grows. If it gets too crowded or the climate becomes too harsh, a part of the population migrates to neighboring cells.

A central element of this simulation is the realization that life does not merely adapt to its environment; it actively rebuilds it. A famous example from Earth's history, which is modeled in this work, is the Great Oxidation Event. Early plant-like life forms produced massive amounts of oxygen. What is vital for us today was pure poison for the anaerobic world back then. By simply spreading, these plants changed the chemical composition of the entire atmosphere. This creates enormous evolutionary pressure: other species must adapt, flee, or go extinct. Such massive, global feedback loops between life and the environment will be realistically modeled in the system developed here.

Furthermore, this thesis addresses a highly fascinating core question: What actually happens when life has it "too easy"? Our hypothesis is that under perfect global living conditions—meaning it is warm everywhere, there is plenty of food, and there are no extreme environmental barriers—the absolute biodiversity on the planet paradoxically decreases rather than increases. If nature does not force species to specialize in certain harsh niches, so-called "generalists" emerge. These are super-species that can survive almost anywhere. They spread unhindered across the globe and ultimately outcompete all other, weaker species. While there might be many animals in one specific location, looking at the planet as a whole, every region looks exactly the same. This theory of biological homogenization will be put to the test through the simulation.

To realize this ambitious project, standard programming is not enough. The simulation is written in the modern, high-performance programming language Rust. Additionally, to ensure that the computer knows how the millions of populations should behave realistically, techniques from the field of artificial intelligence are utilized: instead of training a custom reinforcement learning model, the interaction decisions between populations---how a population reacts to its environment and to other species---are delegated to small, open-source decision models of the "Jev" type, which answer typed questions with calibrated probabilities in a single forward pass. The opposite direction, in which populations actively reshape their environment, is deliberately kept strictly rule-based.

In summary, this thesis combines biology, environmental physics, and cutting-edge software engineering. It provides a computational tool to make the complex, often invisible gears of macroevolution tangible, helping us understand why our planet's biodiversity looks the way it does today—and how quickly it can collapse when the rules of nature change.

== Research Questions <research-questions>

#grid(
  columns: (auto, 1fr),
  inset: (y: 0.65em),
  row-gutter: 0.5em,
  align: (left, left),
  [*RQ1*\ Research Question 1],
  [*How can species-to-species and environment-to-species interaction be approximated at scale without training a custom reinforcement learning model?*

    Instead of training a bespoke RL policy, population-level interaction decisions are answered by small, open-source, typesafe decision models (so-called "System 1" decision models, e.g. JevK5 or a fine-tuned variant of Laya). These models receive the state of a cell and a set of typed questions (yes/no, choice, score) and return calibrated probabilities in a single forward pass, without generating text.],

  [*RQ2*],
  [*How can the reverse direction, species-to-environment interaction, be captured?*

    Species actively reshape their environment (e.g. oxygen production during the Great Oxidation Event). This direction is deliberately modeled as hard, deterministic rule sets---to be defined---rather than learned behavior.],

  [*RQ3*\ Research Question 3],
  [*How far do such generalistic decision models transfer to ecological interaction decisions, and what does their behavior reveal about the biological or systems-level understanding these models have acquired?*

    A secondary, exploratory question: since these models are trained on general-purpose decision data rather than ecological domain data, their applicability in this domain is not self-evident. Deviations and failure patterns are of independent interest, as they hint at what high-level biological or systems understanding such generalistic models actually possess.],
)

== Goals

- Development of a scalable, grid-based planetary simulation in Rust. The environment is updated by physical rules, species populations act as agents.
- Approximation of species-to-species and environment-to-species interaction via open-source, typesafe decision models (JevK5, fine-tuned Laya) instead of a custom RL model.
- Hard rule-based modeling of species-to-environment interaction.
- Evaluation of whether the simulation reproduces plausible macroevolutionary dynamics (e.g. the Great Oxidation Event) and the biodiversity homogenization hypothesis.

== Hypothesis

#math.alpha\-Diversity \~ #math.gamma\-Diversity

Additionally, as a secondary expectation tied to RQ3: the generalistic decision models will be applicable to ecological interaction decisions only to a limited degree---expect transfer gaps and characteristic failure patterns that reveal how much (or how little) high-level biological and systems understanding such models actually encode.

