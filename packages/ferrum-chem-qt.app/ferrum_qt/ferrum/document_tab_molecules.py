"""Durable molecule choices and canvas-authoring admission for Ferrum tabs."""

# standard library
from dataclasses import dataclass

# local repo modules
from ferrum_qt.ferrum.document_tab_errors import (
	FerrumNativeDocumentTabError,
	FerrumNativeDocumentTabUnrenderableMoleculeError,
)


@dataclass(frozen=True)
class FerrumNativeMoleculeChoice:
	"""One direct-root-ordered molecule available to a Ferrum tab."""

	object_id: str
	label: str
	source_order: int


class FerrumNativeDocumentMoleculeChoicesMixin:
	"""Derive durable molecule choices and authorize canvas mutations.

	The composing document tab supplies the installed immutable observations,
	current snapshot, and mutable-state guard.  This mixin keeps molecule choice
	and canvas eligibility on one durable Rust-observation boundary.
	"""

	#============================================
	def durable_molecule_choices(self) -> tuple[FerrumNativeMoleculeChoice, ...]:
		"""Return direct-root-ordered durable molecules from the installed observation."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		observation = self._document_observation
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		projection = observation.projection
		if type(projection) is not engine.DocumentProjectionV1:
			raise FerrumNativeDocumentTabError("Ferrum tab has no exact Rust document projection")
		molecules = projection.molecules
		direct_roots = projection.direct_roots
		if type(molecules) is not list:
			raise FerrumNativeDocumentTabError("Rust molecule projections are not an exact DTO list")
		if type(direct_roots) is not tuple:
			raise FerrumNativeDocumentTabError("Rust document direct roots are not an exact DTO tuple")
		molecules_by_id = {}
		for molecule in molecules:
			if type(molecule) is not engine.MoleculeProjectionV1:
				raise FerrumNativeDocumentTabError("Rust molecule projection has the wrong DTO type")
			object_id = getattr(molecule, "document_object_id", None)
			name = getattr(molecule, "name", None)
			if type(object_id) is not str or not object_id:
				raise FerrumNativeDocumentTabError("Rust molecule projection identity is invalid")
			if name is not None and type(name) is not str:
				raise FerrumNativeDocumentTabError("Rust molecule projection name is invalid")
			if object_id in molecules_by_id:
				raise FerrumNativeDocumentTabError("Rust molecule projection identities are not unique")
			molecules_by_id[object_id] = name
		molecule_roots = []
		root_ids = set()
		paint_orders = set()
		for root in direct_roots:
			object_id = getattr(root, "document_object_id", None)
			kind = getattr(root, "kind", None)
			paint_order = getattr(root, "paint_order", None)
			if type(object_id) is not str or not object_id:
				raise FerrumNativeDocumentTabError("Rust document direct-root identity is invalid")
			if type(kind) is not str or not kind:
				raise FerrumNativeDocumentTabError("Rust document direct-root kind is invalid")
			if type(paint_order) is not int or paint_order < 0 or paint_order >= 2**32:
				raise FerrumNativeDocumentTabError("Rust document direct-root paint order is invalid")
			if object_id in root_ids or paint_order in paint_orders:
				raise FerrumNativeDocumentTabError("Rust document direct roots are not unique")
			root_ids.add(object_id)
			paint_orders.add(paint_order)
			if kind == "molecule":
				molecule_roots.append((paint_order, object_id))
		if len(molecule_roots) != len(molecules_by_id):
			raise FerrumNativeDocumentTabError(
				"Rust molecule projections and molecule direct roots do not agree",
			)
		choices = []
		ordinal = 0
		for paint_order, object_id in sorted(molecule_roots):
			try:
				name = molecules_by_id.pop(object_id)
			except KeyError as exc:
				raise FerrumNativeDocumentTabError(
					"Rust molecule direct root has no molecule projection",
				) from exc
			ordinal += 1
			position_label = f"Molecule {ordinal}"
			label = position_label if name is None or not name.strip() else (
				f"{name} ({position_label})"
			)
			choices.append(
				FerrumNativeMoleculeChoice(object_id, label, paint_order),
			)
		if molecules_by_id:
			raise FerrumNativeDocumentTabError(
				"Rust molecule projection has no molecule direct root",
			)
		return tuple(choices)

	#============================================
	def canvas_authorable_molecule_choices(self) -> tuple[FerrumNativeMoleculeChoice, ...]:
		"""Return durable molecules proven by the installed Rust render plans."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		observation = self._render_observation
		if type(observation) is not engine.RenderObservationV1:
			raise FerrumNativeDocumentTabError(
				"Ferrum tab has no exact installed Rust render observation",
			)
		if (
			observation.document.snapshot.revision != self.current_snapshot.revision
			or observation.document.snapshot.digest != self.current_snapshot.digest
		):
			raise FerrumNativeDocumentTabError(
				"installed Rust render observation does not match the current document snapshot",
			)
		plan_ids = {
			plan.molecule.document_object_id
			for plan in observation.molecule_plans
		}
		return tuple(
			choice for choice in self.durable_molecule_choices()
			if choice.object_id in plan_ids
		)

	#============================================
	def _require_canvas_authorable_molecule(self, molecule_object_id: str) -> None:
		"""Require exact installed Rust render evidence before a canvas mutation."""
		if not any(
			choice.object_id == molecule_object_id
			for choice in self.canvas_authorable_molecule_choices()
		):
			raise FerrumNativeDocumentTabUnrenderableMoleculeError(molecule_object_id)
