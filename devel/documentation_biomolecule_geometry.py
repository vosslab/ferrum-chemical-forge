"""Ferrum-owned RDKit geometry preparation and admission for biomolecule captures."""


# Standard Library
import math

# PIP3 modules
import ferrum_chem

# local repo modules
from documentation_biomolecule_sources import (
	DISTEAROYLPHOSPHATIDYLCHOLINE_SMILES, DNA_BASE_PAIR_CDML,
)
from ferrum_qt.documentation_capture_surfaces import CaptureError


DSPC_DOCUMENT_BOND_LENGTH = 22.0


#============================================
def _angle_degrees(center: object, first: object, second: object) -> float:
	"""Return one interior projected angle for deterministic depiction admission."""
	first_x = first.position.x - center.position.x
	first_y = first.position.y - center.position.y
	second_x = second.position.x - center.position.x
	second_y = second.position.y - center.position.y
	denominator = math.hypot(first_x, first_y) * math.hypot(second_x, second_y)
	if denominator <= 0.0:
		raise CaptureError("RDKit depiction contains coincident bonded atom positions")
	cosine = max(-1.0, min(1.0, (first_x * second_x + first_y * second_y) / denominator))
	return math.degrees(math.acos(cosine))

#============================================
def _connected_atoms(molecule: object, atom: object) -> tuple[object, ...]:
	"""Return the projected neighbors of one atom from the native document graph."""
	atom_id = atom.document_object_id
	atoms_by_id = {candidate.document_object_id: candidate for candidate in molecule.atoms}
	neighbors = []
	for bond in molecule.bonds:
		if bond.start.document_object_id == atom_id:
			neighbors.append(atoms_by_id[bond.end.document_object_id])
		elif bond.end.document_object_id == atom_id:
			neighbors.append(atoms_by_id[bond.start.document_object_id])
	return tuple(neighbors)

#============================================
def distearoylphosphatidylcholine_source() -> str:
	"""Generate compact DSPC CDML through Ferrum-owned RDKit and SDF insertion."""
	# ASVS 1.5.1, 1.5.2, and 2.2.1-2.2.3: this fixed topology enters only through
	# Ferrum's typed local SMILES/SDF boundary; no user source or serializer is admitted.
	molecule = ferrum_chem.parse_smiles(DISTEAROYLPHOSPHATIDYLCHOLINE_SMILES)
	record = ferrum_chem.prepare_sdf_record(
		molecule, "Distearoylphosphatidylcholine (PubChem CID 65146)", (),
	)
	sdf = ferrum_chem.records_to_sdf((record,), ferrum_chem.MolblockVersionV1.v2000)
	# The two C18 chains exceed Ferrum's fixed landscape page at the ordinary
	# 40-point insertion scale. Use the public isotropic placement boundary so the
	# complete RDKit depiction fits without changing any relative length or angle.
	placement = ferrum_chem.validate_insertion_placement_v1(
		DSPC_DOCUMENT_BOND_LENGTH, 420.0, 300.0,
	)
	batch = ferrum_chem.prepare_sdf_molecules_v1(sdf, placement)
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	operation = ferrum_chem.DocumentOperationV1.insert_interchange_record_batch_v1(batch)
	prepared = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(0),
	)
	committed = session.commit_session_operation_transition_v1(prepared)
	return committed.observation.snapshot.cdml

