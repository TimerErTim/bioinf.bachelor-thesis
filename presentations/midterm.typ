#import "libs/sanor/lib.typ" as sr
#import "libs/gh-template.typ" as gh

#let handout = eval(sys.inputs.at("x-handout", default: "false"))
#let title = sys.inputs.at("title", default: "TITLE")
#let (slide, scene) = sr.set-option(handout: handout)

#let presentation-date = datetime(year: 2026, month: 12, day: 03)
#set document(
  title: title + ": Midterm Presentation",
  author: "Tim Peko",
  date: presentation-date,
  keywords: ("midterm", "presentation", "bioinformatics", "fhooe", "hagenberg", "bachelor-thesis"),
)
#show: gh.register.with(
  accent: "green",
)

#scene(s => context {
  show: gh.cover-slide.with(
    badges: ([
      Campus Hagenberg
    ], [
      #set text(fill: gh.gh-accents.purple.light)
      Bachelor of Science] , [
      #set text(fill: gh.gh-accents.green.light)
        Medical & Bioinformatics
    ]),
    kicker: image("../assets/fhooe.png", height: 2em),
    footer-left: [
      #set text(fill: gh.gh-accents.green.light)
      \# #document.author.join(", ") · #document.date.display()
    ],
  )
  set text(size: 46pt)
  title
})

#scene(
  s => {
    show: gh.content-slide.with(title: "Introduction")

    show: s.with("a")
    [ASAD]
    title
  },
  controls: (
    (),
    sr.apply("a"),
  ),
)
