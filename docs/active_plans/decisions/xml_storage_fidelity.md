# XML storage fidelity decision

## Decision

M6 stores CDML as an opaque `xot` XML tree. The storage API is deliberately limited to
parse and structural serialization; it assigns no CDML type, identifier, reference, wire, or
frontend meaning. This establishes the preservation substrate for the M7 and M8 work in
[ferrum-plan-v3.md](../ferrum-plan-v3.md).

Raw source-slice retention is not adopted. The current corpus did not lose structural meaning
when parsed and serialized through `xot`.

## Experiment

On 2026-08-03, a one-time Rust probe parsed, serialized, reparsed, and structurally compared all
three current M1d corpus documents:

- [authored_document_forms.cdml](../../../tests/e2e/corpus/authored_document_forms.cdml): OK,
  3,994 source bytes to 3,953 serialized bytes.
- [legacy_groups_template.cdml](../../../tests/e2e/corpus/legacy_groups_template.cdml): OK,
  1,095 source bytes to 1,054 serialized bytes.
- [opaque_namespace_preservation.cdml](../../../tests/e2e/corpus/opaque_namespace_preservation.cdml): OK,
  893 source bytes to 851 serialized bytes.

The comparison checked document and child order; expanded namespace URI plus local name for
elements and attributes; attribute value sets; text nodes, including mixed-content tails;
comments; and processing-instruction target and data. It intentionally ignored lexical prefix
spelling and attribute order.

## Achieved fidelity

The experiment establishes structural retention for the current three-fixture corpus:

- Unrecognized elements and nested foreign-namespace content survive.
- Namespace identities survive even where multiple prefixes name the same URI.
- QName-like character data remains literal text rather than being interpreted as a QName.
- Mixed text, child-element order, and tail text survive.
- Comments and processing instructions survive, including the pre-root instruction in the
  namespace-preservation fixture.

`xot` preserves these XML tree values, but structural serialization is intentionally not a
byte-preserving codec. The observed normalizations are removal of the XML declaration under the
default serializer and removal of the newline between top-level comment/processing-instruction
nodes and the document element. CDATA boundaries, entity spelling, original namespace prefixes,
attribute order, quote style, and original whitespace spelling are not preservation promises.

## Security boundary

The storage entry point uses `xot` XML 1.0 parsing. `xot` does not support DTDs, so DTD input is
rejected; this prevents the API from expanding external entities or resolving network resources.
Callers provide decoded text, so byte encoding detection remains outside this M6 boundary.

## M6 conclusion

The accepted M1d inventory package bounds the current corpus, and this experiment meets M6's
structural opaque-retention exit criterion. M7 adds stable IDs, order, and reference behavior;
M8 assigns CDML types; and M10 wires the full-corpus preservation gate. Those later dependent
milestones are not M6 blockers. Revisit raw source-slice retention before M8 only if a real
fixture demonstrates lost semantic meaning under this structural contract.
