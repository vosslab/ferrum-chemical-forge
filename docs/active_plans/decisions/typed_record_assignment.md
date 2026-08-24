# Typed versus opaque record assignment

## Decision

This document assigns every CDML element class to one of three M8 dispositions:
`typed`, `opaque payload container`, or `opaque`. It is the assignment table
deliverable of M8 in [ferrum-plan-v3.md](../ferrum-plan-v3.md). It also fixes the
unknown-attribute bag policy, the unrecognized-child policy, and the
additive-promotion rule that later milestones must obey.

What this table governs:

- Which CDML element classes get a typed Rust payload in `ferrum-document`.
- Which documented attributes each typed payload carries as named fields.
- Where every remaining attribute, child node, and subtree goes so it round-trips.
- The direction of future change: opaque may become typed; typed never becomes
  opaque.

What this table explicitly does not govern:

- The typed payload code itself. Struct shapes, field types, and the parser are a
  separate work package. This document names fields, not Rust types.
- Chemistry meaning. Element perception, valence, hydrogen counts, aromaticity,
  and stereo remain owned by the chemistry adapter, per
  [ferrum_core_model.md](ferrum_core_model.md).
- Rendering. A typed field here is a persistence fact, not a paint instruction.
- Identity allocation, reference validation, and mutation. M7 explicitly deferred
  those, and this document does not reopen them; see
  [document_identity_ordering.md](document_identity_ordering.md).
- Byte-level fidelity. Serialization fidelity remains the M6 structural contract
  in [xml_storage_fidelity.md](xml_storage_fidelity.md). Typing an element
  does not upgrade that promise.
- Format authorship. This table describes the CDML that exists. It defines no new
  CDML element, attribute, or version.

## Recognition rule

Recognition is decided before any attribute is read.

An element is recognized when its expanded name matches a documented CDML class:
the local name is in the core element registry and the namespace is exactly the
canonical Ferrum CDML namespace `urn:ferrum:cdml`. Historical BKChem and unqualified
roots are rejected by ordinary Ferrum ingress. This replaces the broader boundary the reference reader uses in
`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_xml.py`
(`CDML_CORE_ELEMENT_NAMES`, `_is_core_cdml_element`) and in
`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_writer.py:_direct_core_children`.

A typed class is keyed by parent context plus expanded name, not by local name
alone. CDML reuses local names across contexts: `<bond>` under `<standard>` is a
default record, under `<molecule>` a chemical bond, and under `<fragment>` an
IDREF member; `<atom>` under `<standard>` is a default record and under
`<molecule>` a chemical vertex; `<text>` is a molecule vertex or a standalone
canvas label; `<arrow>` and `<plus>` are direct-root drawables or reaction
roles. The reference
attribute registry `CDML_CORE_ATTRIBUTE_NAMES` collapses these into one key per
local name, which is why its `arrow` entry carries both `length` and `idref`. M8
must not copy that collapse.

A foreign-namespace element is never recognized, even when its local name looks
like CDML. An unknown local name in the canonical namespace is also not
recognized; it starts an opaque subtree.

A recognized local name in a parent context this table does not list is opaque.
For example `<mark>` directly under `<cdml>` carries a registry local name, but
the table names no `cdml/mark` class, so the element and its subtree become
opaque content of the root record. The reference recognizer decides this case on
local name plus core ancestry alone (`_is_core_cdml_element` at
`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_xml.py:140-148` consults no
context table), so its behavior does not settle which typed class such an
element would be. Choosing opaque is the safe answer rather than a neutral one:
opaque-to-typed is a permitted promotion under the additive rule below, so a
later milestone with a real example can type the context without breaking a
document, while inventing a typed class now would be format authorship this
document disclaims. This disposition applies to every unlisted parent/name pair,
not only to the `<mark>` example.

## Assignment table

`Corpus` names a historical profile that originally evidenced the class: `authored` or `opaque`.
Those external corpus files are retired; their compact XML now lives inline with the semantic
assertions in `packages/ferrum-rust/crates/document/src/typed_tests.rs`. `legacy` identifies
historical evidence from the retired `legacy_groups_template.cdml` probe. `reference only` marks
a class found in reference material but in no historical profile.

