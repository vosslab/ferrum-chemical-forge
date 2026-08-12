"""Per-tab ownership and teardown boundary for Ferrum-Qt documents."""

# Standard Library
import dataclasses
import math
import numbers

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.setup.canvas_setup
import ferrum_qt.setup.mode_setup
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.io.cdml_candidate
import ferrum_qt.io.user_template_catalog
import ferrum_qt.models.backend_revision_history
import ferrum_qt.models.document
import ferrum_qt.models.projection_lifecycle
import ferrum_qt.undo.commands
import ferrum_qt.wavy_geometry
import oasa.cdml_document
import oasa.cdml_ftext
import oasa.cdml_render
import oasa.cdml_writer
import oasa.safe_xml
import oasa.biomolecule_template_placement
import oasa.template_placement


_BLANK_CDML = (
	'<cdml xmlns="%s" version="%s"></cdml>' % (
		oasa.cdml_writer.CDML_NAMESPACE,
		oasa.cdml_writer.DEFAULT_CDML_VERSION,
	)
)

_ORPHANED_IMPORT_WORKERS: set[PySide6.QtCore.QThread] = set()


#============================================
def orphaned_import_worker_count() -> int:
	"""Return the directly disposed workers awaiting their finished signal."""
	return len(_ORPHANED_IMPORT_WORKERS)


#============================================
def _release_orphaned_import_worker(worker: PySide6.QtCore.QThread) -> None:
	"""Release a directly disposed session's worker after native completion."""
	if worker not in _ORPHANED_IMPORT_WORKERS:
		return
	_ORPHANED_IMPORT_WORKERS.discard(worker)
	relay = getattr(worker, "_result_relay", None)
	if relay is not None:
		try:
			relay.deleteLater()
		except RuntimeError:
			pass
		worker._result_relay = None
	try:
		worker.deleteLater()
	except RuntimeError:
		pass


#============================================
def _adopt_orphaned_import_worker(worker: PySide6.QtCore.QThread) -> None:
	"""Give a directly disposed session's worker an explicit terminal owner.

	The set is established before the connection, so a fast completion cannot
	drop the final strong reference.  The post-connection check covers a worker
	that completed before Qt could queue the slot; release is idempotent.
	"""
	_ORPHANED_IMPORT_WORKERS.add(worker)
	worker.finished.connect(
		lambda worker=worker: _release_orphaned_import_worker(worker),
		PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
	)
	if worker.isFinished():
		_release_orphaned_import_worker(worker)


#============================================
def _freeze_plain_payload(value: object) -> object:
	"""Return one recursively immutable value accepted at the Qt/backend boundary."""
	if value is None or isinstance(value, (bool, int, float, str)):
		return value
	if isinstance(value, tuple):
		frozen = tuple(_freeze_plain_payload(item) for item in value)
		return frozen
	raise TypeError("Persistent operation payload must contain immutable plain data")


#============================================
def _direct_core_cdml_children(parent: object, local_name: str) -> tuple[object, ...]:
	"""Return direct legacy-or-canonical core children with one exact name."""
	children = []
	for child in parent.childNodes:
		if getattr(child, "nodeType", None) != child.ELEMENT_NODE:
			continue
		child_name = getattr(child, "localName", None) or getattr(child, "tagName", "")
		if ":" in child_name:
			child_name = child_name.rsplit(":", 1)[1]
		if (
				child_name == local_name
				and getattr(child, "namespaceURI", None) in (
					None, "", oasa.cdml_document.CDML_NAMESPACE_URI,
				)
			):
			children.append(child)
	return tuple(children)


#============================================
def _is_unchanged_authoritative_snapshot(
		before: oasa.cdml_document.CDMLSnapshot,
		after: oasa.cdml_document.CDMLSnapshot,
		) -> bool:
	"""Return whether a successful backend operation changed no persistent state.

	The backend owns this decision through its immutable snapshots.  Qt compares
	the complete canonical content, revision, and saved-baseline state instead of
	consulting a projection model, which can be stale or disposable.
	"""
	unchanged = (
		before.revision == after.revision
		and before.cdml == after.cdml
		and before.is_dirty == after.is_dirty
	)
	return unchanged


