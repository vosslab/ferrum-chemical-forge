"""Installed-extension contract for frozen native template-catalog snapshots."""

from __future__ import annotations

from pathlib import Path

import pytest

import ferrum_chem


_TEMPLATE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07">
 <molecule id="source-molecule" name="Example molecule">
  <atom id="source-a" name="C"><point x="0" y="2"/></atom>
  <atom id="source-b" name="O"><point x="40" y="4"/></atom>
  <bond id="source-bond" start="source-a" end="source-b" type="n1"/>
 </molecule>
</cdml>
"""


def _entry(snapshot: object, source: str) -> object:
	"""Return one native entry with the named source kind."""
	return next(entry for entry in snapshot.entries if entry.source_kind == source)


def test_snapshot_projects_shipped_and_admitted_user_entries(tmp_path: Path) -> None:
	"""The immutable projection publishes facts, never templates or filesystem capability."""
	path = tmp_path / "example.cdml"
	path.write_text(_TEMPLATE, encoding="utf-8")

	snapshot = ferrum_chem.snapshot_template_catalog_v1(str(tmp_path))
	shipped = _entry(snapshot, "shipped")
	user = _entry(snapshot, "user_directory")
	sulfur_entry = next(
		entry
		for entry in snapshot.entries
		if "sulfur" in {term.casefold() for term in entry.search_terms}
	)

	assert (
		snapshot.schema,
		snapshot.snapshot_identity_algorithm,
		len(snapshot.snapshot_identity),
		user.content_identity_algorithm,
		len(user.content_identity),
		user.compatibility_format,
		user.provenance_source_kind,
	) == (
		"ferrum-template-catalog-v1",
		"sha256",
		64,
		"sha256",
		64,
		"cdml",
		"configured_user_directory",
	)
	assert shipped.family and shipped.category and shipped.label
	assert shipped.family_label and shipped.family_order < user.family_order
	assert snapshot.limits_max_candidates >= snapshot.limits_max_entries
	assert snapshot.limits_max_refusals >= 1
	assert "sulfur" in {term.casefold() for term in sulfur_entry.search_terms}
	assert user.label == "Example molecule"
	for value in (snapshot, shipped, user):
		for forbidden in ("cdml", "payload", "path", "recipe", "plan", "fd"):
			assert not hasattr(value, forbidden)
	with pytest.raises(AttributeError):
		user.label = "forged"
	with pytest.raises(TypeError):
		ferrum_chem.snapshot_template_catalog_v1(path)


def test_user_identity_comes_from_admitted_bytes_not_filename(tmp_path: Path) -> None:
	"""Changing same-name content issues a different key and refreshes the snapshot identity."""
	path = tmp_path / "same-name.cdml"
	path.write_text(_TEMPLATE, encoding="utf-8")
	first = ferrum_chem.snapshot_template_catalog_v1(str(tmp_path))
	first_entry = _entry(first, "user_directory")
	path.write_text(_TEMPLATE.replace("Example molecule", "Changed molecule"), encoding="utf-8")
	second = ferrum_chem.snapshot_template_catalog_v1(str(tmp_path))
	second_entry = _entry(second, "user_directory")

	assert (
		first_entry.key != second_entry.key,
		first_entry.content_identity != second_entry.content_identity,
		first.snapshot_identity != second.snapshot_identity,
	) == (True, True, True)


def test_user_placement_and_stale_fence_are_rust_owned(tmp_path: Path) -> None:
	"""A retained selection applies once; a stale document fence makes no mutation."""
	(tmp_path / "example.cdml").write_text(_TEMPLATE, encoding="utf-8")
	snapshot = ferrum_chem.snapshot_template_catalog_v1(str(tmp_path))
	entry = _entry(snapshot, "user_directory")
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	baseline = session.snapshot()
	placed = session.place_template_catalog_entry_v1(
		snapshot, entry.key, baseline, 30.0, 40.0,
	)
	before_refusal = session.snapshot()

	with pytest.raises(ferrum_chem.TemplateCatalogError) as caught:
		session.place_template_catalog_entry_v1(
			snapshot, entry.key, baseline, 30.0, 40.0,
		)
	assert (
		placed.source_kind,
		placed.inserted_molecule_object_id is not None,
		caught.value.category,
		session.snapshot().revision,
	) == ("user_directory", True, "document_stale", before_refusal.revision)


def test_placement_requires_a_native_document_snapshot() -> None:
	"""Placement refuses a caller-supplied revision in place of the native fence."""
	catalog_snapshot = ferrum_chem.snapshot_template_catalog_v1(None)
	entry = _entry(catalog_snapshot, "shipped")
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	document_snapshot = session.snapshot()

	with pytest.raises(TypeError):
		session.place_template_catalog_entry_v1(
			catalog_snapshot, entry.key, document_snapshot.revision, 30.0, 40.0,
		)

	assert session.snapshot().revision == document_snapshot.revision


def test_bad_neighbors_are_typed_refusals_without_breaking_healthy_snapshot(
		tmp_path: Path,
		) -> None:
	"""Malformed and oversized candidates become closed retained refusals."""
	baseline = ferrum_chem.snapshot_template_catalog_v1(str(tmp_path))
	(tmp_path / "healthy.cdml").write_text(_TEMPLATE, encoding="utf-8")
	(tmp_path / "malformed.cdml").write_text("<cdml/>", encoding="utf-8")
	(tmp_path / "oversized.cdml").write_bytes(
		b"x" * (baseline.limits_max_file_bytes + 1)
	)

	snapshot = ferrum_chem.snapshot_template_catalog_v1(str(tmp_path))

	assert _entry(snapshot, "user_directory").label == "Example molecule"
	assert {"document_admission", "file_too_large"} <= {
		refusal.category for refusal in snapshot.refusals
	}
	assert all(refusal.recovery for refusal in snapshot.refusals)
	assert all(refusal.occurrences >= 1 for refusal in snapshot.refusals)
