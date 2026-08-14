"""Installed-extension checks for private document Molfile export V1."""

from pathlib import Path

import pytest

import ferrum_chem


_SOURCE = """\
<cdml version="1.0"><molecule id="m1">
 <atom id="a1" name="N" charge="1" isotope="15" explicit_hydrogens="3">
  <point x="2.5" y="7.5"/>
 </atom>
 <atom id="a2" name="C"><point x="12.5" y="-4"/></atom>
 <bond id="b1" start="a1" end="a2" type="n1"/>
</molecule></cdml>
"""


def _address(source: str = _SOURCE) -> tuple[object, object, str]:
	"""Return one session, frozen observation, and durable direct-root selector."""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	return session, observation, observation.projection.molecules[0].id


@pytest.mark.parametrize(
	("version", "label"),
	(
		(ferrum_chem.MolblockVersionV1.v2000, "V2000"),
		(ferrum_chem.MolblockVersionV1.v3000, "V3000"),
	),
)
def test_private_document_molfile_is_explicit_and_provenance_bound(
		version: object, label: str,
		) -> None:
	"""Each syntax returns one immutable, parseable, coordinate-bearing receipt."""
	session, observation, molecule_id = _address()
	before = session.snapshot()
	receipt = ferrum_chem.export_document_molecule_molblock_v1(
		observation, 0, observation.snapshot.digest, molecule_id, version,
	)

	assert type(receipt) is ferrum_chem.DocumentMoleculeMolblockV1
	assert receipt.schema == "ferrum-document-molecule-molblock-v1"
	assert receipt.profile == "document-xy-to-chemistry-x-minus-y-v1"
	assert receipt.source_revision == 0
	assert receipt.source_digest == observation.snapshot.digest
	assert receipt.molecule_id == molecule_id
	assert receipt.version is version
	assert receipt.title is None
	assert label in receipt.molblock
	parsed = ferrum_chem.molblock_to_molecule(receipt.molblock)
	assert tuple((point.x, point.y) for point in parsed.coordinates) == (
		(2.5, -7.5), (12.5, 4.0),
	)
	after = session.snapshot()
	assert (after.cdml, after.revision, after.digest, after.is_dirty) == (
		before.cdml, before.revision, before.digest, before.is_dirty,
	)


def test_private_document_molfile_publishes_exact_receipt_bytes(
		tmp_path: Path,
		) -> None:
	"""Rust publishes the frozen writer result without altering its session."""
	session, observation, molecule_id = _address()
	receipt = ferrum_chem.export_document_molecule_molblock_v1(
		observation, 0, observation.snapshot.digest, molecule_id,
		ferrum_chem.MolblockVersionV1.v3000,
	)
	before = session.snapshot()
	destination = tmp_path / "molecule.mol"

	publication = ferrum_chem.publish_document_molecule_molblock_v1(
		receipt, destination,
	)

	assert type(publication) is ferrum_chem.DocumentMoleculeMolblockPublicationV1
	assert type(publication.directory_entry_confirmed) is bool
	assert destination.read_bytes() == receipt.molblock.encode("utf-8")
	after = session.snapshot()
	assert (after.cdml, after.revision, after.digest, after.is_dirty) == (
		before.cdml, before.revision, before.digest, before.is_dirty,
	)


def test_private_document_molfile_rejects_source_facts_before_native_export() -> None:
	"""Drawing, stale, and non-root facts remain typed failures."""
	styled = _SOURCE.replace('type="n1"', 'type="w1"')
	_session, observation, molecule_id = _address(styled)
	with pytest.raises(ferrum_chem.DocumentMoleculeMolblockError) as drawing:
		ferrum_chem.export_document_molecule_molblock_v1(
			observation, 0, observation.snapshot.digest, molecule_id,
			ferrum_chem.MolblockVersionV1.v2000,
		)
	assert "drawing style" in drawing.value.reason

	_session, observation, molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeMolblockError) as stale:
		ferrum_chem.export_document_molecule_molblock_v1(
			observation, 1, observation.snapshot.digest, molecule_id,
			ferrum_chem.MolblockVersionV1.v2000,
		)
	assert "document changed" in stale.value.reason
	atom_id = "ferrum-document-object-v1/6d6f6c6563756c652f61746f6d/source/6131"
	with pytest.raises(ferrum_chem.DocumentMoleculeMolblockError) as root:
		ferrum_chem.export_document_molecule_molblock_v1(
			observation, 0, observation.snapshot.digest, atom_id,
			ferrum_chem.MolblockVersionV1.v2000,
		)
	assert "direct-root molecule" in root.value.reason


@pytest.mark.parametrize("title", ("authored \u03b2-lactam", ""))
def test_private_document_molfile_preserves_exact_authored_title(title: str) -> None:
	"""The native writer, not Python text surgery, owns the first title line."""
	named = _SOURCE.replace(
		'id="m1"', f'id="m1" name="{title}"',
	)
	_session, observation, molecule_id = _address(named)
	receipt = ferrum_chem.export_document_molecule_molblock_v1(
		observation, 0, observation.snapshot.digest, molecule_id,
		ferrum_chem.MolblockVersionV1.v3000,
	)

	assert receipt.title == title
	assert receipt.molblock.splitlines()[0] == receipt.title


def test_private_document_molfile_maps_surrogate_text_to_its_error() -> None:
	"""Unencodable Python text stays inside the operation error contract."""
	_session, observation, molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeMolblockError) as digest:
		ferrum_chem.export_document_molecule_molblock_v1(
			observation, 0, "\ud800", molecule_id,
			ferrum_chem.MolblockVersionV1.v2000,
		)
	assert digest.value.reason == "expected digest must be valid UTF-8 text"
	with pytest.raises(ferrum_chem.DocumentMoleculeMolblockError) as selector:
		ferrum_chem.export_document_molecule_molblock_v1(
			observation, 0, observation.snapshot.digest, "\ud800",
			ferrum_chem.MolblockVersionV1.v2000,
		)
	assert selector.value.reason == "molecule selector must be valid UTF-8 text"


def test_private_document_molfile_is_discoverable_but_absent_from_stub() -> None:
	"""The Qt-only operation is runtime-private rather than a wheel promise."""
	assert "export_document_molecule_molblock_v1" in dir(ferrum_chem)
	assert "DocumentMoleculeMolblockV1" in dir(ferrum_chem)
	assert "publish_document_molecule_molblock_v1" in dir(ferrum_chem)
	assert "DocumentMoleculeMolblockPublicationV1" in dir(ferrum_chem)
	stub_path = Path(__file__).resolve().parents[2] / "wheel_metadata" / "ferrum_chem.pyi"
	stub = stub_path.read_text(encoding="utf-8")
	assert "export_document_molecule_molblock_v1" not in stub
	assert "DocumentMoleculeMolblockV1" not in stub
	assert "publish_document_molecule_molblock_v1" not in stub
	assert "DocumentMoleculeMolblockPublicationV1" not in stub