class BackendProjectionOutOfSyncError(RuntimeError):
	"""Raised when Qt's live projection cannot safely use backend CDML."""


class ProjectionReplacementError(RuntimeError):
	"""Raised when a live Qt projection cannot be recovered from backend CDML."""


class BackendFragmentExtractionError(ValueError):
	"""Expose one typed backend fragment-read failure to Qt-only callers."""


@dataclasses.dataclass(frozen=True)
class PersistentOperationRequest:
	"""Immutable plain-data request for one backend-authoritative operation."""

	operation_key: str
	label: str
	payload: tuple[tuple[str, object], ...]
	target_keys: frozenset[tuple[str, str]] = frozenset()

	#============================================
	def __post_init__(self) -> None:
		"""Validate the request cannot retain mutable frontend or backend objects."""
		if not isinstance(self.operation_key, str) or not isinstance(self.label, str):
			raise TypeError("Persistent operation key and label must be strings")
		payload = tuple(
			(key, _freeze_plain_payload(value)) for key, value in self.payload
		)
		if any(not isinstance(key, str) for key, _value in payload):
			raise TypeError("Persistent operation payload keys must be strings")
		if len({key for key, _value in payload}) != len(payload):
			raise ValueError("Persistent operation payload keys must be unique")
		target_keys = frozenset(self.target_keys)
		if any(
			not isinstance(kind, str) or not isinstance(key, str)
			for kind, key in target_keys
		):
			raise TypeError("Persistent target keys must be durable string pairs")
		object.__setattr__(self, "payload", payload)
		object.__setattr__(self, "target_keys", target_keys)
@dataclasses.dataclass(frozen=True)
class _UserTemplateModeDescriptor:
	"""One Qt-mode projection of a session-owned saved-template record."""

	catalog_key: str
	label: str


#============================================
def _freeze_user_template_catalog(
		entries: object,
		) -> tuple[
		tuple[ferrum_qt.io.user_template_catalog.UserTemplateCatalogEntry, ...],
		dict[str, ferrum_qt.io.user_template_catalog.UserTemplateCatalogEntry],
		tuple[_UserTemplateModeDescriptor, ...],
		]:
	"""Copy one admitted immutable catalog into session-owned delivery data."""
	if type(entries) is not tuple:
		raise TypeError("User template catalog must be an immutable tuple")
	frozen_entries = []
	for entry in entries:
		if type(entry) is not ferrum_qt.io.user_template_catalog.UserTemplateCatalogEntry:
			raise TypeError("User template catalog entries must be admitted catalog records")
		if type(entry.catalog_key) is not str or not entry.catalog_key.strip():
			raise ValueError("User template catalog keys must be nonblank strings")
		if type(entry.label) is not str or not entry.label.strip():
			raise ValueError("User template catalog labels must be nonblank strings")
		if type(entry.template_cdml) is not str or not entry.template_cdml:
			raise ValueError("User template catalog CDML must be nonempty text")
		frozen_entries.append(ferrum_qt.io.user_template_catalog.UserTemplateCatalogEntry(
			entry.catalog_key, entry.label, entry.template_cdml,
		))
	if len({entry.catalog_key for entry in frozen_entries}) != len(frozen_entries):
		raise ValueError("User template catalog keys must be unique")
	immutable_entries = tuple(frozen_entries)
	by_key = {entry.catalog_key: entry for entry in immutable_entries}
	descriptors = tuple(
		_UserTemplateModeDescriptor(entry.catalog_key, entry.label)
		for entry in immutable_entries
	)
	return immutable_entries, by_key, descriptors


