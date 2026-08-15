# Ferrum cookbook

These scenarios combine the current protocol and desktop workflows. They do not extend
Ferrum's pre-alpha feature set.

## Review a CDML record

Use this sequence when a collaborator supplies a CDML file and you need to assess its
structure and native rendering before deciding whether to save a structural re-emission.
It leaves the input unchanged until you deliberately use a Qt save action.

1. Inspect the generated protocol schema.

   ```bash
   ferrum protocol schema
   ```

2. Create one `ferrum-operation-request-v1` JSON request with the supplied CDML text
   and run it.

   ```bash
   ferrum protocol run request.json
   ```

3. Open the same document in the ordinary Ferrum-Qt window.

   ```bash
   ferrum-qt supplied.cdml
   ```

Ferrum-Qt is the sole desktop product window. It opens the supplied uncompressed `.cdml`
through Rust, and its native document actions include ordinary editing, Undo/Redo, Save,
Save As, reopening, and complete-document artifact export. A successful protocol rewrite
returns structurally preserved CDML, not byte-for-byte output. Use Save As when you want to
inspect the resulting structural re-emission separately from the supplied source.

File Open also accepts a decoded local `.svg` only when it contains one canonical embedded
CDML payload; the SVG wrapper is discarded. Ferrum refuses `.cdxml`, `.cml`, `.cdsvg`,
`.svgz`, and compressed CDML names without changing the active document. Use the source
application or a converter to produce an uncompressed supported `.cdml` drawing.

Use `File -> Recovery Export CDML...` only for a recovery copy of the current CDML. It does
not replace Save or Save As and does not convert formats. Use `File -> Export...` to publish
the complete supported document as SVG, PDF, or transparent PNG; this is not CD-SVG export or
a wrapper round-trip.

For command arguments and failure behavior, see [USAGE.md](USAGE.md). For Ferrum-Qt
installation and platform limits, see [INSTALL.md](INSTALL.md).
