"""Qt-facing labels and choice sets for canonical CDML bond styles."""


# These choices are editor presentation data. Rust owns code semantics and
# authorable orders without importing this module.
NATIVE_AUTHORABLE_BOND_TYPE_CHOICES = (
	("n", "Normal"),
	("w", "Wedge"),
	("h", "Hashed wedge"),
	("b", "Bold"),
	("d", "Dashed"),
	("s", "Wavy"),
	("q", "Haworth front edge"),
)

#============================================
def native_authorable_choices() -> tuple[tuple[str, str], ...]:
	"""Return the closed Rust-renderable presentation choices for the editor."""
	return NATIVE_AUTHORABLE_BOND_TYPE_CHOICES


#============================================
def label_for_bond_type(bond_type: str) -> str:
	"""Return the accurate presentation label for one known CDML style.

	Args:
		bond_type: Canonical CDML bond type character.

	Returns:
		Human-facing label, including a visible unknown value when unsupported.
	"""
	for code, label in NATIVE_AUTHORABLE_BOND_TYPE_CHOICES:
		if code == bond_type:
			return label
	return f"Unknown bond style ({bond_type})"