Attribute lists are the documented typed fields. Every attribute not listed goes
to that record's unknown-attribute bag.

### Root and header records

| Element class | Corpus | Assignment | Typed payload attributes | Reason and evidence |
| --- | --- | --- | --- | --- |
| `cdml` (root) | authored, legacy, opaque | typed | `version`, `type` | Root grammar in `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md` "Root element". Version drives the migration chain; `type` selects normal/template/standard. M7 already treats the root as a distinct node. |
| `info` | authored, legacy | typed | none | Container only; its payload is its three typed children. |
| `info/author_program` | authored, legacy | typed | `version` | Authoring-program release, independent of root `version`. Text content is the program name. |
| `info/author` | authored | typed | none | Text content only. |
| `info/note` | authored | typed | none | Text content only. |
| `metadata` | authored | typed | none | Container for `doc`. |
| `metadata/doc` | authored | typed | `href` | Documented discovery pointer. |
| `standard` | authored | typed | `line_width`, `font_size`, `font_family`, `line_color`, `area_color`, `paper_type`, `paper_orientation`, `paper_crop_svg`, `paper_crop_margin` | Document drawing defaults. Values are retained document data; M8 stores lexical text and does not convert units. |
| `standard/bond` | authored | typed | `length`, `width`, `wedge-width`, `double-ratio`, `min_wedge_angle` | Distinct class from a molecule bond. Hyphenated spellings are the authored form here. |
| `standard/arrow` | authored | typed | `length` | Distinct class from a drawable arrow. |
| `standard/atom` | authored | typed | `show_hydrogens` | Distinct class from a molecule atom. Reader also accepts `True`/`False` spellings. Whether `show_hydrogens` belongs only to this class rests on a registry-collapse judgment; see open questions. |
| `paper` | authored | typed | `id`, `type`, `orientation`, `crop_svg`, `crop_margin`, `use_real_minus`, `replace_minus`, `size_x`, `size_y` | Documented page record with an existing revision-bound patch operation upstream. `id` is a typed declaration field: `paper` is in `_ID_DECLARATION_ELEMENT_NAMES` at `OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_document.py:40-44` and `_is_id_declaration` at `:1472-1485` grants it declaration behavior. |
| `viewport` | authored, legacy | typed | `viewport`, `id` | Documented canvas record. The four-float string stays one lexical value; M8 does not split it. |

### Direct-root drawable records

| Element class | Corpus | Assignment | Typed payload attributes | Reason and evidence |
| --- | --- | --- | --- | --- |
| `molecule` | authored, legacy, opaque | typed | `id`, `name` | The plan's molecule class. Both attributes are optional source facts, matching the `source_id` policy in [ferrum_core_model.md](ferrum_core_model.md). |
| `arrow` (direct root) | authored | typed | `id`, `type`, `start`, `end`, `width`, `spline`, `shape`, `color` | The plan's arrow class. Ordered `point` children carry the path. `idref` is excluded here: it belongs to the reaction-role class. |
| `plus` (direct root) | authored | typed | `id`, `font_size`, `color`, `background-color` | The plan's plus-sign class. Exactly one `point` child, optional `font`. |
| `text` (direct root) | authored | typed | `id`, `background-color` | The plan's text class, standalone form. Children are optional `font`, one `point`, one `ftext`. Whether `pos`, `number`, and `show_number` also belong here is unresolved; see open questions. |
| `rect` | authored | typed | `id`, `x1`, `y1`, `x2`, `y2`, `area_color`, `line_color`, `width` | Vector graphic. Four bounds are the authored minimum. |
| `square` | authored | typed | `id`, `x1`, `y1`, `x2`, `y2`, `area_color`, `line_color`, `width` | Vector graphic; same shape as `rect` with its own element identity retained. |
| `oval` | authored | typed | `id`, `x1`, `y1`, `x2`, `y2`, `area_color`, `line_color`, `width` | Vector graphic. |
| `circle` | authored | typed | `id`, `x1`, `y1`, `x2`, `y2`, `area_color`, `line_color`, `width` | Vector graphic. |
| `polygon` | authored | typed | `id`, `area_color`, `line_color`, `width` | Vector graphic; three or more ordered `point` children. |
| `polyline` | authored | typed | `id`, `line_color`, `width`, `spline` | Vector graphic; two or more ordered `point` children. This class also carries bracket and wavy artwork. `style` is unresolved; see open questions. |
| `reaction` | authored | typed | `id` | The plan's reaction class. `id` is optional compatibility data in 26.07. |
| `reaction/reactant` | authored | typed | `idref` | Recognized reference field, not an ID definition. |
| `reaction/product` | authored | typed | `idref` | Recognized reference field. |
| `reaction/arrow` | authored | typed | `idref` | Role child, distinct class from the drawable arrow. |
| `reaction/condition` | authored | typed | `idref` | Role child naming a standalone text. |
| `reaction/plus` | authored | typed | `idref` | Role child, distinct class from the drawable plus. |
| `external-data` | authored, opaque | opaque payload container | none | Established durable container with no typed grammar. Its own attributes, including `id`, are literal preservation content per `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md` and the backend contract. Recognition stops at the element boundary. |

