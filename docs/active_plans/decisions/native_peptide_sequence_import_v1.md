# Native peptide sequence import V1 decision

## Status

Selected and implemented as a narrow Qt/PyO3-native-17 insertion path. This
decision defines the shipped contract and its remaining evidence boundary.

## Closed contract

- `prepare_ferrum_peptide_insertion_v1(sequence, placement)` accepts only an
  exact, nonempty uppercase one-letter sequence and returns the existing typed
  detached molecule insertion object. It does not commit a document.
- The only profile is `ferrum-native-peptide-structure-v1`, implemented as
  `Native17ZwitterionicTermini`. It owns typed residue, connectivity, charge,
  and stereochemistry facts with zwitterionic free termini.
- Domain owns sequence parsing and peptide facts; the document adapter owns
  typed-plan conversion, coordinates, placement, and stereo transfer; chemistry
  owns native graph services; Qt owns visible entry and normal insertion; PyO3
  owns only this bridge and its exception mapping.
- `FerrumPeptideSyntaxError` reports invalid syntax. A canonical-alphabet
  residue without a native-17 recipe raises
  `UnsupportedFerrumPeptideProfileError` with position, residue, and profile.
- Native loading, chemistry preparation, placement, or insertion conversion
  follows the existing structured failure path and does not mutate a document
  through this preparation call.

## Exclusions

- No legacy molecular grammar, SMILES compilation, source-string parser, CLI
  command, or operation-protocol route exists for peptide sequences.
- No alternate profile, terminus policy, arbitrary recipe, or user-provided
  chemistry text is admitted.

## Verification gates

- Domain tests cover parser refusal, zwitterionic termini, N-to-C peptide-link
  ownership, stereochemical facts, and unsupported native-17 residues.
- Document-adapter tests cover peptide-link/termini preservation, stereo
  transfer, and profile refusal before a document candidate exists.
- PyO3 tests cover bridge success and typed syntax/profile facts. Qt tests cover
  controller use of the typed candidate and typed error presentation.
- A visible-UI E2E remains required before end-to-end closure is claimed. It
  must create and place a sequence through the real Qt workflow, then prove
  committed structure and recovery without private session access, raw-ID
  assertions, timing gates, or pixel equality.

## Related implementation

- [peptide_insertion_binding.rs](../../../packages/ferrum-rust/crates/api/src/python_binding/peptide_insertion_binding.rs)
- [peptide_structure_plan_document_adapter_v1.rs](../../../packages/ferrum-rust/crates/document/src/chemistry/peptide_structure_plan_document_adapter_v1.rs)
- [peptide_import.py](../../../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/peptide_import.py)
