# CDML preservation coverage

## Purpose and status

This is historical M1d coverage evidence for a possible later M10 preservation
gate. The two compact semantic profiles were retired from the external corpus
and now live inline in the Rust semantic tests. No active `tests/e2e/corpus/`
input set or E2E corpus runner exists in this checkout.

Status: historical evidence only. The inventory records the available format
authority and the missing evidence needed to extend coverage. No real user
documents are available; that remains an explicit evidence limit. This audit
does not claim a current M10 gate, separate-process oracle, or corpus comparison.

## Evidence and authority

| Source | Authority used | Result |
| --- | --- | --- |
| OASA serializers/parsers | `packages/oasa/oasa/cdml_writer.py`, `cdml.py`, `cdml_xml.py`, `cdml_bond_io.py`, `cdml_ftext.py` | Current emitted/read vocabulary, core/opaque boundary, bond attribute variants, and ftext styles. |
| Format and conformance | `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md`; `OTHER_REPOS/bkchem-oasa/docs/cdml_conformance/cdml_26_07_manifest.json` | Grammar, authored/compatibility distinction, opaque namespace cases, and deliberate invalid security cases. |
| Shipped templates and references | `packages/bkchem-app/bkchem_data/templates/*.cdml`; `docs/reference_outputs/` | Four templates exist (40,265 bytes); one informed a retired one-time legacy probe. Reference outputs are Haworth SVG/PNG, not CDML. |
| Real user documents | No supplied user document exists in this checkout. | Coverage is unavailable for unanticipated extensions, namespace combinations, producer quirks, and real CD-SVG. A consented representative set is needed. |

The former compact profiles are historical evidence, not a historical-tree or
production-code import. The reduced legacy probe was removed because no test or
runtime path consumed it. Its findings remain documented here but do not count
as active test, runtime, or E2E corpus evidence.

| Corpus file | Classification | Source-of-truth level | Purpose |
| --- | --- | --- | --- |
| Retired `legacy_groups_template.cdml` probe | Historical, not active corpus | Shipped historical template and legacy reader behavior | Original reduced re-expression of `groups.cdml`; removed because it had no test or runtime consumer. No verbatim template block or OASA code. Upstream BKChem application license: GPL-2.0-or-later; central M1 documentation owner records final disposition. |
| Retired authored-document profile | Historical evidence | Format specification plus OASA core vocabulary | Its original compact XML now lives inline beside the semantic assertions in [packages/ferrum-rust/crates/document/src/typed_tests.rs](../../../packages/ferrum-rust/crates/document/src/typed_tests.rs); it is not a runtime or E2E input. |
| Retired opaque-namespace profile | Historical evidence | Format preservation rules and shipped conformance manifest | Its original compact XML now lives inline beside the semantic assertions in [packages/ferrum-rust/crates/document/src/typed_tests.rs](../../../packages/ferrum-rust/crates/document/src/typed_tests.rs); it is not a runtime or E2E input. |

No known defect is represented as a passing preservation fixture. No implementation accident is
promoted to corpus authority. The 26.07 entries are explicitly marked intended authored behavior;
they do not redefine legacy acceptance.

## Coverage inventory

`Covered` means a retired one-time profile carried the form. It does not mean
current Rust behavior exists or that a structural round-trip gate has passed.

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

Historical verification compared the retired profiles structurally. There is no
current corpus directory, E2E runner, or accepted M10 gate to invoke. A future
gate must define its committed input set, public boundary, comparison rule, and
evidence limits before it can make a passing preservation claim.

No Rust crate, frontend, runner, or root manifest is changed by this package. The central M1
documentation owner records this package in the changelog.
