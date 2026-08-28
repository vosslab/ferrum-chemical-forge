"""Typed identifier receipt validation and display for Molecule Report."""


_UNAVAILABLE_REASONS = {"unsupported_molecule", "chemistry_unavailable"}


#============================================
def valid_identifiers(identifiers: object) -> bool:
	"""Accept exactly one complete Rust-issued identifier outcome."""
	if type(identifiers) is not dict:
		return False
	if identifiers.get("kind") == "available":
		return (
			set(identifiers) == {
				"kind", "canonical_smiles", "standard_inchi", "standard_inchi_key",
			}
			and all(type(identifiers[name]) is str and bool(identifiers[name]) for name in (
				"canonical_smiles", "standard_inchi", "standard_inchi_key",
			))
		)
	if identifiers.get("kind") == "unavailable":
		return (
			set(identifiers) == {"kind", "reason"}
			and type(identifiers["reason"]) is str
			and identifiers["reason"] in _UNAVAILABLE_REASONS
		)
	return False


#============================================
def display_lines(identifiers: dict) -> list[str]:
	"""Render one authenticated outcome without calculating or repairing identities."""
	if identifiers["kind"] == "available":
		lines = [
			"Identifiers:",
			"  Canonical SMILES: {0}".format(identifiers["canonical_smiles"]),
			"  Standard InChI: {0}".format(identifiers["standard_inchi"]),
			"  Standard InChIKey: {0}".format(identifiers["standard_inchi_key"]),
		]
		return lines
	if identifiers["kind"] == "unavailable":
		lines = ["Identifiers: unavailable ({0})".format(identifiers["reason"])]
		return lines
	raise ValueError("unknown Rust molecule-report identifier outcome: {0}".format(
		identifiers["kind"],
	))
