"""Private installed-extension behavior for exact selected SDF export."""

# Standard Library
import pathlib

# PIP3 modules
import ferrum_chem
import pytest


IMPORTED_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m" name="display name">'
	'<atom id="a" name="C"><point x="2" y="7"/></atom>'
	'<f:interchange-record xmlns:f="urn:ferrum-chemical-forge:interchange-import:v1" '
	'encoding="utf8-hex-v1" title="496d706f72746564207469746c65">'
	'<f:property name="4e4f5445" value="6669727374"/>'
	'<f:property name="4e4f5445" value="7365636f6e64"/>'
	'</f:interchange-record></molecule></cdml>'
)


#============================================
def _observation(source: str) -> tuple[object, object, str]:
	"""Return one exact observation, root, and unchanged snapshot."""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].document_object_id
	return observation, molecule_id, observation.snapshot.cdml


#============================================
def test_imported_title_and_duplicate_properties_reach_one_exact_sdf_record() -> None:
	"""Keep authoritative imported metadata outside RDKit's property map."""
	observation, molecule_id, before = _observation(IMPORTED_SOURCE)
	receipt = ferrum_chem.export_document_molecule_sdf_v1(
		observation,
		0,
		observation.snapshot.digest,
		molecule_id,
		ferrum_chem.MolblockVersionV1.v3000,
	)

	assert receipt.schema == "ferrum-document-molecule-sdf-v1"
	assert receipt.profile == "document-xy-to-chemistry-x-minus-y-rust-sdf-envelope-v1"
	assert receipt.title == "Imported title"
	assert receipt.sdf.startswith("Imported title\n")
	assert ">  <NOTE>\nfirst\n\n>  <NOTE>\nsecond\n\n$$$$\n" in receipt.sdf
	assert observation.snapshot.cdml == before


#============================================
def test_ordinary_name_and_explicit_syntax_are_retained() -> None:
	"""Use the molecule name only when no imported record metadata exists."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml"><molecule id="m" name="ordinary title">'
		'<atom id="a" name="O"><point x="0" y="0"/></atom>'
		'</molecule></cdml>'
	)
	observation, molecule_id, _before = _observation(source)
	receipt = ferrum_chem.export_document_molecule_sdf_v1(
		observation,
		0,
		observation.snapshot.digest,
		molecule_id,
		ferrum_chem.MolblockVersionV1.v2000,
	)

	assert receipt.title == "ordinary title"
	assert "V2000" in receipt.sdf
	assert receipt.sdf.endswith("M  END\n$$$$\n")


#============================================
def test_invalid_text_and_stale_provenance_use_the_sdf_error_contract() -> None:
	"""Map Python text and observation failures before native execution."""
	observation, molecule_id, _before = _observation(IMPORTED_SOURCE)
	arguments = (
		observation,
		0,
		observation.snapshot.digest,
		molecule_id,
		ferrum_chem.MolblockVersionV1.v2000,
	)

	with pytest.raises(ferrum_chem.DocumentMoleculeSdfError):
		ferrum_chem.export_document_molecule_sdf_v1(
			observation, 1, *arguments[2:],
		)
	for digest, selector in (("\ud800", molecule_id), (arguments[2], "\ud800")):
		with pytest.raises(ferrum_chem.DocumentMoleculeSdfError) as caught:
			ferrum_chem.export_document_molecule_sdf_v1(
				observation,
				0,
				digest,
				selector,
				ferrum_chem.MolblockVersionV1.v2000,
			)
		assert caught.value.reason


#============================================
def test_authenticated_receipt_publishes_exact_completed_bytes(
		tmp_path: pathlib.Path) -> None:
	"""Give only the immutable Rust receipt to the artifact publisher."""
	observation, molecule_id, before = _observation(IMPORTED_SOURCE)
	receipt = ferrum_chem.export_document_molecule_sdf_v1(
		observation,
		0,
		observation.snapshot.digest,
		molecule_id,
		ferrum_chem.MolblockVersionV1.v2000,
	)
	destination = tmp_path / "record.sdf"
	publication = ferrum_chem.publish_document_molecule_sdf_v1(
		receipt, str(destination),
	)

	assert destination.read_text(encoding="utf-8") == receipt.sdf
	assert type(publication.directory_entry_confirmed) is bool
	assert observation.snapshot.cdml == before
