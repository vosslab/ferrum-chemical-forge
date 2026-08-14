"""Typed public API for Ferrum-Chem's compiled document extension."""

import pathlib
from typing import ClassVar


class FerrumError(Exception): ...


class GeometryError(FerrumError): ...


class PeriodicDisplayError(FerrumError): ...


class UnknownElementDisplaySymbolError(PeriodicDisplayError):
	"""Raised when a symbol is outside the exact periodic-picker V1 contract."""
	symbol: str


class ElementDisplayCategoryV1:
	nonmetal: ClassVar[ElementDisplayCategoryV1]
	halogen: ClassVar[ElementDisplayCategoryV1]
	noble_gas: ClassVar[ElementDisplayCategoryV1]
	metalloid: ClassVar[ElementDisplayCategoryV1]
	metal: ClassVar[ElementDisplayCategoryV1]
	transition_metal: ClassVar[ElementDisplayCategoryV1]
	lanthanide: ClassVar[ElementDisplayCategoryV1]
	actinide: ClassVar[ElementDisplayCategoryV1]


class ElementDisplayFactsV1:
	symbol: str
	category: ElementDisplayCategoryV1
	color: str


class PeriodicDisplayCatalogProvenanceV1:
	catalog_id: str
	revision: str
	source: str
	scope: str


def periodic_display_facts_v1(symbol: str) -> ElementDisplayFactsV1: ...
def periodic_display_entries_v1() -> tuple[ElementDisplayFactsV1, ...]: ...
def periodic_display_catalog_provenance_v1() -> PeriodicDisplayCatalogProvenanceV1: ...


class PaperDimensionsMmV1:
	width: float
	height: float


class PaperSizeV1:
	name: str
	dimensions: PaperDimensionsMmV1 | None


class PaperOrientationV1:
	portrait: ClassVar[PaperOrientationV1]
	landscape: ClassVar[PaperOrientationV1]


class PaperPageIssueV1:
	unsupported_type: ClassVar[PaperPageIssueV1]
	unsupported_orientation: ClassVar[PaperPageIssueV1]
	invalid_custom_dimensions: ClassVar[PaperPageIssueV1]


class PaperPageV1:
	width_mm: float
	height_mm: float
	scene_left: float
	scene_top: float
	scene_right: float
	scene_bottom: float
	issue: PaperPageIssueV1 | None


class PaperAttributesV1:
	id: str | None
	type_name: str | None
	orientation: str | None
	crop_svg: str | None
	crop_margin: str | None
	use_real_minus: str | None
	replace_minus: str | None
	size_x: str | None
	size_y: str | None


class ViewportAttributesV1:
	id: str | None
	viewport: str | None


class PaperLayoutProjectionV1:
	schema: str
	revision: int
	digest: str
	paper_present: bool
	paper_attributes: PaperAttributesV1
	effective_paper_attributes: PaperAttributesV1
	viewport_attributes: ViewportAttributesV1
	default_type: str
	default_orientation: PaperOrientationV1
	page: PaperPageV1


def paper_size_catalog_v1() -> tuple[PaperSizeV1, ...]: ...


class InsertionPlacementV1:
	bond_length_pt: float
	anchor_x: float
	anchor_y: float


def cdml_points_per_cm_v1() -> float: ...
def cm_to_points_v1(centimetres: float) -> float: ...
def points_to_cm_v1(points: float) -> float: ...
def hex_grid_points_v1(
	x_min: float, y_min: float, x_max: float, y_max: float, spacing: float,
) -> tuple[tuple[float, float], ...]: ...
def hex_grid_edges_v1(
	x_min: float, y_min: float, x_max: float, y_max: float, spacing: float,
) -> tuple[tuple[tuple[float, float], tuple[float, float]], ...]: ...
def snap_to_hex_grid_v1(x: float, y: float, spacing: float) -> tuple[float, float]: ...
def normalize_hex_grid_spacing_v1(spacing: float) -> float: ...
def validate_insertion_placement_v1(
	bond_length_pt: float, anchor_x: float, anchor_y: float,
) -> InsertionPlacementV1: ...


class ChemistryError(FerrumError):
	reason: str


class MoleculeInsertionError(FerrumError):
	reason: str


class UnsupportedMoleculeInsertionError(MoleculeInsertionError): ...


class MolblockInputError(MoleculeInsertionError):
	path: str
	stage: str
	limit: int | None
	observed_at_least: int | None


class SdfInputError(MoleculeInsertionError):
	path: str
	stage: str
	limit: int | None
	observed_at_least: int | None


class MoleculeInsertionV1:
	"""Frozen native-handle-free molecule ready for a document transaction."""
	atom_count: int
	bond_count: int


def prepare_smiles_molecule_v1(
	smiles: str,
	placement: InsertionPlacementV1,
) -> MoleculeInsertionV1: ...


def prepare_inchi_molecule_v1(
	inchi: str,
	placement: InsertionPlacementV1,
) -> MoleculeInsertionV1: ...


class DocumentMoleculeInchiError(FerrumError):
	reason: str


class UnsupportedDocumentMoleculeInchiError(DocumentMoleculeInchiError): ...


class DocumentMoleculeInchiV1:
	"""Frozen InChI result tied to one exact Rust document observation."""
	inchi: str
	source_revision: int
	source_digest: str
	molecule_id: str
	mode: InchiModeV1


def export_document_molecule_inchi_v1(
	observation: SessionDocumentObservationV1,
	molecule_id: str,
	mode: InchiModeV1,
) -> DocumentMoleculeInchiV1: ...


