"""Modeless Rust-authoritative reaction inspector and aggregate nudge controller."""

# Standard Library
import html

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors
import ferrum_qt.ferrum.engine


#============================================
def _enum_token(value: object) -> str:
	"""Normalize one closed Rust enum only for readable presentation."""
	return str(getattr(value, "name", value)).rsplit(".", 1)[-1].replace("_", " ")


#============================================
class _ReactionInspectorMembershipChangedError(RuntimeError):
	"""Report a stale role projection through the controller's typed recovery route."""

	category = "membership_changed"
	recovery = "refresh_and_restart"


#============================================
class _ReactionRoleEditor(PySide6.QtWidgets.QDialog):
	"""Edit a complete membership replacement using durable Rust-issued choices."""

	#============================================
	def __init__(self, reaction: object, choices: object,
			parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build a role editor without a CDML or scene-item route."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Edit Reaction"))
		self.setAccessibleName(self.tr("Edit Reaction"))
		self._lists: dict[str, PySide6.QtWidgets.QListWidget] = {}
		self._updating = False
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		layout.addWidget(PySide6.QtWidgets.QLabel(self.tr(
			"Replace every role for {0}. Ferrum validates the complete reaction atomically."
		).format(reaction.reaction_id), self))
		members_by_role: dict[str, list[object]] = {
			"reactant": [], "product": [], "arrow": [], "condition": [], "plus": [],
		}
		for member in reaction.members:
			members_by_role[_enum_token(member.role)].append(member)
		for members in members_by_role.values():
			members.sort(key=lambda member: member.role_ordinal)
		member_role_by_identifier = {
			member.identifier: _enum_token(member.role) for member in reaction.members
		}
		for role, title, kinds in (
			("reactants", self.tr("Reactants"), {"reactant"}),
			("products", self.tr("Products"), {"product"}),
			("arrow", self.tr("Arrow"), {"arrow"}),
			("conditions", self.tr("Conditions"), {"condition"}),
			("pluses", self.tr("Plus signs"), {"plus"}),
		):
			group = PySide6.QtWidgets.QGroupBox(title, self)
			list_widget = PySide6.QtWidgets.QListWidget(group)
			list_widget.setObjectName(f"reaction-inspector-{role}")
			list_widget.setAccessibleName(title)
			list_widget.setSelectionMode(
				PySide6.QtWidgets.QAbstractItemView.SelectionMode.NoSelection,
			)
			member_role = {
				"reactants": "reactant", "products": "product", "arrow": "arrow",
				"conditions": "condition", "pluses": "plus",
			}[role]
			choices_by_identifier = {choice.identifier: choice for choice in choices.choices}
			ordered_choices = [
				choices_by_identifier[member.identifier]
				for member in members_by_role[member_role]
			]
			ordered_choices.extend(
				choice for choice in sorted(choices.choices, key=lambda item: item.source_order)
				if choice.identifier not in {
					member.identifier for member in members_by_role[member_role]
				}
			)
			for choice in ordered_choices:
				kind = _enum_token(choice.kind)
				allowed = (
					(role in ("reactants", "products") and kind == "molecule")
					or (role == "arrow" and kind == "arrow")
					or (role == "conditions" and kind == "condition text")
					or (role == "pluses" and kind == "plus")
				)
				if not allowed:
					continue
				item = PySide6.QtWidgets.QListWidgetItem(
					f"{choice.label} ({choice.identifier})",
				)
				item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, choice.identifier)
				item.setFlags(item.flags() | PySide6.QtCore.Qt.ItemFlag.ItemIsUserCheckable)
				item.setCheckState(
					PySide6.QtCore.Qt.CheckState.Checked
					if member_role_by_identifier.get(choice.identifier) == member_role
					else PySide6.QtCore.Qt.CheckState.Unchecked,
				)
				if (
					_enum_token(choice.availability) != "eligible"
					and choice.identifier not in member_role_by_identifier
				):
					item.setFlags(PySide6.QtCore.Qt.ItemFlag.NoItemFlags)
					item.setToolTip(self.tr("This root already belongs to another reaction."))
				list_widget.addItem(item)
			list_widget.itemChanged.connect(
				lambda item, changed_role=role: self._on_role_changed(changed_role, item),
			)
			group_layout = PySide6.QtWidgets.QVBoxLayout(group)
			group_layout.addWidget(list_widget)
			layout.addWidget(group)
			self._lists[role] = list_widget
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel, self,
		)
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)

	#============================================
	def _on_role_changed(self, role: str, item: PySide6.QtWidgets.QListWidgetItem) -> None:
		"""Keep each durable root in one complete role and the arrow singular."""
		if self._updating or item.checkState() != PySide6.QtCore.Qt.CheckState.Checked:
			return
		self._updating = True
		try:
			identifier = item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
			for other_role, list_widget in self._lists.items():
				for index in range(list_widget.count()):
					candidate = list_widget.item(index)
					if candidate is item:
						continue
					if candidate.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier or (
						role == "arrow" and other_role == "arrow"
					):
						candidate.setCheckState(PySide6.QtCore.Qt.CheckState.Unchecked)
		finally:
			self._updating = False

	#============================================
	def request(self) -> tuple[list[str], list[str], str | None, list[str], list[str]]:
		"""Return a complete durable-role replacement for Rust validation."""
		def selected(role: str) -> list[str]:
			return [
				self._lists[role].item(index).data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
				for index in range(self._lists[role].count())
				if self._lists[role].item(index).checkState()
				== PySide6.QtCore.Qt.CheckState.Checked
			]
		arrows = selected("arrow")
		return selected("reactants"), selected("products"), (
			arrows[0] if len(arrows) == 1 else None
		), selected("conditions"), selected("pluses")


