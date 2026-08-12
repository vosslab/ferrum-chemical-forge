# CDML preservation coverage

## Purpose and status

This is the M1d evidence baseline for the later M10 preservation gate. It states what
the committed `tests/e2e/corpus/` represents; it does not declare the corpus complete.
The separate-process oracle and corpus comparison now exist as independent evidence.

Status: inventory, compact inputs, separate-process harness, and divergence report are
established. M1d remains in progress because no real user documents are available and
the no-namespace, future-version, alternate-prefix, and CD-SVG forms remain uncovered.

## Evidence and authority

| Source | Authority used | Result |
| --- | --- | --- |
| OASA serializers/parsers | `packages/oasa/oasa/cdml_writer.py`, `cdml.py`, `cdml_xml.py`, `cdml_bond_io.py`, `cdml_ftext.py` | Current emitted/read vocabulary, core/opaque boundary, bond attribute variants, and ftext styles. |
| Format and conformance | `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md`; `OTHER_REPOS/bkchem-oasa/docs/cdml_conformance/cdml_26_07_manifest.json` | Grammar, authored/compatibility distinction, opaque namespace cases, and deliberate invalid security cases. |
| Shipped templates and references | `packages/bkchem-app/bkchem_data/templates/*.cdml`; `docs/reference_outputs/` | Four templates exist (40,265 bytes); one informs the reduced legacy probe. Reference outputs are Haworth SVG/PNG, not CDML. |
| Real user documents | No supplied user document exists in this checkout. | Coverage is unavailable for unanticipated extensions, namespace combinations, producer quirks, and real CD-SVG. A consented representative set is needed. |

The corpus is intentionally three XML documents, not a historical-tree or production-code import.
Each fixture declares its classification and source-of-truth level in an XML comment.

| Corpus file | Classification | Source-of-truth level | Purpose |
| --- | --- | --- | --- |
| [`legacy_groups_template.cdml`](../../../tests/e2e/corpus/legacy_groups_template.cdml) | Required compatibility | Shipped historical template and legacy reader behavior | Original reduced re-expression of `groups.cdml`; no verbatim template block or OASA code. Upstream BKChem application license: GPL-2.0-or-later; central M1 documentation owner records final disposition. |
| [`authored_document_forms.cdml`](../../../tests/e2e/corpus/authored_document_forms.cdml) | Required compatibility; intended authored behavior | Format specification plus OASA core vocabulary | Original compact fixture based on documented facts, not copied code or source text; central M1 documentation owner records final provenance/disposition. |
| [`opaque_namespace_preservation.cdml`](../../../tests/e2e/corpus/opaque_namespace_preservation.cdml) | Required compatibility | Format preservation rules and shipped conformance manifest | Original compact probe based on documented cases, not copied manifest XML or OASA code; central M1 documentation owner records final provenance/disposition. |

No known defect is represented as a passing preservation fixture. No implementation accident is
promoted to corpus authority. The 26.07 entries are explicitly marked intended authored behavior;
they do not redefine legacy acceptance.

## Coverage inventory

`Covered` means an input carries the form. It does not mean Rust behavior exists or a future
structural round-trip gate has passed.

