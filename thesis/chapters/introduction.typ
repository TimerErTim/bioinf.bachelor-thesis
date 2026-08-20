= Introduction

== Motivation

When we look at nature, we see an incredibly complex interplay of countless animal and plant species. They prey on one another, compete for space, adapt to temperatures, and change the world around them simply by existing. This process, known as evolution, has shaped our Earth over billions of years. However, modern science faces a fundamental problem: we cannot put an entire planet into a laboratory. If we want to understand how a lifeless rock turned into a green, oxygen-rich world full of life, we run into an issue of time. Real evolution is far too slow to be observed in real-time.

This is exactly where modern bioinformatics comes into play. The overarching goal of this research field is to move evolution into the computer. If we can program realistic virtual worlds, we can simulate millions of years of Earth's history in a single afternoon. This is not just a theoretical game; it has enormous practical value for the real world. By understanding the fundamental rules of how entire ecosystems build up or collapse, we can better predict how our modern environment will react to dramatic changes—whether caused by global climate change, invasive species, or pollution. We are essentially creating a "digital twin" of ecological networks to test the breaking points of life.

This thesis focuses on developing exactly this kind of planetary simulation. The goal is to build a computational framework that can simulate the development of species on a global scale. However, instead of trying to calculate every single animal or every leaf on a tree—which would immediately overload even the most powerful supercomputers—this work introduces a clever, alternative approach. The virtual world is divided into a massive spatial grid. The cells in this grid do not contain individual animals, but rather entire populations of species acting as agents. If a species thrives, its "influence" in that region grows. If it gets too crowded or the climate becomes too harsh, a part of the population migrates to neighboring cells.

A central element of this simulation is the realization that life does not merely adapt to its environment; it actively rebuilds it. A famous example from Earth's history, which is modeled in this work, is the Great Oxidation Event. Early plant-like life forms produced massive amounts of oxygen. What is vital for us today was pure poison for the anaerobic world back then. By simply spreading, these plants changed the chemical composition of the entire atmosphere. This creates enormous evolutionary pressure: other species must adapt, flee, or go extinct. Such massive, global feedback loops between life and the environment will be realistically modeled in the system developed here.

Furthermore, this thesis addresses a highly fascinating core question: What actually happens when life has it "too easy"? Our hypothesis is that under perfect global living conditions—meaning it is warm everywhere, there is plenty of food, and there are no extreme environmental barriers—the absolute biodiversity on the planet paradoxically decreases rather than increases. If nature does not force species to specialize in certain harsh niches, so-called "generalists" emerge. These are super-species that can survive almost anywhere. They spread unhindered across the globe and ultimately outcompete all other, weaker species. While there might be many animals in one specific location, looking at the planet as a whole, every region looks exactly the same. This theory of biological homogenization will be put to the test through the simulation.

To realize this ambitious project, standard programming is not enough. The simulation is written in the modern, high-performance programming language Rust. Additionally, to ensure that the computer knows how the millions of populations should behave realistically, advanced mathematics and techniques from the field of artificial intelligence are utilized in the background. This AI does not actively control the simulation during runtime; rather, it is used beforehand to translate the extremely complex biological rules into simple, highly efficient mathematical formulas.

In summary, this thesis combines biology, environmental physics, and cutting-edge software engineering. It provides a computational tool to make the complex, often invisible gears of macroevolution tangible, helping us understand why our planet's biodiversity looks the way it does today—and how quickly it can collapse when the rules of nature change.

== Informatics Relevancy

== Biological Relevancy

== Hypothesis

#math.alpha\-Diversity \~ #math.gamma\-Diversity