#============================================
class _ReactionDefinitionDeleteDialog(PySide6.QtWidgets.QDialog):
	"""Confirm the safe, definition-only reaction deletion in Ferrum controls."""

	#============================================
	def __init__(self, reaction_id: str, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Explain exactly what deletion changes before exposing one destructive action."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Delete Reaction Definition"))
		self.setObjectName("reaction-inspector-delete-dialog")
		self.setAccessibleName(self.tr("Delete Reaction Definition"))
		self.setModal(True)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		question = PySide6.QtWidgets.QLabel(self.tr(
			"Remove reaction definition {0}?"
		).format(reaction_id), self)
		question.setWordWrap(True)
		layout.addWidget(question)
		consequence = PySide6.QtWidgets.QLabel(self.tr(
			"Only the reaction definition is removed. Its molecule, arrow, text, and plus-sign "
			"member roots remain in the document. Undo restores the definition."
		), self)
		consequence.setObjectName("reaction-inspector-delete-consequence")
		consequence.setAccessibleName(self.tr("Deletion consequence"))
		consequence.setWordWrap(True)
		layout.addWidget(consequence)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		delete = buttons.addButton(
			self.tr("Delete reaction definition"),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.DestructiveRole,
		)
		delete.setObjectName("reaction-inspector-delete-confirm")
		delete.setAccessibleName(self.tr("Delete reaction definition"))
		cancel = buttons.addButton(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel)
		cancel.setObjectName("reaction-inspector-delete-cancel")
		cancel.setAccessibleName(self.tr("Cancel deletion"))
		cancel.setDefault(True)
		delete.clicked.connect(self.accept)
		cancel.clicked.connect(self.reject)
		layout.addWidget(buttons)


