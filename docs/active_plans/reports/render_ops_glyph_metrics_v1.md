# Render operations and glyph metrics

## Verdict

Rust owns atom-label metrics, attachment, ink exclusion, and declarative drawing
operations. The current renderer places the core structural-element ink center at
the atom origin, preserves complete visible label ink for bond exclusion, and
emits plans that Qt consumes without choosing an anchor, shaping replacement text,
or measuring alternative advances. This report records the current contract; it
does not close a parity milestone, cross-platform validation, or human visual
acceptance.

## Current metric and attachment boundary

- Rust reads the hash-verified embedded Telex Regular bytes through `ttf-parser`
  design units and converts them to scene `f64` values with no extra rounding.
- `GlyphBounds` is the exact finite visible-ink rectangle for positioned runs.
  It is not widened merely to contain an atom origin.
- The typed element run determines unversioned `AtomLabelAttachmentGeometry`.
  Its exact ink center must equal local `(0, 0)`; the complete element,
  hydrogen, isotope, and charge union remains the clipping exclusion geometry.
- One shared y-down script-baseline calculation puts subscripts below and
  superscripts above the centered core element. Decorations therefore cannot
  move a bond attachment point.
- `BondInkClearance` is a required positive input. The clipped envelope expands
  complete label ink by the requested gap and the final painted footprint:
  half-width for ordinary and dashed lines, full base width for bold, amplitude
  plus stroke for waves, and the relevant wedge or Haworth radius and axial
  overhang. An empty remaining span is a typed render refusal, not an
  overlapping line.
- Glyph IDs and glyph origins are exact discrete facts. The V1 grammar remains
  unshaped; it does not promise fallback fonts, kerning, ligatures, bidi, or a
  general Unicode layout engine.
- The render DTO compares ordered, typed semantic fields: schema, provenance,
  target/order, operation variant, exact discrete values, and finite `f64` values
  carried through round-trip JSON formatting. It adds no decimal quantum and does
  not treat JSON punctuation as a rendering contract.

## Superseded metric evidence

`devel/measure_m12_font_metrics.py` opened the exact embedded Telex bytes in
Qt `QRawFont` 6.11.1 at 1000 px with `PreferNoHinting`, on macOS 26.6.1 arm64
under CPython 3.12.14. It recorded the asset and Qt module/binary hashes, then
compared the closed corpus `C`, `Cl`, `Br`, `H2`, `NH3+`, and `I`.

- Glyph ID sequences agreed exactly.
- The largest per-run `f64` representation observation was about
  `1.78e-15` scene units.
- Qt's separately recorded baseline descent/height observation was `0.0001875`
  scene units. It remains an observation, not a tolerance, CI threshold, or
  portability promise.

Those measurements established the font asset and former Rust-to-Qt glyph replay
boundary, but they predate the core-centered atom-label contract. They are not
evidence that final bond ink clears the current labels. That former evidence gap
is now closed by `RenderObservationV2` and `RenderPlanV4`: their typed atom payload
publishes the core run, exact core/full Telex ink bounds, and positive bond-ink
clearance, while their typed bond payload publishes the final operations that Qt
must replay.

## Current automated evidence and remaining acceptance

The renderer's deterministic Telex corpus now checks exact core centering,
full-ink containment, y-down script placement, and style-aware final-footprint
clearance, including the Haworth front axial extension. The focused local render
lane passes 158 tests with formatting and strict Clippy clean. A shared 12-row
corpus has both a Rust consumer and an installed Rust-to-Qt E2E consumer; each
requires final bond ink to remain disjoint from full label ink expanded by the
issued clearance. The post-change `./all_test.sh` gate passes 8,297 hygiene tests,
all registered CLI/Qt E2Es, 299 installed PyO3 tests, and 437 Qt tests.

The rebuilt 13-scene screenshot set has independent image-review acceptance.
Human real-window/accessibility acceptance, remote CI, release artifacts,
cross-platform font evidence, and full parity remain open. SVG, PDF, and PNG
consume the same Rust plan, but their automated operation checks are not a
substitute for visual acceptance.