#============================================
def assert_dspc_geometry(molecule: object) -> None:
	"""Require uniform bonds, readable glycerol geometry, and zwitterionic DSPC."""
	atoms_by_id = {atom.document_object_id: atom for atom in molecule.atoms}
	for bond in molecule.bonds:
		start = atoms_by_id[bond.start.document_object_id]
		end = atoms_by_id[bond.end.document_object_id]
		length = math.hypot(
			end.position.x - start.position.x,
			end.position.y - start.position.y,
		)
		if abs(length - DSPC_DOCUMENT_BOND_LENGTH) > 0.1:
			raise CaptureError(f"DSPC depiction has a nonuniform bond length: {length!r}")
	charged_atoms = tuple(
		sorted(
			(atom.element, atom.formal_charge)
			for atom in molecule.atoms
			if atom.formal_charge not in (None, 0)
		)
	)
	if charged_atoms != (("N", 1), ("O", -1)):
		raise CaptureError(f"DSPC did not retain its zwitterionic headgroup: {charged_atoms!r}")
	candidates = []
	for atom in molecule.atoms:
		neighbors = _connected_atoms(molecule, atom)
		if len(neighbors) == 2:
			angle = _angle_degrees(atom, neighbors[0], neighbors[1])
			if angle < 105.0 or angle > 135.0:
				raise CaptureError(
				f"DSPC depiction has a distorted two-bond angle: {angle!r}",
			)
		if atom.element == "C" and tuple(sorted(neighbor.element for neighbor in neighbors)) == (
			"C", "C", "O",
		):
			candidates.append((atom, neighbors))
		if atom.element == "C" and len(neighbors) == 3:
			angles = tuple(
				_angle_degrees(atom, neighbors[first], neighbors[second])
				for first, second in ((0, 1), (0, 2), (1, 2))
			)
			if any(angle < 105.0 or angle > 135.0 for angle in angles):
				raise CaptureError(f"DSPC depiction has a distorted carbon angle: {angles!r}")
	if len(candidates) != 1:
		raise CaptureError("DSPC lacks its unique glycerol-center carbon")
	center, neighbors = candidates[0]
	angles = tuple(
		_angle_degrees(center, neighbors[first], neighbors[second])
		for first, second in ((0, 1), (0, 2), (1, 2))
	)
	if any(angle < 105.0 or angle > 135.0 for angle in angles):
		raise CaptureError(f"DSPC glycerol center is not trigonal: {angles!r}")

#============================================
def _molecule_centroid(molecule: object) -> tuple[float, float]:
	"""Return one molecule's exact projected atom centroid."""
	return (
		sum(atom.position.x for atom in molecule.atoms) / len(molecule.atoms),
		sum(atom.position.y for atom in molecule.atoms) / len(molecule.atoms),
	)

#============================================
def _edge_rotation(molecule: object, first_index: int, second_index: int, side: int) -> float:
	"""Orient a molecular edge vertically on the requested side of its centroid."""
	center_x, center_y = _molecule_centroid(molecule)
	first = molecule.atoms[first_index].position
	second = molecule.atoms[second_index].position
	rotation = math.pi / 2.0 - math.atan2(second.y - first.y, second.x - first.x)
	middle_x = (first.x + second.x) / 2.0
	middle_y = (first.y + second.y) / 2.0
	rotated_middle_x = (
		center_x + math.cos(rotation) * (middle_x - center_x)
		- math.sin(rotation) * (middle_y - center_y)
	)
	if (rotated_middle_x - center_x) * side < 0.0:
		rotation += math.pi
	return rotation

#============================================
def _rotate_molecule(session: ferrum_chem.DocumentSession, molecule: object, angle: float) -> None:
	"""Rotate one molecule through Ferrum's live document boundary."""
	snapshot = session.snapshot()
	center_x, center_y = _molecule_centroid(molecule)
	targets = tuple(
		(molecule.document_object_id, atom.document_object_id)
		for atom in molecule.atoms
	)
	session.rotate_live_document_atoms_v1(
		snapshot.revision, snapshot.digest, targets, center_x, center_y, angle,
	)

