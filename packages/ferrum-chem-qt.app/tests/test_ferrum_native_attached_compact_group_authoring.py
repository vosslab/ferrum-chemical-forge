"""Qt behavior coverage for attached compact-group authoring fences."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.compact_group_authoring
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors
from ferrum_qt.ferrum.interaction_action_handoff import FerrumInteractionActionHandoffRefusal


#============================================
class _Snapshot:
	"""Represent the live document fence exposed by a native tab."""

	def __init__(self, revision: int, digest: str) -> None:
		"""Keep the small fixed fence used by this behavior case."""
		self.revision = revision
		self.digest = digest


#============================================
class _SelectedMoleculeAtomAddress:
	"""Represent the pair-local selection used by compact-group authoring."""

	def __init__(self, atom_id: str) -> None:
		"""Keep the selected molecule fixed while the selected atom may change."""
		self.molecule_id = "molecule-a"
		self.atom_id = atom_id


#============================================
class _ChoiceFact:
	"""Represent the one Rust-projected choice observation used in this case."""

	class _Category:
		"""Provide the closed unknown-anchor marker used by the presentation seam."""

		unknown_anchor = object()

	def __init__(self, anchor_object_id: str, available: bool) -> None:
		"""Create a current available fact for the initial selected atom only."""
		self.revision = 7
		self.digest = "fence-seven"
		self.anchor_object_id = anchor_object_id
		self.catalog_key = "methyl"
		self.category = self._Category()
		self.available = available


#============================================
class _Tab:
	"""Provide the native tab contract needed to exercise chooser completion."""

	def __init__(self) -> None:
		"""Start with the atom whose chooser was opened."""
		self.is_disposed = False
		self.requires_refresh = False
		self.selected_atom = "atom-a"
		self.reject_selection_reads = False
		self.anchor_available = True
		self.current_snapshot = _Snapshot(7, "fence-seven")
		self.mutation_attempts = 0

	def selected_molecule_atom_address(self) -> _SelectedMoleculeAtomAddress:
		"""Expose the current pair-local selection through the tab's normal seam."""
		if self.reject_selection_reads:
			raise AssertionError("selection was read after compact-group admission")
		return _SelectedMoleculeAtomAddress(self.selected_atom)

	def attached_compact_group_choices(self) -> tuple[object, ...]:
		"""Return the one Rust-reviewed presentation choice for this inline tab."""
		choice = ferrum_qt.ferrum.compact_group_authoring._AttachedCompactGroupChoice(
			"methyl", "Me",
		)
		return (choice,)

	def attach_compact_group_availability(self, molecule_object_id: str,
			anchor_object_id: str, catalog_key: str) -> object:
		"""Reject a stale-choice query so fence ordering remains behaviorally visible."""
		if molecule_object_id != "molecule-a":
			raise AssertionError("unexpected selected molecule")
		if anchor_object_id != "atom-a":
			raise AssertionError("stale selection reached choice-specific availability")
		if catalog_key != "methyl":
			raise AssertionError("unexpected compact-group choice")
		return _ChoiceFact(anchor_object_id, self.anchor_available)

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
def test_changed_selection_after_admission_does_not_requery_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Acceptance fence uses captured atom facts, never a later live selection."""
	del qapp
	tab = _Tab()
	window = _Window(tab)
	try:
		command = window._prepare_compact_group_to_attach(False)
		assert command is not None
		tab.selected_atom = "atom-b"
		tab.reject_selection_reads = True
		choice = ferrum_qt.ferrum.compact_group_authoring._AttachedCompactGroupChoice(
			"methyl", "Me",
		)
		window._require_admitted_attach_compact_group_target(
			tab, 7, "fence-seven", "molecule-a", "atom-a", (choice,), choice,
		)

		assert tab.mutation_attempts == 0
		assert window.refusals == []
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_admitted_compact_group_refuses_a_changed_document_fence(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A revision change invalidates the captured capability without selection access."""
	del qapp
	tab = _Tab()
	window = _Window(tab)
	try:
		choice = ferrum_qt.ferrum.compact_group_authoring._AttachedCompactGroupChoice(
			"methyl", "Me",
		)
		tab.current_snapshot = _Snapshot(8, "fence-eight")
		with pytest.raises(
			native_document_tab_errors.FerrumNativeDocumentTabError,
			match="document or admitted atom changed",
		):
			window._require_admitted_attach_compact_group_target(
				tab, 7, "fence-seven", "molecule-a", "atom-a", (choice,), choice,
			)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_admitted_compact_group_refuses_unavailable_captured_anchor(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Rust availability can invalidate the admitted anchor before release."""
	del qapp
	tab = _Tab()
	window = _Window(tab)
	try:
		choice = ferrum_qt.ferrum.compact_group_authoring._AttachedCompactGroupChoice(
			"methyl", "Me",
		)
		tab.anchor_available = False
		with pytest.raises(
			ferrum_qt.ferrum.compact_group_authoring._AttachCompactGroupUnavailableError,
		):
			window._require_admitted_attach_compact_group_target(
				tab, 7, "fence-seven", "molecule-a", "atom-a", (choice,), choice,
			)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_unavailable_attach_admission_preserves_feature_refusal_payload(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Admission retains the exact learner recovery request for an unavailable atom."""
	del qapp
	tab = _Tab()
	tab.anchor_available = False
	window = _Window(tab)
	try:
		with pytest.raises(FerrumInteractionActionHandoffRefusal) as raised:
			window._prepare_compact_group_to_attach(False)
		assert raised.value.payload == (
			"Rust refused Me attachment for the selected atom.",
			"Me cannot attach to the selected atom. Select another atom and try again.",
		)
	finally:
		window.close()
		window.deleteLater()
