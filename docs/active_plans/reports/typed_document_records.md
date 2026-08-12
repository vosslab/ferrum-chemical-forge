# Typed document records

## Verdict

M8 is complete. `ferrum-document` owns the only production CDML reader in the Rust
workspace and projects one retained `xot` tree into content-independent typed records.
Every class in the accepted assignment table is exercised by the authored corpus. The
disposable core-crate corpus reader was deleted rather than becoming a competing parser.

## Design

- `TypedClass` is a context-qualified enum, so a molecule bond, standard bond, and
  fragment bond cannot be confused because they share a local XML name.
- Recognition uses only parent class plus expanded element name. Missing children,
  excess children, malformed values, and unfamiliar attributes never demote a record.
- Present named fields retain exact lexical values in a deterministic map. Conversion
  to numeric chemistry facts occurs only in the separate core projection.
- Every unfamiliar attribute carries its expanded name, literal value, selected QName,
  and in-scope namespaces. QName prefixes follow the established structural-fidelity
  contract rather than claiming byte-level XML preservation.
- Recognized children, typed text segments, and opaque children each retain their
  position in the complete mixed-content sequence. An opaque element owns its whole
  subtree, including canonical-looking descendants.
- Cardinality violations leave the parent typed, retain excess nodes as opaque content,
  and attach an owned diagnostic.
- `external-data`, `molecule/display-form`, and `molecule/user-data` are typed opaque
  containers. Their class, path, and parent-child position are typed facts; their own
  attributes and every descendant remain opaque payload.
- `TypedDocument` keeps the M7 index and typed overlay over one retained tree. Saving
  serializes that tree, so M8 introduces no second source of XML truth.

## Identity correction

The corpus exposed a pre-existing M7 ambiguity: `fragment/bond@id` and
`fragment/vertex@id` are references, not declarations. Indexing every unqualified
`id` made a valid fragment reference collide with the bond it named. The declaration
index now excludes those two recognized reference contexts while continuing to reserve
all opaque `id` values.

## Reader retirement and parity

The comparison route is now:

```text
CDML -> ferrum-document typed overlay -> ferrum-core projection -> comparison JSON
```

The disposable core-crate corpus reader was deleted, `xot` was removed from
`ferrum-core` development dependencies, and
`tests/test_cdml_reader_inventory.py` permits only `ferrum-document` to recognize the
CDML namespace in Rust production code. The isolated corpus comparison reports 96
exact agreements, 29 classified differences, and zero unexpected differences. Its
atom and non-atom mutations each produce exactly one unexpected difference and exit 1.

## Validation

- All 21 `ferrum-document` unit tests pass, including every assigned typed class,
  unknown-attribute retention, opaque-container suppression, excess-child diagnostics,
  fragment-reference identity, typed-to-core projection, and every reachable public
  projection-error class.
- The authoritative comparison reports `match-with-classified-differences` with zero
  unexpected differences.
- `tests/test_cdml_reader_inventory.py` passes with one reader.
- Every new Rust source file remains below the repository's 1,000-line ceiling.
