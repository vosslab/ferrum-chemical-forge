"""Ferrum-owned SMILES geometry preparation for biomolecule documentation captures."""


# Standard Library
import math

# PIP3 modules
import ferrum_chem

# local repo modules
import ferrum_qt.config.geometry_units
from documentation_biomolecule_sources import (
	ADENINE_SMILES, DISTEAROYLPHOSPHATIDYLCHOLINE_SMILES, SUCROSE_SMILES,
	THYMINE_SMILES,
)
from ferrum_qt.documentation_capture_surfaces import CaptureError


#============================================
def _commit(session: ferrum_chem.DocumentSession, operation: object) -> object:
	"""Commit one revision-bound typed operation to a fresh capture document."""
	revision = session.snapshot().revision
	prepared = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(revision),
	)
	return session.commit_session_operation_transition_v1(prepared)

#============================================
def _a2_landscape_session() -> ferrum_chem.DocumentSession:
	"""Create the spacious document needed for canonical-scale lipid depictions."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	changes = (
		ferrum_chem.DocumentPaperPropertyChangeV1.type_name("A2"),
		ferrum_chem.DocumentPaperPropertyChangeV1.orientation(
			ferrum_chem.PaperOrientationV1.landscape,
		),
	)
	_commit(session, ferrum_chem.DocumentOperationV1.set_paper_properties(changes))
	return session

#============================================
def _document_center(session: ferrum_chem.DocumentSession) -> tuple[float, float]:
	"""Return the typed Rust paper center used as the molecule insertion anchor."""
	projection = session.observe(session.snapshot().revision).projection
	page = projection.paper_layout.page
	return ((page.scene_left + page.scene_right) / 2.0, (page.scene_top + page.scene_bottom) / 2.0)

#============================================
def _insert_smiles(session: ferrum_chem.DocumentSession, smiles: str) -> object:
	"""Insert one source molecule via Ferrum's direct native SMILES operation."""
	anchor_x, anchor_y = _document_center(session)
	placement = ferrum_chem.validate_insertion_placement_v1(
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT, anchor_x, anchor_y,
	)
	# The typed operation parses and validates the exact bounded source SMILES.
	# Do not canonicalize first: that would parse a second time and could replace
	# the user-supplied CID 65146 spelling before Ferrum receives it.
	prepared = ferrum_chem.prepare_smiles_molecule_v1(smiles, placement)
	return _commit(session, ferrum_chem.DocumentOperationV1.insert_molecule_v1(prepared))

#============================================
def smiles_document_source(smiles: str) -> str:
	"""Create one A2 landscape CDML document from exactly one source SMILES graph."""
	session = _a2_landscape_session()
	committed = _insert_smiles(session, smiles)
	observation = committed.observation
	molecules = observation.projection.molecules
	if len(molecules) != 1:
		raise CaptureError(
			"single-SMILES documentation source committed "
			f"{len(molecules)!r} molecules instead of exactly one",
		)
	_validate_canonical_scale(molecules[0])
	return observation.snapshot.cdml

#============================================
def sucrose_source() -> str:
	"""Create the CID 5988 sucrose drawing through Ferrum's direct SMILES ingress."""
	return smiles_document_source(SUCROSE_SMILES)

#============================================
def distearoylphosphatidylcholine_source() -> str:
	"""Create user-specified CID 65146 DSPC at the active Ferrum drawing scale."""
	return smiles_document_source(DISTEAROYLPHOSPHATIDYLCHOLINE_SMILES)

#============================================
def _bond_length(start: object, end: object) -> float:
	"""Return one finite rendered bond length."""
	length = math.hypot(end.position.x - start.position.x, end.position.y - start.position.y)
	if not math.isfinite(length) or length <= 0.0:
		raise CaptureError("SMILES depiction contains a nonfinite or coincident bond")
	return length

#============================================
def _angle_degrees(center: object, first: object, second: object) -> float:
	"""Return one interior projected angle for deterministic depiction admission."""
	first_x = first.position.x - center.position.x
	first_y = first.position.y - center.position.y
	second_x = second.position.x - center.position.x
	second_y = second.position.y - center.position.y
	denominator = math.hypot(first_x, first_y) * math.hypot(second_x, second_y)
	if denominator <= 0.0:
		raise CaptureError("SMILES depiction contains coincident bonded atom positions")
	cosine = max(-1.0, min(1.0, (first_x * second_x + first_y * second_y) / denominator))
	return math.degrees(math.acos(cosine))

