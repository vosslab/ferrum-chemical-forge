"""Focused behavior tests for the Ferrum BondDialog adapter."""

# Standard Library
import os
import sys
import enum


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.dialogs.bond_dialog
import ferrum_qt.ferrum.bond_properties


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
	"""Fake exact PyO3 factory owner with the public Ferrum factory names."""

	#============================================
	@staticmethod
	def order(value: int) -> _Change:
		"""Build an order change."""
		return _Change("order", value)

	#============================================
	@staticmethod
	def style(value: str) -> _Change:
		"""Build a style change."""
		return _Change("style", value)

	#============================================
	@staticmethod
	def center(value: bool) -> _Change:
		"""Build a center change."""
		return _Change("center", value)

	#============================================
	@staticmethod
	def line_width(value: float) -> _Change:
		"""Build a line-width change."""
		return _Change("line_width", value)

	#============================================
	@staticmethod
	def bond_width(value: float) -> _Change:
		"""Build a signed bond-width change."""
		return _Change("bond_width", value)

	#============================================
	@staticmethod
	def wedge_width(value: float) -> _Change:
		"""Build a wedge-width change."""
		return _Change("wedge_width", value)

	#============================================
	@staticmethod
	def color(value: str) -> _Change:
		"""Build a color change."""
		return _Change("color", value)


#============================================
class _Order(enum.Enum):
	"""Private exact stand-in for the frozen PyO3 bond-order enum."""

	single = 1
	double = 2
	triple = 3


#============================================
class _Style(enum.Enum):
	"""Private exact stand-in for the frozen PyO3 bond-style enum."""

	normal = "n"
	wedge = "w"
	hashed_wedge = "h"
	adder = "a"
	bold = "b"
	dashed = "d"
	dotted = "o"
	wavy = "s"
	haworth_front = "q"


#============================================
class _Bond:
	"""Fake exact frozen bond projection."""

	#============================================
	def __init__(self, *, order: _Order = _Order.single, style: _Style = _Style.normal,
			line_width: float | None = None,
			bond_width: float | None = None,
			wedge_width: float | None = None) -> None:
		"""Create a projection with deliberately absent optional CDML facts."""
		self.order = order
		self.style = style
		self.center = None
		self.line_width = line_width
		self.bond_width = bond_width
		self.wedge_width = wedge_width
		self.color = None


#============================================
class _FerrumChem:
	"""Private exact-type module seam, not a production compatibility layer."""

	BondProjectionV1 = _Bond
	DocumentBondPropertyChangeV1 = _Changes
	DocumentBondOrderV1 = _Order
	DocumentBondStyleV1 = _Style


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
	"""Install exact private value types only for isolated adapter tests."""
	monkeypatch.setitem(sys.modules, "ferrum_chem", _FerrumChem)


#============================================
def test_absent_optional_facts_do_not_become_authored_on_an_unchanged_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Opening then accepting the form leaves all absent CDML facts absent."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond())
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(model)
	assert dialog.changes() == ()
	dialog.deleteLater()


#============================================
def test_dialog_keeps_its_full_bond_editing_vocabulary(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The shared form keeps fields beyond the currently rendered subset."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond())
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(model)
	assert dialog._type_combo.count() > 1
	assert dialog._type_combo.isEnabled()
	assert dialog._wedge_width_spin.isEnabled()
	assert dialog._center_check.isEnabled()
	assert dialog._bond_width_spin.isEnabled()
	dialog.deleteLater()


#============================================
def test_native_adapter_rejects_a_negative_width_the_shared_dialog_cannot_show(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The adapter refuses to lose the directional negative bond-width sign."""
	_install_ferrum_module(monkeypatch)
	with pytest.raises(ValueError, match="bond width"):
		ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(
			_Bond(bond_width=-2.0),
		)


#============================================
def test_native_adapter_rejects_a_source_style_without_renderer_support(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A retained style outside the closed Ferrum depiction profile is refused."""
	_install_ferrum_module(monkeypatch)
	with pytest.raises(ValueError, match="Normal, Solid wedge, or Hashed wedge"):
		ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(
			_Bond(style=_Style.adder),
		)


#============================================
@pytest.mark.parametrize(
	("field", "value"),
	(("line_width", 20.1), ("bond_width", 0.05), ("wedge_width", 1.23)),
)
def test_native_adapter_rejects_widths_that_a_spin_box_would_change(
		monkeypatch: pytest.MonkeyPatch, field: str, value: float,
		) -> None:
	"""Out-of-range and non-tenth facts fail before a dialog can author a rewrite."""
	_install_ferrum_module(monkeypatch)
	kwargs = {field: value}
	bond = _Bond(**kwargs)
	with pytest.raises(ValueError, match="width"):
		ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(bond)


#============================================
def test_dialog_fields_map_to_closed_rust_property_factories(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Supported BondDialog fields become named frozen Rust property changes."""
	_install_ferrum_module(monkeypatch)
	changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
		_Bond(), (("order", 2), ("type", "n"), ("center", True), ("line_width", 1.5),
			("bond_width", 2.5), ("color", "#123456")),
	)
	assert [(change.kind, change.value) for change in changes] == [
		("order", _Order.double), ("style", _Style.normal), ("center", True), ("line_width", 1.5),
		("bond_width", 2.5), ("color", "#123456"),
	]


#============================================
def test_unrelated_dialog_edit_preserves_absent_optional_facts(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Changing order does not convert absence-derived visual defaults into CDML facts."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond())
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(model)
	dialog._order_combo.setCurrentIndex(1)
	assert dialog.changes() == (("order", 2),)
	dialog.deleteLater()


#============================================
def test_live_native_tab_submits_one_frozen_bond_patch_and_restores_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The installed Rust DTO route keeps a durable selected bond after one edit."""
	del qapp
	ferrum_chem = pytest.importorskip("ferrum_chem")
	import ferrum_qt.ferrum.document_tab
	cdml = (
		'<cdml version="26.08"><molecule id="molecule-1">'
		'<atom id="atom-c" name="C"><point x="0" y="0"/></atom>'
		'<atom id="atom-o" name="O"><point x="30" y="0"/></atom>'
		'</molecule></cdml>'
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(cdml, "bond")
	try:
		tab.select_atoms(("atom-c", "atom-o"))
		created = tab.add_single_bond_between_selected_atoms()
		bond_id = created.observation.projection.molecules[0].bonds[0].source_id
		tab.select_bond(bond_id)
		model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(
			tab.selected_bond_projection(),
		)
		assert model.order == 1 and model.type == "n"
		changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
			tab.selected_bond_projection(), (("order", 2), ("center", True)),
		)
		assert all(type(change) is ferrum_chem.DocumentBondPropertyChangeV1 for change in changes)
		tab.apply_selected_bond_properties(changes)
		assert tab.has_one_selected_bond()
		updated = tab.selected_bond_projection()
		assert updated.order is ferrum_chem.DocumentBondOrderV1.double
		assert updated.center is True
	finally:
		tab.dispose()


#============================================
