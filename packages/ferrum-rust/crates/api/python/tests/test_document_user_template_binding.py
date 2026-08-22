"""Installed-extension checks for private native user-template operations."""

import pytest

import ferrum_chem


_TEMPLATE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07">
 <standard line_width="9"/>
 <paper id="template-paper" type="A4"/>
 <molecule id="source-molecule" name="  Example molecule  ">
  <atom id="source-a" name="C"><point x="0" y="2"/></atom>
  <atom id="source-b" name="O"><point x="10" y="4"/></atom>
  <bond id="source-bond" start="source-a" end="source-b" type="n1"/>
 </molecule>
</cdml>
"""


def _facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return durable facts needed to prove atomic refusal."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


def test_private_template_inspection_derives_catalog_facts() -> None:
	"""Rust derives the catalog name and atom centroid from chemical content."""
	plan = ferrum_chem.prepare_user_template_v1(_TEMPLATE)

	assert (
		plan.display_name,
		plan.atom_centroid_x,
		plan.atom_centroid_y,
	) == ("Example molecule", 5.0, 3.0)


def test_private_template_insertion_keeps_context_separate() -> None:
	"""Insertion retains authored geometry while omitting inspection context."""
	plan = ferrum_chem.prepare_user_template_v1(_TEMPLATE)
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	before = session.snapshot()
	result = session.apply_user_template_v1(0, before.digest, plan, 100.0, 50.0)
	observation = result.operation.observation
	molecule = observation.projection.molecules[0]
	atom_a, atom_b = molecule.atoms

	assert (
		(atom_a.position.x + atom_b.position.x) / 2.0,
		(atom_a.position.y + atom_b.position.y) / 2.0,
		atom_b.position.x - atom_a.position.x,
		atom_b.position.y - atom_a.position.y,
	) == pytest.approx((100.0, 50.0, 10.0, 2.0), abs=0.02)
	assert (
		"template-paper" in observation.snapshot.cdml,
		"line_width=\"9\"" in observation.snapshot.cdml,
		result.inserted_molecule_source_id == "source-molecule",
	) == (False, False, False)


def test_private_template_insertion_is_atomic_history_and_reauthenticates() -> None:
	"""Accepted placement is one history step; stale requests preserve the session."""
	plan = ferrum_chem.prepare_user_template_v1(_TEMPLATE)
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	baseline = session.snapshot()
	accepted = session.apply_user_template_v1(0, baseline.digest, plan, 20.0, 30.0)
	accepted_cdml = accepted.operation.observation.snapshot.cdml
	undone = session.undo(1)
	redone = session.redo(2)

	assert (
		undone.observation.snapshot.cdml,
		redone.observation.snapshot.cdml,
	) == (baseline.cdml, accepted_cdml)
	before_refusal = session.snapshot()
	with pytest.raises(ferrum_chem.DocumentUserTemplateError) as caught:
		session.apply_user_template_v1(2, before_refusal.digest, plan, 0.0, 0.0)
	assert (
		bool(caught.value.reason),
		_facts(session.snapshot()),
	) == (True, _facts(before_refusal))


@pytest.mark.parametrize(
	"source, reason",
	(
		(b"<cdml xmlns='urn:ferrum:cdml'/>", "exact built-in string"),
		("\ud800", "valid UTF-8 text"),
		("<cdml xmlns='urn:ferrum:cdml'><molecule/></cdml>", "at least one direct atom"),
		(
			"<cdml xmlns='urn:ferrum:cdml'><molecule><atom><point x='0' y='0'/></atom>"
			"<template/></molecule></cdml>",
			"legacy template",
		),
	),
)
def test_private_template_admission_has_one_operation_specific_error(
		source: object, reason: str,
		) -> None:
	"""Malformed Python and ineligible CDML remain privately typed."""
	with pytest.raises(ferrum_chem.DocumentUserTemplateError) as caught:
		ferrum_chem.prepare_user_template_v1(source)
	assert reason in caught.value.reason

