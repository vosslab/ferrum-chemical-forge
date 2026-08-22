"""Installed private binding behavior for Rust-owned explicit fragments."""

import ferrum_chem


CDML = (
	"<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'><point x='0' y='0'/></atom>"
	"<atom id='b' name='O'><point x='10' y='0'/></atom>"
	"<bond id='ab' type='n1' start='a' end='b'/></molecule></cdml>"
)


def test_private_explicit_fragment_receipt_commits_authoritative_scalar_facts() -> None:
	"""One exact private receipt returns Rust's owner, closure, and observation."""
	session = ferrum_chem.DocumentSession.load(CDML)
	before = session.snapshot()
	molecule = session.observe(before.revision).projection.molecules[0]
	result = session.create_explicit_fragment_v1(
		before.revision, before.digest, molecule.id, "named part", (), ("ab",),
	)

	assert (result.fragment.name, result.fragment.molecule_id) == ("named part", molecule.id)
