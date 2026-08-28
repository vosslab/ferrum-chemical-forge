"""Focused behavior tests for the Ferrum closed bond-presentation dialog."""

# Standard Library
import enum
import os
import sys


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
	def presentation(value: object) -> _Change:
		"""Build one complete presentation replacement."""
		return _Change("presentation", value)

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
		"""Build a bond-width change."""
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
class _Presentation(enum.Enum):
	"""Private exact stand-in for the frozen PyO3 presentation enum."""

	normal_single = "normal_single"
	normal_double = "normal_double"
	normal_triple = "normal_triple"
	solid_wedge = "solid_wedge"
	hashed_wedge = "hashed_wedge"
	haworth_front = "haworth_front"
	bold = "bold"
	dashed = "dashed"
	wavy = "wavy"


#============================================
class _Bond:
	"""Fake exact frozen bond projection."""

	#============================================
	def __init__(self, presentation: _Presentation = _Presentation.normal_single,
			center: bool | None = None, line_width: float | None = None,
			bond_width: float | None = None,
			wedge_width: float | None = None) -> None:
		"""Create a projection with deliberately absent optional CDML facts."""
		self.presentation = presentation
		self.center = center
		self.line_width = line_width
		self.bond_width = bond_width
		self.wedge_width = wedge_width
		self.color = None


#============================================
class _FerrumChem:
	"""Private exact-type module seam, not a production compatibility layer."""

	BondProjectionV1 = _Bond
	DocumentBondPropertyChangeV1 = _Changes
	DocumentBondPresentationV1 = _Presentation


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
@pytest.mark.parametrize(
	("presentation", "order", "bond_type"),
	(
		(_Presentation.normal_single, 1, "n"),
		(_Presentation.normal_double, 2, "n"),
		(_Presentation.normal_triple, 3, "n"),
		(_Presentation.solid_wedge, 1, "w"),
		(_Presentation.hashed_wedge, 1, "h"),
		(_Presentation.haworth_front, 1, "q"),
		(_Presentation.bold, 1, "b"),
		(_Presentation.dashed, 1, "d"),
		(_Presentation.wavy, 1, "s"),
	),
)
def test_closed_presentations_map_to_exact_dialog_values(
		monkeypatch: pytest.MonkeyPatch, presentation: _Presentation, order: int,
		bond_type: str,
		) -> None:
	"""Every PyO3 presentation maps once to its visible order/style controls."""
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond(presentation))
	assert (model.order, model.type) == (order, bond_type)


#============================================
def test_absent_optional_facts_do_not_become_authored_on_an_unchanged_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Opening then accepting the form leaves all absent CDML facts absent."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond())
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(
		model,
	)
	assert dialog.changes() == ()
	dialog.deleteLater()


#============================================
def test_native_dialog_exposes_only_the_supported_closed_presentation_vocabulary(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Adder and Dotted remain outside this renderer-backed property surface."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond())
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(
		model,
	)
	assert tuple(dialog._type_combo.itemData(index) for index in range(dialog._type_combo.count())) == (
		"n", "w", "h", "b", "d", "s", "q",
	)
	dialog.deleteLater()


#============================================
@pytest.mark.parametrize("presentation", (
	_Presentation.solid_wedge, _Presentation.hashed_wedge,
	_Presentation.haworth_front, _Presentation.bold, _Presentation.dashed,
	_Presentation.wavy,
))
def test_fixed_single_presentations_lock_incompatible_controls(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		presentation: _Presentation,
		) -> None:
	"""Non-normal render forms are intrinsically Single and reject order edits."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond(presentation))
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(
		model,
	)
	assert dialog._order_combo.currentText() == "Single"
	assert not dialog._order_combo.isEnabled()
	assert not dialog._center_check.isEnabled()
	assert not dialog._bond_width_spin.isEnabled()
	assert dialog._wedge_width_spin.isEnabled() == (presentation in (
		_Presentation.solid_wedge, _Presentation.hashed_wedge,
	))
	dialog.deleteLater()


#============================================
def test_switching_normal_double_to_wedge_submits_one_fixed_single_presentation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A style switch is one complete document presentation change, never two facts."""
	del qapp
	_install_ferrum_module(monkeypatch)
	model = ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(
		_Bond(_Presentation.normal_double),
	)
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(
		model,
	)
	dialog._type_combo.setCurrentIndex(dialog._type_combo.findData("w"))
	assert dialog._order_combo.currentText() == "Single"
	assert not dialog._order_combo.isEnabled()
	assert dialog.changes() == (("presentation", (1, "w")),)
	changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
		_Bond(_Presentation.normal_double), dialog.changes(),
	)
	assert [(change.kind, change.value) for change in changes] == [
		("presentation", _Presentation.solid_wedge),
	]
	dialog.deleteLater()


