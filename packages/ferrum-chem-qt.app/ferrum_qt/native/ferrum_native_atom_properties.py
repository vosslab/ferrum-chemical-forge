"""Typed AtomDialog adaptation for the Rust-native Ferrum editor."""

# Standard Library
import dataclasses


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeAtomDialogModel:
	"""Scalar AtomDialog inputs copied from one frozen Rust atom projection."""

	symbol: str
	charge: int
	valency: int
	isotope: int | None
	multiplicity: int
	show: bool
	show_hydrogens: bool
	font_size: int
	line_color: str


#============================================
def dialog_model_from_projection(atom: object) -> FerrumNativeAtomDialogModel:
	"""Copy one exact Rust atom DTO into values representable by AtomDialog.

	Absent optional CDML facts are shown as ordinary presentation values.  The
	dialog's ``changes()`` comparison means they remain absent unless the user
	actually changes that field.  Facts the existing visual form cannot display
	faithfully are rejected instead of being silently clamped or remapped.
	"""
	import ferrum_chem
	if type(atom) is not ferrum_chem.AtomProjectionV1:
		raise TypeError("native atom properties require an exact Ferrum atom projection")
	if type(atom.element) is not str or not atom.element:
		raise ValueError("selected Rust atom has no editable element spelling")
	charge = 0 if atom.formal_charge is None else atom.formal_charge
	valency = 0 if atom.valence is None else atom.valence
	multiplicity = 1 if atom.multiplicity is None else atom.multiplicity
	show = True if atom.show is None else atom.show
	show_hydrogens = False if atom.show_hydrogens is None else atom.show_hydrogens
	font_size, line_color = _dialog_font_values(atom.label_font, ferrum_chem)
	_require_int_in_range(charge, -9, 9, "formal charge")
	_require_int_in_range(valency, 0, 10, "valence")
	if atom.isotope is not None:
		_require_int_in_range(atom.isotope, 1, 300, "isotope")
	if multiplicity not in (1, 2, 3):
		raise ValueError("selected Rust atom multiplicity is not representable by AtomDialog")
	if type(show) is not bool or type(show_hydrogens) is not bool:
		raise TypeError("selected Rust atom visibility facts must be booleans")
	return FerrumNativeAtomDialogModel(
		atom.element, charge, valency, atom.isotope, multiplicity, show,
		show_hydrogens, font_size, line_color,
	)


#============================================
def property_changes_from_dialog(changes: tuple[tuple[str, object], ...]) -> tuple[object, ...]:
	"""Map AtomDialog's changed fields to closed frozen Rust change values."""
	import ferrum_chem
	if type(changes) is not tuple:
		raise TypeError("native atom property changes must be an exact tuple")
	converted = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("native atom property changes must be exact field/value pairs")
		field, value = change
		converted.append(_property_change(field, value, ferrum_chem))
	return tuple(converted)


#============================================
def _dialog_font_values(font: object, ferrum_chem: object) -> tuple[int, str]:
	"""Return the exact label-font values that the integer visual control can show."""
	if font is None:
		return 12, "#000000"
	if type(font) is not ferrum_chem.FontFactsV1:
		raise TypeError("selected Rust atom label font must be exact Ferrum font facts")
	size = 12 if font.size is None else font.size
	color = "#000000" if font.color is None else font.color
	if type(size) is not float or not size.is_integer():
		raise ValueError("selected Rust atom font size is not representable by AtomDialog")
	font_size = int(size)
	_require_int_in_range(font_size, 4, 72, "font size")
	if type(color) is not str:
		raise TypeError("selected Rust atom label color must be a string")
	return font_size, color


#============================================
def _require_int_in_range(value: object, minimum: int, maximum: int, label: str) -> None:
	"""Reject a valid Rust fact only when this particular visual form cannot show it."""
	if type(value) is not int or value < minimum or value > maximum:
		raise ValueError(f"selected Rust atom {label} is not representable by AtomDialog")


#============================================
def _property_change(field: object, value: object, ferrum_chem: object) -> object:
	"""Convert one closed AtomDialog field without accepting legacy-shaped input."""
	if field == "element" and type(value) is str:
		return ferrum_chem.DocumentAtomPropertyChangeV1.element(value)
	if field == "charge" and type(value) is int:
		return ferrum_chem.DocumentAtomPropertyChangeV1.formal_charge(value)
	if field == "valency" and type(value) is int:
		valence = None if value == 0 else value
		return ferrum_chem.DocumentAtomPropertyChangeV1.valence(valence)
	if field == "isotope" and (value is None or type(value) is int):
		return ferrum_chem.DocumentAtomPropertyChangeV1.isotope(value)
	if field == "multiplicity" and type(value) is int:
		return ferrum_chem.DocumentAtomPropertyChangeV1.multiplicity(value)
	if field == "show" and type(value) is bool:
		return ferrum_chem.DocumentAtomPropertyChangeV1.show(value)
	if field == "show_hydrogens" and type(value) is bool:
		return ferrum_chem.DocumentAtomPropertyChangeV1.show_hydrogens(value)
	if field == "font_size" and type(value) is int:
		return ferrum_chem.DocumentAtomPropertyChangeV1.font_size(float(value))
	if field == "line_color" and type(value) is str:
		return ferrum_chem.DocumentAtomPropertyChangeV1.label_color(value)
	raise ValueError("native AtomDialog supplied an unsupported property change")
