// Thanks for using this thesis template!
// Please submit any issues or feature requests to https://github.com/TimerErTim/hagenberg-thesis-typst/issues
// Refer to the documentation at https://github.com/TimerErTim/hagenberg-thesis-typst/tree/main/easy-hgb-thesis-manual.pdf for more information.

#import "@preview/easy-hgb-thesis:0.2.1": full-thesis, titlepage, WORK_TYPES

#let title = sys.inputs.at("title", default: "TITLE MISSING")

#set document(
  title: title,
  author: ("Tim Peko"),
  description: ```
  This work presents a new way to simulate how large systems behave over time in a grid-like world, using groups of agents that each stand for a species. The simulation is programmed in Rust.

  The core concepts are:
  1. The environment is set up as a grid, similar to cellular automata, which allows things like gas concentrations to spread from cell to cell.
  2. Each cell holds agents that represent whole species populations, not individual organisms.

  Concept 1. time step updates are based on physical laws.
  Concept 2. time step updates are based on an approximative MARL model.
  ```,
  keywords: ("simulation", "cellular automata", "multi-agent systems", "evolution", "population dynamics"),
)
#set text(lang: "en")

//#import "abbrev.typ": abbr
#show: full-thesis.with(
  titlepage: titlepage(
    "Medical and Bioinformatics",
    "FH-Prof. PD DI Dr. Stephan Winkler",
    work-type: WORK_TYPES.bachelor-thesis,
  ),
  acknowledgement: include "chapters/acknowledgement.typ", // Can be deleted if not required
  kurzfassung: include "chapters/kurzfassung.typ",
  abstract: include "chapters/abstract.typ",
  //appendix: include "chapters/appendix.typ", // Can be deleted if not required
  bibl: bibliography("bib.yaml"), // Can be replaced with a BibLaTex file,

  // Demonstration of how to apply custom styles to sections, can be deleted if not required.
  content-style: it => {
    show table.cell.where(y: 0): strong
    it
  },
)

#include "chapters/introduction.typ"
#include "chapters/foundation.typ"
#include "chapters/methodology.typ"
#include "chapters/results.typ"
#include "chapters/conclusion.typ"
