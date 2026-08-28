"""Installed-wheel semantics for the private ordinary native artifact bridge."""

from pathlib import Path

import pytest

import ferrum_chem


CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
	'<point x="10" y="20"/></atom></molecule></cdml>'
)


def local_cdml_handle() -> object:
	"""Return the extension-issued V2 File/Open handle for native CDML."""
	return next(
		descriptor.route_handle
		for descriptor in ferrum_chem.DocumentSession.local_document_open_descriptors_v2()
		if ".cdml" in descriptor.suffixes
	)


def test_preparation_refuses_a_mismatched_observation_fence_without_mutation() -> None:
	"""The worker receipt is bound to the exact captured document observation."""
	session = ferrum_chem.DocumentSession.load(CDML)
	before = session.snapshot()
	observation = session.observe(before.revision)

	with pytest.raises(ferrum_chem.DocumentNativeArtifactError) as error:
		ferrum_chem.prepare_document_native_artifact_v1(
			observation, before.revision, "0" * 64, "svg",
		)

	after = session.snapshot()
	assert error.value.category == "provenance_mismatch" and (
		after.revision, after.digest,
	) == (before.revision, before.digest)


def test_local_origin_publication_uses_one_immutable_receipt_without_mutation(
		tmp_path: Path,
		) -> None:
	"""A local tab publishes through Rust while preserving its live document state."""
	source = tmp_path / "source.cdml"
	destination = tmp_path / "artifact.svg"
	source.write_text(CDML, encoding="utf-8")
	prepared_open = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
		str(source), local_cdml_handle(),
	)
	session, _render_observation, origin, _source_kind, _summary = prepared_open.take_admission_v2()
	before = session.snapshot()
	receipt = ferrum_chem.prepare_document_native_artifact_v1(
		session.observe(before.revision), before.revision, before.digest, "svg",
	)
	ferrum_chem.publish_prepared_document_native_artifact_v1(
		receipt, str(destination), origin,
	)

	assert destination.is_file()
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)


def test_local_origin_token_refuses_a_hard_link_destination(tmp_path: Path) -> None:
	"""The bridge carries the retained local source into artifact publication."""
	source = tmp_path / "source.cdml"
	alias = tmp_path / "source-alias.svg"
	source.write_text(CDML, encoding="utf-8")
	alias.hardlink_to(source)
	prepared_open = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
		str(source), local_cdml_handle(),
	)
	session, _render_observation, origin, _source_kind, _summary = prepared_open.take_admission_v2()
	before = session.snapshot()
	receipt = ferrum_chem.prepare_document_native_artifact_v1(
		session.observe(before.revision), before.revision, before.digest, "svg",
	)

	with pytest.raises(ferrum_chem.InvalidDestinationError):
		ferrum_chem.publish_prepared_document_native_artifact_v1(receipt, str(alias), origin)

	assert alias.read_text(encoding="utf-8") == CDML
