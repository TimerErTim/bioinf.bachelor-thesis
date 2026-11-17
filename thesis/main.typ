// Thanks for using this thesis template!
// Please submit any issues or feature requests to https://github.com/TimerErTim/hagenberg-thesis-typst/issues
// Refer to the documentation at https://github.com/TimerErTim/hagenberg-thesis-typst/tree/main/easy-hgb-thesis-manual.pdf for more information.

#import "@preview/easy-hgb-thesis:0.2.2": WORK_TYPES, full-thesis, titlepage
#import "style.typ": style-document

#let title = sys.inputs.at("title", default: "TITLE MISSING")
#let printing = eval(sys.inputs.at("printing", default: "false"))

#set document(
  title: title,
  author: "Tim Peko",
  description: ```
  This work investigates how to design large-scale, deterministic discrete simulation frameworks in conjunction with small, open-source typesafe decision models (JevK5, fine-tuned Laya), implemented in Rust.

  The core concepts are:
  1. A chunk-parallel cellular-automaton environment grid updated by physical and explicit rule-based dynamics.
  2. Population-level multi-agent entities whose interaction decisions are delegated behind modular kernel ports to typed decision models or offline-distilled runtime formulas instead of custom reinforcement learning.

  Planetary ecosystem dynamics—specifically biodiversity under hyperoptimal versus harsh conditions and species-driven atmospheric feedback—serve as the concrete evaluation scenario for testing scalability, reproducibility, and decision-model transfer.
  ```,
  keywords: (
    "simulation framework",
    "cellular automata",
    "multi-agent systems",
    "decision models",
    "population dynamics",
  ),
)
#set text(lang: "en")
#import "abbrev.typ": abbr

#let apply-link-style(it) = {
  show link: it => if type(it.dest) == location {
    it
  } else {
    underline(it)
  }
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

#let apply-autoplaced-figures(it) = {
  set figure(placement: auto)
  it
}

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
  bibl: bibliography("../bib.yaml"), // Can be replaced with a BibLaTex file,
  abbreviations: abbr,
  include-print-size-control: printing,

  document-style: it => {
    set text(font: "Libertinus Serif")
    let sans-fonts = state("_eht-sans-fonts")
    sans-fonts.update(("Libertinus Sans",))
    show raw: set text(font: "JetBrains Mono")
    show math.equation: set text(font: "Libertinus Math")

    show: style-document

    it
  },
  content-style: it => {
    show table.cell.where(y: 0): strong
    show: apply-link-style
    show: apply-abbreviation-highlighting
    show: apply-autoplaced-figures
    it
  },
  appendix-style: it => {
    show: apply-link-style
    show: apply-abbreviation-highlighting
    show: apply-autoplaced-figures
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
#include "chapters/implementation.typ"
#include "chapters/results.typ"
#include "chapters/conclusion.typ"