#============================================
def build_user_template_insert_request(
		expected_revision: int, catalog_key: str, anchor: tuple[float, float],
		) -> PersistentOperationRequest:
	"""Build one immutable detached saved-template insertion intent."""
	if type(expected_revision) is not int:
		raise TypeError("User template insertion expected_revision must be an integer")
	if type(catalog_key) is not str or not catalog_key.strip():
		raise ValueError("User template insertion catalog_key must be nonblank")
	if (
			type(anchor) is not tuple or len(anchor) != 2
			or any(
				isinstance(value, bool) or not isinstance(value, numbers.Real)
				or not math.isfinite(value)
				for value in anchor
			)
		):
		raise ValueError("User template insertion anchor must be a finite point tuple")
	return PersistentOperationRequest(
		"user-template.insert", "Place User Template",
		(
			("expected_revision", expected_revision),
			("catalog_key", catalog_key),
			("anchor", (float(anchor[0]), float(anchor[1]))),
		),
		frozenset(),
	)


#============================================
def build_atom_element_request(
		expected_revision: int, molecule_id: str, atom_id: str, element: str,
		) -> PersistentOperationRequest:
	"""Build the one immutable request grammar for an atom element substitution."""
	request = PersistentOperationRequest(
		"atom.element.set", "Change Atom Element",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("atom_id", atom_id),
			("element", element),
		),
		frozenset({("molecule", molecule_id), ("atom", atom_id)}),
	)
	return request


#============================================
def build_atom_align_request(
		expected_revision: int, axis: str, targets: tuple[tuple[str, str], ...],
		) -> PersistentOperationRequest:
	"""Build one immutable request for direct-core atom depiction alignment."""
	return PersistentOperationRequest(
		"atom.align", "Align Selected Atoms",
		(
			("expected_revision", expected_revision),
			("axis", axis),
			("targets", targets),
		),
		frozenset(
			("molecule", molecule_id) for molecule_id, _atom_id in targets
		) | frozenset(("atom", atom_id) for _molecule_id, atom_id in targets),
	)


#============================================
def build_atom_translate_request(
		expected_revision: int, targets: tuple[tuple[str, str], ...],
		delta: tuple[float, float],
		) -> PersistentOperationRequest:
	"""Build one immutable request for direct-core atom translation."""
	return PersistentOperationRequest(
		"atom.translate", "Nudge Selected Atoms",
		(
			("expected_revision", expected_revision),
			("targets", targets),
			("delta", delta),
		),
		frozenset(
			("molecule", molecule_id) for molecule_id, _atom_id in targets
		) | frozenset(("atom", atom_id) for _molecule_id, atom_id in targets),
	)


#============================================
def build_selection_translate_request(
		expected_revision: int, atom_targets: tuple[tuple[str, str], ...],
		presentation_root_keys: tuple[tuple[str, str], ...], delta: tuple[float, float],
		) -> PersistentOperationRequest:
	"""Build one immutable mixed atom/presentation translation request.

	The frontend retains root kinds only to restore canonical selection. OASA
	receives durable presentation IDs and validates authoritative geometry.
	"""
	if type(expected_revision) is not int:
		raise TypeError("Selection translation expected_revision must be an integer")
	if type(atom_targets) is not tuple:
		raise TypeError("Selection translation atom targets must be an immutable tuple")
	if type(presentation_root_keys) is not tuple:
		raise TypeError("Selection translation presentation roots must be an immutable tuple")
	if type(delta) is not tuple:
		raise TypeError("Selection translation delta must be an immutable tuple")
	if not atom_targets:
		raise ValueError("Selection translation requires durable atom targets")
	if not presentation_root_keys:
		raise ValueError("Selection translation requires durable presentation roots")
	if any(
			type(target) is not tuple or len(target) != 2
			or type(target[0]) is not str or not target[0].strip()
			or type(target[1]) is not str or not target[1].strip()
			for target in atom_targets
		):
		raise ValueError("Selection translation atom targets must be durable ID pairs")
	if any(
			type(key) is not tuple or len(key) != 2
			or key[0] != "presentation"
			or type(key[1]) is not str or not key[1].strip()
			for key in presentation_root_keys
		):
		raise ValueError("Selection translation presentation roots must be durable keys")
	presentation_root_ids = tuple(identifier for _kind, identifier in presentation_root_keys)
	target_keys = (
		frozenset(("molecule", molecule_id) for molecule_id, _atom_id in atom_targets)
		| frozenset(("atom", atom_id) for _molecule_id, atom_id in atom_targets)
		| frozenset(presentation_root_keys)
	)
	return PersistentOperationRequest(
		"selection.translate", "Move Selected",
		(
			("expected_revision", expected_revision),
			("atom_targets", atom_targets),
			("presentation_root_ids", presentation_root_ids),
			("delta", delta),
		),
		target_keys,
	)