#============================================
def _translate_molecule(
		session: ferrum_chem.DocumentSession, molecule: object, delta_x: float, delta_y: float,
		) -> None:
	"""Translate one molecular render root through Ferrum's public interaction API."""
	snapshot = session.snapshot()
	interaction = session.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
	query = ferrum_chem.RenderInteractionQueryV1.root(molecule.document_object_id)
	selection = session.select_render_interaction_roots_v1(interaction, None, query)
	gesture = session.begin_render_interaction_translation_v1(
		selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	session.commit_render_interaction_translation_v1(gesture, delta_x, delta_y)

#============================================
def _append_hydrogen_bond_guides(cdml: str, lanes: tuple[tuple[object, object], ...]) -> str:
	"""Append four presentation dashes per actual donor-acceptor coordinate lane."""
	if not cdml.endswith("</cdml>"):
		raise CaptureError("Ferrum geometry operation did not return complete CDML")
	guides = []
	for lane_index, (left, right) in enumerate(lanes, start=1):
		if abs(left.position.y - right.position.y) > 0.001 or right.position.x - left.position.x < 80.0:
			raise CaptureError("Watson-Crick anchors did not form readable horizontal lanes")
		start_x = left.position.x + 8.0
		length = right.position.x - left.position.x - 16.0
		for dash_index in range(4):
			dash_start = start_x + length * dash_index / 4.0
			dash_end = dash_start + length * 0.14
			guides.append(
				f"<polyline id='hbond-{lane_index}-{dash_index + 1}' line_color='#5f6f8f' "
				f"width='1.5'><point x='{dash_start}' y='{left.position.y}'/>"
				f"<point x='{dash_end}' y='{left.position.y}'/></polyline>",
			)
	return f"{cdml[:-7]}{'\n'.join(guides)}\n</cdml>"

#============================================
def _assert_base_pair_geometry(thymine: object, adenine: object) -> None:
	"""Reject malformed carbonyl or fused purine geometry before GUI capture."""
	carbonyl_angles = (
		_angle_degrees(thymine.atoms[1], thymine.atoms[0], thymine.atoms[2]),
		_angle_degrees(thymine.atoms[1], thymine.atoms[2], thymine.atoms[3]),
		_angle_degrees(thymine.atoms[5], thymine.atoms[3], thymine.atoms[6]),
		_angle_degrees(thymine.atoms[5], thymine.atoms[6], thymine.atoms[7]),
	)
	purine_angles = (
		_angle_degrees(adenine.atoms[0], adenine.atoms[1], adenine.atoms[5]),
		_angle_degrees(adenine.atoms[3], adenine.atoms[2], adenine.atoms[4]),
		_angle_degrees(adenine.atoms[9], adenine.atoms[3], adenine.atoms[10]),
		_angle_degrees(adenine.atoms[10], adenine.atoms[9], adenine.atoms[11]),
		_angle_degrees(adenine.atoms[11], adenine.atoms[10], adenine.atoms[4]),
		_angle_degrees(adenine.atoms[4], adenine.atoms[11], adenine.atoms[3]),
		_angle_degrees(adenine.atoms[3], adenine.atoms[4], adenine.atoms[9]),
	)
	if (
		any(angle < 105.0 or angle > 135.0 for angle in carbonyl_angles)
		or any(angle < 95.0 or angle > 140.0 for angle in purine_angles)
		):
		raise CaptureError(
			f"RDKit base-pair depiction has malformed ring or carbonyl angles: "
			f"{carbonyl_angles!r} {purine_angles!r}",
		)

#============================================
def dna_base_pair_source() -> str:
	"""Generate and place the fixed A-T topology through Ferrum's RDKit geometry owner."""
	# ASVS 1.5.1, 1.5.2, and 2.2.1-2.2.3: the topology is a repository constant;
	# all geometry operations use typed Ferrum/PyO3 values rather than a new parser.
	session = ferrum_chem.DocumentSession.load(DNA_BASE_PAIR_CDML)
	observation = session.observe(0)
	molecule_ids = tuple(molecule.document_object_id for molecule in observation.projection.molecules)
	prepared = ferrum_chem.prepare_clean_geometry_v1(observation, molecule_ids, 40.0)
	session.apply_clean_geometry_v1(0, prepared)
	observation = session.observe(1)
	thymine, adenine = observation.projection.molecules
	_rotate_molecule(session, thymine, _edge_rotation(thymine, 4, 6, 1))
	observation = session.observe(2)
	thymine, adenine = observation.projection.molecules
	_rotate_molecule(session, adenine, _edge_rotation(adenine, 0, 7, -1))
	observation = session.observe(3)
	thymine, adenine = observation.projection.molecules
	delta_y = (
		(thymine.atoms[4].position.y + thymine.atoms[6].position.y)
		- adenine.atoms[0].position.y - adenine.atoms[7].position.y
	) / 2.0
	# Keep the full purine inside the fixed capture canvas while preserving both lanes.
	_translate_molecule(session, thymine, -70.0, 0.0)
	observation = session.observe(4)
	thymine, adenine = observation.projection.molecules
	_translate_molecule(session, adenine, -70.0, delta_y)
	observation = session.observe(5)
	thymine, adenine = observation.projection.molecules
	_assert_base_pair_geometry(thymine, adenine)
	return _append_hydrogen_bond_guides(
		observation.snapshot.cdml,
		((thymine.atoms[4], adenine.atoms[0]), (thymine.atoms[6], adenine.atoms[7])),
	)
