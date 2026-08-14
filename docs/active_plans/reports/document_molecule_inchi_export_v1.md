# Document molecule InChI export V1

## Boundary

The standalone native editor can export one durable direct-root molecule as
Standard or Fixed-H InChI. One immutable document observation supplies the source
revision, digest, molecule identity, and complete projected graph. Rust validates
that graph before the packaged ABI-4 RDKit adapter is located or loaded. Qt does not
parse CDML, reconstruct chemistry, or mutate the document for this operation.

Unsupported graph facts, drawing-only bond styles, an unknown molecule identity, a
stale result, or native chemistry failure remain typed failures. The worker can
invalidate delivery, but does not claim to interrupt an in-flight native call. A
successful still-current identifier is copied to the clipboard.

## Durable verification

- Rust tests prove complete graph conversion, closed mode routing, exact source
  provenance, and rejection before the chemistry engine is invoked.
- The installed-extension test proves an unsupported document graph fails before
  packaged-adapter loading and leaves the session snapshot unchanged.
- Existing native coordinate and public-window behavior tests protect the shared
  worker lifecycle after its mixin extraction.

No permanent Qt test mocks the private worker or adapter. A disposable direct-wheel
offscreen exercise opened methane in the public native window, exported
`InChI=1S/CH4/h1H4`, confirmed unchanged revision and digest, and observed no OASA
import. That rebuild receipt is one-time evidence, not a byte, pixel, timing, or
network acceptance gate.

## Remaining scope

This is partial M16 evidence. The ordinary `MainWindow` codec registry, other
document-molecule export formats, and full FQ-005 closure remain legacy or open.
