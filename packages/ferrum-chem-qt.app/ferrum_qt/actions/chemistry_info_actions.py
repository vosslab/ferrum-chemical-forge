"""Read-only chemistry queries and synchronized molecule edits."""

import PySide6.QtWidgets

import ferrum_qt.bridge.oasa_bridge
import ferrum_qt.models.document_session

def _get_mols_for_info(app: object) -> list:
	"""Return selected molecules, or all document molecules if none selected.

	Args:
		app: MainWindow instance with document attribute.

	Returns:
		List of MoleculeModel instances.
	"""
	mols = app.document.selected_mols
	if not mols:
		mols = app.document.molecules
	return mols


#============================================
def _compute_formula(mol: object) -> str:
	"""Compute the molecular formula string for a MoleculeModel.

	Uses Hill system ordering: C first, H second, then alphabetical.

	Args:
		mol: MoleculeModel instance.

	Returns:
		Formula string, e.g. 'C6H12O6'.
	"""
	symbols = tuple(atom_model.symbol for atom_model in mol.atoms)
	return ferrum_qt.bridge.oasa_bridge.molecule_summary_facts(symbols).formula


#============================================
def _compute_molecular_weight(mol: object) -> float:
	"""Compute the molecular weight for a MoleculeModel.

	Sums atomic weights from the OASA periodic table for each atom.

	Args:
		mol: MoleculeModel instance.

	Returns:
		Molecular weight as a float.
	"""
	symbols = tuple(atom_model.symbol for atom_model in mol.atoms)
	return ferrum_qt.bridge.oasa_bridge.molecule_summary_facts(symbols).molecular_weight


#============================================
def _chemistry_info(app: object) -> None:
	"""Display summary info on selected (or all) molecules.

	Shows atom count, bond count, formula, and molecular weight for
	each molecule in a QMessageBox.

	Args:
		app: MainWindow instance.
	"""
	mols = _get_mols_for_info(app)
	if not mols:
		PySide6.QtWidgets.QMessageBox.information(
			app, "Molecule Info", "No molecules in the document."
		)
		return
	# build info text for each molecule
	lines = []
	for idx, mol in enumerate(mols, start=1):
		n_atoms = len(mol.atoms)
		n_bonds = len(mol.bonds)
		formula = _compute_formula(mol)
		mw = _compute_molecular_weight(mol)
		mol_name = mol.name if mol.name else f"Molecule {idx}"
		lines.append(f"--- {mol_name} ---")
		lines.append(f"  Atoms: {n_atoms}")
		lines.append(f"  Bonds: {n_bonds}")
		lines.append(f"  Formula: {formula}")
		lines.append(f"  Molecular weight: {mw:.2f}")
		lines.append("")
	info_text = "\n".join(lines)
	PySide6.QtWidgets.QMessageBox.information(
		app, "Molecule Info", info_text
	)


#============================================
def _chemistry_check(app: object) -> None:
	"""Check selected molecules for valency violations.

	For each selected molecule, checks each atom's free_valency.
	Reports any atoms with free_valency < 0 in a QMessageBox.

	Args:
		app: MainWindow instance.
	"""
	session = _active_smiles_document_session(app)
	if session is not None:
		if not session.can_write_authoritative_snapshot:
			PySide6.QtWidgets.QMessageBox.warning(
				app, "Check Chemistry",
				"Chemistry check is unavailable until the document projection recovers.",
			)
			return
		_selected_chemistry_check(app, session)
		return
	mols = app.document.selected_mols
	if not mols:
		PySide6.QtWidgets.QMessageBox.information(
			app, "Check Chemistry", "No molecules selected."
		)
		return
	# check each atom in each molecule
	problems = []
	for mol in mols:
		chemistry = ferrum_qt.bridge.oasa_bridge.standalone_atom_chemistry(mol)
		for atom_model in mol.atoms:
			free_val, _oxidation_number = chemistry[id(atom_model)]
			if free_val < 0:
				msg = (
					f"Atom {atom_model.symbol} "
					f"(charge={atom_model.charge}): "
					f"free valency = {free_val}"
				)
				problems.append(msg)
	if problems:
		result_text = "Valency violations found:\n\n" + "\n".join(problems)
	else:
		result_text = "All atoms pass valency check."
	PySide6.QtWidgets.QMessageBox.information(
		app, "Check Chemistry", result_text
	)


