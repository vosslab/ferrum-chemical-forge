"""Read-only Properties client for the ordinary Ferrum product window."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
def _count_label(count: int, singular: str) -> str:
	"""Return one readable semantic count."""
	return f"{count} {singular if count == 1 else singular + 's'}"


#============================================
def _value_label(value: object | None) -> str:
	"""Turn one closed PyO3 enum or absent authored fact into readable text."""
	if value is None:
		return "Not authored"
	name = getattr(value, "name", None)
	if isinstance(name, str):
		return name.replace("_", " ").title()
	return str(value).rsplit(".", 1)[-1].replace("_", " ").title()


#============================================
def _document_lines(document: object) -> tuple[str, ...]:
	"""Summarize current Rust-owned drawable facts without interpreting CDML."""
	molecules = tuple(document.molecules)
	atoms = sum(len(molecule.atoms) for molecule in molecules)
	bonds = sum(len(molecule.bonds) for molecule in molecules)
	drawings = len(document.presentation_stack.entries)
	if not molecules and not drawings:
		return ("No drawable objects",)
	lines = []
	if molecules:
		lines.extend((
			_count_label(len(molecules), "molecule"),
			_count_label(atoms, "atom"),
			_count_label(bonds, "bond"),
		))
	if drawings:
		lines.append(_count_label(drawings, "drawing object"))
	issues = len(document.issues) + len(document.presentation_stack.issues)
	if issues:
		lines.append(_count_label(issues, "projection warning"))
	return tuple(lines)


#============================================

def _atom_for_target(document: object, molecule_id: str,
		atom_id: str) -> object | None:
	"""Resolve one Rust-addressed atom within its immutable Rust projection."""
	match = None
	for molecule in document.molecules:
		if molecule.document_object_id != molecule_id:
			continue
		for atom in molecule.atoms:
			if atom.document_object_id != atom_id:
				continue
			if match is not None:
				return None
			match = atom
	return match


#============================================

def _bond_for_target(document: object, molecule_id: str,
		bond_id: str) -> object | None:
	"""Resolve one Rust-addressed bond within its immutable Rust projection."""
	match = None
	for molecule in document.molecules:
		if molecule.document_object_id != molecule_id:
			continue
		for bond in molecule.bonds:
			if bond.document_object_id != bond_id:
				continue
			if match is not None:
				return None
			match = bond
	return match


#============================================
def _atom_lines(atom: object) -> tuple[str, ...]:
	"""Present authored atom facts without manufacturing absent defaults."""
	position = atom.position
	formal_charge = atom.formal_charge if atom.formal_charge is not None else "Not authored"
	lines = [
		f"Element: {atom.element or 'Unspecified'}",
		f"Formal charge: {formal_charge}",
		f"Position: {position.x:g}, {position.y:g}",
	]
	if atom.number is not None:
		lines.append(f"Atom number: {atom.number}")
	if atom.marks:
		lines.append(_count_label(len(atom.marks), "mark"))
	return tuple(lines)


#============================================
def _bond_lines(bond: object) -> tuple[str, ...]:
	"""Present typed bond facts and their durable endpoint identities."""
	start = bond.start.object_id or "Unresolved"
	end = bond.end.object_id or "Unresolved"
	return (
		f"Order: {_value_label(bond.order)}",
		f"Style: {_value_label(bond.style)}",
		f"Endpoints: {start} to {end}",
	)


#============================================
def _selection_lines(document: object, selection: tuple[object, ...]) -> tuple[str, ...]:
	"""Summarize generic targets using Rust-owned top-level root kinds when present."""
	root_kinds = {
		root.document_object_id: root.kind
		for root in document.direct_roots
	}
	counts: dict[str, int] = {}
	for target in selection:
		root_kind = root_kinds.get(target.document_object_id)
		label = "Selected Object" if root_kind is None else root_kind.replace("_", " ").title()
		counts[label] = counts.get(label, 0) + 1
	return tuple(
		_count_label(counts[label], label.lower())
		for label in sorted(counts)
	)


#============================================
class FerrumNativePropertyDock(PySide6.QtWidgets.QDockWidget):
	"""Show current Rust projection facts and reuse established edit actions."""

	#============================================
	def __init__(self, atom_action: PySide6.QtGui.QAction,
			bond_action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Build one dock whose buttons remain clients of window-owned commands."""
		super().__init__(parent.tr("Properties"), parent)
		self.setObjectName("native-properties-dock")
		self.setAccessibleName(parent.tr("Document properties"))
		self.setAllowedAreas(
			PySide6.QtCore.Qt.DockWidgetArea.LeftDockWidgetArea
			| PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea,
		)
		panel = PySide6.QtWidgets.QWidget(self)
		layout = PySide6.QtWidgets.QVBoxLayout(panel)
		layout.setContentsMargins(12, 12, 12, 12)
		layout.setSpacing(8)
		self._heading = PySide6.QtWidgets.QLabel(parent.tr("No document"), panel)
		font = self._heading.font()
		font.setBold(True)
		self._heading.setFont(font)
		self._heading.setAccessibleName(parent.tr("Properties heading"))
		layout.addWidget(self._heading)
		self._summary = PySide6.QtWidgets.QLabel(parent.tr("Open or create a document."), panel)
		self._summary.setObjectName("native-properties-summary")
		self._summary.setAccessibleName(parent.tr("Current properties"))
		self._summary.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByMouse,
		)
		self._summary.setWordWrap(True)
		self._summary.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignTop)
		layout.addWidget(self._summary)
		self._atom_button = self._action_button(atom_action, panel)
		self._bond_button = self._action_button(bond_action, panel)
		layout.addWidget(self._atom_button)
		layout.addWidget(self._bond_button)
		layout.addStretch()
		self.setWidget(panel)
		self._show("No document", ("Open or create a document.",), None)

	#============================================
	def _action_button(self, action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QWidget) -> PySide6.QtWidgets.QToolButton:
		"""Create one visible client of an already-owned Ferrum edit action."""
		button = PySide6.QtWidgets.QToolButton(parent)
		button.setDefaultAction(action)
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextOnly)
		button.setAccessibleName(action.text())
		return button

	#============================================
	@property
	def summary_text(self) -> str:
		"""Return the visible summary for semantic UI checks."""
		return self._summary.text()

	#============================================
	def refresh(self, tab: object | None) -> None:
		"""Observe one active tab without retaining its session or scene objects."""
		if tab is None or tab.is_disposed:
			self._show("No document", ("Open or create a document.",), None)
			return
		if tab.requires_refresh:
			self._show(
				"Refresh required",
				("Properties will return after the authoritative Rust view is refreshed.",),
				None,
			)
			return
		observation = tab.observe_properties()
		selection = observation.selection
		if len(selection) != 1:
			if selection:
				self._show("Selection", _selection_lines(observation.document, selection), None)
			else:
				self._show("Document", _document_lines(observation.document), None)
			return
		try:
			atom_address = tab.selected_molecule_atom_address()
		except native_document_tab_errors.FerrumNativeDocumentTabError:
			atom_address = None
		if atom_address is not None:
			atom = _atom_for_target(
				observation.document, atom_address.molecule_id, atom_address.atom_id,
			)
			if atom is not None:
				self._show("Atom", _atom_lines(atom), "atom")
				return
		try:
			bond_address = tab.selected_molecule_bond_address()
		except native_document_tab_errors.FerrumNativeDocumentTabError:
			bond_address = None
		if bond_address is not None:
			bond = _bond_for_target(
				observation.document, bond_address.molecule_id, bond_address.bond_id,
			)
			if bond is not None:
				self._show("Bond", _bond_lines(bond), "bond")
				return
		self._show(
			"Selected object",
			("No typed structural details are available.",), None,
		)

	#============================================
	def _show(self, heading: str, lines: tuple[str, ...], edit_kind: str | None) -> None:
		"""Replace only visible inspector state; actions retain their own enablement."""
		self._heading.setText(self.tr(heading))
		self._summary.setText("\n".join(self.tr(line) for line in lines))
		self._atom_button.setVisible(edit_kind == "atom")
		self._bond_button.setVisible(edit_kind == "bond")


#============================================
def install_native_property_dock(window: PySide6.QtWidgets.QMainWindow,
		atom_action: PySide6.QtGui.QAction,
		bond_action: PySide6.QtGui.QAction) -> FerrumNativePropertyDock:
	"""Install one right-side inspector and register its View action."""
	dock = FerrumNativePropertyDock(atom_action, bond_action, window)
	window.addDockWidget(PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea, dock)
	toggle = dock.toggleViewAction()
	toggle.setText(window.tr("Properties"))
	toggle.setToolTip(window.tr("Show or hide the current document properties"))
	window._register_action("view.properties.toggle", toggle,
		lifecycle="stateful-visibility")
	return dock