#============================================
def test_normal_double_and_triple_own_only_their_relevant_controls(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Center is double-only while bond width is normal double/triple-only."""
	del qapp
	_install_ferrum_module(monkeypatch)
	double = ferrum_qt.dialogs.bond_dialog.BondDialog(
		ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(
			_Bond(_Presentation.normal_double),
		),
	)
	triple = ferrum_qt.dialogs.bond_dialog.BondDialog(
		ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(
			_Bond(_Presentation.normal_triple),
		),
	)
	assert double._center_check.isEnabled() and double._bond_width_spin.isEnabled()
	assert not double._wedge_width_spin.isEnabled()
	assert not triple._center_check.isEnabled() and triple._bond_width_spin.isEnabled()
	assert not triple._wedge_width_spin.isEnabled()
	double.deleteLater()
	triple.deleteLater()


#============================================
def test_presentation_and_scalar_changes_use_closed_factories(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""One combined presentation factory accompanies independent scalar changes."""
	_install_ferrum_module(monkeypatch)
	changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
		_Bond(), (("presentation", (2, "n")), ("center", True),
			("line_width", 1.5), ("bond_width", 2.5), ("color", "#123456")),
	)
	assert [(change.kind, change.value) for change in changes] == [
		("presentation", _Presentation.normal_double), ("center", True),
		("line_width", 1.5), ("bond_width", 2.5), ("color", "#123456"),
	]


#============================================
def test_normal_double_accepts_both_explicit_center_values(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Centering is a normal-double boolean, not a true-only command."""
	_install_ferrum_module(monkeypatch)
	changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
		_Bond(_Presentation.normal_double, center=True), (("center", False),),
	)
	assert [(change.kind, change.value) for change in changes] == [("center", False)]


#============================================
def test_presentation_change_clears_inapplicable_authored_fields(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Presentation transitions clear retained facts that no longer have meaning."""
	_install_ferrum_module(monkeypatch)
	changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
		_Bond(_Presentation.normal_double, center=True, bond_width=2.5),
		(("presentation", (1, "w")),),
	)
	assert [(change.kind, change.value) for change in changes] == [
		("presentation", _Presentation.solid_wedge), ("center", None), ("bond_width", None),
	]


#============================================
def test_leaving_a_wedge_clears_its_authored_width(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Wedge width does not survive a replacement normal presentation."""
	_install_ferrum_module(monkeypatch)
	changes = ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(
		_Bond(_Presentation.solid_wedge, wedge_width=9.0),
		(("presentation", (1, "n")),),
	)
	assert [(change.kind, change.value) for change in changes] == [
		("presentation", _Presentation.normal_single), ("wedge_width", None),
	]


#============================================
@pytest.mark.parametrize(
	("changes", "message"),
	(
		((("presentation", (2, "w")),), "unsupported bond presentation"),
		((("bond_width", 2.5),), "bond width only"),
		((("wedge_width", 2.5),), "solid or hashed wedge"),
		((("center", True),), "normal double"),
	),
)
def test_adapter_refuses_invalid_control_combinations(
		monkeypatch: pytest.MonkeyPatch, changes: tuple[tuple[str, object], ...],
		message: str,
		) -> None:
	"""Programmatic callers cannot bypass the same closed UI combinations."""
	_install_ferrum_module(monkeypatch)
	with pytest.raises(ValueError, match=message):
		ferrum_qt.ferrum.bond_properties.property_changes_from_dialog(_Bond(), changes)


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
	with pytest.raises(ValueError, match="width"):
		ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(_Bond(**kwargs))


#============================================
