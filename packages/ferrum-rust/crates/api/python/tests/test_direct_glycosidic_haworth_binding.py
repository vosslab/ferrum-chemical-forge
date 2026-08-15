"""Installed private behavior for Rust-owned direct-glycosidic Haworth insertion."""

import ferrum_chem


SMILES = "O1CCCC1OC2CCCCO2"


def _is_direct_glycosidic_profile(molecule: object) -> bool:
	"""Recognize the closed two-ring bridge and Haworth-depth contract semantically."""
	atoms = {atom.source_id: atom for atom in molecule.atoms}
	if not atoms or any(atom.element not in {"C", "O"} for atom in atoms.values()):
		return False
	adjacency = {identifier: [] for identifier in atoms}
	for bond in molecule.bonds:
		if bond.start.source_id not in atoms or bond.end.source_id not in atoms:
			return False
		adjacency[bond.start.source_id].append((bond.end.source_id, bond))
		adjacency[bond.end.source_id].append((bond.start.source_id, bond))
	bridges = [
		identifier
		for identifier, neighbors in adjacency.items()
		if atoms[identifier].element == "O" and len(neighbors) == 2
		and all(atoms[neighbor].element == "C" for neighbor, _ in neighbors)
		and all(
			bond.source_type == "n1" and bond.haworth_position is None
			for _, bond in neighbors
		)
	]
	if len(bridges) != 1:
		return False
	bridge = bridges[0]
	bridge_bonds = {bond.source_id for _, bond in adjacency[bridge]}
	if any(
		bond.source_type != "n1" or bond.haworth_position is not None
		for _, bond in adjacency[bridge]
	):
		return False
	remaining = set(atoms) - {bridge}
	components: list[set[str]] = []
	while remaining:
		component = {remaining.pop()}
		frontier = list(component)
		while frontier:
			current = frontier.pop()
			for neighbor, bond in adjacency[current]:
				if bond.source_id in bridge_bonds or neighbor not in remaining:
					continue
				remaining.remove(neighbor)
				component.add(neighbor)
				frontier.append(neighbor)
		components.append(component)
	if len(components) != 2:
		return False
	for component in components:
		ring_bonds = [
			bond
			for atom_id in component
			for neighbor, bond in adjacency[atom_id]
			if neighbor in component and atom_id < neighbor
		]
		if (
			len(component) not in {5, 6}
			or sum(atoms[atom_id].element == "O" for atom_id in component) != 1
			or len(ring_bonds) != len(component)
		):
			return False
		if not all(
			sum(neighbor in component for neighbor, _ in adjacency[atom_id]) == 2
			for atom_id in component
		):
			return False
		front_strokes = [
			bond for bond in ring_bonds
			if bond.source_type == "q1"
			and bond.haworth_position == ferrum_chem.DocumentHaworthPositionV1.front
		]
		front_wedges = [
			bond for bond in ring_bonds
			if bond.source_type == "w1"
			and bond.haworth_position == ferrum_chem.DocumentHaworthPositionV1.front
		]
		if len(front_strokes) != 1 or len(front_wedges) != 2:
			return False
		if any(
			bond.source_type != "n1"
			or bond.haworth_position != ferrum_chem.DocumentHaworthPositionV1.back
			for bond in ring_bonds
			if bond not in front_strokes and bond not in front_wedges
		):
			return False
	return True


def test_private_direct_haworth_receipt_previews_and_commits_rust_owned_profile() -> None:
	"""One parsed receipt previews and commits one semantic direct Haworth drawing."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	parsed = ferrum_chem.prepare_direct_haworth_from_smiles_v1(SMILES)
	prepared = session.prepare_create_direct_haworth_v1(0, parsed, 13.0, -7.0)
	result = session.commit_create_direct_haworth_v1(0, prepared)
	molecule = next(
		candidate
		for candidate in result.observation.projection.molecules
		if candidate.source_id == prepared.molecule_identifier
	)

	assert parsed.local_scale > 0.0
	assert {batch.display_layer for batch in prepared.preview_batches} == {
		"ordinary", "haworth_front_stroke", "haworth_front_wedge",
	}
	assert _is_direct_glycosidic_profile(molecule)


def test_private_direct_haworth_profile_refusal_preserves_document() -> None:
	"""A non-profile structural request returns a typed refusal without mutation."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	baseline = session.snapshot()
	baseline_facts = (baseline.cdml, baseline.revision, baseline.digest)

	try:
		ferrum_chem.prepare_direct_haworth_from_smiles_v1("O1CCCC1CC2CCCCO2")
	except ferrum_chem.DirectHaworthProfileError as error:
		assert error.reason == (
			"use a neutral, single-bond C/O two-ring structure with one exterior oxygen bridge"
		)
	else:
		raise AssertionError("non-profile request unexpectedly prepared a receipt")

	after = session.snapshot()
	assert (after.cdml, after.revision, after.digest) == baseline_facts
