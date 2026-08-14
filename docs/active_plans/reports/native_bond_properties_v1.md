# Native bond properties V1

## Outcome

The OASA-free native preview can edit one durably selected, normally depicted
bond through one Rust document operation. The closed patch carries order,
depiction style, center intent, line width, signed bond width, wedge width, and
color. Rust owns CDML mutation, validation, revision history, and persistence;
the shared Qt `BondDialog` is a visual-only source of intent.

The Rust and PyO3 contract retains nine closed CDML style values and signed
nonzero bond width. The current native dialog is narrower because the current
renderer is narrower: it exposes only normal single, double, and triple bonds,
positive widths it can represent exactly, and values that survive the widget's
one-decimal controls unchanged. Unsupported source facts fail visibly before
submission rather than being clamped, rounded, or silently reinterpreted.

## Ownership and security boundary

- PyO3 accepts an exact built-in tuple of exact frozen change values. The tuple
  is rejected before extraction when it exceeds the seven-field grammar.
- Rust validates the complete unique-field patch and durable direct-child bond
  target at the expected revision, applies it to a detached candidate, reparses
  the result, and appends at most one history revision.
- Rejected duplicate, malformed, stale, unknown-target, or unsupported-type
  requests retain the authoritative revision, digest, and history.
- Mutation preserves endpoints, durable identity, namespace meaning, source
  order, unrelated attributes, and opaque children.
- Qt consumes frozen projections only. It does not parse CDML, call RDKit, or
  own persistent bond state.

This operation adds no external parser, network, filesystem, or C ABI boundary.
Its bounded resource grammar is exactly seven property kinds, not an arbitrary
numeric ceiling. CDML and CD-SVG resource admission are handled separately by
the caller-owned XML budget preflight.

## Grounded verification

Focused Rust tests cover all fields, optional clearing, signed negative width,
type composition, normal no-op, undo/redo, alternate canonical namespace,
opaque preservation, invalid input, unknown targets, unsupported source types,
and stale revision. PyO3 installed-wheel tests cover exact frozen values,
closed enums, tuple/subclass/class rejection, the seven-field preflight,
projection facts, save/reopen, and no-mutation failures.

The focused direct-extension wheel has SHA-256
`559345a7398e3818ff2e3d5b7ae32e7dc7c9f46f2c61b8944fa792a7ca2159ff`.
Its OASA-free public native E2E applies all seven property kinds using a normal
double bond and positive lane width, retains durable selection through
undo/redo, saves and reopens the semantic facts and opaque content, and reports
a clean document. This wheel is a direct boundary artifact, not a native
chemistry release-closure receipt.

## Deliberate renderer limits

The native visual adapter rejects non-normal styles and negative bond width
because the current render plan cannot represent them faithfully. Direct Rust
and PyO3 tests still prove those persistent facts are retained. A negative
signed width now produces one durable target-owned `UnsupportedFeature` render
issue and no graphics batch; it is neither silently omitted nor converted to a
positive width. Normal double geometry still does not express every authored
center and sign combination. That remains an explicit renderer gap; Ferrum
does not invent a lane side or weaken the E2E to claim support.

Acceptance is semantic: durable facts, target identity, history, preservation,
visible rejection, and selection continuity. It has no pixel-equivalence,
byte-equivalence, or invented timing threshold.

## Boundary

This is accepted pre-milestone evidence. It does not complete M8a, M9, M12,
M16, or OASA removal. The legacy editor remains a separate OASA-backed route,
and broader bond-style rendering needs a separately reviewed closed render-op
slice before the native dialog may expose it.
