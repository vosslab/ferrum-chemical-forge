# Ferrum cookbook

These scenarios combine the current commands into bounded, repeatable workflows. They do
not extend Ferrum's pre-alpha feature set.

## Review a CDML record

Use this sequence when a collaborator supplies a CDML file and you need to assess its
structure and native rendering before deciding whether to save a structural re-emission.
It leaves the input unchanged until you deliberately use a Qt save action.

1. Inspect the document and its declared identities.

   ```bash
   ferrum cdml inspect supplied.cdml
   ```

2. Validate its retained structure and require current typed molecule facts.

   ```bash
   ferrum cdml validate supplied.cdml --typed
   ```

3. Check the structural rewrite contract without writing an output file.

   ```bash
   ferrum cdml rewrite supplied.cdml --check
   ```

4. Produce the Rust render observation that the native Qt projection consumes.

   ```bash
   ferrum cdml render-observation supplied.cdml
   ```

5. Open the same document in the OASA-free native bounded editor.

   ```bash
   ferrum-qt --native supplied.cdml
   ```

The native route can open, render, change an atom element, add one free-standing atom,
undo/redo, save, save as, reopen, and close CDML documents. It does not create bonds or
provide an export workflow. A successful `rewrite --check` establishes structural
preservation, including retained opaque content; it does not promise byte-for-byte output.
Use Save As when you want to inspect the resulting structural re-emission separately from
the supplied source.

For command arguments and failure behavior, see [USAGE.md](USAGE.md). For native-route
installation and platform limits, see [INSTALL.md](INSTALL.md).