Reaction role cardinality and repetition are settled here, not deferred. The
format spec explicitly declines to define cardinality, ordering, stoichiometry,
or repeated-target semantics for reaction roles
(`OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md:1009`), so M8 stores the five
role classes above as one ordered list per reaction with no uniqueness rule and
no role-count constraint. That choice preserves every accepted historical
reaction structure. A later typed reaction model may add semantics on top of the
stored order; under the additive rule it may not remove a role a document
carries.

### Molecule-scoped records

| Element class | Corpus | Assignment | Typed payload attributes | Reason and evidence |
| --- | --- | --- | --- | --- |
| `molecule/atom` | authored, legacy, opaque | typed | `id`, `name`, `charge`, `pos`, `show`, `hydrogens`, `show_number`, `number`, `background-color`, `multiplicity`, `valency`, `free_sites`, `isotope`, `explicit_hydrogens` | Documented atom grammar. `Option` presence preserves absent versus default, per the core-model source-field mapping. This list is the collapsed registry `atom` key minus `show_hydrogens`, which this table assigns to `standard/atom`; that split is unresolved, see open questions. |
| `molecule/group` | authored | typed | `id`, `name`, `group-type`, `pos`, `background-color`, `show_number`, `number` | The plan's group class. Chemical vertex, not a visual container. |
| `molecule/text` | authored | typed | `id`, `pos`, `background-color`, `show_number`, `number` | Molecule-local text vertex, introduced in 0.14. Distinct class from direct-root text. The collapsed registry `text` key supplies this whole list; which names the direct-root form also carries is unresolved, see open questions. |
| `molecule/query` | authored | typed | `id`, `name`, `pos`, `background-color`, `show_number`, `number`, `free_sites` | Query vertex; participates in bond endpoints in the authored fixture. |
| `molecule/bond` | authored, legacy | typed | `id`, `type`, `start`, `end`, `line_width`, `bond_width`, `wedge_width`, `double_ratio`, `center`, `auto_sign`, `equithick`, `simple_double`, `color`, `wavy_style`, `haworth_position` | Documented core plus depiction attributes. `start`/`end` order is persistent depiction data for `w1` and `h1` and must not be rederived from geometry. `distance` and `width` are unresolved; see open questions. |
| `molecule/template` | authored, legacy | typed | `atom`, `bond_first`, `bond_second` | Molecule-local IDREF attachment metadata. |
| `molecule/fragment` | authored | typed | `id`, `type` | Named substructure. Richer historical shapes stay typed with their extra content in the unrecognized-child list rather than becoming opaque. |
| `fragment/name` | authored | typed | none | Text content is the fragment name. |
| `fragment/bond` | authored | typed | `id` | Documented IDREF, not an ID definition. |
| `fragment/vertex` | authored | typed | `id` | Documented IDREF, not an ID definition. |
| `fragment/property` | authored | typed | `name`, `value`, `type` | Arbitrary key-value property with a documented attribute triple. |
| `molecule/display-form` | authored | opaque payload container | none | Documented preservation-only container. Descendants receive no CDML lookup even with canonical names; the authored fixture proves this with a canonical `future-local` child. |
| `molecule/user-data` | authored | opaque payload container | none | Documented preservation-only container. |

