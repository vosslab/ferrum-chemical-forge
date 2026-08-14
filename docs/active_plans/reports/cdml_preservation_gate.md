# CDML preservation gate

## Result

The M10 preservation gate passes over every committed CDML corpus document. Ferrum's
public CLI parsed each source, serialized the retained document, reparsed the result,
and accepted the rewrite only after the original and rewritten structural observations
matched.

The accepted local command was:

```bash
source source_me.sh && python3 -B tests/e2e/e2e_cdml_preservation.py \
  --ferrum packages/ferrum-rust/target/debug/ferrum
```

It emitted `ferrum-cdml-preservation-corpus-v1` with status `preserved` for all three
current corpus documents. The runner discovers `tests/e2e/corpus/*.cdml` rather than
freezing a filename or document-count inventory, so later corpus additions enter the
same gate automatically.

## Comparison boundary

This is structural preservation, not byte equivalence. The Rust comparison permits
serializer-normalized prefixes and attribute order. It compares parsed node kinds,
expanded element and attribute names, literal values, namespace URI context, and
ordered children. Its typed observation separately compares persistent identities,
direct-root source order, typed record classes, diagnostics, and opaque-child counts.

The gate is backend-only. Qt does not reconstruct or reinterpret persistent content.
It invokes the public `ferrum cdml rewrite --check` contract, writes no document, makes
no network request, and uses no historical OASA process. Atomic file replacement is
covered separately by the CLI publication tests.

## Permanent test decision

`tests/e2e/e2e_cdml_preservation.py` is retained as a slow-lane E2E rather than regular
pytest. It protects user documents through the public executable and has no mocks,
private wiring, timing threshold, raster comparison, or platform-specific fixture.
The JSON printed by an individual run is a one-time receipt and is not checked in as
a golden file.

## Known evidence limits

The coverage inventory still records unavailable real-user, future-version, and
CD-SVG evidence explicitly. Those absences are evidence limitations, not fabricated
passing fixtures and not reasons to block the bounded inventory forever. New consented
documents can extend the corpus without changing this gate. CD-SVG wrapper admission
and embedded-payload preservation remain covered by their separate parser boundary.
