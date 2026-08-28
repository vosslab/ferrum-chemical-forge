"""Pure receipt-to-text presentation helpers for Molecule Report."""

# local repo modules
import ferrum_qt.ferrum.molecule_report_identifier_contract
import ferrum_qt.ferrum.molecule_report_stereo_contract


#============================================
def record_text(record: dict) -> str:
	"""Format record facts supplied by Rust without deriving new chemistry."""
	name = record["authored_name"]
	label = "(unnamed)" if name is None else name
	elements = ", ".join(
		"{0}: {1}".format(entry["symbol"], entry["atom_count"])
		for entry in record["authored_elements"]
	)
	charge = record["authored_charge"]
	charge_text = "not completely authored" if charge is None else "{0:+d}".format(charge)
	lines = [
		"Name: {0}".format(label),
		"Authored graph: {0} atoms, {1} bonds".format(
			record["atom_count"], record["bond_count"],
		),
		"Authored elements: {0}".format(elements),
		"Complete authored formal charge: {0}".format(charge_text),
	]
	lines.extend(ferrum_qt.ferrum.molecule_report_stereo_contract.display_lines(
		record["stereo_semantics"], record["stereo_depiction"],
	))
	lines.extend(ferrum_qt.ferrum.molecule_report_identifier_contract.display_lines(
		record["identifiers"],
	))
	composition = record["composition"]
	if composition is None:
		lines.append("Composition: unavailable (see diagnostics)")
	else:
		lines.extend(composition_lines(composition))
	lines.append("Neutral bond-capacity result: {0}".format(record["neutral_bond_capacity"]))
	text = "\n".join(lines)
	return text


#============================================
def composition_lines(composition: dict) -> list[str]:
	"""Render one complete Rust composition DTO without recalculating its facts."""
	lines = [
		"Formula: {0}".format(composition["formula"]),
		"Net formal charge: {0:+d}".format(composition["net_formal_charge"]),
		"Average molecular weight: {0:.6f} Da".format(
			composition["average_molecular_weight_da"],
		),
		"Monoisotopic mass: {0:.6f} Da".format(
			composition["monoisotopic_mass_da"],
		),
		"Isotope-aware element contributions:",
	]
	for element in composition["elements"]:
		isotope = element["isotope"]
		isotope_label = element["symbol"] if isotope is None else "{0}{1}".format(
			isotope, element["symbol"],
		)
		lines.append("  {0}: {1} atoms; {2:.6f} Da ({3:.4f}%)".format(
			isotope_label,
			element["atom_count"],
			element["average_mass_contribution_da"],
			element["mass_percentage"],
		))
	return lines


#============================================
def aggregate_text(aggregate: dict) -> str:
	"""Render the tagged Rust aggregate outcome without interpreting its chemistry."""
	kind = aggregate["kind"]
	if kind == "complete":
		lines = ["Aggregate composition: complete"]
		lines.extend(composition_lines(aggregate["composition"]))
		text = "\n".join(lines)
		return text
	if kind == "omitted":
		lines = [
			"Aggregate composition: omitted",
			"Reason: {0}".format(aggregate["reason"]),
			"Recovery: {0}".format(aggregate["recovery"]),
		]
		text = "\n".join(lines)
		return text
	raise ValueError("unknown Rust molecule-report aggregate outcome: {0}".format(kind))


#============================================
def finding_text(finding: dict) -> str:
	"""Present one complete ordered Rust finding without deriving chemistry in Qt."""
	lines = [
		"Severity: {0}".format(finding["severity"]),
		"Code: {0}".format(finding["code"]),
		"Location: {0}".format(finding_location_text(finding["location"])),
		"Recovery: {0}".format(finding["recovery"]),
	]
	if finding["detail"] is not None:
		lines.append("Detail: {0}".format(finding["detail"]))
	text = "\n".join(lines)
	return text


#============================================
def finding_location_text(location: dict) -> str:
	"""Render one authenticated diagnostic location without locating scene items."""
	kind = location["kind"]
	if kind == "root":
		text = "root"
	elif kind == "unaddressable":
		text = "unaddressable {0}".format(location["subject"])
	else:
		text = "{0}: {1}".format(kind, location["identifier"])
	return text