#============================================
def _connected_atoms(molecule: object, atom: object) -> tuple[object, ...]:
	"""Return one projected atom's topology-derived neighbor set."""
	atoms_by_id = {candidate.document_object_id: candidate for candidate in molecule.atoms}
	atom_id = atom.document_object_id
	neighbors = []
	for bond in molecule.bonds:
		if bond.start.document_object_id == atom_id:
			neighbors.append(atoms_by_id[bond.end.document_object_id])
		elif bond.end.document_object_id == atom_id:
			neighbors.append(atoms_by_id[bond.start.document_object_id])
	return tuple(neighbors)

#============================================
def _validate_canonical_scale(molecule: object) -> None:
	"""Require every finite bond to use the active UI drawing scale."""
	atoms_by_id = {atom.document_object_id: atom for atom in molecule.atoms}
	lengths = tuple(
		_bond_length(atoms_by_id[bond.start.document_object_id], atoms_by_id[bond.end.document_object_id])
		for bond in molecule.bonds
	)
	if not lengths:
		raise CaptureError("SMILES depiction has no bonds")
	mean_length = sum(lengths) / len(lengths)
	target = ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT
	tolerance = target * 0.02
	minimum = min(lengths)
	maximum = max(lengths)
	if any(abs(length - target) > tolerance for length in lengths):
		raise CaptureError(
			"SMILES depiction bond lengths "
			f"(min={minimum!r}, mean={mean_length!r}, max={maximum!r}) "
			f"differ from active scale {target!r} by more than {tolerance!r}",
		)

#============================================
def assert_dspc_geometry(molecule: object) -> None:
	"""Require canonical-scale DSPC, its zwitterion, and a readable glycerol center."""
	_validate_canonical_scale(molecule)
	charged_atoms = tuple(sorted(
		(atom.element, atom.formal_charge)
		for atom in molecule.atoms if atom.formal_charge not in (None, 0)
	))
	if charged_atoms != (("N", 1), ("O", -1)):
		raise CaptureError(f"DSPC did not retain its zwitterionic headgroup: {charged_atoms!r}")
	candidates = []
	for atom in molecule.atoms:
		neighbors = _connected_atoms(molecule, atom)
		if atom.element == "C" and tuple(sorted(neighbor.element for neighbor in neighbors)) == (
			"C", "C", "O",
		):
			candidates.append((atom, neighbors))
	if len(candidates) != 1:
		raise CaptureError("DSPC lacks its unique glycerol-center carbon")
	center, neighbors = candidates[0]
	angles = tuple(
		_angle_degrees(center, neighbors[first], neighbors[second])
		for first, second in ((0, 1), (0, 2), (1, 2))
	)
	if any(angle < 100.0 or angle > 140.0 for angle in angles):
		raise CaptureError(f"DSPC glycerol center is not readable: {angles!r}")

#============================================
def _molecule_centroid(molecule: object) -> tuple[float, float]:
	"""Return one molecule's exact projected atom centroid."""
	return (
		sum(atom.position.x for atom in molecule.atoms) / len(molecule.atoms),
		sum(atom.position.y for atom in molecule.atoms) / len(molecule.atoms),
	)

#============================================
def _rotate_molecule(session: ferrum_chem.DocumentSession, molecule: object, angle: float) -> None:
	"""Rotate one molecule through Ferrum's live document boundary."""
	center_x, center_y = _molecule_centroid(molecule)
	targets = tuple((molecule.document_object_id, atom.document_object_id) for atom in molecule.atoms)
	snapshot = session.snapshot()
	session.rotate_live_document_atoms_v1(
		snapshot.revision, snapshot.digest, targets, center_x, center_y, angle,
	)

#============================================
def _translate_molecule(session: ferrum_chem.DocumentSession, molecule: object,
		delta_x: float, delta_y: float) -> None:
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
def _carbonyl_oxygen_atoms(molecule: object) -> tuple[object, ...]:
	"""Find acceptor oxygens from their carbonyl topology rather than atom ordering."""
	atoms_by_id = {atom.document_object_id: atom for atom in molecule.atoms}
	result = []
	for bond in molecule.bonds:
		start = atoms_by_id[bond.start.document_object_id]
		end = atoms_by_id[bond.end.document_object_id]
		if {start.element, end.element} == {"C", "O"} and bond.source_type in ("n2", "d1"):
			result.append(start if start.element == "O" else end)
	return tuple(result)

