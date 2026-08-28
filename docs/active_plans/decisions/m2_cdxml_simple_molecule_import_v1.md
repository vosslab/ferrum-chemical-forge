# M2 CDXML simple-molecule import

## Decision

Ferrum defines one pre-production, Rust-owned **CDXML simple-molecule import V1**
profile. It accepts a deliberately small CDXML grammar, imports every admitted
fragment as one record into one new Ferrum document, and records only declared
losses. It is input-only: CDML remains the local save format, and the profile has
no CDXML encoder, source-save baseline, or round-trip-fidelity claim.

The delivered profile includes one closed presentation slice:
`Display="Wavy"`, `Display="Bold"`, and `Display="Dash"` are accepted only on a
single, non-directed bond. They become durable CDML `s1`, `b1`, and `d1`,
respectively, and must render cleanly before the new document is published.
Rendering is admission, not optional client polish.

The local implementation and focused acceptance evidence below establish this
bounded slice. They do not close M2 or full parity, and they do not replace final
post-change aggregate, CI, release, screenshot, or human acceptance evidence.

## Source basis

| Source | Status | What it establishes |
| --- | --- | --- |
| [CDXML.dtd](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) | Primary vendor grammar evidence | Unprefixed names, tree shape, attributes, and display tokens. |
| [ChemDraw Suite 23.0 release notes](https://revvitysignals.com/sites/default/files/2024-02/rs-release-notes-DT-23.0-chemdraw.pdf) | Primary current-producer evidence | ChemDraw 23.0 writes the current Revvity DTD URL in CDXML DOCTYPE declarations. |
| [CDX SDK simple example](https://chemapps.stolaf.edu/iupac/cdx/sdk/IntroExampleSimple.htm) | Corroborating historical example | Ordinary coordinate and omitted-order producer spelling. |

The vendor sources establish grammar facts. Ferrum owns the bounded semantic,
resource, loss, and presentation contracts below.

## Accepted profile

```text
document := declaration? vendor_doctype? CDXML
vendor_doctype := <!DOCTYPE CDXML SYSTEM
  "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd">
CDXML := page*
page := fragment*
fragment := n* b*
```

The molecule tree uses literal, unprefixed `CDXML`, `page`, `fragment`, `n`, `b`,
`t`, and `s` names. A default namespace or a prefix is outside this profile.
Closed root/page view metadata and well-formed `fonttable`/`font` and
`colortable`/`color` metadata may be admitted when ordinary producer files require
them, but never become Ferrum document state.

Each direct admitted `fragment` becomes one imported record in source order. One
import creates one new-document transaction for all records. It does not merge
fragments, infer reaction roles, or retain CDXML source identifiers as Ferrum IDs.

### Atoms and bonds

- A node with no `Element` and no direct `t > s` element label is carbon.
- `Element` is an unsigned decimal atomic number. A direct `t > s` label may state
  an element symbol; when both are present they must agree.
- Coordinates use the admitted finite two-number `p` spelling.
- Omitted `Order` and `Order="1"` are normal single bonds; `Order="2"` and
  `Order="3"` are normal double and triple bonds.
- Absent or `Solid` display is normal depiction. `WedgeBegin` and
  `WedgedHashBegin` are authored begin-directed solid and hashed single bonds.
- `Wavy`, `Bold`, and `Dash` are accepted only with omitted `Order` or `Order="1"`
  and no directional display. They become fixed-single document presentations
  `Wavy`, `Bold`, and `Dashed`, serialized as `s1`, `b1`, and `d1`.

All other `Display` tokens, end-directed wedges, hash variants, aromatic forms,
unsupported orders, non-single presentation/order pairs, and conflicting directional
facts refuse. Ferrum does not normalize them into nearby chemistry or depiction.

### Source carrier and document conversion

`InterchangeRecordV1` stays format-neutral chemistry: graph, title, and properties
only. It must not receive a CDXML-driven presentation vector or a bond-index side
invariant that every CML, SDF, SMILES, and future codec must construct. `MolBond`
also remains chemistry/stereochemistry, not a source-view display container.

`ferrum-chemistry` owns the source-specific validated carrier
`CdxmlDecodedRecordV1`. Its private fields pair the graph-bearing
`InterchangeRecordV1` with presentation facts in graph-bond source order. Its
constructor validates exact vector cardinality and permits a presentation only on a
single, non-directed bond. `None` is the sole ordinary-presentation state.

Parser state is an unversioned closed `ParsedBondDepiction`:
`Ordinary`, `Stereo(BondDirection)`, or
`Presentation(CdxmlBondPresentationV1)`. One CDXML `Display` attribute therefore
cannot become both stereochemistry and presentation:

```text
none | Solid                 => Ordinary
WedgeBegin                   => Stereo(BeginWedge)
WedgedHashBegin              => Stereo(BeginDash)
Wavy | Bold | Dash           => Presentation(...), only Order absent or "1"
all other Display tokens     => UnrepresentedSemanticFact
recognized presentation with non-single Order => InvalidScalar
```

`ferrum-document` owns the only source-to-document conversion. Its private,
unversioned `cdxml_record_insertion` adapter reuses generic preparation for graph
validation, aromaticity, coordinates, placement, reports, metadata, and row layout.
It overlays the validated presentation on the detached insertion by the same bond
index before durable IDs are allocated or CDML is mutated. It repeats cardinality,
single-order, and no-direction validation and returns a concrete redacted error for
an internal-contract breach.

`DocumentBondPresentationV1` is the sole durable document representation. Its
fixed-single variants own the only CDML conversion: `Wavy -> s1`, `Bold -> b1`, and
`Dashed -> d1`. The insertion constructor derives `Single` from every fixed-single
variant, making `s2`, `b2`, and `d2` unrepresentable. Typed bond properties and
CDML parsing first validate into this same closed type.

`V1` is retained only for durable serialized or cross-crate contracts:
`CdxmlDecodedRecordV1`, `CdxmlBondPresentationV1`, and
`DocumentBondPresentationV1`. Parser state, adapter functions, correspondence
checks, and renderer geometry remain private and unversioned.

### Charge and isotope

Optional `Charge` and `Isotope` node attributes are bounded atom facts. Ferrum owns
the XML canonicalization.

| Attribute | Accepted ASCII token | Numeric range | Ferrum atom value |
| --- | --- | ---: | --- |
| `Charge` | `0` or `-?[1-9][0-9]*` | `-128..=127` | `0` and absence become no formal charge; otherwise `Some(i32)` |
| `Isotope` | `0` or `[1-9][0-9]*` | `0..=32767` | `0` and absence become natural abundance; otherwise `Some(u16)` |

The grammar rejects padding, plus signs, leading zeros other than `0`, negative
zero, decimal points, exponents, embedded whitespace, and Unicode numerals.

## Renderer admission

`ferrum-render` owns every styled-bond primitive. It consumes native projected
style after complete visible-ink clipping with explicit bond clearance and final
painted-footprint reserve; it never reads raw CDXML, accepts
CDXML geometry, or delegates dash interpretation to Qt, SVG, PDF, or PNG. One
private, unversioned styled-axis lowerer constructs a finite clipped axis and emits
existing renderer-neutral `LineOp` and `PathOpV3` operations.

Styled bonds initially require two atom endpoints. Compact-group exterior bonds stay
normal single until they have their own correspondence contract.

| Style | Required renderer policy |
| --- | --- |
| Bold | One butt-capped clipped `LineOp`, semantic bond paint, width `2 * base_width`. Source `BoldWidth` is not retained. |
| Dashed | Explicit finite butt-capped `LineOp`s: period `6w`, dash `3w`, gap `3w`; whole dashes are symmetric after clipping; cap at 4096. No sink dash primitive or solid fallback. |
| Wavy | One finite round-capped cubic `PathOpV3`: wavelength `12w`, amplitude `min(2w, span/6)`, ties-even whole-wave count capped at 4096, and exact clipped endpoints. |

All primitives retain resolved `RenderPaintV3`, source paint order, ordinary display
layer, and strictly increasing local z. Non-finite arithmetic, coincident anchors,
total label overlap, or an operation-cap breach produces a typed render issue and no
partial batch. Any renderer error or issue discards the private candidate; no CLI
output, PyO3 result, or Qt tab is published.

## Resource and publication boundary

The parser accepts no DOCTYPE or exactly the documented vendor marker. It never
fetches, loads, validates against, or resolves a DTD. It refuses `PUBLIC`
identifiers, another system identifier, internal subsets, entity declarations or
references, and declarations after the root.

| Limit | Maximum |
| --- | ---: |
| Source bytes | 1,048,576 |
| XML start elements | 50,000 |
| Decoded attribute value bytes | 1,024 |
| Fragment/node/bond identifier bytes | 128 |
| Imported records | 1,024 |
| Atoms per record | 10,000 |
| Bonds per record | 20,000 |

Only the CDXML branch of the existing generic new-document interchange route changes:

```text
admit source
  -> decode CDXML into source-specific validated records
  -> build CDXML record insertion
  -> admit one new-document transition
  -> observe and render the private committed candidate once
  -> publish the generic result only when that observation is clean
```

CML and SDF continue through their graph-only builder. No new descriptor, CLI verb,
Python parser, Qt branch, protocol field, or public route handle is introduced.

## Exclusions and non-goals

- No generic CDXML presentation fidelity, presentation-bearing interchange record,
  CDXML writer, raw `BoldWidth`, source layout, source identifier, or provenance store.
- No end-directed wedges, namespace variants, reactions, arrows, annotations,
  page layout, compact-group styled bonds, aromatic/dotted/adder styles, queries,
  attachment models, or CDX binary decoding.
- No sink-specific dash state, pixel-equivalence testing, or reuse of authored Wavy
  presentation-root geometry.
- No output on refusal and no replacement of an active Qt document on refusal.

## Delivered local evidence and remaining completion gates

Permanent, deterministic evidence proves:

- decoder token matrix, source-order correspondence, loss ordering, resource limits,
  and typed refusals;
- single, non-directed admission and exact `s1`/`b1`/`d1` persistence through history
  and reopen, with a malformed later record leaving the candidate unpublished;
- common clipping, finite geometry, translation/rotation equivariance, explicit
  Bold/dash/wave invariants, and typed no-partial-batch failure;
- clean generic CLI/PyO3/Qt results for each style and an unchanged active Qt tab
  after refusal; and
- affected Rust package gates, installed-wheel boundary test, registered CLI E2E,
  Qt lane, and `./build.sh`.

The local receipts at this checkpoint are: chemistry CDXML suite 22 tests,
document suite 517 tests, render suite 144 tests, API/PyO3 suite 181 tests,
installed Python suite 294 tests, focused Qt presentation suite 47 tests, and
registered CLI and real Qt CDXML E2Es. `./check_rust.sh` and `./build.sh` exited
0 for the delivered M2 slice. A prior `./all_test.sh` pass after the audit
corrections is retained as M2 evidence; the subsequent atom-label clearance
redesign requires a fresh aggregate rerun before a current end-state claim.

Real macOS 16:10 outer-window review, keyboard/accessibility walkthrough,
screenshots, independent audit, CI, and release approval are separate acceptance
evidence; they do not replace permanent contracts.

The alignment redesign's focused renderer receipt does not by itself prove the
installed Qt consumer, fresh screenshots, human visual/accessibility review, CI,
or release artifacts. Those gates remain open and are intentionally separate from
the deterministic CDXML contract.

## Ownership boundaries

| Owner | Responsibility |
| --- | --- |
| Chemistry | Parse the closed token grammar, retain source-specific presentation facts, and validate the carrier. |
| Document | Convert one validated carrier into the sole durable presentation type and CDML tokens. |
| Render | Build every clipped Bold/dashed/wavy primitive from native projected style. |
| Document-render and API | Render-admit the unpublished candidate and publish only a clean generic result. |
| PyO3, CLI, and Qt | Present or exercise issued generic facts only; add no presentation interpretation. |

The dependency direction remains chemistry to document, document to document-render,
API to lower layers, and Qt as a presentation consumer. A later codec gets its own
source adapter until evidence justifies a real cross-codec presentation contract.
