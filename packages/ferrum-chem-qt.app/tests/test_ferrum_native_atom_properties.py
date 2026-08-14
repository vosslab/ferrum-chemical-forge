"""Focused behavior tests for the OASA-free native AtomDialog adapter."""

# Standard Library
import os
import sys
import types


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
sys.modules.setdefault("ferrum_chem", types.ModuleType("ferrum_chem"))

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.dialogs.atom_dialog
import ferrum_qt.native.ferrum_native_atom_properties


#============================================
class _Change:
	"""One minimal exact frozen-change stand-in for the private test seam."""

	#============================================
	def __init__(self, kind: str, value: object) -> None:
		"""Retain one closed factory payload for behavioral verification."""
		self.kind = kind
		self.value = value


#============================================
class _Changes:
	"""Fake exact PyO3 factory owner with the public native factory names."""

	#============================================
	@staticmethod
	def element(value: str) -> _Change:
		"""Build an element change."""
		return _Change("element", value)

	#============================================
	@staticmethod
	def formal_charge(value: int) -> _Change:
		"""Build a charge change."""
		return _Change("formal_charge", value)

	#============================================
	@staticmethod
	def valence(value: int | None) -> _Change:
		"""Build a valence change."""
		return _Change("valence", value)

	#============================================
	@staticmethod
	def isotope(value: int | None) -> _Change:
		"""Build an isotope change."""
		return _Change("isotope", value)

	#============================================
	@staticmethod
	def multiplicity(value: int) -> _Change:
		"""Build a multiplicity change."""
		return _Change("multiplicity", value)

	#============================================
	@staticmethod
	def show(value: bool) -> _Change:
		"""Build an atom visibility change."""
		return _Change("show", value)

	#============================================
	@staticmethod
	def show_hydrogens(value: bool) -> _Change:
		"""Build a hydrogen visibility change."""
		return _Change("show_hydrogens", value)

	#============================================
	@staticmethod
	def font_size(value: float) -> _Change:
		"""Build a font-size change."""
		return _Change("font_size", value)

	#============================================
	@staticmethod
	def label_color(value: str) -> _Change:
		"""Build a label-color change."""
		return _Change("label_color", value)


#============================================
class _Font:
	"""Fake exact frozen font facts."""

	#============================================
	def __init__(self, size: float | None, color: str | None) -> None:
		"""Retain optional font facts."""
		self.size = size
		self.color = color


#============================================
class _Atom:
	"""Fake exact frozen atom projection."""

	#============================================
	def __init__(self, *, multiplicity: int | None = None) -> None:
		"""Create a projection with deliberately absent optional CDML facts."""
		self.element = "C"
		self.formal_charge = None
		self.valence = None
		self.isotope = None
		self.multiplicity = multiplicity
		self.show = None
		self.show_hydrogens = None
		self.label_font = None


#============================================
class _FerrumChem:
	"""Private exact-type module seam, not a production compatibility layer."""

	AtomProjectionV1 = _Atom
	FontFactsV1 = _Font
	DocumentAtomPropertyChangeV1 = _Changes


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an offscreen QApplication for the shared visual form."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _install_ferrum_module(monkeypatch: pytest.MonkeyPatch) -> None:
	"""Install exact private value types only for this adapter's isolated tests."""
	monkeypatch.setitem(sys.modules, "ferrum_chem", _FerrumChem)


#============================================
def test_absent_optional_facts_do_not_become_authored_on_an_unchanged_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Opening then accepting the visual form does not manufacture optional CDML."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.native.ferrum_native_atom_properties.dialog_model_from_projection(_Atom())
	dialog = ferrum_qt.dialogs.atom_dialog.AtomDialog(model)
	assert dialog.changes() == ()
	dialog.deleteLater()


#============================================
def test_native_adapter_rejects_a_multiplicity_the_shared_dialog_cannot_show(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An unsupported source multiplicity is never silently displayed as singlet."""
	_install_ferrum_module(monkeypatch)
	with pytest.raises(ValueError, match="multiplicity"):
		ferrum_qt.native.ferrum_native_atom_properties.dialog_model_from_projection(
			_Atom(multiplicity=4),
		)


#============================================
def test_dialog_fields_map_to_closed_rust_property_factories(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Legacy form names become only the named frozen Rust change values."""
	_install_ferrum_module(monkeypatch)
	changes = ferrum_qt.native.ferrum_native_atom_properties.property_changes_from_dialog(
		(("charge", 0), ("valency", 0), ("isotope", None), ("line_color", "#123456")),
	)
	assert [(change.kind, change.value) for change in changes] == [
		("formal_charge", 0), ("valence", None), ("isotope", None),
		("label_color", "#123456"),
	]
