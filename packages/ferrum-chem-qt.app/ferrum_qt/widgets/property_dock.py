"""Read-only property projection dock backed by immutable observations."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def _count_label(count: int, singular: str) -> str:
	"""Return one readable semantic count."""
	return f"{count} {singular if count == 1 else singular + 's'}"


#============================================
def _value_label(value: object | None) -> str:
	"""Present a closed projection value without interpreting document data."""
	if value is None:
		return "Not authored"
	name = getattr(value, "name", None)
	if type(name) is str:
		return name.replace("_", " ").title()
	return str(value).rsplit(".", 1)[-1].replace("_", " ").title()


#============================================
def _document_lines(document: object) -> tuple[str, ...]:
	"""Summarize fields exposed by the immutable Rust observation DTO."""
	molecules = tuple(getattr(document, "molecules", ()))
	atoms = sum(len(getattr(molecule, "atoms", ())) for molecule in molecules)
	bonds = sum(len(getattr(molecule, "bonds", ())) for molecule in molecules)
	presentation_stack = getattr(document, "presentation_stack", None)
	drawings = len(getattr(presentation_stack, "roots", ()))
	if not molecules and not drawings:
		return ("No drawable objects",)
	lines: list[str] = []
	if molecules:
		lines.extend((
			_count_label(len(molecules), "molecule"),
			_count_label(atoms, "atom"),
			_count_label(bonds, "bond"),
		))
	if drawings:
		lines.append(_count_label(drawings, "drawing object"))
	issues = len(getattr(document, "issues", ())) + len(getattr(presentation_stack, "issues", ()))
	if issues:
		lines.append(_count_label(issues, "projection warning"))
	return tuple(lines)


#============================================
def _find_projection(document: object, collection_name: str, identifier: str) -> object | None:
	"""Find one durable projection item by its Rust-issued source identifier."""
	for molecule in getattr(document, "molecules", ()):
		for item in getattr(molecule, collection_name, ()):
			if getattr(item, "source_id", None) == identifier:
				return item
	return None


#============================================
def _selection_lines(selection: tuple[object, ...]) -> tuple[str, ...]:
	"""Summarize mixed selection DTOs without defining edit behavior."""
	counts: dict[str, int] = {}
	for target in selection:
		label = str(getattr(target, "kind", "drawing object")).replace("_", " ").title()
		counts[label] = counts.get(label, 0) + 1
	return tuple(_count_label(counts[label], label.lower()) for label in sorted(counts))


#============================================
class PropertyDock(PySide6.QtWidgets.QDockWidget):
	"""Show immutable document facts and client actions supplied by a registry."""

	#============================================
	def __init__(self, registry: object,
			parent: PySide6.QtWidgets.QMainWindow | None = None) -> None:
		"""Build a dock that accepts observation DTOs rather than a document owner."""
		super().__init__(self.tr("Properties"), parent)
		if not callable(getattr(registry, "get_qt_action", None)):
			raise TypeError("Ferrum property dock needs an ActionRegistry-like client")
		self.setObjectName("properties-dock")
		self.setAccessibleName(self.tr("Document properties"))
		self.setAllowedAreas(
			PySide6.QtCore.Qt.DockWidgetArea.LeftDockWidgetArea
			| PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea,
		)
		panel = PySide6.QtWidgets.QWidget(self)
		layout = PySide6.QtWidgets.QVBoxLayout(panel)
		layout.setContentsMargins(12, 12, 12, 12)
		layout.setSpacing(8)
		self._heading = PySide6.QtWidgets.QLabel(self.tr("No document"), panel)
		font = self._heading.font()
		font.setBold(True)
		self._heading.setFont(font)
		self._heading.setAccessibleName(self.tr("Properties heading"))
		layout.addWidget(self._heading)
		self._summary = PySide6.QtWidgets.QLabel(self.tr("Open or create a document."), panel)
		self._summary.setObjectName("properties-summary")
		self._summary.setAccessibleName(self.tr("Current properties"))
		self._summary.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByMouse,
		)
		self._summary.setWordWrap(True)
		layout.addWidget(self._summary)
		self._atom_button = self._action_button(registry, "edit.atom.properties", panel)
		self._bond_button = self._action_button(registry, "edit.bond.properties", panel)
		layout.addWidget(self._atom_button)
		layout.addWidget(self._bond_button)
		layout.addStretch()
		self.setWidget(panel)
		self.refresh(None)

	#============================================
	def _action_button(self, registry: object, action_id: str,
			parent: PySide6.QtWidgets.QWidget) -> PySide6.QtWidgets.QToolButton:
		"""Create one button client of a registry action, never a duplicate command."""
		button = PySide6.QtWidgets.QToolButton(parent)
		action = registry.get_qt_action(action_id)
		if isinstance(action, PySide6.QtGui.QAction):
			button.setDefaultAction(action)
			button.setAccessibleName(action.text())
		else:
			button.setVisible(False)
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextOnly)
		return button

	#============================================
	@property
	def summary_text(self) -> str:
		"""Return visible summary text for lightweight widget verification."""
		return self._summary.text()

	#============================================
	def refresh(self, observation: object | None) -> None:
		"""Project one immutable observation, retaining no tab, session, or scene."""
		if observation is None:
			self._show("No document", ("Open or create a document.",), None)
			return
		document = getattr(observation, "document", None)
		if document is None:
			self._show("Refresh required", ("Properties will return after the authoritative view is refreshed.",), None)
			return
		selection = tuple(getattr(observation, "selection", ()))
		if len(selection) != 1:
			if selection:
				self._show("Selection", _selection_lines(selection), None)
			else:
				self._show("Document", _document_lines(document), None)
			return
		target = selection[0]
		kind = getattr(target, "kind", "drawing object")
		identifier = getattr(target, "identifier", None)
		if kind == "atom" and type(identifier) is str:
			atom = _find_projection(document, "atoms", identifier)
			if atom is not None:
				position = getattr(atom, "position", None)
				lines = (
					f"Element: {getattr(atom, 'element', None) or 'Unspecified'}",
					f"Formal charge: {getattr(atom, 'formal_charge', None) if getattr(atom, 'formal_charge', None) is not None else 'Not authored'}",
					f"Position: {getattr(position, 'x', 0.0):g}, {getattr(position, 'y', 0.0):g}",
				)
				self._show("Atom", lines, "atom")
				return
		if kind == "bond" and type(identifier) is str:
			bond = _find_projection(document, "bonds", identifier)
			if bond is not None:
				start = getattr(getattr(bond, "start", None), "source_id", None) or "Unresolved"
				end = getattr(getattr(bond, "end", None), "source_id", None) or "Unresolved"
				self._show("Bond", (
					f"Order: {_value_label(getattr(bond, 'order', None))}",
					f"Style: {_value_label(getattr(bond, 'style', None))}",
					f"Endpoints: {start} to {end}",
				), "bond")
				return
		self._show(str(kind).replace("_", " ").title(), ("Drawing object selected",), None)

	#============================================
	def _show(self, heading: str, lines: tuple[str, ...], edit_kind: str | None) -> None:
		"""Replace displayed facts while each shared action owns enablement."""
		self._heading.setText(self.tr(heading))
		self._summary.setText("\n".join(self.tr(line) for line in lines))
		self._atom_button.setVisible(edit_kind == "atom" and not self._atom_button.defaultAction() is None)
		self._bond_button.setVisible(edit_kind == "bond" and not self._bond_button.defaultAction() is None)
