"""Stable behavior checks for Rust-owned new-document construction."""

import defusedxml.ElementTree
import ferrum_chem


def test_create_empty_document_v1_is_clean_and_projects_no_selectable_roots() -> None:
	"""New documents have Rust-owned canonical root facts and no selectable content."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	observation = session.observe(0)
	root = defusedxml.ElementTree.fromstring(observation.snapshot.cdml)

	assert (
		root.tag == "{urn:ferrum:cdml}cdml"
		and root.attrib["version"] == "26.07"
	)
	assert (
		observation.snapshot.is_dirty is False
		and observation.projection.molecules == []
		and observation.projection.presentation_stack.entries == ()
	)


def test_create_empty_document_v1_semantically_reopens_at_revision_zero() -> None:
	"""The saved canonical empty baseline is accepted by the normal backend loader."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	reopened = ferrum_chem.DocumentSession.load(session.snapshot().cdml)

	assert reopened.observe(0).snapshot.revision == 0 and reopened.snapshot().is_dirty is False