def prepare_molblock_molecule_v1(
	molblock: str,
	placement: InsertionPlacementV1,
) -> MoleculeInsertionV1: ...


def prepare_molblock_file_v1(
	path: str,
	placement: InsertionPlacementV1,
) -> MoleculeInsertionV1: ...


class SdfMoleculeBatchInsertionV1:
	"""Frozen ordered SDF records ready for one document transaction."""
	record_count: int


def prepare_sdf_molecules_v1(
	source: str,
	placement: InsertionPlacementV1,
) -> SdfMoleculeBatchInsertionV1: ...


def prepare_sdf_file_v1(
	path: str,
	placement: InsertionPlacementV1,
) -> SdfMoleculeBatchInsertionV1: ...


class InvalidSmiles(ChemistryError): ...


class InvalidSdf(ChemistryError): ...


class InvalidMolblock(ChemistryError): ...


class InvalidInchi(ChemistryError): ...


class ChemistryUnavailable(ChemistryError):
	operation: str
	library_path: str


class ChemistryParse(ChemistryError):
	status: int


class ChemistryCodec(ChemistryError):
	codec: str
	operation: str
	library_path: str


class ChemistryBoundary(ChemistryError): ...


class SmilesPoint2V1:
	x: float
	y: float


class SmilesAtomChiralityV1:
	unspecified: ClassVar[SmilesAtomChiralityV1]
	tetrahedral_cw: ClassVar[SmilesAtomChiralityV1]
	tetrahedral_ccw: ClassVar[SmilesAtomChiralityV1]
	other: ClassVar[SmilesAtomChiralityV1]


class SmilesBondOrderV1:
	aromatic: ClassVar[SmilesBondOrderV1]
	single: ClassVar[SmilesBondOrderV1]
	double: ClassVar[SmilesBondOrderV1]
	triple: ClassVar[SmilesBondOrderV1]
	quadruple: ClassVar[SmilesBondOrderV1]


class SmilesBondStereoV1:
	none: ClassVar[SmilesBondStereoV1]
	any: ClassVar[SmilesBondStereoV1]
	z: ClassVar[SmilesBondStereoV1]
	e: ClassVar[SmilesBondStereoV1]
	cis: ClassVar[SmilesBondStereoV1]
	trans: ClassVar[SmilesBondStereoV1]
	other: ClassVar[SmilesBondStereoV1]


class SmilesBondDirectionV1:
	none: ClassVar[SmilesBondDirectionV1]
	begin_wedge: ClassVar[SmilesBondDirectionV1]
	begin_dash: ClassVar[SmilesBondDirectionV1]
	end_up_right: ClassVar[SmilesBondDirectionV1]
	end_down_right: ClassVar[SmilesBondDirectionV1]
	other: ClassVar[SmilesBondDirectionV1]


class SmilesAtomV1:
	atomic_number: int
	formal_charge: int | None
	isotope: int | None
	explicit_hydrogens: int | None
	aromatic: bool
	chirality: SmilesAtomChiralityV1
	radical_electrons: int
	no_implicit: bool
	atom_map_number: int | None


class SmilesBondV1:
	start: int
	end: int
	order: SmilesBondOrderV1
	aromatic: bool
	stereo: SmilesBondStereoV1
	direction: SmilesBondDirectionV1
	stereo_atoms: tuple[int, int] | None


class SmilesMoleculeV1:
	canonical_smiles: str
	atoms: tuple[SmilesAtomV1, ...]
	bonds: tuple[SmilesBondV1, ...]
	coordinates: tuple[SmilesPoint2V1, ...]


class MolblockVersionV1:
	v2000: ClassVar[MolblockVersionV1]
	v3000: ClassVar[MolblockVersionV1]


class InchiModeV1:
	standard: ClassVar[InchiModeV1]
	fixed_hydrogen: ClassVar[InchiModeV1]


class SdfPropertyV1:
	name: str
	value: str


class SdfRecordV1:
	molecule: SmilesMoleculeV1
	title: str
	properties: tuple[SdfPropertyV1, ...]


class ImportedSdfRecordV1:
	molecule: SmilesMoleculeV1
	title: str
	properties: tuple[SdfPropertyV1, ...]


def parse_smiles(smiles: str) -> SmilesMoleculeV1: ...
def molblock_to_molecule(molblock: str) -> SmilesMoleculeV1: ...
def parse_inchi(inchi: str) -> SmilesMoleculeV1: ...
def molecule_to_smarts(molecule: SmilesMoleculeV1) -> str: ...
def molecule_to_molblock(
	molecule: SmilesMoleculeV1, version: MolblockVersionV1,
) -> str: ...
def molecule_to_inchi(
	molecule: SmilesMoleculeV1, mode: InchiModeV1,
) -> str: ...
def inchi_to_inchi_key(inchi: str) -> str: ...
def prepare_sdf_record(
	molecule: SmilesMoleculeV1,
	title: str,
	properties: tuple[tuple[str, str], ...],
) -> SdfRecordV1: ...
def records_to_sdf(
	records: tuple[SdfRecordV1, ...], version: MolblockVersionV1,
) -> str: ...
def sdf_to_records(sdf: str) -> tuple[ImportedSdfRecordV1, ...]: ...


class DocumentError(FerrumError): ...


class DocumentInputError(DocumentError):
	origin: str
	stage: str
	limit: int | None
	actual: int | None
	observed_at_least: int | None


class DocumentLoadError(DocumentError): ...