#============================================
def build_atom_rotate_request(
		expected_revision: int, targets: tuple[tuple[str, str], ...],
		center: tuple[float, float], angle_radians: float,
		) -> PersistentOperationRequest:
	"""Build one immutable request for direct-core atom rotation."""
	return PersistentOperationRequest(
		"atom.rotate", "Rotate Selected Atoms",
		(
			("expected_revision", expected_revision),
			("targets", targets),
			("center", center),
			("angle_radians", angle_radians),
		),
		frozenset(
			("molecule", molecule_id) for molecule_id, _atom_id in targets
		) | frozenset(("atom", atom_id) for _molecule_id, atom_id in targets),
	)


#============================================
def build_bond_order_request(
		expected_revision: int, molecule_id: str, bond_id: str, order: int,
		) -> PersistentOperationRequest:
	"""Build one immutable request for an exact direct-core bond-order edit."""
	return PersistentOperationRequest(
		"bond.order.set", "Set Bond Order",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("bond_id", bond_id),
			("order", order),
		),
		frozenset({("molecule", molecule_id), ("bond", bond_id)}),
	)


#============================================
def build_bond_type_request(
		expected_revision: int, molecule_id: str, bond_id: str, bond_type: str,
		) -> PersistentOperationRequest:
	"""Build one immutable request for an exact direct-core bond-type edit."""
	return PersistentOperationRequest(
		"bond.type.set", "Set Bond Type",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("bond_id", bond_id),
			("bond_type", bond_type),
		),
		frozenset({("molecule", molecule_id), ("bond", bond_id)}),
	)


#============================================
def build_bond_properties_patch_request(
		expected_revision: int, molecule_id: str, bond_id: str,
		changes: tuple[tuple[str, object], ...] = (),
		) -> PersistentOperationRequest:
	"""Build one immutable explicit-field direct-core bond patch request."""
	return PersistentOperationRequest(
		"bond.properties.patch", "Edit Bond Properties",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("bond_id", bond_id),
			("changes", changes),
		),
		frozenset({("molecule", molecule_id), ("bond", bond_id)}),
	)


#============================================
def build_atom_properties_patch_request(
		expected_revision: int, molecule_id: str, atom_id: str,
		changes: tuple[tuple[str, object], ...] = (),
		) -> PersistentOperationRequest:
	"""Build one immutable explicit-field direct-core atom patch request."""
	return PersistentOperationRequest(
		"atom.properties.patch", "Edit Atom Properties",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("atom_id", atom_id),
			("changes", changes),
		),
		frozenset({("molecule", molecule_id), ("atom", atom_id)}),
	)


#============================================
def build_text_properties_patch_request(
		expected_revision: int, text_id: str,
		changes: tuple[tuple[str, object], ...],
		) -> PersistentOperationRequest:
	"""Build one immutable explicit-field direct-root plain Text patch request."""
	return PersistentOperationRequest(
		"text.properties.patch", "Edit Text Properties",
		(
			("expected_revision", expected_revision),
			("text_id", text_id),
			("changes", changes),
		),
		frozenset({("presentation", text_id)}),
	)


#============================================
def build_rich_text_patch_request(
		expected_revision: int, text_id: str,
		runs: tuple[tuple[str, tuple[str, ...]], ...],
		changes: tuple[tuple[str, object], ...] = (),
		) -> PersistentOperationRequest:
	"""Build one immutable plain-run direct-root rich Text patch request."""
	return PersistentOperationRequest(
		"text.rich.patch", "Edit Rich Text",
		(
			("expected_revision", expected_revision),
			("text_id", text_id),
			("runs", runs),
			("changes", changes),
		),
		frozenset({("presentation", text_id)}),
	)


