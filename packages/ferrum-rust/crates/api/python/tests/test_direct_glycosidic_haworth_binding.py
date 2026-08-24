"""Generic Python contract for direct-glycosidic Haworth insertion."""

import ferrum_chem


SMILES = "O1CCCC1OC2CCCCO2"


def test_direct_haworth_uses_generic_transition_and_renderer_overlay() -> None:
	"""A supported source paints before one generic transition commits its molecule."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	source = ferrum_chem.prepare_direct_haworth_from_smiles_v1(SMILES)
	request = session.resolve_direct_haworth_transition_v1(0, source, 13.0, -7.0)
	prepared = session.prepare_session_operation_transition_v1(request)
	assert prepared.presentation_v1().precommit_overlay is not None
	result = session.commit_session_operation_transition_v1(prepared)
	assert any(
		molecule.source_id == result.outcome.molecule_inserted.molecule_identifier
		for molecule in result.observation.projection.molecules
	)