class DocumentSerializationError(DocumentError): ...


class RevisionConflictError(DocumentError):
	expected: int
	actual: int


class RevisionExhaustedError(DocumentError): ...


class HistoryUnavailableError(DocumentError): ...


class ProjectionError(DocumentError):
	reason: str


class OperationValidationError(DocumentError): ...


class InvalidAtomElementError(OperationValidationError): ...


class InvalidDocumentObjectIdError(OperationValidationError):
	object_id: str


class UnknownDocumentObjectError(OperationValidationError):
	object_id: str


class PreparedOperationError(DocumentError): ...


class PreparedOperationConsumedError(PreparedOperationError): ...


class PreparedOperationForeignSessionError(PreparedOperationError): ...


class PublicationError(FerrumError):
	path: str
	reason: str


class InvalidDestinationError(PublicationError): ...


class PublicationNotStartedError(PublicationError): ...


class PublicationPossiblyCompletedError(PublicationError): ...


class DocumentSnapshot:
	"""Immutable independent copy of one authoritative CDML revision."""
	cdml: str
	revision: int
	digest: str
	is_dirty: bool


class Point3V1:
	x: float
	y: float
	z: float


class FontFactsV1:
	family: str | None
	size: float | None
	color: str | None


class BondEndpointV1:
	source_id: str | None
	object_id: str | None
	kind: str


class AtomProjectionV1:
	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	element: str | None
	position: Point3V1
	formal_charge: int | None
	isotope: int | None
	explicit_hydrogens: int | None
	valence: int | None
	multiplicity: int | None
	free_sites: int | None
	number: int | None
	show_number: bool | None
	label_font: FontFactsV1 | None
	label_text: str | None
	show: bool | None
	show_hydrogens: bool | None
	marks: list[AtomMarkProjectionV1]


class AtomMarkActionV1:
	"""Closed add/remove intent for one authored atom mark."""
	add: ClassVar[AtomMarkActionV1]
	remove: ClassVar[AtomMarkActionV1]


class AtomMarkKindV1:
	"""Closed atom-mark vocabulary supported by Ferrum V1."""
	plus: ClassVar[AtomMarkKindV1]
	minus: ClassVar[AtomMarkKindV1]
	radical: ClassVar[AtomMarkKindV1]
	biradical: ClassVar[AtomMarkKindV1]
	electronpair: ClassVar[AtomMarkKindV1]
	dotted_electronpair: ClassVar[AtomMarkKindV1]
	pz_orbital: ClassVar[AtomMarkKindV1]


class AtomMarkProjectionV1:
	kind: AtomMarkKindV1
	source_order: int
	same_type_ordinal: int
	angle_degrees: float
	radial_offset: float
	size: float
	draw_circle: bool
	line_width: float


class BondProjectionV1:
	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	start: BondEndpointV1
	end: BondEndpointV1
	source_type: str | None
	order: DocumentBondOrderV1 | None
	style: DocumentBondStyleV1 | None
	haworth_position: DocumentHaworthPositionV1 | None
	line_width: float | None
	bond_width: float | None
	wedge_width: float | None
	center: bool | None
	color: str | None


class MoleculeProjectionV1:
	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	name: str | None
	atoms: list[AtomProjectionV1]
	bonds: list[BondProjectionV1]


class ProjectionIssueV1:
	code: str
	path: str
	detail: str


class DocumentProjectionV1:
	schema: str
	revision: int
	digest: str
	is_dirty: bool
	paper_layout: PaperLayoutProjectionV1
	molecules: list[MoleculeProjectionV1]
	presentation_stack: PresentationStackProjectionV1
	issues: list[ProjectionIssueV1]


class PresentationStackProjectionV1:
	schema: str
	revision: int
	digest: str
	roots: list[PresentationRootProjectionV1]
	bracket_pairs: list[BracketPairProjectionV1]
	issues: list[PresentationProjectionIssueV1]


class BracketPairProjectionV1:
	pair_id: str
	member_ids: list[str]
	style: DocumentBracketStyleV1
	line_width: float | None
	line_color: str | None


class PresentationRootProjectionV1:
	kind: str
	arrow: ArrowProjectionV1 | None
	plus: PlusProjectionV1 | None
	text: TextProjectionV1 | None
	polyline: PolylineProjectionV1 | None
	shape: BoxShapeProjectionV1 | None
	polygon: PolygonProjectionV1 | None


class ArrowProjectionV1:
	target: PresentationTargetV1
	source_path: ArrowPathV1
	axis_path: ArrowPathV1
	head_shape: ArrowHeadShapeV1
	start_head: bool
	end_head: bool
	heads: list[ArrowHeadV1]
	stroke: PresentationStrokeV1


class ArrowPathV1:
	points: list[Point3V1]


class ArrowHeadShapeV1:
	line_inset: float
	total_length: float
	half_width: float


class ArrowHeadV1:
	position: str
	points: list[Point3V1]


class PresentationFontV1:
	family: str | None
	family_provenance: str
	size: float
	size_provenance: str
	color: str
	color_provenance: str


class PlusProjectionV1:
	target: PresentationTargetV1
	anchor: Point3V1
	font: PresentationFontV1
	background: PresentationFillV1


class PresentationTextRunV1:
	text: str
	styles: tuple[str, ...]


class PresentationTextFontV1:
	family: str | None
	family_provenance: str
	size: float
	size_provenance: str
	color: str
	color_provenance: str


