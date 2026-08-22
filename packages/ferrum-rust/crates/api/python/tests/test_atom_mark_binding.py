"""Installed-extension behavior for the closed Rust atom-mark boundary."""

import ferrum_chem
import pytest


SOURCE = (
	"<cdml xmlns='urn:ferrum:cdml'><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
	"<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)


def test_atom_marks_are_typed_rendered_and_undoable() -> None:
	"""One accepted mark remains a typed projection and semantic render batch."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	added = session.submit(
		0,
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", ferrum_chem.AtomMarkActionV1.add,
			ferrum_chem.AtomMarkKindV1.plus, None,
		),
	).observation
	atom = added.projection.molecules[0].atoms[0]

	assert atom.formal_charge == 1 and len(atom.marks) == 1
	mark = atom.marks[0]
	assert mark.kind == ferrum_chem.AtomMarkKindV1.plus
	assert mark.same_type_ordinal == 0 and mark.radial_offset > 0.0
	plan = session.observe_render(1).molecule_plans[0].plan
	batch = next(
		batch for batch in plan.batches if batch.target.record_id.id == "a"
	)
	assert tuple(operation.kind for operation in batch.operations)[-3:] == (
		"ellipse", "line", "line",
	)
	assert isinstance(batch.operations[-3].operation, ferrum_chem.EllipseOpV1)

	removed = session.submit(
		1,
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", ferrum_chem.AtomMarkActionV1.remove,
			ferrum_chem.AtomMarkKindV1.plus, 0,
		),
	).observation
	assert removed.projection.molecules[0].atoms[0].formal_charge is None
	assert removed.projection.molecules[0].atoms[0].marks == []
	assert len(session.undo(2).observation.projection.molecules[0].atoms[0].marks) == 1


def test_atom_mark_intent_rejects_hostile_selectors_before_mutation() -> None:
	"""Malformed Python selectors never reach revisioned document mutation."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()

	for invalid in (True, -1, 2**64):
		with pytest.raises(ferrum_chem.OperationValidationError):
			ferrum_chem.DocumentOperationV1.apply_atom_mark(
				"m", "a", ferrum_chem.AtomMarkActionV1.remove,
				ferrum_chem.AtomMarkKindV1.radical, invalid,
			)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", ferrum_chem.AtomMarkActionV1.add,
			ferrum_chem.AtomMarkKindV1.radical, 0,
		)
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", "add", ferrum_chem.AtomMarkKindV1.radical, None,
		)
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)
