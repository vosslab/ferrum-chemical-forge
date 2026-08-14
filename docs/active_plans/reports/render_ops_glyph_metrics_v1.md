# M12 render ops and glyph metrics

## Verdict

M12 is complete for the currently evidenced macOS arm64 Telex/PySide6 reference
boundary. Rust owns the declarative operation and glyph facts; Ferrum-Qt consumes
them without choosing a font, shaping text, or measuring advances. This is not
pixel or byte equivalence, cross-platform coverage, a timing gate, or M13 backend
work.

## Metric boundary

- Rust reads the hash-verified embedded Telex Regular bytes through `ttf-parser`
  design units and converts them to scene `f64` values with no extra rounding.
- Run and centered-plus bounds are true outline ink bounds. Atom-label clipping
  alone expands its outline union to include the durable atom origin, because a
  bond starts at that anchor.
- Glyph IDs and glyph origins are exact discrete facts. The V1 grammar remains
  unshaped; it does not promise fallback fonts, kerning, ligatures, bidi, or a
  general Unicode layout engine.
- The render DTO compares ordered, typed semantic fields: schema, provenance,
  target/order, operation variant, exact discrete values, and finite `f64` values
  carried through round-trip JSON formatting. It adds no decimal quantum and does
  not treat JSON punctuation as a rendering contract.

## One-time target evidence

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

A disposable current-wheel offscreen Qt proof consumed the Rust-issued iodine and
plus glyph IDs and origins using `QRawFont.pathForGlyph`. It found nonempty
outlines for both supplied glyphs and installed the projection. That proves the
current PyO3-to-Qt consumer path, not a permanent GUI, screenshot, pixel, byte, or
timing gate.

## Permanent evidence and limits

Semantic Cargo render/API tests and focused Qt projection tests remain the
permanent evidence. The QRawFont comparison and current-wheel proof are
implementation receipts only. M20 must refresh equivalent evidence for each added
release target. Cairo raster/PDF and `xot` SVG remain M13 work.