### Shared child records

| Element class | Corpus | Assignment | Typed payload attributes | Reason and evidence |
| --- | --- | --- | --- | --- |
| `point` | authored, legacy, opaque | typed | `x`, `y`, `z` | Coordinates for vertices, marks, and drawable paths. M8 stores lexical text; unit and precision policy stays with the geometry owner. |
| `font` | authored | typed | `size`, `family`, `color` | Per-object font override. |
| `ftext` | authored | typed | none | Rich text is escaped character data since 0.16. M8 stores the character-data value verbatim and does not parse the inner markup grammar; pre-0.16 direct child markup is preservation content, not CDML vocabulary. |
| `mark` | authored | typed | `type`, `x`, `y`, `auto`, `size`, `line_width`, `draw_circle`, `text`, `refname` | Documented mark attributes across all mark types. Marks carry no `id` in 26.07; identity is parent atom plus type plus direct-child order. |

### Reference-only and non-classes

| Element class | Corpus | Assignment | Typed payload attributes | Reason and evidence |
| --- | --- | --- | --- | --- |
| Rectangular bracket | reference only, as `polyline` | not a class | n/a | The plan names bracket as an object class, but CDML has no `<bracket>` record. `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md` "Deferred document concepts" states the bracket tool persists its artwork as top-level `<polyline>` records, and the backend contract commits a bracket as two new direct-root polylines. M8 therefore assigns bracket to the `polyline` row and creates no bracket payload. |
| `bracket` element | reference only | opaque | none | Named in the spec as a deferred proposal and absent from the core element registry. Legacy or extension `<bracket>` content is preserve-only, receives no provisional-ID allocation, and is a promotion candidate only under a separately versioned grammar. Inference: no fixture or production writer emits it, so its real-world shape is unknown. |
| `vector` element | reference only | opaque | none | Same deferred-proposal status as `bracket`. The plan's "vector graphic" class maps to the six shape rows above, not to a `<vector>` element. |
| Foreign-namespace element | opaque | opaque | none | Begins an opaque subtree that retains lexical QNames, literal values, and complete in-scope namespace context. The opaque fixture covers `v:extension` with a foreign child, QName-like attribute values such as `q:kind="q:widget"`, and mixed text around that child. |
| Unknown canonical-namespace local name | authored (inside `display-form` only) | opaque | none | Covered only inside a preservation-only container. Outside one, the M1d audit records this form as uncovered; M8 must not invent its semantics. |
| Recognized local name in an unlisted parent context | reference only | opaque | none | No row above names the pair, so the element and its subtree are opaque content of its parent record. See the recognition rule for why opaque is the safe disposition here. |

Every element name in the reference core registry
(`CDML_CORE_ELEMENT_NAMES`, 40 names) appears in the corpus, so no typed row above
is reference-only. The reference-only rows are exactly the deferred proposals and
the concepts the plan named that CDML does not implement as elements.

## Unknown-attribute bag and unrecognized-child list

This is the mechanism that makes the M8 exit criterion hold: a recognized element
carrying one unfamiliar attribute stays typed and still round-trips.

### Recognition is independent of content

An element's class is decided by expanded name and parent context only. No
attribute, missing child, malformed value, or extra child can change a typed
element into an opaque one. The authored fixture pins this with
`atom@local_extension="literal"`: the atom is a recognized `molecule/atom`, its
fourteen documented attributes populate typed fields, and `local_extension` lands
in the bag.

### Unknown-attribute bag

Every typed record carries a bag holding one entry per attribute that is not one
of that class's typed fields. Each entry retains:

- the QName selected by the retained namespace context; prefix spelling remains
  subject to M6 structural normalization and is not a lexical-fidelity promise;
- the expanded name (namespace URI plus local name), with an empty URI for an
  unqualified attribute;
