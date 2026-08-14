# Native atom properties V1

## Outcome

The standalone OASA-free `ferrum-qt --native` route can edit the authored
properties of one durably selected atom through one Rust document operation.
The shared Qt `AtomDialog` supplies controls only; it neither persists changes
nor receives mutable document objects.

The closed V1 property set is element, formal charge, valence, isotope,
multiplicity, atom-label visibility, hydrogen-label visibility, label font
size, and label colour. The PyO3 boundary accepts only exact frozen
`DocumentAtomPropertyChangeV1` values in an exact built-in tuple, then Rust
validates the complete patch before it can enter the document session. Input
longer than the closed nine-field grammar is rejected before value extraction.

## Ownership and semantics

- Qt resolves one selected durable atom ID and converts dialog intent once.
- Rust applies the patch to a detached CDML candidate, reparses it, and commits
  it through the normal revision and history authority.
- One accepted non-no-op patch creates one revision and one undo/redo entry.
- An unchanged dialog produces no changes. Clearing authored optional values
  removes the applicable CDML fact rather than storing a UI default.
- The patch retains unrelated attributes, opaque children, root siblings,
  source order, identities, and untouched atom/bond endpoints.
- Multiple direct canonical `font` children make a font edit ambiguous and are
  rejected before session mutation. A missing direct canonical font is created
  in the atom's namespace only when a font property is changed.
- Values the retained dialog cannot represent are rejected visibly. The route
  does not clamp or remap them to a nearby value.

## Security boundary

This operation introduces no new external-input parser, FFI library load, or
package boundary. It receives a closed in-process PyO3 value tuple; the Rust
document layer owns validation and returns typed failure before mutation for a
wrong value, duplicate property, unknown atom ID, or ambiguous direct font.
The underlying CDML candidate remains subject to the existing hardened XML
policy: no DTD, external entities, entity resolution, network access, recovery
mode, or opt-in huge-tree mode. That policy does not yet impose a measured byte,
node, depth, attribute, or text budget on otherwise valid CDML. A separate shared
ingestion-boundary work package is adding explicit caller-supplied budget
preflight support; production limits will be selected only after representative
user documents are measured. Focused malformed-intent and adversarial namespace
tests exercise this mutation boundary.

Every later external-input, FFI, parser, or package work package must name its
trust boundary, validation owner, resource limits, typed malformed-input failure,
and adversarial tests when it is introduced. M22's security sweep confirms those
controls; it is not the first point at which security work occurs.

## Grounded verification

Focused Rust tests cover all nine fields, optional-fact clearing, no-op
behavior, invalid/duplicate intent, unknown IDs, opaque retention, undo/redo,
and canonical namespace handling. The PyO3 installed-wheel tests cover frozen
values, exact tuple input, rejection at the public boundary, atomic
publication, save/reopen, and the expanded authored atom projection. Focused
native tests drive both the live tab and the real window action, including
selection restoration after a projection replacement.

These are semantic gates: durable facts, session history, preservation, and
visible error behavior. They do not require byte-equivalent CDML,
pixel-equivalent rendering, or an invented timing threshold.

## Boundary

This is accepted pre-milestone evidence for native document-session adoption.
It does not close M8a, M9, M16, or M22: the normal Ferrum-Qt editor remains a
legacy route, full persistent-object coverage is unproven, and OASA has not
been removed from the production tree.

The final fresh-wheel end-to-end receipt used the hardened nine-change request
preflight and a direct CPython 3.12 extension wheel. Exact built-in tuple
enforcement landed after that wheel was built and has its own no-mutation
binding regression; a later aggregate wheel receipt must cover it alongside
the next native slice. The tested wheel's SHA-256 is
`85354da03c5dffdcaacb06127fae72e61171cd11e2c2fd99db93093dc3645275` and its
size is 963,930 bytes. The OASA-free public native route changed all nine facts
in one revision, retained durable selection through undo/redo, saved and
reopened the semantic facts and opaque extension, and reported a clean document.
This is a focused semantic receipt, not a full release-wheel closure claim.