class TextProjectionV1:
	target: PresentationTargetV1
	anchor: Point3V1
	runs: tuple[PresentationTextRunV1, ...]
	font: PresentationTextFontV1
	background: PresentationFillV1


class PolylineProjectionV1:
	target: PresentationTargetV1
	path: PolylinePathV1
	stroke: PresentationStrokeV1


class BoxShapeProjectionV1:
	target: PresentationTargetV1
	bounds: PresentationBoundsV1
	stroke: PresentationStrokeV1
	fill: PresentationFillV1


class PolygonProjectionV1:
	target: PresentationTargetV1
	path: PolygonPathV1
	stroke: PresentationStrokeV1
	fill: PresentationFillV1


class PresentationTargetV1:
	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	record_kind: str


class PolylinePathV1:
	points: list[Point3V1]


class PolygonPathV1:
	points: list[Point3V1]


class PresentationStrokeV1:
	color: str
	color_provenance: str
	width: float
	width_provenance: str


class PresentationBoundsV1:
	left: float
	top: float
	right: float
	bottom: float


class PresentationFillV1:
	color: str | None
	color_provenance: str


class PresentationProjectionIssueV1:
	target: PresentationTargetV1
	code: str
	detail: str


class SessionDocumentObservationV1:
	"""One immutable authoritative snapshot and matching Rust projection."""
	snapshot: DocumentSnapshot
	projection: DocumentProjectionV1


class SessionOperationResultV1:
	"""Immutable post-mutation envelope for one authoritative revision."""
	observation: SessionDocumentObservationV1


class PreparedMoleculeCoordinatesV1:
	"""Immutable complete molecule coordinates bound to one source observation."""
	molecule_id: str
	atom_count: int
	source_revision: int
	source_digest: str


class PreparedCleanGeometryV1:
	"""Immutable multi-molecule geometry bound to one source observation."""
	molecule_ids: tuple[str, ...]
	atom_counts: tuple[int, ...]
	source_revision: int
	source_digest: str


class MoleculeCoordinateError(FerrumError):
	reason: str


class UnsupportedMoleculeCoordinateError(MoleculeCoordinateError):
	pass


def prepare_molecule_coordinates_v1(
	observation: SessionDocumentObservationV1,
	molecule_id: str,
) -> PreparedMoleculeCoordinatesV1: ...


def prepare_clean_geometry_v1(
	observation: SessionDocumentObservationV1,
	molecule_ids: tuple[str, ...],
	target_spacing_points: float,
) -> PreparedCleanGeometryV1: ...


class SaveOutcome:
	"""Closed publication outcome created only by an immutable Publication."""
	is_confirmed: bool
	requires_destination_verification: bool


class Publication:
	"""Immutable result of one ordinary save or recovery export."""
	snapshot: DocumentSnapshot
	published_snapshot: DocumentSnapshot
	outcome: SaveOutcome


