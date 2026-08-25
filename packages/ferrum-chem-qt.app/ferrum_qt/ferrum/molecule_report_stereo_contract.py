"""Typed P0 stereo receipt validation and display for Molecule Report."""


_TETRAHEDRAL_PARITIES = {"clockwise", "counter_clockwise"}
_DOUBLE_BOND_CONFIGURATIONS = {"e", "z"}
_DIRECTED_BOND_PRESENTATIONS = {"solid_wedge", "hashed_wedge"}
_DOUBLE_BOND_CARRIER_MARKS = {"up", "down"}


#============================================
def valid_stereo_semantics(semantics: object) -> bool:
	"""Accept only complete Rust-issued P0 stereo descriptor collections."""
	return (
		type(semantics) is dict
		and set(semantics) == {"tetrahedral", "double_bonds"}
		and type(semantics["tetrahedral"]) is list
		and all(_valid_tetrahedral(item) for item in semantics["tetrahedral"])
		and type(semantics["double_bonds"]) is list
		and all(_valid_double_bond(item) for item in semantics["double_bonds"])
	)


#============================================
def display_lines(semantics: dict | None, depiction: dict | None) -> list[str]:
	"""Render descriptor facts without calculating chemical meaning in Qt."""
	if semantics is None:
		lines = ["Stereo semantics: none"]
	else:
		lines = ["Stereo semantics:"]
		if not semantics["tetrahedral"] and not semantics["double_bonds"]:
			lines.append("  none")
		for descriptor in semantics["tetrahedral"]:
			ligands = ", ".join(
				"explicit hydrogen" if ligand["kind"] == "explicit_hydrogen" else str(ligand["index"])
				for ligand in descriptor["ligands"]
			)
			lines.append("  tetrahedral atom {0}: [{1}], {2}".format(
				descriptor["center"], ligands, descriptor["parity"],
			))
		for descriptor in semantics["double_bonds"]:
			lines.append("  double bond {0}: ligands {1}/{2}, {3}".format(
				descriptor["bond_index"], descriptor["start_ligand"],
				descriptor["end_ligand"], descriptor["configuration"],
			))
	lines.append("Stereo depiction:" if depiction is not None else "Stereo depiction: none")
	if depiction is not None:
		if not depiction["directed_bonds"] and not depiction["double_bond_carrier_marks"]:
			lines.append("  none")
		for bond in depiction["directed_bonds"]:
			lines.append("  directed bond {0}: {1} -> {2}, {3}".format(
				bond["bond_index"], bond["start"], bond["end"], bond["presentation"],
			))
		for mark in depiction["double_bond_carrier_marks"]:
			lines.append("  double bond carrier: double bond {0}, carrier bond {1}, {2}".format(
				mark["double_bond_index"], mark["carrier_bond_index"], mark["mark"],
			))
	return lines


#============================================
def valid_stereo_depiction(depiction: object) -> bool:
	"""Accept only complete Rust-issued stereo drawing descriptor collections."""
	return (
		type(depiction) is dict
		and set(depiction) == {"directed_bonds", "double_bond_carrier_marks"}
		and type(depiction["directed_bonds"]) is list
		and all(_valid_directed_bond(item) for item in depiction["directed_bonds"])
		and type(depiction["double_bond_carrier_marks"]) is list
		and all(_valid_double_bond_carrier_mark(item) for item in depiction["double_bond_carrier_marks"])
	)


#============================================
def _valid_tetrahedral(item: object) -> bool:
	"""Validate one ordered tetrahedral descriptor without deriving parity."""
	return (
		type(item) is dict
		and set(item) == {"center", "ligands", "parity"}
		and type(item["center"]) is int and item["center"] >= 0
		and type(item["ligands"]) is list and len(item["ligands"]) == 4
		and all(_valid_ligand(ligand) for ligand in item["ligands"])
		and type(item["parity"]) is str
		and item["parity"] in _TETRAHEDRAL_PARITIES
	)


#============================================
def _valid_ligand(ligand: object) -> bool:
	"""Validate an atom-index ligand or the explicit-hydrogen sentinel."""
	if type(ligand) is not dict:
		return False
	if ligand.get("kind") == "atom":
		return set(ligand) == {"kind", "index"} and type(ligand.get("index")) is int and ligand["index"] >= 0
	return ligand == {"kind": "explicit_hydrogen"}


#============================================
def _valid_double_bond(item: object) -> bool:
	"""Validate one typed E/Z descriptor without interpreting its configuration."""
	return (
		type(item) is dict
		and set(item) == {"bond_index", "start_ligand", "end_ligand", "configuration"}
		and all(type(item[name]) is int and item[name] >= 0 for name in (
			"bond_index", "start_ligand", "end_ligand",
		))
		and type(item["configuration"]) is str
		and item["configuration"] in _DOUBLE_BOND_CONFIGURATIONS
	)


#============================================
def _valid_directed_bond(item: object) -> bool:
	"""Validate one Rust-issued tetrahedral drawing fact without deriving parity."""
	return (
		type(item) is dict
		and set(item) == {"bond_index", "start", "end", "presentation"}
		and all(type(item[name]) is int and item[name] >= 0 for name in (
			"bond_index", "start", "end",
		))
		and item["start"] != item["end"]
		and type(item["presentation"]) is str
		and item["presentation"] in _DIRECTED_BOND_PRESENTATIONS
	)


#============================================
def _valid_double_bond_carrier_mark(item: object) -> bool:
	"""Validate a Rust-issued E/Z carrier mark without deriving configuration."""
	return (
		type(item) is dict
		and set(item) == {"double_bond_index", "carrier_bond_index", "mark"}
		and all(type(item[name]) is int and item[name] >= 0 for name in (
			"double_bond_index", "carrier_bond_index",
		))
		and type(item["mark"]) is str
		and item["mark"] in _DOUBLE_BOND_CARRIER_MARKS
	)
