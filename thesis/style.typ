#import "deps.typ": *

#let flavor = latte
#let accent-colors = flavor.colors.filter(it => it.accent == true).map(it => it.rgb)
#let base-colors = flavor.colors.filter(it => it.accent == false).map(it => it.rgb)

#let style-document(doc) = {
  show: set-code-theme.with(flavor)
  show: zebraw.with()

  doc
}