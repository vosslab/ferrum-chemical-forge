"""Stage and dispose scene-less Qt graphics for hydrated CDML."""

# Standard Library
import dataclasses

# local repo modules
import oasa.cdml_document

import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.molecule_projection
import ferrum_qt.io.cdml_document_hydration
import ferrum_qt.models.document


@dataclasses.dataclass(frozen=True)
class PreparedProjection:
	"""Fresh, detached Qt projection decoded from complete canonical CDML.

	The bundle is deliberately frontend-only: it holds a new Qt ``Document``
	and scene-less graphics wrappers that a document session may install later.
	It neither changes OASA state nor serializes a candidate document.
	"""
	document: ferrum_qt.models.document.Document
	molecule_projections: tuple[tuple[object, tuple[object, ...]], ...]
	presentation_items: tuple[object, ...]
	# Atom-owned marks are declared with their parent wrapper.  They are not
	# scene roots and installation must never infer that ownership from scene().
	mark_parent_items: tuple[tuple[object, tuple[object, ...]], ...]
	mark_items: tuple[object, ...]


#============================================
#============================================
def prepare_synchronized_projection(
		projection_snapshot: oasa.cdml_document.CDMLProjectionSnapshot,
		retirement_reaper: object | None = None,
		) -> PreparedProjection:
	"""Prepare a scene-less projection from one exact backend snapshot."""
	document = ferrum_qt.io.cdml_document_hydration.hydrate_synchronized_cdml_document(
		projection_snapshot,
	)
	return _prepare_projection_from_document(document, retirement_reaper)


#============================================
def prepare_compatibility_projection_from_cdml(
		complete_cdml: str, retirement_reaper: object | None = None,
		) -> PreparedProjection:
	"""Prepare one standalone compatibility projection from raw CDML text.

	This route retains local legacy decoding and rendering behavior.  Session
	staging and snapshot rendering use :func:`prepare_synchronized_projection`
	with a complete backend projection envelope instead.
	"""
	document = ferrum_qt.io.cdml_document_hydration.decode_compatibility_cdml_string(
		complete_cdml,
	)
	return _prepare_projection_from_document(document, retirement_reaper)


#============================================
def _prepare_projection_from_document(
		document: ferrum_qt.models.document.Document,
		retirement_reaper: object | None,
		) -> PreparedProjection:
	"""Create detached graphics only after one route has hydrated a document."""
	molecule_projections = []
	presentation_items = []
	mark_items = []
	mark_parent_items = []
	try:
		built_molecules = ferrum_qt.canvas.molecule_projection.build_molecule_projections(
			document.molecules,
		)
		for molecule, items in built_molecules:
			molecule_projections.append((molecule, tuple(items)))

		atom_items = {}
		for _molecule, items in molecule_projections:
			for item in items:
				atom_model = getattr(item, "atom_model", None)
				if atom_model is not None:
					atom_items[atom_model] = item

		for model in document.presentation_objects:
			item = ferrum_qt.canvas.document_projection.create_presentation_item(
				model,
			)
			if item is not None:
				presentation_items.append(item)
		for model in document.marks:
			atom_item = atom_items.get(model.atom_model)
			if atom_item is None:
				continue
			item = ferrum_qt.canvas.document_projection.create_mark_item(
				model, atom_item,
			)
			if item is not None:
				mark_items.append(item)
				mark_parent_items.append((atom_item, (item,)))
		return PreparedProjection(
			document=document,
			molecule_projections=tuple(molecule_projections),
			presentation_items=tuple(presentation_items),
			mark_parent_items=tuple(mark_parent_items),
			mark_items=tuple(mark_items),
		)
	except Exception:
		_dispose_projection_parts(
			document, molecule_projections, presentation_items, mark_items,
			retirement_reaper,
		)
		raise


#============================================
def dispose_prepared_projection(
		prepared: PreparedProjection, retirement_reaper: object | None = None,
		) -> None:
	"""Release a prepared projection that was never installed into a scene."""
	_dispose_projection_parts(
		prepared.document,
		list(prepared.molecule_projections),
		list(prepared.presentation_items),
		list(prepared.mark_items), retirement_reaper,
	)