#============================================
def rich_text_patch_from_plain_runs(
		expected_revision: int, text_id: str,
		runs: tuple[tuple[str, tuple[str, ...]], ...],
		changes: tuple[tuple[str, object], ...] = (),
		) -> oasa.cdml_document.CDMLRichTextPatch:
	"""Adapt exact frontend plain runs to the OASA rich-text patch at one seam."""
	if type(runs) is not tuple:
		raise ValueError("Rich Text runs must be an immutable tuple")
	backend_runs = []
	for run in runs:
		if type(run) is not tuple or len(run) != 2:
			raise ValueError("Rich Text runs must be exact text/style pairs")
		text, styles = run
		if type(text) is not str or type(styles) is not tuple:
			raise ValueError("Rich Text runs must contain plain immutable values")
		backend_runs.append(oasa.cdml_ftext.CDMLFTextRun(text, styles))
	try:
		normalized = oasa.cdml_ftext.normalize(tuple(backend_runs))
	except oasa.cdml_ftext.CDMLFTextCodecError as exc:
		raise ValueError("Rich Text runs are invalid: %s" % exc) from exc
	canonical_runs = tuple((run.text, run.styles) for run in normalized)
	if runs != canonical_runs:
		raise ValueError("Rich Text runs must use canonical styles and adjacent spans")
	patch = oasa.cdml_document.CDMLRichTextPatch(
		expected_revision, text_id, normalized, changes,
	)
	return patch


#============================================
def build_plus_properties_patch_request(
		expected_revision: int, plus_id: str,
		changes: tuple[tuple[str, object], ...],
		) -> PersistentOperationRequest:
	"""Build one immutable explicit-field direct-root plain Plus patch request."""
	return PersistentOperationRequest(
		"plus.properties.patch", "Edit Plus Properties",
		(
			("expected_revision", expected_revision),
			("plus_id", plus_id),
			("changes", changes),
		),
		frozenset({("presentation", plus_id)}),
	)


#============================================
def build_wavy_properties_patch_request(
		expected_revision: int, wavy_id: str,
		changes: tuple[tuple[str, object], ...],
		) -> PersistentOperationRequest:
	"""Build one immutable explicit-field direct-root plain Wavy patch request."""
	return PersistentOperationRequest(
		"wavy.properties.patch", "Edit Wavy Properties",
		(
			("expected_revision", expected_revision),
			("wavy_id", wavy_id),
			("changes", changes),
		),
		frozenset({("presentation", wavy_id)}),
	)


#============================================
def build_fragment_create_request(
		expected_revision: int, molecule_id: str, name: str, fragment_type: str,
		atom_ids: tuple[str, ...], bond_ids: tuple[str, ...],
		) -> PersistentOperationRequest:
	"""Build one immutable ordinary fragment metadata creation request."""
	return PersistentOperationRequest(
		"fragment.create", "Create Fragment",
		(
			("expected_revision", expected_revision), ("molecule_id", molecule_id),
			("name", name), ("fragment_type", fragment_type),
			("atom_ids", atom_ids), ("bond_ids", bond_ids),
		),
		frozenset({("molecule", molecule_id)})
		| frozenset(("atom", atom_id) for atom_id in atom_ids)
		| frozenset(("bond", bond_id) for bond_id in bond_ids),
	)


#============================================
def build_fragment_delete_request(
		expected_revision: int, molecule_id: str, fragment_id: str,
		) -> PersistentOperationRequest:
	"""Build one immutable ordinary fragment metadata deletion request."""
	return PersistentOperationRequest(
		"fragment.delete", "Delete Fragment",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("fragment_id", fragment_id),
		),
		frozenset({("molecule", molecule_id)}),
	)


#============================================
def build_implicit_group_expand_request(
		expected_revision: int, molecule_id: str, group_id: str,
		) -> PersistentOperationRequest:
	"""Build one immutable backend-owned implicit-group expansion request."""
	return PersistentOperationRequest(
		"group.expand.implicit", "Expand Implicit Group",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("group_id", group_id),
		),
		frozenset({("molecule", molecule_id), ("group", group_id)}),
	)