class DocumentOperationV1:
	"""Closed Rust-owned V1 operation grammar."""
	@staticmethod
	def set_atom_element(atom_id: str, element: str) -> "DocumentOperationV1": ...
	@staticmethod
	def set_atom_properties(
		atom_id: str,
		changes: tuple["DocumentAtomPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_paper_properties(
		changes: tuple["DocumentPaperPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_atom_number(
		molecule_id: str,
		atom_id: str,
		number: int,
		show_number: bool,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def clear_atom_number(
		molecule_id: str,
		atom_id: str,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def apply_atom_mark(
		molecule_id: str,
		atom_id: str,
		action: AtomMarkActionV1,
		kind: AtomMarkKindV1,
		matching_mark_index: int | None,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_bond_properties(
		bond_id: str,
		changes: tuple["DocumentBondPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_plus_properties(
		plus_id: str,
		changes: tuple["DocumentPlusPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_text_properties(
		text_id: str,
		changes: tuple["DocumentTextPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_arrow_properties(
		arrow_id: str,
		changes: tuple["DocumentArrowPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_geometric_properties(
		presentation_id: str,
		changes: tuple["DocumentGeometricPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_wavy_properties(
		wavy_id: str,
		changes: tuple["DocumentWavyPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_bracket_properties(
		pair_id: str,
		changes: tuple["DocumentBracketPropertyChangeV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_atom_position(
		atom_id: str,
		x: float,
		y: float,
		z: float,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def delete_atom(atom_id: str) -> "DocumentOperationV1": ...
	@staticmethod
	def delete_bond(bond_id: str) -> "DocumentOperationV1": ...
	@staticmethod
	def delete_presentation_root(
		presentation_id: str,
		kind: "DocumentPresentationRootKindV1",
	) -> "DocumentOperationV1": ...
	@staticmethod
	def delete_presentation_roots(
		targets: tuple["DocumentPresentationRootSelectorV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def reorder_presentation_roots(
		order: "DocumentPresentationStackOrderV1",
		targets: tuple["DocumentPresentationRootSelectorV1", ...],
	) -> "DocumentOperationV1": ...
	@staticmethod
	def translate_top_level_roots(
		targets: tuple["DocumentTopLevelRootSelectorV1", ...],
		dx: int | float,
		dy: int | float,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def align_top_level_roots(
		targets: tuple["DocumentTopLevelRootSelectorV1", ...],
		alignment: "DocumentTopLevelAlignmentV1",
	) -> "DocumentOperationV1": ...
	@staticmethod
	def scale_top_level_roots(
		targets: tuple["DocumentTopLevelRootSelectorV1", ...],
		scale_x: int | float,
		scale_y: int | float,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def mirror_top_level_roots(
		targets: tuple["DocumentTopLevelRootSelectorV1", ...],
		orientation: "DocumentTopLevelMirrorV1",
	) -> "DocumentOperationV1": ...
	@staticmethod
	def rotate_atoms(
		targets: tuple["DocumentAtomRotationTargetV1", ...],
		center_x: int | float,
		center_y: int | float,
		angle_radians: int | float,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def repair_geometry(
		molecule_ids: tuple[str, ...],
		kind: "DocumentGeometryRepairKindV1",
		target_spacing_points: int | float,
	) -> "DocumentOperationV1": ...
	@staticmethod
	def set_bond_order(
		bond_id: str,
		order: "DocumentBondOrderV1",
	) -> "DocumentOperationV1": ...


class DocumentPaperPropertyChangeV1:
	"""One frozen, closed paper-property change accepted by a Rust patch."""
	@staticmethod
	def type_name(value: str) -> "DocumentPaperPropertyChangeV1": ...
	@staticmethod
	def orientation(value: PaperOrientationV1) -> "DocumentPaperPropertyChangeV1": ...
	@staticmethod
	def crop_svg(value: bool) -> "DocumentPaperPropertyChangeV1": ...
	@staticmethod
	def crop_margin(value: int) -> "DocumentPaperPropertyChangeV1": ...
	@staticmethod
	def use_real_minus(value: bool) -> "DocumentPaperPropertyChangeV1": ...
	@staticmethod
	def replace_minus(value: bool) -> "DocumentPaperPropertyChangeV1": ...
	@staticmethod
	def dimensions(width: float, height: float) -> "DocumentPaperPropertyChangeV1": ...


class DocumentAtomPropertyChangeV1:
	"""One frozen, closed atom-property change accepted by a Rust patch."""
	@staticmethod
	def element(value: str) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def formal_charge(value: int) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def valence(value: int | None) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def isotope(value: int | None) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def multiplicity(value: int) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def show(value: bool) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def show_hydrogens(value: bool) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def font_size(value: float) -> "DocumentAtomPropertyChangeV1": ...
	@staticmethod
	def label_color(value: str) -> "DocumentAtomPropertyChangeV1": ...


class PreparedAtomInsertion:
	"""Opaque revision-bound one-use prepared atom insertion."""
	identifier: str


class PreparedWavyInsertion:
	"""Opaque revision-bound one-use prepared Wavy insertion."""
	identifier: str


class DocumentBracketStyleV1:
	"""Closed persistent bracket families."""
	rectangular: ClassVar[DocumentBracketStyleV1]
	round: ClassVar[DocumentBracketStyleV1]


class DocumentBracketBoundsV1:
	"""Exact finite normalized bounds for one bracket pair."""
	left: float
	top: float
	right: float
	bottom: float
	def __init__(self, left: float, top: float, right: float, bottom: float) -> None: ...


class PreparedBracketInsertion:
	"""Opaque revision-bound one-use prepared bracket-pair insertion."""
	pair_identifier: str
	left_identifier: str
	right_identifier: str


class DocumentBondOrderV1:
	"""Closed CDML bond orders persisted without approximation."""
	single: ClassVar[DocumentBondOrderV1]
	double: ClassVar[DocumentBondOrderV1]
	triple: ClassVar[DocumentBondOrderV1]


class DocumentBondStyleV1:
	"""Closed CDML bond-depiction styles supported by the V1 editor."""
	normal: ClassVar[DocumentBondStyleV1]
	wedge: ClassVar[DocumentBondStyleV1]
	hashed_wedge: ClassVar[DocumentBondStyleV1]
	adder: ClassVar[DocumentBondStyleV1]
	bold: ClassVar[DocumentBondStyleV1]
	dashed: ClassVar[DocumentBondStyleV1]
	dotted: ClassVar[DocumentBondStyleV1]
	wavy: ClassVar[DocumentBondStyleV1]
	haworth_front: ClassVar[DocumentBondStyleV1]


class DocumentHaworthPositionV1:
	front: ClassVar[DocumentHaworthPositionV1]
	back: ClassVar[DocumentHaworthPositionV1]


class DocumentBondPropertyChangeV1:
	"""One frozen, closed bond-property change accepted by a Rust patch."""
	@staticmethod
	def order(value: DocumentBondOrderV1) -> "DocumentBondPropertyChangeV1": ...
	@staticmethod
	def style(value: DocumentBondStyleV1) -> "DocumentBondPropertyChangeV1": ...
	@staticmethod
	def center(value: bool | None) -> "DocumentBondPropertyChangeV1": ...
	@staticmethod
	def line_width(value: float | None) -> "DocumentBondPropertyChangeV1": ...
	@staticmethod
	def bond_width(value: float | None) -> "DocumentBondPropertyChangeV1": ...
	@staticmethod
	def wedge_width(value: float | None) -> "DocumentBondPropertyChangeV1": ...
	@staticmethod
	def color(value: str | None) -> "DocumentBondPropertyChangeV1": ...


class DocumentPlusPropertyChangeV1:
	"""One frozen, closed direct-root Plus property change."""
	@staticmethod
	def font_family(value: str) -> "DocumentPlusPropertyChangeV1": ...
	@staticmethod
	def font_size(value: int) -> "DocumentPlusPropertyChangeV1": ...
	@staticmethod
	def color(value: str) -> "DocumentPlusPropertyChangeV1": ...
	@staticmethod
	def background_color(value: str | None) -> "DocumentPlusPropertyChangeV1": ...


class DocumentTextEditStyleV1:
	"""Closed formatted-text styles accepted by the V1 editor."""
	bold: ClassVar[DocumentTextEditStyleV1]
	italic: ClassVar[DocumentTextEditStyleV1]
	subscript: ClassVar[DocumentTextEditStyleV1]
	superscript: ClassVar[DocumentTextEditStyleV1]


class DocumentTextEditRunV1:
	"""One frozen validated character-data run."""
	@staticmethod
	def create(
		text: str,
		styles: tuple[DocumentTextEditStyleV1, ...],
	) -> "DocumentTextEditRunV1": ...


class DocumentTextPropertyChangeV1:
	"""One frozen, closed direct-root Text property change."""
	@staticmethod
	def runs(
		values: tuple[DocumentTextEditRunV1, ...],
	) -> "DocumentTextPropertyChangeV1": ...
	@staticmethod
	def font_family(value: str | None) -> "DocumentTextPropertyChangeV1": ...
	@staticmethod
	def font_size(value: int) -> "DocumentTextPropertyChangeV1": ...
	@staticmethod
	def color(value: str) -> "DocumentTextPropertyChangeV1": ...
	@staticmethod
	def background_color(value: str | None) -> "DocumentTextPropertyChangeV1": ...


class DocumentPresentationRootKindV1:
	"""Closed durable direct-root presentation kinds accepted by mutation."""
	arrow: ClassVar[DocumentPresentationRootKindV1]
	plus: ClassVar[DocumentPresentationRootKindV1]
	text: ClassVar[DocumentPresentationRootKindV1]
	polyline: ClassVar[DocumentPresentationRootKindV1]
	rectangle: ClassVar[DocumentPresentationRootKindV1]
	square: ClassVar[DocumentPresentationRootKindV1]
	oval: ClassVar[DocumentPresentationRootKindV1]
	circle: ClassVar[DocumentPresentationRootKindV1]
	polygon: ClassVar[DocumentPresentationRootKindV1]


@final
class DocumentPresentationStackOrderV1:
	bring_to_front: ClassVar[DocumentPresentationStackOrderV1]
	send_to_back: ClassVar[DocumentPresentationStackOrderV1]
	reverse_selected_slots: ClassVar[DocumentPresentationStackOrderV1]


@final
class DocumentPresentationRootSelectorV1:
	presentation_id: str
	kind: DocumentPresentationRootKindV1

	@staticmethod
	def create(
		presentation_id: str, kind: DocumentPresentationRootKindV1,
		) -> DocumentPresentationRootSelectorV1: ...


@final
class DocumentTopLevelRootKindV1:
	molecule: ClassVar[DocumentTopLevelRootKindV1]
	arrow: ClassVar[DocumentTopLevelRootKindV1]
	plus: ClassVar[DocumentTopLevelRootKindV1]
	text: ClassVar[DocumentTopLevelRootKindV1]
	rectangle: ClassVar[DocumentTopLevelRootKindV1]
	square: ClassVar[DocumentTopLevelRootKindV1]
	oval: ClassVar[DocumentTopLevelRootKindV1]
	circle: ClassVar[DocumentTopLevelRootKindV1]
	polygon: ClassVar[DocumentTopLevelRootKindV1]
	polyline: ClassVar[DocumentTopLevelRootKindV1]


@final
class DocumentTopLevelRootSelectorV1:
	root_id: str
	kind: DocumentTopLevelRootKindV1

	@staticmethod
	def create(
		root_id: str, kind: DocumentTopLevelRootKindV1,
		) -> DocumentTopLevelRootSelectorV1: ...


@final
class DocumentTopLevelAlignmentV1:
	top: ClassVar[DocumentTopLevelAlignmentV1]
	bottom: ClassVar[DocumentTopLevelAlignmentV1]
	left: ClassVar[DocumentTopLevelAlignmentV1]
	right: ClassVar[DocumentTopLevelAlignmentV1]
	center_x: ClassVar[DocumentTopLevelAlignmentV1]
	center_y: ClassVar[DocumentTopLevelAlignmentV1]


@final
class DocumentTopLevelMirrorV1:
	vertical: ClassVar[DocumentTopLevelMirrorV1]
	horizontal: ClassVar[DocumentTopLevelMirrorV1]


@final
class DocumentAtomRotationTargetV1:
	molecule_id: str
	atom_id: str

	@staticmethod
	def create(molecule_id: str, atom_id: str) -> DocumentAtomRotationTargetV1: ...


@final
class DocumentGeometryRepairKindV1:
    normalize_bond_angles: ClassVar[DocumentGeometryRepairKindV1]
    normalize_bond_lengths: ClassVar[DocumentGeometryRepairKindV1]
    normalize_rings: ClassVar[DocumentGeometryRepairKindV1]
    snap_to_hex_grid: ClassVar[DocumentGeometryRepairKindV1]
    straighten_bonds: ClassVar[DocumentGeometryRepairKindV1]


class DocumentArrowPropertyChangeV1:
	"""One frozen, closed direct-root Arrow property change."""
	@staticmethod
	def start_head(value: bool) -> "DocumentArrowPropertyChangeV1": ...
	@staticmethod
	def end_head(value: bool) -> "DocumentArrowPropertyChangeV1": ...
	@staticmethod
	def spline(value: bool) -> "DocumentArrowPropertyChangeV1": ...
	@staticmethod
	def line_width(value: int | float) -> "DocumentArrowPropertyChangeV1": ...
	@staticmethod
	def color(value: str) -> "DocumentArrowPropertyChangeV1": ...


class DocumentGeometricPropertyChangeV1:
	"""One frozen, closed geometric presentation property change."""
	@staticmethod
	def line_width(value: int | float) -> "DocumentGeometricPropertyChangeV1": ...
	@staticmethod
	def stroke_color(value: str) -> "DocumentGeometricPropertyChangeV1": ...
	@staticmethod
	def fill_color(value: str | None) -> "DocumentGeometricPropertyChangeV1": ...


class DocumentWavyPropertyChangeV1:
	"""One frozen, closed Wavy presentation property change."""
	@staticmethod
	def line_width(value: int | float) -> "DocumentWavyPropertyChangeV1": ...
	@staticmethod
	def line_color(value: str) -> "DocumentWavyPropertyChangeV1": ...


class DocumentBracketPropertyChangeV1:
	"""One frozen, closed common bracket-pair property change."""
	@staticmethod
	def line_width(value: int | float) -> "DocumentBracketPropertyChangeV1": ...
	@staticmethod
	def line_color(value: str) -> "DocumentBracketPropertyChangeV1": ...


class PreparedBondInsertion:
	"""Opaque revision-bound one-use prepared molecule-local bond insertion."""
	identifier: str


class PreparedBondedAtomInsertion:
	"""Opaque revision-bound one-use atom-plus-bond insertion."""
	atom_identifier: str
	bond_identifier: str


class PreparedMoleculeInsertion:
	"""Opaque revision-bound one-use prepared complete molecule insertion."""
	molecule_identifier: str


class PreparedSdfRecordInsertion:
	"""Opaque revision-bound one-use prepared SDF record batch."""
	molecule_identifiers: tuple[str, ...]
	atom_identifiers: tuple[tuple[str, ...], ...]
	bond_identifiers: tuple[tuple[str, ...], ...]


class RenderPointV1:
	x: float
	y: float


class RenderRecordIdV1:
	kind: str
	id: str | None


class RenderTargetV1:
	record_id: RenderRecordIdV1
	source_order: int


class AtomLocalSpaceV1:
	kind: str
	anchor: RenderPointV1


class SceneSpaceV1:
	kind: str


class GlyphPlacementV1:
	glyph_index: int
	origin: RenderPointV1


class TextRunV1:
	text: str
	script: str
	origin: RenderPointV1
	glyphs: tuple[GlyphPlacementV1, ...]
	scale: float


class TextOpV1:
	origin: RenderPointV1
	runs: tuple[TextRunV1, ...]
	face: str
	size: float
	paint: str
	z: int


class LineOpV1:
	start: RenderPointV1
	end: RenderPointV1
	width: float
	paint: str
	z: int


class MaskOpV1:
	origin: RenderPointV1
	width: float
	height: float
	paint: str
	z: int


class EllipseOpV1:
	center: RenderPointV1
	radius_x: float
	radius_y: float
	rotation_degrees: float
	stroke_width: float | None
	stroke_paint: str | None
	fill_paint: str | None
	z: int


class RenderOperationV1:
	kind: str
	operation: TextOpV1 | LineOpV1 | MaskOpV1 | EllipseOpV1


class RenderBatchV1:
	target: RenderTargetV1
	coordinate_space: AtomLocalSpaceV1 | SceneSpaceV1
	operations: tuple[RenderOperationV1, ...]


class RenderIssueV1:
	target: RenderTargetV1
	kind: str
	detail: str


class RenderProvenanceV1:
	revision: int
	digest: str


class RenderPlanV1:
	schema: str
	provenance: RenderProvenanceV1
	batches: tuple[RenderBatchV1, ...]
	issues: tuple[RenderIssueV1, ...]


class MoleculeRenderRootV1:
	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int


class DocumentMoleculeRenderPlanV1:
	molecule: MoleculeRenderRootV1
	plan: RenderPlanV1


class PresentationTextBoundsV1:
	left: float
	top: float
	right: float
	bottom: float


class DocumentPlusRenderV1:
	target: PresentationTargetV1
	anchor: RenderPointV1
	operation: TextOpV1
	bounds: PresentationTextBoundsV1
	background: str | None


class PresentationTextSourceRunV1:
	text: str
	script: str


class PresentationGlyphRunV1:
	text: str
	script: str
	origin: RenderPointV1
	glyphs: tuple[GlyphPlacementV1, ...]
	scale: float


class PresentationTextOpV1:
	runs: tuple[PresentationGlyphRunV1, ...]
	face: str
	size: float
	paint: str
	z: int


class DocumentTextRenderV1:
	target: PresentationTargetV1
	anchor: RenderPointV1
	source_runs: tuple[PresentationTextSourceRunV1, ...]
	operation: PresentationTextOpV1
	bounds: PresentationTextBoundsV1
	background: str | None


class DepictionIssueV1:
	code: str
	target: str
	detail: str


class RenderObservationV1:
	schema: str
	document: SessionDocumentObservationV1
	profile: str
	molecule_plans: tuple[DocumentMoleculeRenderPlanV1, ...]
	plus_renders: tuple[DocumentPlusRenderV1, ...]
	text_renders: tuple[DocumentTextRenderV1, ...]
	issues: tuple[DepictionIssueV1, ...]
	suppression: str | None


class VerifiedTelexRegularV1:
	resource_id: str
	data: bytes
	byte_length: int
	sha256: str
	family: str
	postscript_name: str


class RenderObservationError(FerrumError): ...
class RenderDepictionError(RenderObservationError): ...
class RenderProvenanceError(RenderObservationError): ...
def verified_telex_regular() -> VerifiedTelexRegularV1: ...


class XmlInputBudgetV1:
	"""Immutable caller-owned XML admission limits; no defaults are selected."""
	max_utf8_bytes: int
	max_elements: int
	max_depth: int
	max_attributes: int
	max_text_bytes: int
	def __init__(
		self,
		max_utf8_bytes: int,
		max_elements: int,
		max_depth: int,
		max_attributes: int,
		max_text_bytes: int,
	) -> None: ...


class DocumentSession:
	"""Thread-affine mutable session with synchronous, owned-value methods only."""
	@staticmethod
	def create_empty_document_v1() -> "DocumentSession":
		"""Create a canonical clean empty CDML baseline owned by Rust."""
		...
	@staticmethod
	def load(cdml: str) -> "DocumentSession":
		"""Unbounded compatibility route for an already-allocated string.

		External input must use an explicit-budget byte or local-file admission method.
		"""
		...
	@staticmethod
	def load_utf8_bytes_with_budget(
		source: bytes, budget: "XmlInputBudgetV1",
	) -> "DocumentSession": ...
	@staticmethod
	def load_file_with_budget(
		path: str, budget: "XmlInputBudgetV1",
	) -> "DocumentSession": ...
	@staticmethod
	def load_cdsvg_utf8_bytes_with_budget(
		source: bytes,
		wrapper_budget: "XmlInputBudgetV1",
		payload_budget: "XmlInputBudgetV1",
	) -> "DocumentSession": ...
	@staticmethod
	def load_cdsvg_file_with_budget(
		path: str,
		wrapper_budget: "XmlInputBudgetV1",
		payload_budget: "XmlInputBudgetV1",
	) -> "DocumentSession": ...
	def snapshot(self) -> DocumentSnapshot: ...
	def observe(self, expected_revision: int) -> SessionDocumentObservationV1: ...
	def observe_render(self, expected_revision: int) -> RenderObservationV1: ...
	def submit(
		self,
		expected_revision: int,
		operation: DocumentOperationV1,
	) -> SessionOperationResultV1: ...
	def apply_molecule_coordinates_v1(
		self,
		expected_revision: int,
		prepared: PreparedMoleculeCoordinatesV1,
	) -> SessionOperationResultV1: ...
	def apply_clean_geometry_v1(
		self,
		expected_revision: int,
		prepared: PreparedCleanGeometryV1,
	) -> SessionOperationResultV1: ...
	def undo(self, expected_revision: int) -> SessionOperationResultV1: ...
	def redo(self, expected_revision: int) -> SessionOperationResultV1: ...
	def prepare_create_atom_v1(
		self,
		expected_revision: int,
		molecule_object_id: str,
		element: str,
		x: float,
		y: float,
		z: float,
	) -> PreparedAtomInsertion: ...
	def commit_create_atom(
		self,
		expected_revision: int,
		prepared: PreparedAtomInsertion,
	) -> SessionOperationResultV1: ...
	def prepare_create_wavy_v1(
		self,
		expected_revision: int,
		start_x: float,
		start_y: float,
		end_x: float,
		end_y: float,
	) -> PreparedWavyInsertion: ...
	def commit_create_wavy(
		self,
		expected_revision: int,
		prepared: PreparedWavyInsertion,
	) -> SessionOperationResultV1: ...
	def prepare_create_bracket_v1(
		self,
		expected_revision: int,
		style: DocumentBracketStyleV1,
		bounds: DocumentBracketBoundsV1,
	) -> PreparedBracketInsertion: ...
	def commit_create_bracket(
		self,
		expected_revision: int,
		prepared: PreparedBracketInsertion,
	) -> SessionOperationResultV1: ...
	def prepare_create_bond_v1(
		self,
		expected_revision: int,
		start_atom_object_id: str,
		end_atom_object_id: str,
		order: DocumentBondOrderV1,
	) -> PreparedBondInsertion: ...
	def commit_create_bond(
		self,
		expected_revision: int,
		prepared: PreparedBondInsertion,
	) -> SessionOperationResultV1: ...
	def prepare_create_bonded_atom_v1(
		self,
		expected_revision: int,
		start_atom_object_id: str,
		element: str,
		x: float,
		y: float,
		z: float,
		order: DocumentBondOrderV1,
	) -> PreparedBondedAtomInsertion: ...
	def commit_create_bonded_atom(
		self,
		expected_revision: int,
		prepared: PreparedBondedAtomInsertion,
	) -> SessionOperationResultV1: ...
	def prepare_insert_molecule_v1(
		self,
		expected_revision: int,
		molecule: MoleculeInsertionV1,
	) -> PreparedMoleculeInsertion: ...
	def commit_create_molecule(
		self,
		expected_revision: int,
		prepared: PreparedMoleculeInsertion,
	) -> SessionOperationResultV1: ...
	def prepare_insert_sdf_records_v1(
		self,
		expected_revision: int,
		batch: SdfMoleculeBatchInsertionV1,
	) -> PreparedSdfRecordInsertion: ...
	def commit_create_sdf_records(
		self,
		expected_revision: int,
		prepared: PreparedSdfRecordInsertion,
	) -> SessionOperationResultV1: ...
	def save_atomic(
		self,
		path: str | pathlib.Path,
		expected_revision: int,
	) -> Publication: ...
	def recovery_export(
		self,
		path: str | pathlib.Path,
		expected_revision: int,
	) -> Publication: ...
