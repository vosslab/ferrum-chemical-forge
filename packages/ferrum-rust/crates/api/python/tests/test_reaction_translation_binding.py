"""Installed-extension contract for opaque Rust reaction translation."""

from __future__ import annotations

import lxml.etree

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="left"><atom id="left-a" name="C">'
	'<point x="0" y="0"/></atom></molecule>'
	'<molecule id="right"><atom id="right-a" name="O">'
	'<point x="100" y="0"/></atom></molecule>'
	'<arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/></arrow>'
	'<reaction id="strict"><reactant idref="left"/><product idref="right"/>'
	'<arrow idref="arrow"/></reaction></cdml>'
)
_XML_PARSER = lxml.etree.XMLParser(
	load_dtd=False,
	resolve_entities=False,
	no_network=True,
	huge_tree=False,
)


def _reaction_members(session: object) -> tuple[str, ...]:
	"""Return the strict reaction's durable member-root identities."""
	snapshot = session.snapshot()
	observation = session.observe_reaction_list_v1(snapshot.revision, snapshot.digest)
	reaction, = observation.reactions
	assert reaction.strict is True
	return tuple(member.document_object_id for member in reaction.members)


def _reaction_references(cdml: str) -> set[str]:
	"""Return the strict reaction's source relationships, excluding document identity."""
	root = lxml.etree.fromstring(cdml.encode("utf-8"), parser=_XML_PARSER)
	reaction, = root.findall("{urn:ferrum:cdml}reaction")
	return {member.attrib["idref"] for member in reaction}


def test_translation_resolves_to_generic_transition_preserves_references_and_undoes() -> None:
	"""One semantic gesture resolves into the sole generic transition receipt."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	initial_members = _reaction_members(session)
	initial_references = _reaction_references(before.cdml)
	interaction = session.observe_render_interaction_v1(before.revision, before.digest)
	selection = None
	for member_id in _reaction_members(session):
		selection = session.select_render_interaction_roots_v1(
			interaction,
			selection,
			ferrum_chem.RenderInteractionQueryV1.root(
				member_id, ferrum_chem.RenderInteractionModifierV1.toggle,
			),
		)
	assert selection is not None
	gesture = session.begin_render_interaction_translation_v1(
		selection, 10.0, 20.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	accepted = session.commit_render_interaction_translation_v1(gesture, 45.0, 40.0)
	changed = accepted.result.observation.snapshot

	assert accepted.changed is True
	assert changed.revision == before.revision + 1
	assert _reaction_references(changed.cdml) == initial_references
	assert _reaction_members(session) == initial_members
	undone = session.undo(changed.revision).observation.snapshot
	assert _reaction_references(undone.cdml) == initial_references
	assert _reaction_members(session) == initial_members