#============================================
def build_linear_form_convert_request(
		expected_revision: int, molecule_id: str, atom_ids: tuple[str, ...],
		) -> PersistentOperationRequest:
	"""Build one immutable backend-owned linear-form conversion request."""
	return PersistentOperationRequest(
		"linear-form.convert", "Convert to Linear Form",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("atom_ids", atom_ids),
		),
		frozenset({("molecule", molecule_id)})
		| frozenset(("atom", atom_id) for atom_id in atom_ids),
	)


#============================================
def build_atom_mark_request(
		expected_revision: int, molecule_id: str, atom_id: str,
		action: str, mark_type: str, matching_mark_index: int | None = None,
		) -> PersistentOperationRequest:
	"""Build one immutable direct-atom chemical-mark operation."""
	payload = (
		("expected_revision", expected_revision),
		("molecule_id", molecule_id),
		("atom_id", atom_id),
		("action", action),
		("mark_type", mark_type),
	)
	if matching_mark_index is not None:
		payload += (("matching_mark_index", matching_mark_index),)
	return PersistentOperationRequest(
		"atom.mark.apply", "Apply Atom Mark",
		payload,
		frozenset({("molecule", molecule_id), ("atom", atom_id)}),
	)


#============================================
def build_structure_delete_request(
		expected_revision: int, molecule_id: str,
		atom_ids: tuple[str, ...], bond_ids: tuple[str, ...],
		) -> PersistentOperationRequest:
	"""Build one immutable direct-atom/bond structural deletion request."""
	if type(expected_revision) is not int:
		raise TypeError("Structure Delete expected_revision must be an integer")
	if type(molecule_id) is not str or not molecule_id.strip():
		raise ValueError("Structure Delete molecule_id must be a nonblank durable ID")
	for name, identifiers in (("atom_ids", atom_ids), ("bond_ids", bond_ids)):
		if type(identifiers) is not tuple:
			raise TypeError("Structure Delete %s must be an immutable tuple" % name)
		if any(type(identifier) is not str or not identifier.strip() for identifier in identifiers):
			raise ValueError("Structure Delete IDs must be nonblank strings")
		if len(set(identifiers)) != len(identifiers):
			raise ValueError("Structure Delete IDs must be unique")
	if not atom_ids and not bond_ids:
		raise ValueError("Structure Delete requires at least one atom or bond")
	if set(atom_ids).intersection(bond_ids):
		raise ValueError("Structure Delete atom and bond IDs must be distinct")
	target_keys = (
		frozenset({("molecule", molecule_id)})
		| frozenset(("atom", identifier) for identifier in atom_ids)
		| frozenset(("bond", identifier) for identifier in bond_ids)
	)
	return PersistentOperationRequest(
		"structure.delete", "Delete",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("atom_ids", atom_ids),
			("bond_ids", bond_ids),
		),
		target_keys,
	)


#============================================
def build_structure_fragment_extraction_query(
		expected_revision: int, molecule_id: str,
		atom_ids: tuple[str, ...], bond_ids: tuple[str, ...],
		) -> oasa.cdml_document.CDMLStructureFragmentExtractionQuery:
	"""Build the immutable read-only structural clipboard query."""
	return oasa.cdml_document.CDMLStructureFragmentExtractionQuery(
		expected_revision, molecule_id, atom_ids, bond_ids,
	)


#============================================
def build_top_level_fragment_extraction_query(
		expected_revision: int, root_ids: tuple[str, ...],
		) -> oasa.cdml_document.CDMLTopLevelFragmentExtractionQuery:
	"""Build one immutable backend-owned direct-root clipboard query."""
	return oasa.cdml_document.CDMLTopLevelFragmentExtractionQuery(
		expected_revision, root_ids,
	)


#============================================
def build_molecule_name_request(
		expected_revision: int, molecule_id: str, name: str,
		) -> PersistentOperationRequest:
	"""Build the immutable request grammar for one molecule display-name edit."""
	return PersistentOperationRequest(
		"molecule.name.set", "Set Molecule Name",
		(
			("expected_revision", expected_revision),
			("molecule_id", molecule_id),
			("name", name),
		),
		frozenset({("molecule", molecule_id)}),
	)


