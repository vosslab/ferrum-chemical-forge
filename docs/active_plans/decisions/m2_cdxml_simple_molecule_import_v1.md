# M2 CDXML simple-molecule import

## Decision

Ferrum supports one bounded, Rust-owned **CDXML simple-molecule import V1**
profile. It accepts a small, explicit XML CDXML grammar, imports each admitted
fragment as one record into a new Ferrum document, and reports every admitted
loss. It is an input-only interchange capability: CDXML has no Ferrum output
encoder, save baseline, or round-trip fidelity claim.

This decision records delivered implementation and its present evidence. It
does not close M2, full Rust/OASA/BKChem parity, or the manual 16:10 and
accessibility evidence gates.

## Source basis

| Source | Status | What it establishes |
| --- | --- | --- |
| [CDXML.dtd](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) | Primary vendor grammar evidence | Unprefixed names, tree shape, attributes, and display tokens. |
| [ChemDraw Suite 23.0 release notes](https://revvitysignals.com/sites/default/files/2024-02/rs-release-notes-DT-23.0-chemdraw.pdf) | Primary current-producer evidence | ChemDraw 23.0 writes the current Revvity DTD URL in CDXML DOCTYPE declarations. |
| [CDX SDK simple example](https://chemapps.stolaf.edu/iupac/cdx/sdk/IntroExampleSimple.htm) | Corroborating historical example | Ordinary coordinate and omitted-order producer spelling. |

The DTD and release notes are facts about the vendor grammar and current
producer prolog. The profile decisions below are Ferrum inferences: they make
that broad grammar into a safe, chemistry-only import contract. The historical
example corroborates lexical patterns; it does not define current vendor policy.

## Accepted profile

The accepted document shape is:

```text
document := declaration? vendor_doctype? CDXML
vendor_doctype := <!DOCTYPE CDXML SYSTEM
  "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd">
CDXML := page*
page := fragment*
fragment := n* b*
```

The molecule tree uses literal, unprefixed `CDXML`, `page`, `fragment`, `n`,
`b`, `t`, and `s` names. A default namespace or a prefix is outside V1. The
profile admits a closed DTD-derived table of root/page view metadata and
well-formed `fonttable`/`font` and `colortable`/`color` metadata when needed by
ordinary producer files. Metadata does not become Ferrum document state.

Each direct admitted `fragment` becomes one imported record, in source order.
An import creates one new document transaction for all records; it does not
merge fragments, infer reaction roles, or retain CDXML source identifiers as
Ferrum durable IDs.

### Atoms and bonds

- A node with no `Element` and no direct `t > s` element label is carbon.
- `Element` is an unsigned decimal atomic number. A direct `t > s` label may
  state an element symbol; when both are present they must agree.
- Coordinates use the admitted finite two-number `p` spelling.
- An omitted `Order` and `Order="1"` are normal single bonds; `Order="2"` and
  `Order="3"` are normal double and triple bonds.
- An absent or `Solid` display is normal bond depiction. `WedgeBegin` and
  `WedgedHashBegin` map to Ferrum's authored begin-directed solid and hashed
  single-bond depictions.

This is an inference from vendor token names plus Ferrum's existing
begin-directed depiction model. End-directed wedges, wavy/bold/dashed/hash
display variants, unsupported orders, aromatic forms, and conflicting or
non-single directional facts refuse instead of being normalized into a nearby
meaning.

### C2 atomic charge and isotope

Optional `Charge` and `Isotope` node attributes are admitted as bounded atom
facts. The current vendor DTD establishes their names and optionality.
Historical vendor CDX property material establishes non-fractional signed
eight-bit charge and a sixteen-bit isotope field where zero means natural
abundance. Ferrum owns the XML canonicalization below; the vendor material
does not establish every whitespace or leading-zero spelling.

| Attribute | Accepted ASCII token | Numeric range | Ferrum atom value |
| --- | --- | ---: | --- |
| `Charge` | `0` or `-?[1-9][0-9]*` | `-128..=127` | `0` and absence become no formal charge; otherwise `Some(i32)` |
| `Isotope` | `0` or `[1-9][0-9]*` | `0..=32767` | `0` and absence become natural abundance; otherwise `Some(u16)` |

The grammar admits decimal ASCII only. It rejects padding, plus signs, leading
zeros other than `0`, negative zero, decimal points, exponents, embedded
whitespace, and Unicode numerals. `Isotope` is admitted only where C1 already
lowers the node to an elemental atom. Explicit neutral charge and
natural-abundance isotope intentionally normalize to the same native value as
absence: CDXML V1 imports chemistry meaning, not raw source spelling or
provenance.

## Prolog and resource boundary

The parser accepts no DOCTYPE or exactly the `vendor_doctype` lexical marker
above. It never fetches, loads, validates against, or resolves the DTD. It
refuses `PUBLIC` identifiers, another system identifier, internal subsets,
entity declarations/references, and any declaration after the root. This keeps
current producer spelling compatible with offline parsing without treating the
remote DTD as executable input.

Only resource limits that profile-valid input can reach first are public:

| Limit | Maximum |
| --- | ---: |
| Source bytes | 1,048,576 |
| XML start elements | 50,000 |
| Decoded attribute value bytes | 1,024 |
| Fragment/node/bond identifier bytes | 128 |
| Imported records | 1,024 |
| Atoms per record | 10,000 |
| Bonds per record | 20,000 |

The chemistry-owned source-byte cap, closed grammar, and element cap bound XML nesting,
aggregate lexical content, and aggregate object count. They replace misleading
separate limits for those unreachable categories.

## Loss and refusal

Ferrum records `lexical_syntax` for an admitted XML declaration and
`document_view_metadata` for admitted document/view metadata. These declared import losses
use canonical category order: `lexical_syntax`, then `document_view_metadata`. Ferrum imports
molecule semantics rather than a ChemDraw presentation document.

Chemistry-bearing facts outside the profile refuse before document mutation.
Examples include radical, hydrogen-count, non-element node types,
query/list/alternative-group forms, external connections, attachment models,
unsupported text-as-chemistry forms, unsupported bond order/display, and
malformed or over-budget XML. A present malformed or out-of-range `Charge` or
`Isotope` is the typed `InvalidScalar` refusal; unknown node attributes remain
`AttributeUnsupported`. The public adapter maps closed decoder reasons to
typed, redacted interchange refusals and reports no conversion outcome on
refusal.

## Capability and clients

`ferrum formats` lists CDXML as canonical `.cdxml`, runtime-free, input-only, and eligible
only for a new-document Open operation; the machine-readable catalog represents its output as
`null`. `ferrum open --format cdxml --output result.cdml` and the generic document interchange route use the
same Rust registry descriptor. PyO3 issues an opaque registry route handle and
redeems it in Rust, so Python and Qt do not choose a parser by suffix or string.

`ferrum convert` refuses CDXML before source read because its descriptor does not advertise
chemistry-conversion eligibility. Its recovery guidance directs the caller to `ferrum open`.

Qt File/Open discovers CDXML from that descriptor and passes it to the existing
detached interchange worker. A successful receipt has typed `CDXML` provenance,
is presented as an imported ChemDraw XML document, and has no source save
baseline. Later save uses Ferrum's CDML document format. A refused import keeps
the active document unchanged and uses the descriptor-neutral open-document
recovery.

## Evidence and remaining work

Permanent evidence is deliberately semantic and offline:

- decoder grammar, loss/refusal, and exact/one-over reachable resource limits;
- document record order, atomic admission, durable-ID reallocation, and redaction;
- public `formats` input-only projection and PyO3 opaque-route behavior;
- built CLI success/refusal E2E with inline temporary input; and
- Qt local File/Open success, nonmutation refusal, and provenance boundaries.

C2 extends existing permanent evidence without a fixture catalog: decoder
tests prove scalar boundaries, zero/absence equivalence, and the closed
`InvalidScalar` refusal table; the public CLI E2E proves nonzero facts survive
into CDML and malformed scalar input publishes no output. C2 adds no parser,
API/protocol field, PyO3 or Qt route, CDXML encoder, source-provenance store,
or save-format change.

One-time/release evidence remains separate: a real macOS 16:10 outer-window
screenshot, keyboard/accessibility walkthrough, fresh repository-wide
`all_test.sh`, and independent multi-reviewer audit. These validate the
integrated local product; they are not parser fixtures, timing gates, or
pixel-equivalence tests.

Final bounded receipts are: post-audit `./build.sh` exited 0; the registered
`tests/e2e/run_all.sh` exited 0, including CDXML; staged Python bindings passed
281 tests; Qt passed 238 tests with one intentional skip; focused chemistry and
API libraries passed 124 and 117 tests; and `cargo check --workspace` passed.

`./all_test.sh` is not aggregate-green. It recorded 7,759 passes and then
stopped at five Markdown-link failures. Each canonical link targets this
present decision artifact, which is absent only from the tracked-file catalog;
the later aggregate phases therefore did not run through `all_test.sh`. Those
later phases were run directly and passed as listed above. Real macOS 16:10
outer-window and keyboard/accessibility evidence remains separate release
evidence. The still-open evidence above prevents this bounded decision from
closing full M2 or parity.

Focused C2 receipts are: `cargo fmt -p ferrum-chemistry` exited 0; the initial
CDXML-focused chemistry target passed 17 tests; the built CLI E2E exited 0;
the completed scalar-contract target passed 3 tests; and the full chemistry
library target passed 127 tests. These do not supersede the aggregate
`all_test.sh` limitation or prove full M2, parity, or GUI evidence.

## Explicit next extensions

- CDX binary decoding.
- Reaction, arrow, annotation, page-layout, and ChemDraw presentation import.
- Namespace variants and end-directed wedge semantics, after corpus evidence.
- Additional atom, bond, query, and attachment models, each with an owned loss
  or refusal contract before implementation.