#============================================
class ReactionInspectorController(PySide6.QtCore.QObject):
	"""Own modeless reaction inspection and opaque lifecycle actions for one window."""

	#============================================
	def __init__(self, window: object) -> None:
		"""Bind a document-scoped inspector without retaining Rust capabilities."""
		super().__init__(window)
		self._window = window
		self._dock: PySide6.QtWidgets.QDockWidget | None = None
		self._list: PySide6.QtWidgets.QListWidget | None = None
		self._detail: PySide6.QtWidgets.QTextBrowser | None = None
		self._tab: object | None = None
		self._observation: object | None = None
		self._reaction_id: str | None = None
		self._owned_dialog: PySide6.QtWidgets.QDialog | None = None
		self._window.installEventFilter(self)

	#============================================
	def install_action(self, menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
		"""Install the ordinary menu and ribbon command route."""
		action = PySide6.QtGui.QAction(self.tr("Reaction Inspector"), self._window)
		action.setObjectName("reaction-inspector-action")
		action.setToolTip(self.tr("Inspect and edit Rust-owned reaction definitions"))
		action.triggered.connect(self.open)
		menu.addAction(action)
		return action

	#============================================
	def refresh_action(self, action: PySide6.QtGui.QAction) -> None:
		"""Expose inspection for any live mutable Ferrum tab."""
		tab = self._window._active_native_tab()
		action.setEnabled(tab is not None and not tab._disposed and not tab.requires_refresh)

	#============================================
	def open(self) -> None:
		"""Open a fresh list view after retiring competing disposable tools."""
		self.close()
		for method_name in (
			"_cancel_atom_insertion", "_cancel_line_gesture", "_cancel_structure_selection",
			"_cancel_catalog_placement",
		):
			getattr(self._window, method_name, lambda: None)()
		tab = self._window._active_native_tab()
		if tab is None:
			return
		self._tab = tab
		self._dock = PySide6.QtWidgets.QDockWidget(self.tr("Reaction Inspector"), self._window)
		self._dock.setObjectName("reaction-inspector-dock")
		self._dock.setAccessibleName(self.tr("Reaction Inspector"))
		self._dock.setFeatures(PySide6.QtWidgets.QDockWidget.DockWidgetFeature.DockWidgetClosable)
		panel = PySide6.QtWidgets.QWidget(self._dock)
		layout = PySide6.QtWidgets.QVBoxLayout(panel)
		layout.setContentsMargins(10, 10, 10, 10)
		layout.addWidget(PySide6.QtWidgets.QLabel(self.tr(
			"Reaction membership, diagnostics, and bounds are issued by Rust."
		), panel))
		self._list = PySide6.QtWidgets.QListWidget(panel)
		self._list.setObjectName("reaction-inspector-list")
		self._list.setAccessibleName(self.tr("Rust-issued reactions"))
		self._list.currentItemChanged.connect(self._on_current_changed)
		layout.addWidget(self._list)
		self._detail = PySide6.QtWidgets.QTextBrowser(panel)
		self._detail.setObjectName("reaction-inspector-detail")
		self._detail.setAccessibleName(self.tr("Reaction details"))
		layout.addWidget(self._detail)
		buttons = PySide6.QtWidgets.QGridLayout()
		for text, slot, row, column in (
			(self.tr("Refresh"), self.refresh, 0, 0),
			(self.tr("Highlight Members"), self.highlight_members, 0, 1),
			(self.tr("Edit Roles..."), self.edit_roles, 1, 0),
			(self.tr("Delete Definition..."), self.delete_definition, 1, 1),
			(self.tr("Nudge Left"), lambda: self.nudge(-10.0, 0.0), 2, 0),
			(self.tr("Nudge Right"), lambda: self.nudge(10.0, 0.0), 2, 1),
			(self.tr("Nudge Up"), lambda: self.nudge(0.0, -10.0), 3, 0),
			(self.tr("Nudge Down"), lambda: self.nudge(0.0, 10.0), 3, 1),
		):
			button = PySide6.QtWidgets.QPushButton(text, panel)
			button.setAccessibleName(text)
			button.clicked.connect(slot)
			buttons.addWidget(button, row, column)
		self._snap = PySide6.QtWidgets.QCheckBox(self.tr("Snap nudge to View Hex Grid"), panel)
		self._snap.setAccessibleName(self.tr("Snap nudge to View Hex Grid"))
		buttons.addWidget(self._snap, 4, 0, 1, 2)
		layout.addLayout(buttons)
		self._dock.setWidget(panel)
		self._dock.visibilityChanged.connect(self._on_visibility_changed)
		self._window.addDockWidget(PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea, self._dock)
		self._dock.show()
		self.refresh()

	#============================================
	def close(self) -> None:
		"""Retire only Qt observation state and return canvas focus when possible."""
		dock = self._dock
		tab = self._tab
		self._dock = None
		self._list = None
		self._detail = None
		self._tab = None
		self._observation = None
		self._reaction_id = None
		if dock is not None:
			dock.hide()
			dock.deleteLater()
		if tab is not None and not tab._disposed:
			tab.view.viewport().setFocus()

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Retire on hide, close, or external deactivation of this modeless tool."""
		if watched is self._window and event.type() == PySide6.QtCore.QEvent.Type.WindowDeactivate:
			if self._has_foreground_owned_modal():
				return super().eventFilter(watched, event)
			self.close()
		elif watched is self._window and event.type() in (
			PySide6.QtCore.QEvent.Type.Hide, PySide6.QtCore.QEvent.Type.Close,
		):
			self.close()
		return super().eventFilter(watched, event)

	#============================================
	def _has_foreground_owned_modal(self) -> bool:
		"""Return whether the inspector's modal owns the application's foreground focus."""
		dialog = self._owned_dialog
		if dialog is None or not dialog.isVisible():
			return False
		application = PySide6.QtWidgets.QApplication.instance()
		return application.activeModalWidget() is dialog and dialog.isActiveWindow()

	#============================================
	def _on_visibility_changed(self, visible: bool) -> None:
		"""Dispose inspection state when the modeless dock is closed."""
		if not visible:
			self.close()

	#============================================
	def refresh(self) -> None:
		"""Replace every observation with one current Rust-issued list."""
		if self._tab is None or self._list is None:
			return
		selected = self._reaction_id
		try:
			self._observation = self._tab.observe_reaction_list()
		except Exception as exc:
			self._recover(exc)
			return
		self._list.blockSignals(True)
		self._list.clear()
		for reaction in self._observation.reactions:
			item = PySide6.QtWidgets.QListWidgetItem(
				f"{reaction.reaction_id} [{_enum_token(reaction.disposition)}]",
			)
			item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, reaction.reaction_id)
			self._list.addItem(item)
			if reaction.reaction_id == selected:
				self._list.setCurrentItem(item)
		self._list.blockSignals(False)
		if self._list.currentItem() is None and self._list.count():
			self._list.setCurrentRow(0)
		else:
			self._on_current_changed(self._list.currentItem(), None)

	#============================================
	def _on_current_changed(self, current: object, _previous: object) -> None:
		"""Render a faithful readable projection of selected Rust facts."""
		if current is None or self._observation is None or self._detail is None:
			return
		self._reaction_id = current.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
		reaction = self._reaction()
		member_lines = "<br/>".join(
			"{0} {1}: {2}".format(
				html.escape(str(member.role)), member.role_ordinal + 1,
				html.escape(str(member.identifier)),
			)
			for member in reaction.members
		)
		diagnostics = "<br/>".join(
			"{0}. Recovery: {1}.".format(
				html.escape(_enum_token(item.reason)), html.escape(_enum_token(item.recovery)),
			)
			for item in reaction.diagnostics
		) or self.tr("None")
		bounds = reaction.union_bounds
		bounds_text = self.tr("No complete render bounds") if bounds is None else (
			f"{bounds.left:g}, {bounds.top:g} to {bounds.right:g}, {bounds.bottom:g}"
		)
		self._detail.setHtml(
			f"<b>{html.escape(str(reaction.reaction_id))}</b><br/>"
			f"Disposition: {html.escape(_enum_token(reaction.disposition))}<br/>"
			f"Bounds: {html.escape(bounds_text)}<br/><br/><b>Members</b><br/>{member_lines}"
			f"<br/><br/><b>Diagnostics</b><br/>{diagnostics}",
		)

	#============================================
	def _reaction(self) -> object:
		"""Resolve the selected projection only from the active Rust list."""
		if self._observation is None or self._reaction_id is None:
			raise RuntimeError("select a reaction first")
		for reaction in self._observation.reactions:
			if reaction.reaction_id == self._reaction_id:
				return reaction
		raise RuntimeError("the selected reaction is no longer in the current Rust observation")

	#============================================
	def _selection(self) -> object:
		"""Acquire a fresh opaque selection, never retaining it in Qt state."""
		if self._tab is None or self._observation is None or self._reaction_id is None:
			raise RuntimeError("refresh and select a reaction first")
		return self._tab.select_reaction(self._observation, self._reaction_id)

	#============================================
	def highlight_members(self) -> None:
		"""Project exact Rust member identifiers through existing root selection."""
		if self._tab is None:
			return
		try:
			reaction = self._reaction()
			observation = self._tab.observe_direct_root_interaction()
			selection = None
			for member in reaction.members:
				modifier = (
					ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
					if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
				)
				selection = self._tab.select_direct_roots(
					observation, selection,
					ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(member.identifier, modifier),
				)
			self._window._replace_render_interaction_selection(selection, self._tab)
			self._window.statusBar().showMessage(
				self.tr("Highlighted all Rust-issued members of {0}.").format(reaction.reaction_id), 5000,
			)
		except Exception as exc:
			self._recover(exc)

	#============================================
	def edit_roles(self) -> None:
		"""Submit a complete role replacement through an opaque fresh selection."""
		if self._tab is None:
			return
		try:
			reaction = self._reaction()
			choices = self._tab.observe_reaction_authoring_choices()
		except Exception as exc:
			self._recover(exc)
			return
		try:
			editor = _ReactionRoleEditor(reaction, choices, self._window)
		except KeyError:
			self._recover(_ReactionInspectorMembershipChangedError())
			return
		if self._run_owned_dialog(editor) != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		reactants, products, arrow, conditions, pluses = editor.request()
		if not reactants or not products or arrow is None:
			self._window.statusBar().showMessage(
				self.tr("Choose at least one reactant, one product, and one arrow."), 5000,
			)
			return
		try:
			self._tab.patch_reaction_membership(
				self._selection(), reactants, products, arrow, conditions, pluses,
			)
		except ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabMutationPresentationError:
			self._recover_accepted_mutation(
				"Updated reaction roles", rehighlight=True,
			)
			return
		except Exception as exc:
			self._recover(exc)
			return
		self.refresh()
		self.highlight_members()

	#============================================
	def delete_definition(self) -> None:
		"""Confirm and remove only the reaction record, retaining all member roots."""
		try:
			reaction = self._reaction()
		except Exception as exc:
			self._recover(exc)
			return
		dialog = _ReactionDefinitionDeleteDialog(reaction.reaction_id, self._window)
		if self._run_owned_dialog(dialog) != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			self._tab.delete_reaction_definition(self._selection())
		except ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabMutationPresentationError:
			self._recover_accepted_mutation("Deleted the reaction definition", rehighlight=False)
			return
		except Exception as exc:
			self._recover(exc)
			return
		self._window._replace_render_interaction_selection(None, self._tab)
		self.refresh()
		self._window.statusBar().showMessage(
			self.tr("Deleted the reaction definition. Its member roots remain; Undo restores it."), 5000,
		)

	#============================================
	def nudge(self, delta_x: float, delta_y: float) -> None:
		"""Move every selected reaction member through one opaque Rust receipt."""
		try:
			self._tab.translate_reaction(
				self._selection(), delta_x, delta_y, self._snap.isChecked(),
			)
		except ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabMutationPresentationError:
			self._recover_accepted_mutation("Moved all reaction members", rehighlight=True)
			return
		except Exception as exc:
			self._recover(exc)
			return
		self.refresh()
		self.highlight_members()

	#============================================
	def _run_owned_dialog(self, dialog: PySide6.QtWidgets.QDialog) -> int:
		"""Run one inspector-owned modal without treating its parent deactivation as loss."""
		self._owned_dialog = dialog
		try:
			result = dialog.exec()
		finally:
			self._owned_dialog = None
		return result

	#============================================
	def _recover_accepted_mutation(self, action: str, rehighlight: bool) -> None:
		"""Refresh only after Rust accepted a mutation but Qt projection installation failed."""
		if self._tab is None:
			return
		tab = self._tab
		self._window._replace_render_interaction_selection(None, tab)
		recovered = tab.refresh_authoritative()
		self._window._refresh_actions()
		if not recovered:
			self.close()
			self._window.statusBar().showMessage(
				self.tr(
					"{0}; Rust accepted the change, but display recovery is required before further editing."
				).format(action), 7000,
			)
			return
		self.refresh()
		if rehighlight and self._tab is tab and self._observation is not None:
			self.highlight_members()
		self._window.statusBar().showMessage(
			self.tr("{0}; the display was refreshed after installation recovery.").format(action),
			7000,
		)

	#============================================
	def _recover(self, error: Exception) -> None:
		"""Map typed Rust refusal facts to one no-mutation refresh boundary."""
		category = _enum_token(getattr(error, "category", "observation"))
		recovery = _enum_token(getattr(error, "recovery", "refresh and restart"))
		self._window.statusBar().showMessage(
			self.tr("Reaction action refused: {0}. Recovery: {1}.").format(category, recovery), 7000,
		)
		if category in {"stale snapshot", "foreign session", "session conflict", "membership changed"}:
			self.close()