#============================================
def _carbonyl_carbon_atoms(molecule: object) -> tuple[object, ...]:
	"""Return the carbon partners of carbonyl oxygens without atom-number assumptions."""
	atoms_by_id = {atom.document_object_id: atom for atom in molecule.atoms}
	result = []
	for bond in molecule.bonds:
		start = atoms_by_id[bond.start.document_object_id]
		end = atoms_by_id[bond.end.document_object_id]
		if {start.element, end.element} == {"C", "O"} and bond.source_type in ("n2", "d1"):
			result.append(start if start.element == "C" else end)
	return tuple(result)

#============================================
def _is_methyl_carbon(molecule: object, atom: object) -> bool:
	"""Recognize thymine's methyl terminus from graph degree and element only."""
	neighbors = _connected_atoms(molecule, atom)
	return atom.element == "C" and len(neighbors) == 1 and neighbors[0].element == "C"

#============================================
def _base_pair_anchors(thymine: object, adenine: object) -> tuple[object, object, object, object]:
	"""Locate T/O4, T/N3, A/N1, and A/N6 from molecular topology."""
	carbonyl_oxygens = _carbonyl_oxygen_atoms(thymine)
	carbonyl_carbons = _carbonyl_carbon_atoms(thymine)
	if len(carbonyl_oxygens) != 2 or len(carbonyl_carbons) != 2:
		raise CaptureError("thymine lacks its two carbonyl acceptors")
	carbonyl_carbon_ids = {atom.document_object_id for atom in carbonyl_carbons}
	thymine_donors = tuple(
		atom for atom in thymine.atoms if atom.element == "N"
		and len(_connected_atoms(thymine, atom)) == 2
		and all(
			neighbor.document_object_id in carbonyl_carbon_ids
			for neighbor in _connected_atoms(thymine, atom)
		)
	)
	adenine_donors = tuple(
		atom for atom in adenine.atoms if atom.element == "N"
		and len(_connected_atoms(adenine, atom)) == 1
	)
	if len(thymine_donors) != 1 or len(adenine_donors) != 1:
		raise CaptureError("base-pair donor topology is ambiguous")
	thymine_donor = thymine_donors[0]
	adenine_donor = adenine_donors[0]
	thymine_acceptors = tuple(
		oxygen for oxygen in carbonyl_oxygens
		if any(
			any(_is_methyl_carbon(thymine, candidate) for candidate in _connected_atoms(thymine, side))
			for side in _connected_atoms(thymine, _connected_atoms(thymine, oxygen)[0])
			if side.element == "C"
		)
	)
	if len(thymine_acceptors) != 1:
		raise CaptureError("thymine O4 topology is ambiguous")
	thymine_acceptor = thymine_acceptors[0]
	adenine_donor_carbon = _connected_atoms(adenine, adenine_donor)[0]
	adenine_acceptors = tuple(
		atom for atom in adenine.atoms if atom.element == "N"
		and atom.document_object_id != adenine_donor.document_object_id
		and len(_connected_atoms(adenine, atom)) == 2
		and any(
			neighbor.document_object_id == adenine_donor_carbon.document_object_id
			for neighbor in _connected_atoms(adenine, atom)
		)
	)
	if len(adenine_acceptors) != 1:
		raise CaptureError("adenine N1 topology is ambiguous")
	adenine_acceptor = adenine_acceptors[0]
	return thymine_acceptor, thymine_donor, adenine_acceptor, adenine_donor

#============================================
def _edge_rotation_for_face(molecule: object, first: object, second: object,
		face_right: bool) -> float:
	"""Make a Watson-Crick edge vertical while placing it on its intended side."""
	edge_angle = math.atan2(
		second.position.y - first.position.y,
		second.position.x - first.position.x,
	)
	rotation = math.pi / 2.0 - edge_angle
	centroid_x, centroid_y = _molecule_centroid(molecule)
	edge_mid_x = (first.position.x + second.position.x) / 2.0
	edge_mid_y = (first.position.y + second.position.y) / 2.0
	face_x = edge_mid_x - centroid_x
	face_y = edge_mid_y - centroid_y
	rotated_face_x = face_x * math.cos(rotation) - face_y * math.sin(rotation)
	if (rotated_face_x > 0.0) != face_right:
		rotation += math.pi
	return rotation