#============================================
def _require_complete_molecule_render_batches(
		molecule_core_observation: oasa.cdml_document.CDMLMoleculeCoreObservation | None,
		molecule_render_observation: oasa.cdml_document.CDMLMoleculeRenderObservation | None,
		) -> None:
	"""Require one portable paint batch for every synchronized renderable core child."""
	if molecule_core_observation is None or molecule_render_observation is None:
		return
	expected = {}
	accepted_molecules = set()
	for molecule_record in molecule_core_observation.records:
		if not molecule_record.renderable:
			continue
		if molecule_record.source_position in accepted_molecules:
			raise ValueError("molecule core render association is ambiguous")
		accepted_molecules.add(molecule_record.source_position)
		atom_source_ids = {}
		ambiguous_atom_ids = set()
		for atom_record in molecule_record.atoms:
			if atom_record.renderable:
				atom_key = (molecule_record.source_position, "atom", atom_record.source_position)
				if atom_key in expected:
					raise ValueError("molecule core render association is ambiguous")
				expected[atom_key] = None
				if atom_record.identifier is not None:
					if atom_record.identifier in atom_source_ids:
						ambiguous_atom_ids.add(atom_record.identifier)
						atom_source_ids.pop(atom_record.identifier)
					elif atom_record.identifier not in ambiguous_atom_ids:
						atom_source_ids[atom_record.identifier] = atom_record.source_position
		for bond_record in molecule_record.bonds:
			if (
					bond_record.renderable
					and bond_record.start_id in atom_source_ids
					and bond_record.end_id in atom_source_ids
					and bond_record.order is not None
					and bond_record.bond_type is not None
					):
				bond_key = (molecule_record.source_position, "bond", bond_record.source_position)
				if bond_key in expected:
					raise ValueError("molecule core render association is ambiguous")
				expected[bond_key] = None
	for batch in molecule_render_observation.batches:
		batch_key = (batch.molecule_source_position, batch.kind, batch.source_position)
		if batch.molecule_source_position not in accepted_molecules:
			raise ValueError("molecule render batch belongs to no accepted molecule core record")
		if batch_key not in expected:
			wrong_kind = any(
				molecule_position == batch.molecule_source_position
				and source_position == batch.source_position
				for molecule_position, _kind, source_position in expected
			)
			message = (
				"molecule render batch kind does not match its core child"
				if wrong_kind else "molecule render batch has no renderable core child"
			)
			raise ValueError(message)
		if expected[batch_key] is not None:
			raise ValueError("molecule render batch association is ambiguous")
		expected[batch_key] = batch
	missing = next((key for key, batch in expected.items() if batch is None), None)
	if missing is not None:
		raise ValueError("molecule render batch coverage is incomplete")


#============================================
def _dispose_projection_parts(
		document: ferrum_qt.models.document.Document,
		molecule_projections: list, presentation_items: list,
		mark_items: list, retirement_reaper: object | None = None,
		) -> None:
	"""Disconnect detached graphics bindings before releasing their models."""
	items = list(mark_items)
	items.extend(presentation_items)
	for _molecule, molecule_items in molecule_projections:
		items.extend(molecule_items)
	first_error = None
	try:
		ferrum_qt.canvas.document_projection.dispose_detached_items(
			items, reaper=retirement_reaper,
		)
	except Exception as exc:
		first_error = exc
	try:
		document.clear()
	except Exception as exc:
		if first_error is None:
			first_error = exc
	finally:
		# Always sever the prepared Document's QObject ownership after an item
		# cleanup fault.  It must never survive as a hidden owner of detached
		# models or graphics callbacks.
		try:
			document.setParent(None)
		except Exception as exc:
			if first_error is None:
				first_error = exc
		try:
			document.deleteLater()
		except Exception as exc:
			if first_error is None:
				first_error = exc
	if first_error is not None:
		raise RuntimeError(
			"Prepared projection was released after a disposal failure",
		) from first_error


#============================================