- the literal attribute value, uninterpreted; and
- the in-scope namespace context needed to resolve a QName-like value.

The bag does not retain attribute order, because M6 already excluded attribute
order from the fidelity promise. Serialization emits typed fields, then bag
entries, in a deterministic order.

Bag entries are literal. They receive no IDREF resolution, provisional-token
allocation, or value normalization, and a bag entry never makes its record a
reference source.

One behavior does reach a bag `id`, and M7 already owns it. The M7 index covers
every unqualified `id` in the document, including opaque content, so a bag `id`
is indexed and reserves a document-wide collision name exactly as M7 states in
[document_identity_ordering.md](document_identity_ordering.md). Being
indexed is the whole of that behavior: it neither promotes the attribute to a
typed declaration nor grants it reference semantics. This distinction matters
now that `paper@id` is a typed field, because it fixes what changes on any
future promotion of a bag `id` -- the collision reservation is already in place
and stays unchanged; only declaration and reference behavior is added.

Typed fields store the source text verbatim. An unmodified typed field serializes
with its original lexical spelling, so typing `bond@width="1.0"` never rewrites it
to `1`. Only an explicit edit changes a field's spelling.

### Unrecognized-child list

Every typed record carries an ordered list of child nodes that are not recognized
typed children of that class. It holds unknown or foreign elements, comments,
processing instructions, character data including CDATA, and tail text.

Each entry records its index within the element's complete child sequence, so
serialization reconstructs the exact original child order and mixed content. An
unrecognized element entry owns its whole subtree, which stays opaque even where a
descendant uses a canonical CDML name.

Recognized children that a class permits go to typed slots. When a class specifies
a cardinality (for example one `point` per atom) and the source violates it, the
extra nodes go to the unrecognized-child list rather than being dropped, and the
record stays typed and carries a diagnostic.

### Preservation-only containers

`display-form`, `user-data`, and handler-less `external-data` suppress recognition
for their entire subtree. Their element identity and document position are typed
facts; their own attributes and every descendant are opaque payload. This matches
the format spec, the backend contract, and the M1d audit row that classifies a
canonical `future-local` inside `display-form` as preservation-only.

### Round-trip statement

For a recognized element, output equals: typed fields serialized from their stored
lexical values, plus bag attributes, plus the recognized and unrecognized children
re-emitted in their recorded combined order. Fidelity is M6 structural fidelity.
Typing adds no lexical promise about prefix spelling, attribute order, quote
style, entity spelling, or CDATA boundaries.

## Additive-promotion rule

Promotion is the only permitted direction of change.

Permitted:

- An opaque element class becomes typed in a later milestone.
- An attribute moves out of a record's unknown-attribute bag into a named typed
  field.
- A child node kind moves out of the unrecognized-child list into a typed slot.
- A typed payload gains a field.

Forbidden:

- Demoting a typed element class to opaque.
- Moving a typed field back into the unknown-attribute bag.
- Moving a typed child slot back into the unrecognized-child list.
- Removing a typed field, or narrowing its accepted value space so that a
  previously accepted document stops round-tripping.
- Demotion triggered by content. An unfamiliar attribute, a malformed value, a
  missing required child, or a violated cardinality leaves the record typed and
  produces a diagnostic. A record may be typed and unusable for editing at the
  same time; it never becomes opaque.
- Promoting a descendant of a preservation-only container without an explicitly
  versioned typed handler for that container. Even then, the handler normalizes
  only content its own schema governs.
- Treating opaque reference-like attributes or text as references. Promotion of a
  reference field requires a documented recognized-reference decision, not
  inference from a value that looks like an ID.

Compatibility requirement on any promotion: for every document that does not use
the newly typed behavior, serialized output must be unchanged. A promotion that
alters output for existing corpus fixtures is a format change, not a promotion,
and needs its own decision.

## Open questions

These are unresolved by the evidence read for this document. No answer is invented
to fill a cell.

