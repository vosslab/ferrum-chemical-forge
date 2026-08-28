"""Installed PyO3 coverage for live Rust-owned template publication receipts."""

# Standard Library
import pathlib

# PIP3 modules
import pytest

# local repo modules
import ferrum_chem


SOURCE = (
	"<cdml xmlns='urn:ferrum:cdml'><molecule id='m' name='Reusable'>"
	"<atom id='a' name='C'><point x='0' y='0'/></atom>"
	"</molecule></cdml>"
)


#============================================
def test_live_receipt_publishes_without_exposing_cdml_or_a_plan(
		tmp_path: pathlib.Path,
		) -> None:
	"""The PyO3 handoff retains opaque Rust authority and only safe fence facts."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	receipt = session.prepare_user_template_publication_v1()

	assert receipt.display_name == "Reusable"
	assert isinstance(receipt.revision, int)
	assert len(receipt.digest) == 64
	assert not hasattr(receipt, "cdml")
	assert not hasattr(receipt, "plan")
	assert not hasattr(receipt, "document")

	destination = tmp_path / "reusable.cdml"
	publication = session.publish_user_template_v1(receipt, str(destination))

	assert destination.read_text() == publication.published_snapshot.cdml
	assert publication.published_snapshot.revision == receipt.revision


#============================================
def test_ineligible_live_document_refuses_before_creating_a_file(
		tmp_path: pathlib.Path,
		) -> None:
	"""An empty live document never gets a receipt or a publication side effect."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	destination = tmp_path / "not-a-template.cdml"

	with pytest.raises(ferrum_chem.UserTemplatePublicationError) as caught:
		session.prepare_user_template_publication_v1()

	assert caught.value.reason == "ineligible"
	assert not destination.exists()


#============================================
def test_stale_receipt_cannot_publish(tmp_path: pathlib.Path) -> None:
	"""A receipt's Rust revision fence prevents an old UI intent from writing."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	receipt = session.prepare_user_template_publication_v1()
	snapshot = session.snapshot()
	plan = ferrum_chem.prepare_user_template_v1(SOURCE)
	session.apply_user_template_v1(
		snapshot.revision, snapshot.digest, plan, 20.0, 20.0,
	)
	destination = tmp_path / "stale.cdml"

	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.publish_user_template_v1(receipt, str(destination))

	assert not destination.exists()
