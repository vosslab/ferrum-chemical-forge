"""Durable identity allocation helpers for the Document facade."""

# local repo modules
import ferrum_qt.io.cdml_inspection
import ferrum_qt.models.molecule_model


#============================================
class DocumentIdentity:
	def unique_cdml_id(self, prefix: str) -> str:
		"""Return a document-global CDML identifier using a stable prefix.

		IDs share one XML namespace across molecule metadata, atoms, bonds, and
		top-level objects.  Retained raw fragments are parsed safely so a new
		editable fragment cannot silently collide with lossless raw XML.
		"""
		used = self._used_cdml_ids()
		index = 1
		candidate = "%s%d" % (prefix, index)
		while candidate in used:
			index += 1
			candidate = "%s%d" % (prefix, index)
		return candidate

	#============================================
	def planned_fragment_id_changes(
			self, molecule: ferrum_qt.models.molecule_model.MoleculeModel,
			) -> tuple[tuple[tuple[object, str, str], ...], tuple[tuple[object, str, str], ...]]:
		"""Plan global-safe atom and bond IDs without mutating live models."""
		used = self._used_cdml_ids(exclude_molecule=molecule)
		if molecule.mol_id:
			used.add(molecule.mol_id)
		used.update(fragment.fragment_id for fragment in molecule.fragments)
		for raw_xml in molecule.unsupported_fragment_xml:
			used.update(self._raw_fragment_ids(raw_xml))
		atom_changes = self._planned_model_ids(molecule.atoms, "atom_id", "atom", used)
		bond_changes = self._planned_bond_ids(molecule.bonds, used)
		return tuple(atom_changes), tuple(bond_changes)

	#============================================
	def _used_cdml_ids(
			self, exclude_molecule: ferrum_qt.models.molecule_model.MoleculeModel | None = None,
			) -> set[str]:
		"""Collect IDs from durable projected and retained document content."""
		used = set()
		for molecule in self._molecules:
			if molecule is exclude_molecule:
				continue
			if molecule.mol_id:
				used.add(molecule.mol_id)
			for atom_model in molecule.atoms:
				identifier = str(atom_model.atom_id or "")
				if identifier:
					used.add(identifier)
			for bond_model in molecule.bonds:
				identifier = str(bond_model.bond_id or "")
				if identifier:
					used.add(identifier)
			for group_model in molecule.groups:
				if group_model.group_id:
					used.add(group_model.group_id)
			for fragment in molecule.fragments:
				used.add(fragment.fragment_id)
			for raw_xml in molecule.unsupported_fragment_xml:
				used.update(self._raw_fragment_ids(raw_xml))
		for object_model in self._presentation_objects:
			if object_model.object_id:
				used.add(object_model.object_id)
		for unsupported in self._unsupported_content:
			if unsupported.object_id:
				used.add(unsupported.object_id)
		return used

	#============================================
	def _raw_fragment_ids(self, raw_xml: str) -> set[str]:
		"""Read retained raw fragment IDs without treating XML as text."""
		identifier = ferrum_qt.io.cdml_inspection.root_id(raw_xml)
		return {identifier} if identifier is not None else set()

	#============================================
	def _planned_model_ids(
			self, models: list, chemistry_name: str, prefix: str, used: set[str],
			) -> list[tuple[object, str, str]]:
		"""Return deterministic before/after IDs while reserving each result."""
		changes = []
		for model in models:
			before = str(getattr(model, chemistry_name) or "")
			after = before
			if not after or after in used:
				index = 1
				after = "%s%d" % (prefix, index)
				while after in used:
					index += 1
					after = "%s%d" % (prefix, index)
			used.add(after)
			changes.append((model, before, after))
		return changes

	#============================================
	def _planned_bond_ids(
			self, models: list, used: set[str],
			) -> list[tuple[object, str, str]]:
		"""Return deterministic scalar BondModel ID assignments."""
		changes = []
		for model in models:
			before = str(model.bond_id or "")
			after = before
			if not after or after in used:
				index = 1
				after = "bond%d" % index
				while after in used:
					index += 1
					after = "bond%d" % index
			used.add(after)
			changes.append((model, before, after))
		return changes
