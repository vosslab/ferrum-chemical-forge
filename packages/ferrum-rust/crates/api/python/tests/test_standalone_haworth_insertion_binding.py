"""Installed private seam for Rust-owned standalone D-glucose Haworth insertion."""

import ferrum_chem


def test_private_standalone_haworth_receipt_commits_rust_authored_semantic_drawing() -> None:
	"""The native receipt preview and committed projection retain one Haworth recipe."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	prepared = session.prepare_create_standalone_haworth_v1(
		0, "beta-D-glucofuranose", 13.0, -7.0,
	)
	result = session.commit_create_standalone_haworth_v1(0, prepared)
	molecule = next(
		candidate
		for candidate in result.observation.projection.molecules
		if candidate.source_id == prepared.molecule_identifier
	)

	assert (
		sum(atom.element == "C" for atom in molecule.atoms),
		sum(atom.element == "O" for atom in molecule.atoms),
		len(molecule.bonds),
		{bond.source_type for bond in molecule.bonds},
		any(
			bond.source_type == "q1"
			and bond.haworth_position == ferrum_chem.DocumentHaworthPositionV1.front
			for bond in molecule.bonds
		),
		all(
			bond.haworth_position == ferrum_chem.DocumentHaworthPositionV1.front
			for bond in molecule.bonds
			if bond.source_type == "w1"
		),
	) == (6, 6, 12, {"n1", "q1", "w1"}, True, True)


def test_haworth_render_stays_accepted_when_live_smarts_has_no_correspondence() -> None:
	"""A valid Haworth render remains openable without a paintable SMARTS plan."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	prepared = session.prepare_create_standalone_haworth_v1(
		0, "beta-D-glucofuranose", 13.0, -7.0,
	)
	result = session.commit_create_standalone_haworth_v1(0, prepared)
	assert len(result.observation.projection.molecules) == 1
	published = session._publish_live_render_plan_v1(session.snapshot().revision)

	assert len(published.molecule_plans) == 1
	assert published.molecule_plans[0].plan.batches
	assert len(published.issues) == 0
	try:
		session._run_live_document_smarts_query_v1("C", 1, 1)
	except ferrum_chem.LiveDocumentSmartsError as error:
		assert (error.category, error.reason) == (
			ferrum_chem.LiveDocumentSmartsCategoryV1.unsupported_document,
			ferrum_chem.LiveDocumentSmartsReasonV1.unsupported_document,
		)
	else:
		raise AssertionError("Haworth SMARTS query unexpectedly received a live plan")
