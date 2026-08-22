"""Semantic installed-extension checks for direct-root Text editing."""

import pytest

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><text id="label" background-color="#fff" keep="yes">'
	'<point x="10" y="20"/><font family="Arial" size="12" color="#000" '
	'v:keep="yes"><v:font-child/></font><v:between/>'
	'<ftext>old</ftext></text><v:root/></cdml>'
)


def text_projection(observation: object) -> object:
	"""Return the one Text projection from this focused source."""
	root = observation.projection.presentation_stack.roots[0]
	assert root.kind == "text"
	return root.text


def test_text_properties_are_one_frozen_atomic_edit_with_history() -> None:
	style = ferrum_chem.DocumentTextEditStyleV1
	run_type = ferrum_chem.DocumentTextEditRunV1
	change_type = ferrum_chem.DocumentTextPropertyChangeV1
	runs = (
		run_type.create("H", ()),
		run_type.create("2", (style.subscript,)),
		run_type.create("O <&>", ()),
	)
	changes = (
		change_type.runs(runs),
		change_type.font_family(None),
		change_type.font_size(18),
		change_type.color("#AbC"),
		change_type.background_color(None),
	)
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_text_properties("label", changes),
	).observation
	text = text_projection(changed)

	assert changed.snapshot.revision == 1
	assert (text.font.family, text.font.size, text.font.color) == (None, 18.0, "#aabbcc")
	assert [(run.text, run.styles) for run in text.runs] == [
		("H", ()), ("2", ("subscript",)), ("O <&>", ()),
	]
	assert text.background.color is None
	assert all(value in changed.snapshot.cdml for value in (
		'keep="yes"', '<v:font-child', '<v:between', '<v:root',
	))
	assert text_projection(session.undo(1).observation).runs[0].text == "old"
	assert text_projection(session.redo(2).observation).runs[-1].text == "O <&>"
	reopened = ferrum_chem.DocumentSession.load(changed.snapshot.cdml)
	assert text_projection(reopened.observe(0)).runs[-1].text == "O <&>"
	with pytest.raises(AttributeError):
		changes[0].value = "forged"


def test_text_properties_reject_non_closed_python_intent_without_mutation() -> None:
	class TupleSubclass(tuple):
		pass

	style = ferrum_chem.DocumentTextEditStyleV1
	run_type = ferrum_chem.DocumentTextEditRunV1
	change_type = ferrum_chem.DocumentTextPropertyChangeV1
	run = run_type.create("x", (style.superscript,))
	change = change_type.runs((run,))
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()

	with pytest.raises(TypeError):
		run_type.create("x", [style.bold])
	with pytest.raises(ferrum_chem.OperationValidationError):
		run_type.create("x", (style.bold,) * 5)
	with pytest.raises(ferrum_chem.OperationValidationError):
		run_type.create("x", (style.subscript, style.superscript))
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.runs(())
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.set_text_properties("label", [change])
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_text_properties(
			"label", TupleSubclass((change,)),
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_text_properties("label", (change,) * 6)

	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)