#============================================
def _selected_chemistry_check(app: object, session: object) -> None:
	"""Render one synchronized complete-graph valency observation."""
	molecule_ids = session.document.selected_direct_root_molecule_ids
	if not molecule_ids:
		PySide6.QtWidgets.QMessageBox.information(
			app, "Check Chemistry", "No molecules selected.",
		)
		return
	response = ferrum_qt.bridge.oasa_bridge.observe_atom_chemistry_facts(
		session, session.backend_snapshot.revision,
	)
	if response.failure is not None:
		message = response.failure.message
		if response.failure.kind == "revision-conflict":
			message = "Document changed; try again.\n%s" % message
		PySide6.QtWidgets.QMessageBox.warning(app, "Check Chemistry", message)
		return
	observation = response.value
	if observation is None:
		return
	selected = set(molecule_ids)
	problems = [
		"Atom %s: free valency = %d" % (_atom_chemistry_display_label(record), record.free_valency)
		for record in observation.records
		if record.molecule_id in selected and record.disposition == "usable"
		and record.free_valency is not None and record.free_valency < 0
	]
	unavailable = [record for record in observation.records
		if record.molecule_id in selected and record.disposition != "usable"]
	if problems:
		text = "Valency violations found:\n\n" + "\n".join(problems)
	elif unavailable:
		text = "Chemistry check is unavailable for one or more selected atoms."
	else:
		text = "All selected atoms pass the OASA complete-graph valency check."
	PySide6.QtWidgets.QMessageBox.information(app, "Check Chemistry", text)


#============================================
def _expand_groups(app: object) -> None:
	"""Expand one current implicit group through the owning backend session."""
	session = _active_smiles_document_session(app)
	document = getattr(app, "document", None)
	if session is None or not session.can_write_authoritative_snapshot or document is None:
		return
	groups = tuple(document.selected_groups)
	if len(groups) != 1:
		return
	item = groups[0]
	model = getattr(item, "group_model", None)
	molecule = model.parent() if model is not None else None
	if (
		not document.is_current_projection_item(item)
		or model is None
		or not model.implicit_expandable
		or type(model.group_id) is not str or not model.group_id
		or molecule not in document.molecules
		or type(getattr(molecule, "mol_id", None)) is not str or not molecule.mol_id
	):
		return
	expected_revision = session.backend_snapshot.revision
	molecule_id = molecule.mol_id
	group_id = model.group_id
	submit = session.submit_implicit_group_expand
	del groups
	del item
	del model
	del molecule
	del document
	outcome = submit(expected_revision, molecule_id, group_id)
	if outcome.status != "accepted":
		PySide6.QtWidgets.QMessageBox.warning(app, "Expand Group", outcome.message)


