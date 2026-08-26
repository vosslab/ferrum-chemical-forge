"""Installed-extension checks for the private native bond-capacity receipt."""

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.08">
 <molecule id="within">
  <atom id="within-c" name="C" explicit_hydrogens="4">
   <point x="0" y="0"/>
  </atom>
 </molecule>
 <molecule id="exceeds">
  <atom id="excess-c" name="C" explicit_hydrogens="4">
   <point x="2" y="0"/>
  </atom>
  <atom id="excess-o" name="O"><point x="3" y="0"/></atom>
  <bond id="excess-bond" start="excess-c" end="excess-o" type="n1"/>
 </molecule>
</cdml>
"""


def test_private_bond_capacity_orders_roots_and_preserves_observation() -> None:
	"""Frozen selected roots yield ordered Rust outcomes without a session edit."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	before = session.snapshot()
	roots = observation.projection.molecules

	receipt = ferrum_chem.inspect_document_bond_capacity_v1(
		observation, 0, observation.snapshot.digest,
		(roots[1].document_object_id, roots[0].document_object_id),
	)

	assert [record.source_id for record in receipt.records] == ["within", "exceeds"]
	assert receipt.records[0].category == "within_capacity"
	excess = receipt.records[1].atoms[0]
	assert (excess.source_id, excess.category, excess.demand, excess.capacity) == (
		"excess-c", "exceeds_capacity", 5, 4,
	)
	assert (
		session.snapshot().revision,
		session.snapshot().digest,
		session.snapshot().is_dirty,
	) == (before.revision, before.digest, before.is_dirty)


def test_private_bond_capacity_refuses_incomplete_profile_without_partial_atoms() -> None:
	"""Unsupported authored chemistry remains a root-level non-result."""
	source = """\
<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="unsupported">
 <atom id="c1" name="C" valency="4"><point x="0" y="0"/></atom>
</molecule></cdml>
"""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	root_id = observation.projection.molecules[0].document_object_id

	receipt = ferrum_chem.inspect_document_bond_capacity_v1(
		observation, 0, observation.snapshot.digest, (root_id,),
	)

	assert receipt.records[0].category == "not_checked"
	assert receipt.records[0].atoms == ()
	with pytest.raises(ferrum_chem.DocumentBondCapacityError):
		ferrum_chem.inspect_document_bond_capacity_v1(
			observation, 0, observation.snapshot.digest, (root_id, root_id),
		)
	with pytest.raises(ferrum_chem.DocumentBondCapacityError):
		ferrum_chem.inspect_document_bond_capacity_v1(
			observation, 1, observation.snapshot.digest, (root_id,),
		)


def test_private_bond_capacity_retains_absent_and_authored_neutral_facts() -> None:
	"""The read-only receipt keeps authored presence apart from calculation zero."""
	source = """\
<cdml xmlns="urn:ferrum:cdml" version="26.08">
 <molecule id="absent"><atom id="missing" name="C"><point x="0" y="0"/></atom></molecule>
 <molecule id="authored"><atom id="zero" name="C" charge="0" explicit_hydrogens="0">
  <point x="1" y="0"/>
 </atom></molecule>
</cdml>
"""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	root_ids = tuple(root.document_object_id for root in observation.projection.molecules)

	receipt = ferrum_chem.inspect_document_bond_capacity_v1(
		observation, 0, observation.snapshot.digest, root_ids,
	)

	absent, authored = (record.atoms[0] for record in receipt.records)
	assert (
		absent.explicit_hydrogens.was_authored,
		absent.explicit_hydrogens.value_or_zero,
		absent.formal_charge.was_authored,
		absent.formal_charge.value_or_zero,
	) == (False, 0, False, 0)
	assert (
		authored.explicit_hydrogens.was_authored,
		authored.explicit_hydrogens.value_or_zero,
		authored.formal_charge.was_authored,
		authored.formal_charge.value_or_zero,
	) == (True, 0, True, 0)
