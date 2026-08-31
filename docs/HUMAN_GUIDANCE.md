# Human guidance

<!-- VENDORED HEADER: START -->
Record the durable guidance Neil Voss states, or approves for preservation here, in his own words:
first person or close paraphrase, one to three lines per bullet. Material he supplies as a source
may inform [DESIGN_DECISIONS.md](DESIGN_DECISIONS.md) once it is settled, and an entry of uncertain
origin belongs there too. Rules: [REPO_STYLE.md](REPO_STYLE.md).
[PROPAGATED HEADER - ENTRIES BELOW ARE YOURS]
<!-- VENDORED HEADER: END -->

## Direct preferences

- Atkinson HyperLegible https://www.brailleinstitute.org/freefont/ is my
  favorite for written text and mononoki font https://madmalik.github.io/mononoki/ for monospace
- Keep it simple: avoid speculative machinery when a focused durable design will do.
- Plans must continue to completion while I am unavailable; do not make my interaction a gate.
- Run `all_test.sh` periodically to detect repository drift and overly complicated permanent
  tests. Use focused checks during active edits; treat full runs as a design-quality signal, not
  merely a pass/fail gate.
- Fix the design that permits a problem rather than masking it with incidental fallbacks. Prefer
  adaptable ownership boundaries and durable improvements when their value justifies the cost;
  stop at a good, correct system when further refinement would not materially improve it.
- Target a 16:10 desktop aspect for Ferrum screenshots. Measure the complete outer application
  window, including the ribbon/menu and status bar; the canvas is not the aspect-ratio boundary.
- I noticed the letters are poorly aligned with the bond lines, we spend significant time and make
  crazy tests for OASA to perfect this.