#============================================
def _int_to_roman_oxidation(n: int) -> str:
	"""Convert an integer to a signed Roman numeral oxidation state string.

	Standard chemistry convention: +III, -II, 0, etc.

	Args:
		n: Integer oxidation number.

	Returns:
		Signed Roman numeral string (e.g. -2 -> '-II', +3 -> '+III', 0 -> '0').
	"""
	if n == 0:
		return "0"
	# build the roman numeral from the absolute value
	abs_n = abs(n)
	roman_pairs = [
		(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
		(100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
		(10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
	]
	parts = []
	for value, numeral in roman_pairs:
		while abs_n >= value:
			parts.append(numeral)
			abs_n -= value
	sign = "+" if n > 0 else "-"
	roman_str = sign + "".join(parts)
	return roman_str


#============================================
def _atom_chemistry_display_label(record: object) -> str:
	"""Return a plain backend-owned label without consulting Qt atom state."""
	symbol = record.symbol or "atom"
	identifier = record.atom_id or "unnamed"
	label = "%s (%s)" % (symbol, identifier)
	if record.charge is not None and record.charge != 0:
		label += ", charge=%+d" % record.charge
	return label


#============================================
def _oxidation_number(app: object) -> None:
	"""Compute and display oxidation numbers for atoms in selected molecules.

	For each selected molecule, iterates atoms and computes the oxidation
	number via the OASA electronegativity-based algorithm. Results are
	displayed in a QMessageBox with per-molecule headings.

	Args:
		app: MainWindow instance.
	"""
	session = _active_smiles_document_session(app)
	if session is not None:
		if not session.can_write_authoritative_snapshot:
			PySide6.QtWidgets.QMessageBox.warning(
				app, "Oxidation Number",
				"Oxidation results are unavailable until the document projection recovers.",
			)
			return
		_selected_oxidation_number(app, session)
		return
	mols = app.document.selected_mols
	if not mols:
		PySide6.QtWidgets.QMessageBox.information(
			app, "Oxidation Number", "No molecules selected."
		)
		return
	# build result text for each molecule
	lines = []
	for idx, mol in enumerate(mols, start=1):
		mol_name = mol.name if mol.name else f"Molecule {idx}"
		lines.append(f"--- {mol_name} ---")
		chemistry = ferrum_qt.bridge.oasa_bridge.standalone_atom_chemistry(mol)
		for atom_model in mol.atoms:
			_free_valency, ox_num = chemistry[id(atom_model)]
			roman = _int_to_roman_oxidation(ox_num)
			lines.append(f"  {atom_model.symbol}: {roman}")
		lines.append("")
	result_text = "\n".join(lines)
	PySide6.QtWidgets.QMessageBox.information(
		app, "Oxidation Number", result_text
	)


#============================================
def _selected_oxidation_number(app: object, session: object) -> None:
	"""Render one synchronized OASA-derived oxidation observation."""
	molecule_ids = session.document.selected_direct_root_molecule_ids
	if not molecule_ids:
		PySide6.QtWidgets.QMessageBox.information(
			app, "Oxidation Number", "No molecules selected.",
		)
		return
	response = ferrum_qt.bridge.oasa_bridge.observe_atom_chemistry_facts(
		session, session.backend_snapshot.revision,
	)
	if response.failure is not None:
		message = response.failure.message
		if response.failure.kind == "revision-conflict":
			message = "Document changed; try again.\n%s" % message
		PySide6.QtWidgets.QMessageBox.warning(app, "Oxidation Number", message)
		return
	observation = response.value
	if observation is None:
		return
	selected = set(molecule_ids)
	lines = ["OASA-derived electronegativity results (not universal formal assignments):"]
	for record in observation.records:
		if record.molecule_id not in selected or record.disposition != "usable":
			continue
		if record.oxidation_number is not None:
			lines.append("  %s: %s" % (
				_atom_chemistry_display_label(record), _int_to_roman_oxidation(record.oxidation_number),
			))
	if len(lines) == 1:
		lines.append("  No selected atoms have an available complete-graph result.")
	PySide6.QtWidgets.QMessageBox.information(
		app, "Oxidation Number", "\n".join(lines),
	)


#============================================
def _active_smiles_document_session(app: object) -> object | None:
	"""Return the live registered session that owns the active projection."""
	session = getattr(app, "_active_session", None)
	document = getattr(app, "document", None)
	scene = getattr(app, "scene", None)
	view = getattr(app, "view", None)
	sessions = getattr(app, "sessions", ())
	if session is None or document is None or scene is None or view is None:
		return None
	if session.is_disposed or session not in sessions:
		return None
	if (
		session.document is not document
		or session.scene is not scene
		or session.view is not view
	):
		return None
	return session


#============================================
def _gen_smiles(app: object) -> None:
	"""Export one selected direct-root molecule through authoritative CDML.

	A selected compatibility child may resolve its durable direct-root molecule,
	but the request never contains a child identifier.

	Copies the SMILES string to the clipboard and displays it
	in a dialog.

	Args:
		app: MainWindow instance.
	"""
	session = _active_smiles_document_session(app)
	if session is None:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Export SMILES",
			"SMILES export requires an active synchronized document session.",
		)
		return
	if not session.can_write_authoritative_snapshot:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Export SMILES",
			"SMILES export is unavailable until the document projection recovers.",
		)
		return
	molecule_ids = session.document.selected_direct_root_molecule_ids
	if len(molecule_ids) != 1:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Export SMILES",
			"Please select exactly one molecule and no presentation objects."
		)
		return
	response = ferrum_qt.bridge.oasa_bridge.query_molecule_smiles(
		session, session.backend_snapshot.revision, molecule_ids[0],
	)
	if response.failure is not None:
		failure = response.failure
		if failure.kind == "projection-unavailable":
			title = "Export SMILES"
			message = "SMILES export is unavailable until the document projection recovers."
		elif failure.kind == "revision-conflict":
			title = "Export SMILES"
			message = "SMILES export used an older document revision. Please try again:\n%s" % failure.message
		elif failure.kind == "unavailable":
			title = "Export SMILES"
			message = "SMILES export is unavailable for this molecule:\n%s" % failure.message
		else:
			title = "SMILES Export Error"
			message = "Failed to generate SMILES:\n%s" % failure.message
		PySide6.QtWidgets.QMessageBox.warning(app, title, message)
		return
	result = response.value
	if result is None:
		return
	smiles_str = result.smiles
	# copy to clipboard
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	clipboard.setText(smiles_str)
	# show in dialog
	PySide6.QtWidgets.QMessageBox.information(
		app, "Export SMILES",
		f"SMILES (copied to clipboard):\n\n{smiles_str}"
	)


#============================================
def _set_name(app: object) -> None:
	"""Set one selected direct-root molecule name through its backend session."""
	session = _active_smiles_document_session(app)
	if session is None or not session.can_write_authoritative_snapshot:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Set Molecule Name",
			"Set molecule name requires an active synchronized document session.",
		)
		return
	molecule_ids = session.document.selected_direct_root_molecule_ids
	if len(molecule_ids) != 1:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Set Molecule Name", "Please select exactly one molecule.",
		)
		return
	molecule_id = molecule_ids[0]
	mol = next(
		(molecule for molecule in session.document.molecules if molecule.mol_id == molecule_id),
		None,
	)
	if mol is None:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Set Molecule Name", "Selected molecule is unavailable in this projection.",
		)
		return
	current_name = mol.name or ""
	new_name, ok = PySide6.QtWidgets.QInputDialog.getText(
		app, "Set Molecule Name", "Molecule name:",
		text=current_name
	)
	if not ok:
		return
	snapshot = session.backend_snapshot
	request = ferrum_qt.models.document_session.build_molecule_name_request(
		snapshot.revision, molecule_id, new_name,
	)
	outcome = session.submit_persistent_operation(request)
	if outcome.status != "accepted":
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Set Molecule Name", outcome.message,
		)


#============================================