#============================================
def build_paper_properties_request(
		expected_revision: int, changes: tuple[tuple[str, object], ...],
		) -> PersistentOperationRequest:
	"""Build one immutable explicit-field paper-properties patch request."""
	return PersistentOperationRequest(
		"paper.properties.set", "Edit Paper Properties",
		(
			("expected_revision", expected_revision),
			("changes", changes),
		),
	)


#============================================
def build_presentation_stack_request(
		expected_revision: int, mode: str, root_ids: tuple[str, ...],
		) -> PersistentOperationRequest:
	"""Build one immutable direct-presentation-root reorder request."""
	labels = {
		"bring-to-front": "Bring to Front",
		"send-back": "Send to Back",
		"swap-at-slots": "Swap on Stack",
	}
	if mode not in labels:
		raise ValueError("Presentation stack mode is unsupported")
	return PersistentOperationRequest(
		"presentation.stack.reorder", labels[mode],
		(
			("expected_revision", expected_revision),
			("mode", mode),
			("root_ids", root_ids),
		),
		frozenset(("presentation", identifier) for identifier in root_ids),
	)


#============================================
def build_top_level_transform_request(
		expected_revision: int, mode: str,
		root_keys: tuple[tuple[str, str], ...],
		scale_x: float | None = None, scale_y: float | None = None,
		delta: tuple[float, float] | None = None,
		) -> PersistentOperationRequest:
	"""Build one immutable mixed direct-root transform request.

	The backend request contains durable root IDs plus only the documented scalar
	intent for its mode. The paired root kinds stay at the frontend boundary
	solely to restore the selected roots after canonical reprojection.
	"""
	allowed_modes = {
		"align-top", "align-bottom", "align-left", "align-right",
		"align-center-x", "align-center-y", "scale", "mirror-vertical",
		"mirror-horizontal", "translate",
	}
	labels = {
		"align-top": "Align Top",
		"align-bottom": "Align Bottom",
		"align-left": "Align Left",
		"align-right": "Align Right",
		"align-center-x": "Align Center Horizontally",
		"align-center-y": "Align Center Vertically",
		"scale": "Scale",
		"mirror-vertical": "Vertical Mirror",
		"mirror-horizontal": "Horizontal Mirror",
		"translate": "Move Selected",
	}
	if type(expected_revision) is not int:
		raise TypeError("Top-level transform expected_revision must be an integer")
	if mode not in allowed_modes:
		raise ValueError("Top-level transform mode is unsupported")
	if type(root_keys) is not tuple or not root_keys:
		raise ValueError("Top-level transform roots must be a nonempty immutable tuple")
	if any(
		type(key) is not tuple or len(key) != 2
		or key[0] not in {"molecule", "presentation"}
		or type(key[1]) is not str or not key[1]
		for key in root_keys
	):
		raise ValueError("Top-level transform roots must be supported durable root keys")
	if len({identifier for _kind, identifier in root_keys}) != len(root_keys):
		raise ValueError("Top-level transform root IDs must be unique")
	if mode == "scale":
		if delta is not None:
			raise ValueError("Only translate accepts a top-level transform delta")
		if any(
			type(value) not in (int, float) or not math.isfinite(value) or value <= 0
			for value in (scale_x, scale_y)
		):
			raise ValueError("Top-level transform scale factors must be finite positive numbers")
	elif mode == "translate":
		if scale_x is not None or scale_y is not None:
			raise ValueError("Only scale accepts top-level transform scale factors")
		if (
			type(delta) is not tuple or len(delta) != 2
			or any(type(value) not in (int, float) or not math.isfinite(value) for value in delta)
		):
			raise ValueError("Top-level transform delta must be two finite non-bool numbers")
	else:
		if scale_x is not None or scale_y is not None or delta is not None:
			raise ValueError("Only scale or translate accepts top-level transform parameters")
	return PersistentOperationRequest(
		"top-level.transform.apply", labels[mode],
		(
			("expected_revision", expected_revision), ("mode", mode),
			("root_ids", tuple(identifier for _kind, identifier in root_keys)),
			("scale_x", scale_x), ("scale_y", scale_y), ("delta", delta),
		),
		frozenset(root_keys),
	)
