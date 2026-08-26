"""Generic Python contract for standalone D-glucose Haworth insertion."""

import ferrum_chem


def test_standalone_haworth_uses_generic_transition_and_renderer_overlay() -> None:
	"""A supported recipe paints before one generic transition commits its molecule."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	request = session.resolve_standalone_haworth_transition_v1(
		0, "beta-D-glucofuranose", 13.0, -7.0,
	)
	prepared = session.prepare_session_operation_transition_v1(request)
	assert prepared.presentation_v1().precommit_overlay is not None
	result = session.commit_session_operation_transition_v1(prepared)
	inserted = result.outcome.molecule_inserted
	assert inserted is not None
	assert inserted.molecule_identifier
	assert len(inserted.atom_identifiers) == len(result.observation.projection.molecules[0].atoms)
	assert result.observation.projection.molecules[0].document_object_id
