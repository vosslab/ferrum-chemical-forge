"""Behavior tests for the Ferrum persistent atom-number seam."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.atom_number
import ferrum_qt.ferrum.main_window


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one isolated offscreen Qt application."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
class _Atom:
	"""Small frozen-projection-shaped source for the public window action."""

	#============================================
	def __init__(self, number: int | None, show_number: bool | None) -> None:
		"""Retain explicit optional atom-number facts."""
		self.number = number
		self.show_number = show_number


#============================================
class _NativeTab:
	"""Record exact public action submissions without document interpretation."""

	#============================================
	def __init__(self, atom: _Atom) -> None:
		"""Retain one selected projected atom."""
		self.atom = atom
		self.assignments: list[tuple[int, bool]] = []
		self.clears = 0

	#============================================
	def selected_atom_projection(self) -> _Atom:
		"""Return the current projected atom."""
		return self.atom

	#============================================
	def set_selected_atom_number(self, number: int, show_number: bool) -> None:
		"""Record one exact assignment."""
		self.assignments.append((number, show_number))

	#============================================
	def clear_selected_atom_number(self) -> None:
		"""Record one exact clear."""
		self.clears += 1


#============================================
class _AcceptedDialog:
	"""Deterministic accepted dialog used only through monkeypatch."""

	#============================================
	def __init__(self, number: int | None, show_number: bool | None,
			_parent: object) -> None:
		"""Record source facts without changing their meaning."""
		self.source = number, show_number

	#============================================
	def exec(self) -> PySide6.QtWidgets.QDialog.DialogCode:
		"""Return accepted without entering a modal test loop."""
		return PySide6.QtWidgets.QDialog.DialogCode.Accepted

	#============================================
	def assignment(self) -> tuple[int, bool]:
		"""Return one explicit changed value pair."""
		return 42, False


#============================================
def test_dialog_uses_the_protocol_range_without_an_arbitrary_widget_ceiling(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The text form accepts the complete positive u64 protocol and rejects coercion."""
	dialog = (
		ferrum_qt.ferrum.atom_number.
		FerrumNativeAtomNumberDialog((1 << 64) - 1, True)
	)
	ok = dialog.buttons.button(
		PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
	)
	assert ok.isEnabled()
	assert dialog.assignment() == ((1 << 64) - 1, True)
	for invalid in ("", "0", "01", "-1", "1.0", str(1 << 64)):
		dialog.number_edit.setText(invalid)
		qapp.processEvents()
		assert not ok.isEnabled()
	dialog.deleteLater()


#============================================
def test_public_window_actions_submit_one_assignment_and_one_clear(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The Ferrum actions carry only explicit scalar intent to the active tab."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = _NativeTab(_Atom(7, True))
	monkeypatch.setattr(window, "_active_native_tab", lambda: tab)
	monkeypatch.setattr(window, "_refresh_actions", lambda *_unused: None)
	monkeypatch.setattr(
		ferrum_qt.ferrum.atom_number,
		"FerrumNativeAtomNumberDialog",
		_AcceptedDialog,
	)

	window._on_set_atom_number()
	window._on_clear_atom_number()

	assert tab.assignments == [(42, False)] and tab.clears == 1
	window.deleteLater()