#============================================
def _edge_midpoint(first: object, second: object) -> tuple[float, float]:
	"""Return the exact midpoint of the two Watson-Crick edge anchors."""
	return (
		(first.position.x + second.position.x) / 2.0,
		(first.position.y + second.position.y) / 2.0,
	)

#============================================
def _append_hydrogen_bond_guides(cdml: str, lanes: tuple[tuple[object, object], ...]) -> str:
	"""Append presentation dashes from the final topology-derived anchor positions."""
	if not cdml.endswith("</cdml>"):
		raise CaptureError("Ferrum geometry operation did not return complete CDML")
	guides = []
	for lane_index, (left, right) in enumerate(lanes, start=1):
		if right.position.x - left.position.x < 80.0:
			raise CaptureError("Watson-Crick anchors are too close for readable guides")
		for dash_index in range(4):
			fraction = dash_index / 4.0
			start_x = left.position.x + 8.0 + (right.position.x - left.position.x - 16.0) * fraction
			end_x = start_x + (right.position.x - left.position.x - 16.0) * 0.14
			start_y = left.position.y + (right.position.y - left.position.y) * fraction
			end_y = left.position.y + (right.position.y - left.position.y) * (fraction + 0.14)
			guides.append(
				f"<polyline id='hbond-{lane_index}-{dash_index + 1}' line_color='#5f6f8f' "
				f"width='1.5'><point x='{start_x}' y='{start_y}'/>"
				f"<point x='{end_x}' y='{end_y}'/></polyline>",
			)
	return f"{cdml[:-7]}{'\\n'.join(guides)}\\n</cdml>"

#============================================
def dna_base_pair_source() -> str:
	"""Build a Watson-Crick A-T display from two PubChem SMILES insertions."""
	session = _a2_landscape_session()
	_insert_smiles(session, THYMINE_SMILES)
	_insert_smiles(session, ADENINE_SMILES)
	observation = session.observe(session.snapshot().revision)
	thymine, adenine = observation.projection.molecules
	_validate_canonical_scale(thymine)
	_validate_canonical_scale(adenine)
	left_acceptor, left_donor, right_acceptor, right_donor = _base_pair_anchors(thymine, adenine)
	_rotate_molecule(
		session, thymine,
		_edge_rotation_for_face(thymine, left_acceptor, left_donor, face_right=True),
	)
	observation = session.observe(session.snapshot().revision)
	thymine, adenine = observation.projection.molecules
	left_acceptor, left_donor, right_acceptor, right_donor = _base_pair_anchors(thymine, adenine)
	_rotate_molecule(
		session, adenine,
		_edge_rotation_for_face(adenine, right_acceptor, right_donor, face_right=False),
	)
	observation = session.observe(session.snapshot().revision)
	thymine, adenine = observation.projection.molecules
	left_acceptor, left_donor, right_acceptor, right_donor = _base_pair_anchors(thymine, adenine)
	page_center_x, page_center_y = _document_center(session)
	inter_base_gap = 150.0
	left_mid_x, left_mid_y = _edge_midpoint(left_acceptor, left_donor)
	_translate_molecule(
		session, thymine,
		page_center_x - inter_base_gap / 2.0 - left_mid_x,
		page_center_y - left_mid_y,
	)
	observation = session.observe(session.snapshot().revision)
	thymine, adenine = observation.projection.molecules
	left_acceptor, left_donor, right_acceptor, right_donor = _base_pair_anchors(thymine, adenine)
	right_mid_x, right_mid_y = _edge_midpoint(right_acceptor, right_donor)
	_translate_molecule(
		session, adenine,
		page_center_x + inter_base_gap / 2.0 - right_mid_x,
		page_center_y - right_mid_y,
	)
	observation = session.observe(session.snapshot().revision)
	thymine, adenine = observation.projection.molecules
	left_acceptor, left_donor, right_acceptor, right_donor = _base_pair_anchors(thymine, adenine)
	return _append_hydrogen_bond_guides(
		observation.snapshot.cdml, ((left_acceptor, right_donor), (left_donor, right_acceptor)),
	)
