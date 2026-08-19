"""Private Rust explicit-fragment seam for one Ferrum document tab."""


#============================================
class FerrumNativeExplicitFragmentTabMixin:
	"""Submit a frozen explicit-fragment request and retain durable selection."""

	#============================================
	def create_explicit_fragment_v1(self, expected_revision: int,
			expected_digest: str, molecule_id: str, name: str,
			atom_ids: tuple[str, ...], bond_ids: tuple[str, ...]) -> object:
		"""Create one Rust-owned annotation from exact durable source facts."""
		self._require_mutable()
		if (
			type(expected_revision) is not int
			or type(expected_digest) is not str
			or type(molecule_id) is not str
			or type(name) is not str
			or type(atom_ids) is not tuple
			or type(bond_ids) is not tuple
		):
			raise TypeError("Ferrum fragment creation requires exact frozen inputs")
		if (
			not atom_ids and not bond_ids
			or any(type(identifier) is not str or not identifier for identifier in atom_ids)
			or any(type(identifier) is not str or not identifier for identifier in bond_ids)
			or len(frozenset(atom_ids)) != len(atom_ids)
			or len(frozenset(bond_ids)) != len(bond_ids)
		):
			raise ValueError("Ferrum fragment creation requires distinct durable members")
		snapshot = self.current_snapshot
		if snapshot.revision != expected_revision or snapshot.digest != expected_digest:
			raise RuntimeError("document changed before fragment creation")
		result = self._session.create_explicit_fragment_v1(
			expected_revision, expected_digest, molecule_id, name, atom_ids, bond_ids,
		)
		authoritative = result.operation.observation.snapshot
		if (
			authoritative.revision != snapshot.revision
			or authoritative.digest != snapshot.digest
		):
			selection = tuple(("atom", identifier) for identifier in atom_ids) + tuple(
				("bond", identifier) for identifier in bond_ids
			)
			self._install_mutation_result(result.operation, selection)
		return result