| Form and attributes | Namespace and reference case | Status | Evidence or precise next evidence |
| --- | --- | --- | --- |
| Root `cdml@version,type` | Canonical default namespace | Covered | Authored and legacy probes use the URI; versions 26.07 and 0.8 occur. |
| Legacy no-namespace root | No namespace direct root | Uncovered | The manifest has inline XML but no shipped file. Obtain a historical-release or user document. |
| Unknown future root version | Canonical namespace | Uncovered | Need a real future-version document; a made-up version only tests plumbing. |
| `info`, `author_program@version`, `author`, `note`, `metadata/doc@href` | No references | Covered | Authored probe covers all; shipped templates independently contain `info`. |
| `standard` and nested defaults | Unit-bearing and legacy spellings | Covered | Authored probe has documented nested records; legacy probe has historical syntax. |
| `paper` and `viewport` | Custom dimensions and scalar viewport | Covered | Authored has custom paper; legacy has viewport. |
| `molecule@id,name` and direct-child order | Durable ID definition | Covered | Both probes use molecules; authored document carries header, drawing, and reaction order. |
| `atom` attributes and child point/font/ftext/mark | Atom ID and no reference | Covered | Authored probe carries chemistry, display, numbering, and children. |
| `group`, molecule-local `text`, `query` | Bond endpoint IDREF targets | Covered | Authored probe provides each; `group`, text, and query each participate in a bond endpoint. |
| Bond core and current depiction fields | Ordered `start,end` IDREF; `n`, `w`, `q` | Covered | Authored probe includes every current `cdml_bond_io.py` metadata name. |
| Other bond styles and aliases | `h,a,b,d,o,s,l,r`; hyphenated aliases | Partly covered | Legacy probe has `s`, `d`, and `distance`; authored has underscore names. Need real examples for remaining styles and hyphen aliases. |
| Coordinate lexical forms | Bare, `cm`, `px`, `mm`, `in`; `z` | Partly covered | Bare/z legacy and `cm` authored are present. Obtain historical `px`, `mm`, and `in` documents. |
| `font` and ftext styles | Escaped character data with unqualified b/i/sub/sup only | Covered | Authored probe stores style markup as escaped ftext character data and uses b/i/sub/sup, never direct child markup or unsupported u. Invalid markup belongs in a negative harness case. |
| Atom mark types | Optional mark attributes | Partly covered | One plus mark covers structure. Need saved cases for minus, radicals, electron pairs, and presentation-only marks. |
| `template@atom,bond_first,bond_second` | Molecule-local IDREF | Covered | Authored has all; legacy confirms atom-only form. |
| `fragment`, `name`, `bond@id`, `vertex@id`, `property` | Fragment child IDs are IDREFs | Covered | Authored linear-form fragment covers all. Need real explicit and implicit fragments. |
| `display-form` and `user-data` | Descendants opaque; literal IDs reserve names | Covered | Authored probe carries opaque descendants with IDs. |
| Arrow, plus, standalone text | Durable IDs and direct point paths | Covered | Authored probe supplies all forms. |
| Rect, square, oval, circle | Durable IDs and four bounds | Covered | Authored probe supplies each documented shape. |
| Polygon and polyline | Ordered point sequences | Covered | Authored probe supplies minimum ordered sequences. |
| Reaction roles | Five `@idref` forms to direct root | Covered | Authored probe covers reactant, product, arrow, condition, plus. |
| `external-data` | Literal ID and foreign descendants | Covered | Authored and opaque probes cover both. |
| Foreign elements and namespaced unknown attributes | Foreign subtree, QName-like literal, namespace context | Covered | Opaque probe has `v` and `q` bindings, foreign attributes, nesting, and literal QName text. |
| Unqualified unknown attribute on known CDML element | Lexical local name and literal value | Covered | Authored probe has `atom@local_extension="literal"`; it is distinct from the foreign namespaced attributes in the opaque probe. |
| Canonical CDML alternate prefix | Expanded-name equivalence | Uncovered | Need genuine CD-SVG/CDML with a non-default canonical prefix. |
| Embedded CDML in SVG/CD-SVG | SVG wrapper and canonical CDML | Uncovered | No shipped CD-SVG exists. Obtain one from a supported release or user document. |
| Comments, PI, and text/tail order | Pre-root and internal-root positions | Covered | Opaque probe contains a pre-root PI, internal comment/PI, and foreign-element text/tail. It makes no after-root claim; CDATA spelling is intentionally not preserved lexically. |
| Foreign unknown element outside an opaque container | Foreign namespace begins opaque subtree | Covered | Opaque probe has direct-root `v:extension` with a foreign descendant. |
| Unknown canonical-CDML local name outside opaque content | Canonical namespace, future extension | Uncovered | Need real future-extension CDML; do not invent semantics. |
| Unknown canonical-CDML local name inside opaque container | No CDML lookup below `display-form`, `user-data`, or handler-less `external-data` | Covered | Authored probe has canonical `future-local` inside `display-form`; it is preservation-only despite its canonical namespace. |
| XML security rejection | DOCTYPE, entities, malformed XML | Uncovered by design | Manifest supplies negative inputs; these belong to the separate harness, not preservable corpus. |
| Real user extension schemas and quirks | Any user namespaces or legacy producers | Unavailable | Request a small consented sample with producer/version/provenance notes; classify before adding it. |

## Gate use and verification

The table deliberately gives no brittle total: rows can cover several XML forms. A later gate must
enumerate expanded names and reference classes, then fail only when a form is neither represented
nor accompanied by its recorded reason and next evidence. This baseline cannot prove real-user
extension coverage.

Verification for this package:

- Parse every corpus XML document with `xml.etree.ElementTree` in the required environment.
- Check the audit and corpus for ASCII-only bytes.
- Run the Markdown-link check on this audit's relative links.
- Cross-check corpus element names and documented reference attributes against this table.

No Rust crate, frontend, runner, or root manifest is changed by this package. The central M1
documentation owner records this package in the changelog.
