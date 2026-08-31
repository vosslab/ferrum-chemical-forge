# M13 renderer-neutral backend V1

## Status

M13 is complete. `ferrum-render` lowers every supported whole-document plan once through
its private checked `DrawSinkV1` stream to owned, in-memory `xot` SVG, direct pure-Rust
PNG, and direct pure-Rust vector PDF. This is an independent Rust backend boundary, not
the existing Qt snapshot route.

## Accepted boundary

The backend preserves plan batch source order and batch-local paint order. It lowers
the closed V1 line, mask, ellipse, verified molecule-label text, and nine direct-root vector
kinds: Arrow, Polyline, Wavy, round-bracket, Rectangle, Square, Oval, Circle, and
Polygon. Vector paths retain issued commands and normalized shapes retain issued
bounds. Their V1 paint profile explicitly uses butt caps, miter joins with a fixed 4.0
bevel fallback, and even-odd fill; no sink inherits an appearance default. Degenerate
box shapes are rejected at document projection rather than becoming renderer-specific
hairlines or points. Text becomes outline paths from the bundled digest-verified molecule-label
face, using the explicit TrueType-up to Ferrum-scene-down Y conversion. Atom batches
use their supplied anchor; scene batches gain no invented transform.

The whole-page plan holds one exact revision/digest provenance, a finite physical page,
and one source-order sequence of paintable roots or named exclusions. The API composer
derives the page from the authoritative paper fact in one final observation; molecule,
fixed-plus, and Text roots retain their issued durable-or-local identity, anchor,
bounds, optional background, and existing molecule-label layout. Roots without a supported
operation stay as named profile, rejected-projection, or `not_yet_lowered` exclusions.
Invalid presentation suppression returns a typed no-plan result before a partial page
can be formed.

Each whole-document sink returns the same external receipt, containing exact
revision/digest provenance, the full page rectangle, and exclusions in source order.
Those Ferrum facts are not embedded as output artifact metadata. SVG may retain only
serializer-local rendering diagnostics. Its only inputs are typed plans and requests; it
reads or writes no files, selects no system font, shapes no text, and exposes no CLI,
PyO3, Qt, CD-SVG, or RDKit route. Invalid viewport, non-finite conversion, finite-plan
outline arithmetic overflow, and sink failure return typed failures before an owned
artifact and receipt are published.

## Evidence and limits

Focused offline Cargo tests cover common stream ordering, page provenance, named
exclusions, typed suppression, geometry failures, explicit paint, and the resource
boundaries. SVG is parsed structurally with `xot`; these are semantic/structural tests,
not XML-byte, pixel, timing, GUI, file, network, or golden-artifact tests. The independent
combined review passed 70 `ferrum-render` tests, fmt, clippy, docs, and locked-offline
macOS arm64 checking.

PNG requires nonzero caller-owned dimensions, a required transparent or RGB background,
and explicit raw-RGBA and encoded-artifact limits. Raw admission occurs before pixmap
allocation, and a bounded writer refuses an overflowing whole write. PDF requires
caller-selected structural limits for plan traversal, lowered path commands, and
exclusion-report UTF-8 before it creates a `pdf-writer` document, and a nonzero
post-build artifact limit. An over-cap PDF is withheld; that publication limit does not
claim to bound `pdf-writer` allocation or process memory. The current locked pure-Rust
surface is `tiny-skia`/`tiny-skia-path` (BSD-3-Clause), `png`, and `pdf-writer` (MIT OR
Apache-2.0); it has no build scripts or native-graphics linkage. Their provenance and
the internal `unsafe` boundary are recorded in [docs/PROVENANCE.md](../../PROVENANCE.md).

A disposable current-source integration proof composed one A4 page with six supported
recognizable roots and no exclusions. It produced an 800 x 1131 opaque PNG and a
one-page PDF, decoded the PNG, compared SVG/PNG/PDF receipts, accepted the PDF with
`qpdf` and `pdfinfo`, and visually inspected a local PDF raster beside the PNG. This
evidence records semantic structure, dimensions, and recognizability; it does not create
a byte, pixel, timing, or perceptual threshold. Profile, rejected-projection, and
`not_yet_lowered` outcomes remain named exclusions for future source coverage. Cairo is
not an M13 obligation; a native graphics library needs a separate M20 packaging decision.
