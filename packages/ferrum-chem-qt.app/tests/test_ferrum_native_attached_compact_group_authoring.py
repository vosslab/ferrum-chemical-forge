"""Qt behavior coverage for attached compact-group authoring fences."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.compact_group_authoring


#============================================
class _Snapshot:
	"""Represent the live document fence exposed by a native tab."""

	def __init__(self, revision: int, digest: str) -> None:
		"""Keep the small fixed fence used by this behavior case."""
		self.revision = revision
		self.digest = digest


#============================================
class _ChoiceFact:
	"""Represent the one Rust-projected choice observation used in this case."""

	class _Category:
		"""Provide the closed unknown-anchor marker used by the presentation seam."""

		unknown_anchor = object()

	def __init__(self, anchor_object_id: str) -> None:
		"""Create a current available fact for the initial selected atom only."""
		self.revision = 7
		self.digest = "fence-seven"
		self.anchor_object_id = anchor_object_id
		self.catalog_key = "methyl"
		self.category = self._Category()
		self.available = True


#============================================
class _Tab:
	"""Provide the native tab contract needed to exercise chooser completion."""

	def __init__(self) -> None:
		"""Start with the atom whose chooser was opened."""
		self.is_disposed = False
		self.selected_atom = "atom-a"
		self.current_snapshot = _Snapshot(7, "fence-seven")
		self.mutation_attempts = 0

	def _selected_atom_identifier(self) -> str:
		"""Expose the current durable selection through the tab's normal seam."""
		return self.selected_atom

	def attached_compact_group_choices(self) -> tuple[object, ...]:
		"""Return the one Rust-reviewed presentation choice for this inline tab."""
		choice = ferrum_qt.ferrum.compact_group_authoring._AttachedCompactGroupChoice(
			"methyl", "Me",
		)
		return (choice,)

	def attach_compact_group_availability(self, anchor_object_id: str,
			catalog_key: str) -> object:
		"""Reject a stale-choice query so fence ordering remains behaviorally visible."""
		if anchor_object_id != "atom-a":
			raise AssertionError("stale selection reached choice-specific availability")
		if catalog_key != "methyl":
			raise AssertionError("unexpected compact-group choice")
		return _ChoiceFact(anchor_object_id)

	def begin_attached_compact_group(self, *unused: object) -> object:
		"""Record any mutation attempt that would violate the stale-intent contract."""
		del unused
		self.mutation_attempts += 1
		raise AssertionError("stale intent reached native compact-group mutation")


#============================================
class _Window(PySide6.QtWidgets.QMainWindow,
		ferrum_qt.ferrum.compact_group_authoring.FerrumNativeCompactGroupAuthoringWindowMixin):
	"""Expose the Qt window seams required by the real authoring mixin."""

	def __init__(self, tab: _Tab) -> None:
		"""Register one active tab and collect learner-facing refusals."""
		super().__init__()
		self._tab = tab
		self._native_tabs_by_page = {tab: tab}
		self.refusals: list[tuple[str, str | None]] = []
		self._initialize_compact_group_authoring()

	def _active_native_tab(self) -> _Tab:
		"""Return the selected test tab through the production lookup seam."""
		return self._tab

	def _unavailable_edit_refusal(self, detail: str,
			primary_message: str | None = None) -> tuple[str, str | None]:
		"""Keep the presented guidance inspectable without a modal dialog."""
		return detail, primary_message

	def _show_edit_refusal(self, refusal: tuple[str, str | None]) -> None:
		"""Record the public refusal that the completed chooser would present."""
		self.refusals.append(refusal)

	def _refresh_actions(self) -> None:
		"""Supply the normal window refresh seam without unrelated actions."""


#============================================
def test_changed_selection_after_chooser_acceptance_refuses_without_mutation(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A changed durable selection rejects the accepted chooser before native mutation."""
	tab = _Tab()
	window = _Window(tab)
	try:
		window._choose_compact_group_to_attach()
		dialog = window.findChild(PySide6.QtWidgets.QDialog)
		if dialog is None:
			raise AssertionError("Attach Compact Group chooser did not open")
		tab.selected_atom = "atom-b"
		dialog.accept()
		qapp.processEvents()

		assert tab.mutation_attempts == 0
		assert window.refusals == [(
			"The selected atom changed; choose Attach Compact Group again.", None,
		)]
	finally:
		window.close()
		window.deleteLater()