1. `polyline@style` and legacy `polyline@color`. The reference document layer
   reads `style != "wavy"` as a rejection condition at
   `OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_document.py:4847`, and the Qt
   contract names a durable top-level `<polyline style="wavy">` as an editable
   target. The same layer reads legacy `color` as a fallback at
   `OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_document.py:4919-4920`, where
   the visible color is `line_color`, then `color`, then `#000000`. Both names
   are absent from the reference polyline attribute registry
   (`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_xml.py:57`) and from the
   polyline table in
   `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md:967-976`, so they share one
   evidentiary status: read by the implementation, undeclared by the registry and
   the spec. Unresolved whether each is a typed polyline field or a bag
   attribute; both are bag attributes above. Either choice round-trips; only the
   typed choice lets Ferrum recognize a wavy record or its legacy color.
2. `molecule/bond@distance` and `molecule/bond@width`. The legacy fixture carries
   both on a real molecule bond, and the reference attribute registry lists them
   under the collapsed `bond` key alongside `standard/bond` names such as
   `length`, `min_wedge_angle`, `wedge-width`, and `double-ratio`. Unresolved
   which of these are legitimate molecule-bond compatibility fields and which are
   registry collapse artifacts. Assigned to the bag above, pending evidence.
3. Hyphenated versus underscored depiction aliases. The registry lists both
   `double-ratio`/`double_ratio` and `wedge-width`/`wedge_width`. Unresolved
   whether a molecule bond may legally carry the hyphenated spelling, or whether
   those spellings belong only to `standard/bond`. The M1d audit already records
   hyphen aliases as needing a real example.
4. `paper@id` prevalence. The disposition is decided, not open: `paper` is in
   `_ID_DECLARATION_ELEMENT_NAMES` and `_is_id_declaration` grants it declaration
   behavior, so `id` is a typed `paper` field above. What remains unresolved is
   frequency and provenance. No corpus fixture carries `paper@id`, and the
   attribute registry omits `id` from its `paper` key while listing it for
   `viewport`, so this table cannot say whether any real document or production
   writer emits one. That gap affects test coverage and migration expectations,
   not the assignment.
5. `atom@show_hydrogens` context. This table gives `show_hydrogens` to
   `standard/atom` and withholds it from `molecule/atom`, but the registry
   collapses both classes into one `atom` key that carries the name. The
   judgment is the same kind open question 2 leaves unresolved for `bond`.
   Unresolved whether a molecule atom may legally carry `show_hydrogens`; it is a
   bag attribute on `molecule/atom` until a real example decides it.
6. Direct-root `text` versus `molecule/text` fields. This table gives `pos`,
   `number`, and `show_number` to `molecule/text` only, inferred from their
   absence on the authored fixture's direct-root text. Absence in one fixture is
   not evidence of prohibition, and the collapsed registry `text` key carries all
   three. Unresolved whether a direct-root text may legally carry them; they are
   bag attributes on that class until a real example decides it.
7. Unknown canonical-namespace local names outside a preservation-only container.
   The M1d audit marks this uncovered, and no fixture or production writer
   supplies one. Unresolved how a future canonical extension should be classified;
   this table treats it as opaque, which is the safe default but is an inference,
   not a read fact.
8. `bracket` and `vector` element shapes. Both are named as deferred proposals
   with no grammar and no example. Unresolved what a legacy `<bracket>` or
   `<vector>` document actually contains, so their promotion path cannot be
   designed yet.
9. `ftext` inner markup. M8 stores the escaped character-data value verbatim.
   Unresolved which milestone owns the typed run grammar, whether that grammar
   lives in the document layer or above it, and how pre-0.16 direct child markup
   coexists with it in one payload.
10. Coordinate lexical forms. `point@x`, `point@y`, mark coordinates, and shape
    bounds accept bare, `cm`, `px`, `mm`, and `in` spellings, and the M1d audit
    records `px`, `mm`, and `in` as uncovered. M8 stores lexical text, so the
    round trip is safe, but the conversion owner is unresolved.
11. Diagnostics representation. This document requires that a malformed typed
    record stay typed and carry a diagnostic. The diagnostic value's shape, and
    whether it lives on the record or in a document-level list, is unresolved and
    belongs to the typed-payload work package.
