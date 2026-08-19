// Thanks for using this thesis template!
// Please submit any issues or feature requests to https://github.com/TimerErTim/hagenberg-thesis-typst/issues
// Refer to the documentation at https://github.com/TimerErTim/hagenberg-thesis-typst/tree/main/easy-hgb-thesis-manual.pdf for more information.

#import "@preview/easy-hgb-thesis:0.2.1": WORK_TYPES, full-thesis, titlepage

#let title = sys.inputs.at("title", default: "TITLE MISSING")

#set document(
  title: title,
  author: "Tim Peko",
  description: ```
  This work presents a new way to simulate how large systems behave over time in a grid-like world, using groups of agents that each stand for a species. The simulation is programmed in Rust.

  The core concepts are:
  1. The environment is set up as a grid, similar to cellular automata, which allows things like gas concentrations to spread from cell to cell.
  2. Each cell holds agents that represent whole species populations, not individual organisms.

  Time step updates of concept 1 are based on physical laws.
  Time step updates of concept 2 are based on an approximative MARL model.
  ```,
  keywords: (
    "simulation",
    "cellular automata",
    "multi-agent systems",
    "evolution",
    "population dynamics",
  ),
)
#set text(lang: "en")
#import "abbrev.typ": abbr

#let apply-link-style(it) = {
  show link: underline
  it
}

#let apply-abbreviation-highlighting(it) = (
  abbr
    .keys()
    .fold(it, (it, key) => {
      show key: emph
      it
    })
)

#show: full-thesis.with(
  titlepage: titlepage(
    "Medical and Bioinformatics",
    "FH-Prof. PD DI Dr. Stephan Winkler",
    work-type: WORK_TYPES.bachelor-thesis,
  ),
  acknowledgement: include "chapters/acknowledgement.typ", // Can be deleted if not required
  kurzfassung: include "chapters/kurzfassung.typ",
  abstract: include "chapters/abstract.typ",
  appendix: include "chapters/appendix.typ",
  bibl: bibliography("bib.yaml"), // Can be replaced with a BibLaTex file,
  abbreviations: abbr,

  content-style: it => {
    show table.cell.where(y: 0): strong
    show: apply-link-style
    show: apply-abbreviation-highlighting
    it
  },
  appendix-style: it => {
    show: apply-link-style
    show: apply-abbreviation-highlighting
    it
  },
  acknowledgement-style: it => {
    show: apply-link-style
    it
  },
  abstract-style: it => {
    show: apply-link-style
    show: apply-abbreviation-highlighting
    it
  },
)

#include "chapters/introduction.typ"
#include "chapters/foundation.typ"
#include "chapters/methodology.typ"
#include "chapters/results.typ"
#include "chapters/conclusion.typ"
