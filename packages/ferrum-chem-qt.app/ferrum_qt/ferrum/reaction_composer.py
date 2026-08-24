"""Modeless Rust-authoritative reaction role composer for one Ferrum window."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors
import ferrum_qt.ferrum.engine


#============================================
def _enum_token(value: object) -> str:
	"""Normalize one frozen PyO3 enum only for presentation routing."""
	return str(getattr(value, "name", value)).rsplit(".", 1)[-1].lower()


#============================================
class _ReactionComposerPanel(PySide6.QtWidgets.QWidget):
	"""Accessible role lists that retain only durable backend-issued identifiers."""

	changed = PySide6.QtCore.Signal()
	cancelled = PySide6.QtCore.Signal()
	submitted = PySide6.QtCore.Signal()

	#============================================
	def __init__(self, choices: tuple[object, ...], exclusions: tuple[object, ...],
			parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build role lists from an immutable Rust observation, not canvas items."""
		super().__init__(parent)
		self._choices = tuple(sorted(choices, key=lambda choice: choice.source_order))
		self._updating = False
		self.setAccessibleName(self.tr("Define Reaction"))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		layout.setContentsMargins(10, 10, 10, 10)
		layout.setSpacing(8)
		heading = PySide6.QtWidgets.QLabel(
			self.tr("Create reaction from {0} selected roots").format(
				len(self._choices) + len(exclusions),
			), self,
		)
		heading.setObjectName("reaction-composer-heading")
		font = heading.font()
		font.setBold(True)
		heading.setFont(font)
		layout.addWidget(heading)
		requirements = PySide6.QtWidgets.QLabel(self.tr(
			"Choose at least one reactant, at least one product, and exactly one arrow.",
		), self)
		requirements.setWordWrap(True)
		requirements.setAccessibleName(self.tr("Reaction requirements"))
		layout.addWidget(requirements)
		layout.addWidget(self._new_source_list())
		if exclusions:
			layout.addWidget(self._new_exclusion_list(exclusions))
		self._lists: dict[str, PySide6.QtWidgets.QListWidget] = {}
		for role, label, kinds in (
			("reactants", self.tr("Reactants"), {"molecule"}),
			("products", self.tr("Products"), {"molecule"}),
			("arrow", self.tr("Arrow"), {"arrow"}),
		):
			layout.addWidget(self._new_role_list(role, label, kinds))
		optional_toggle = PySide6.QtWidgets.QToolButton(self)
		optional_toggle.setText(self.tr("Optional annotation roots"))
		optional_toggle.setCheckable(True)
		optional_toggle.setChecked(False)
		optional_toggle.setToolButtonStyle(
			PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon,
		)
		optional_toggle.setArrowType(PySide6.QtCore.Qt.ArrowType.RightArrow)
		optional_toggle.setAccessibleName(self.tr("Optional annotation roots"))
		layout.addWidget(optional_toggle)
		self._optional = PySide6.QtWidgets.QWidget(self)
		optional_layout = PySide6.QtWidgets.QVBoxLayout(self._optional)
		optional_layout.setContentsMargins(12, 0, 0, 0)
		optional_layout.setSpacing(6)
		optional_layout.addWidget(self._new_role_list("pluses", self.tr("Plus signs"), {"plus"}))
		optional_layout.addWidget(self._new_role_list(
			"conditions", self.tr("Condition text"), {"condition_text"},
		))
		self._optional.setVisible(False)
		optional_toggle.toggled.connect(
			lambda checked: self._set_optional_visible(optional_toggle, checked),
		)
		layout.addWidget(self._optional)
		self._validation = PySide6.QtWidgets.QLabel(self)
		self._validation.setObjectName("reaction-composer-validation")
		self._validation.setWordWrap(True)
		self._validation.setAccessibleName(self.tr("Reaction validation"))
		layout.addWidget(self._validation)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self._create_button = buttons.addButton(
			self.tr("Create Reaction"), PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole,
		)
		cancel = buttons.addButton(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel)
		self._create_button.setAccessibleName(self.tr("Create Reaction"))
		cancel.setAccessibleName(self.tr("Cancel reaction definition"))
		self._create_button.clicked.connect(self.submitted)
		cancel.clicked.connect(self.cancelled)
		layout.addWidget(buttons)
		self._refresh_validation()

	#============================================
	def _new_source_list(self) -> PySide6.QtWidgets.QGroupBox:
		"""Show only selected Rust-issued roots that can receive a reaction role."""
		group = PySide6.QtWidgets.QGroupBox(self.tr("Selected usable roots"), self)
		list_widget = PySide6.QtWidgets.QListWidget(group)
		list_widget.setObjectName("reaction-composer-selected-usable-roots")
		list_widget.setAccessibleName(self.tr("Selected usable roots"))
		list_widget.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.NoSelection,
		)
		for choice in self._choices:
			item = PySide6.QtWidgets.QListWidgetItem(
				f"{choice.label} ({choice.identifier}) [{_enum_token(choice.kind)}; {_enum_token(choice.availability)}]",
			)
			item.setToolTip(item.text())
			list_widget.addItem(item)
		layout = PySide6.QtWidgets.QVBoxLayout(group)
		layout.addWidget(list_widget)
		return group

	#============================================
	def _new_exclusion_list(self, exclusions: tuple[object, ...]) -> PySide6.QtWidgets.QGroupBox:
		"""Display Rust diagnostics as unavailable facts, never selectable roots."""
		group = PySide6.QtWidgets.QGroupBox(self.tr("Unavailable selected roots"), self)
		list_widget = PySide6.QtWidgets.QListWidget(group)
		list_widget.setObjectName("reaction-composer-unavailable-roots")
		list_widget.setAccessibleName(self.tr("Unavailable selected roots"))
		list_widget.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.NoSelection,
		)
		for exclusion in exclusions:
			reason = _enum_token(exclusion.reason)
			recovery = _enum_token(exclusion.recovery)
			message = self._exclusion_recovery_message(recovery)
			text = self.tr("{0}\nReason: {1}\nRecovery: {2}. {3}").format(
				exclusion.label, reason, recovery, message,
			)
			item = PySide6.QtWidgets.QListWidgetItem(text)
			item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, exclusion.diagnostic_key)
			item.setFlags(PySide6.QtCore.Qt.ItemFlag.NoItemFlags)
			item.setToolTip(text)
			list_widget.addItem(item)
		layout = PySide6.QtWidgets.QVBoxLayout(group)
		layout.addWidget(list_widget)
		return group

	#============================================
	def _exclusion_recovery_message(self, recovery: str) -> str:
		"""Translate the closed Rust recovery token without inventing a Qt repair path."""
		if recovery == "choose_supported_member":
			return self.tr("Choose a supported molecule, arrow, plus sign, or condition text.")
		if recovery == "repair_document":
			return self.tr("Repair the document, then refresh and select the reaction members again.")
		return self.tr("Refresh and select the reaction members again.")

	#============================================
	def _new_role_list(self, role: str, label: str, kinds: set[str]) -> PySide6.QtWidgets.QGroupBox:
		"""Create one role list containing only its exact immutable kinds."""
		group = PySide6.QtWidgets.QGroupBox(label, self)
		list_widget = PySide6.QtWidgets.QListWidget(group)
		list_widget.setObjectName(f"reaction-composer-{role}")
		list_widget.setAccessibleName(label)
		list_widget.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.NoSelection,
		)
		for choice in self._choices:
			if _enum_token(choice.kind) not in kinds:
				continue
			item = PySide6.QtWidgets.QListWidgetItem(f"{choice.label} ({choice.identifier})")
			item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, choice.identifier)
			item.setFlags(item.flags() | PySide6.QtCore.Qt.ItemFlag.ItemIsUserCheckable)
			item.setCheckState(PySide6.QtCore.Qt.CheckState.Unchecked)
			if _enum_token(choice.availability) != "eligible":
				item.setFlags(PySide6.QtCore.Qt.ItemFlag.NoItemFlags)
				item.setToolTip(self.tr("Already belongs to an authored reaction."))
			list_widget.addItem(item)
		list_widget.itemChanged.connect(
			lambda item, changed_role=role: self._on_role_changed(changed_role, item),
		)
		self._lists[role] = list_widget
		layout = PySide6.QtWidgets.QVBoxLayout(group)
		layout.addWidget(list_widget)
		return group

	#============================================
	def _set_optional_visible(self, toggle: PySide6.QtWidgets.QToolButton,
			checked: bool) -> None:
		"""Expand optional exact-type roles without changing any assignment."""
		toggle.setArrowType(
			PySide6.QtCore.Qt.ArrowType.DownArrow if checked
			else PySide6.QtCore.Qt.ArrowType.RightArrow,
		)
		self._optional.setVisible(checked)

	#============================================
	def _on_role_changed(self, role: str, item: PySide6.QtWidgets.QListWidgetItem) -> None:
		"""Keep each durable root in one visible role and arrow singular."""
		if self._updating or item.checkState() != PySide6.QtCore.Qt.CheckState.Checked:
			self._refresh_validation()
			return
		self._updating = True
		try:
			identifier = item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
			for other_role, list_widget in self._lists.items():
				for index in range(list_widget.count()):
					candidate = list_widget.item(index)
					if candidate is item:
						continue
					if (
						candidate.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier
						or (role == "arrow" and other_role == "arrow")
					):
						candidate.setCheckState(PySide6.QtCore.Qt.CheckState.Unchecked)
		finally:
			self._updating = False
		self._refresh_validation()
		self.changed.emit()

	#============================================
	def _selected(self, role: str) -> list[str]:
		"""Return one source-ordered role sequence from checked UI rows."""
		return [
			self._lists[role].item(index).data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
			for index in range(self._lists[role].count())
			if self._lists[role].item(index).checkState() == PySide6.QtCore.Qt.CheckState.Checked
		]

	#============================================
	def request(self) -> tuple[list[str], list[str], str | None, list[str], list[str]]:
		"""Return only durable role identifiers for the opaque mutation boundary."""
		arrows = self._selected("arrow")
		return (
			self._selected("reactants"), self._selected("products"),
			arrows[0] if len(arrows) == 1 else None,
			self._selected("conditions"), self._selected("pluses"),
		)

	#============================================
	def _refresh_validation(self) -> None:
		"""Make incomplete role assignment visible without colour-only feedback."""
		reactants, products, arrow, _conditions, _pluses = self.request()
		valid = bool(reactants and products and arrow is not None)
		self._create_button.setEnabled(valid)
		self._validation.setText(
			self.tr("Reaction roles are complete.") if valid else self.tr(
				"Choose at least one reactant, at least one product, and one arrow.",
			),
		)

	#============================================
	def show_refusal(self, message: str) -> None:
		"""Keep a selector-correctable form open with an inline typed message."""
		self._validation.setText(message)
		self._validation.setFocus()

	#============================================
	def keyPressEvent(self, event: PySide6.QtGui.QKeyEvent) -> None:
		"""Make Escape a strict no-mutation cancellation path."""
		if event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
			self.cancelled.emit()
			event.accept()
			return
		super().keyPressEvent(event)


#============================================
class ReactionComposerController(PySide6.QtCore.QObject):
	"""Own one modeless reaction composer and its fenced Qt lifecycle."""

	#============================================
	def __init__(self, window: object) -> None:
		"""Attach lifecycle observation without acquiring document ownership."""
		super().__init__(window)
		self._window = window
		self._dock: PySide6.QtWidgets.QDockWidget | None = None
		self._panel: _ReactionComposerPanel | None = None
		self._tab: object | None = None
		self._choices: object | None = None
		self._revision: int | None = None
		self._digest: str | None = None
		self._poller = PySide6.QtCore.QTimer(self)
		self._poller.setInterval(200)
		self._poller.timeout.connect(self._retire_if_stale)
		self._window.installEventFilter(self)
		application = PySide6.QtWidgets.QApplication.instance()
		if application is not None:
			application.focusChanged.connect(self._on_application_focus_changed)

	#============================================
	def install_action(self, menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
		"""Install the one ordinary non-tool entry route."""
		action = PySide6.QtGui.QAction(self.tr("Create Reaction..."), self._window)
		action.setObjectName("create-reaction-action")
		action.setToolTip(self.tr(
			"Classify selected complete roots as reactants, products, arrow, pluses, or condition text through Rust.",
		))
		action.triggered.connect(self.open)
		menu.addAction(action)
		return action

	#============================================
	def refresh_action(self, action: PySide6.QtGui.QAction) -> None:
		"""Keep the command reachable for a live tab and explain missing selection on use."""
		tab = self._window._active_native_tab()
		action.setEnabled(tab is not None and not tab.is_disposed and not tab.requires_refresh)

	#============================================
	def open(self) -> None:
		"""Freeze current Rust choices after terminally retiring pointer authoring."""
		self.close()
		self._window.cancel_active_pointer_authoring()
		tab = self._window._active_native_tab()
		selection = getattr(self._window, "_render_interaction_selection", None)
		if tab is None or selection is None or not selection.roots:
			self._window.statusBar().showMessage(
				self.tr("Select complete roots first, then choose Create Reaction."), 5000,
			)
			return
		try:
			choices = tab.observe_reaction_authoring_choices()
			tab.validate_reaction_authoring_choices(choices)
		except ferrum_qt.ferrum.engine.ReactionAuthoringChoicesError:
			self._window._replace_render_interaction_selection(None, tab)
			self._window.statusBar().showMessage(
				self.tr("Refresh and select the reaction members again."), 5000,
			)
			return
		selected_ids = {root.identifier for root in selection.roots}
		selected_choices = tuple(
			choice for choice in choices.choices if choice.identifier in selected_ids
		)
		selected_exclusions = tuple(
			exclusion for exclusion in choices.exclusions
			if exclusion.diagnostic_key in selected_ids
		)
		if not selected_choices and not selected_exclusions:
			self._window.statusBar().showMessage(
				self.tr("The selected roots are not eligible for reaction authoring."), 5000,
			)
			return
		self._tab = tab
		self._choices = choices
		self._revision = choices.revision
		self._digest = choices.digest
		self._dock = PySide6.QtWidgets.QDockWidget(self.tr("Define Reaction"), self._window)
		self._dock.setObjectName("reaction-composer-dock")
		self._dock.setAccessibleName(self.tr("Define Reaction"))
		self._dock.setFeatures(PySide6.QtWidgets.QDockWidget.DockWidgetFeature.DockWidgetClosable)
		self._panel = _ReactionComposerPanel(selected_choices, selected_exclusions, self._dock)
		self._panel.cancelled.connect(self.close)
		self._panel.submitted.connect(self._submit)
		self._dock.setWidget(self._panel)
		self._dock.visibilityChanged.connect(self._on_visibility_changed)
		self._window.addDockWidget(PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea, self._dock)
		self._dock.show()
		self._poller.start()
		self._panel.setFocus()

	#============================================
	def _on_visibility_changed(self, visible: bool) -> None:
		"""Retire invisible modeless state without changing CDML."""
		if not visible:
			self.close()

	#============================================
	def close(self, *, restore_canvas_focus: bool = True) -> None:
		"""Dispose composer-only state without changing the authoritative document."""
		self._poller.stop()
		dock = self._dock
		tab = self._tab
		self._dock = None
		self._panel = None
		self._tab = None
		self._choices = None
		self._revision = None
		self._digest = None
		if dock is not None:
			dock.hide()
			dock.deleteLater()
		if restore_canvas_focus:
			self._restore_canvas_focus(tab)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Retire a modeless composer when its owning Ferrum window deactivates."""
		if (
			watched is self._window
			and event.type() in (
				PySide6.QtCore.QEvent.Type.WindowDeactivate,
				PySide6.QtCore.QEvent.Type.Hide,
			)
		):
			self._retire_for_focus_loss()
		return super().eventFilter(watched, event)

	#============================================
	def _on_application_focus_changed(self, _previous: PySide6.QtWidgets.QWidget | None,
			current: PySide6.QtWidgets.QWidget | None) -> None:
		"""Keep canvas and dock focus transitions live, but retire another window's form."""
		if self._tab is None or current is None:
			return
		if current is self._window or self._window.isAncestorOf(current):
			return
		self._retire_for_focus_loss()

	#============================================
	def _retire_for_focus_loss(self) -> None:
		"""Terminally clear disposable authoring state after leaving this document context."""
		tab = self._tab
		if tab is None:
			return
		self.close(restore_canvas_focus=False)
		if self._window._active_native_tab() is tab and not tab.is_disposed:
			self._window._replace_render_interaction_selection(None, tab)
			self._restore_canvas_focus(tab)
		self._window.statusBar().showMessage(
			self.tr("Reaction definition cancelled because focus left this document. "
				"Select the reaction members again."), 5000,
		)
		self._window._refresh_actions()

	#============================================
	def _restore_canvas_focus(self, tab: object | None) -> None:
		"""Return keyboard ownership to the still-active Ferrum canvas when possible."""
		if (
			tab is None or tab.is_disposed or self._window._active_native_tab() is not tab
			or not self._window.isActiveWindow()
		):
			return
		tab.view.setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)

	#============================================
	def _retire_if_stale(self) -> None:
		"""Close rather than submit after tab switch, disposal, or accepted mutation."""
		if self._tab is None:
			return
		snapshot = self._tab.current_snapshot
		if (
			self._window._active_native_tab() is not self._tab
			or self._tab.requires_refresh
			or snapshot.revision != self._revision
			or snapshot.digest != self._digest
		):
			self.close()
			self._window.statusBar().showMessage(
				self.tr("The document changed. Refresh and select the reaction members again."), 5000,
			)

	#============================================
	def _submit(self) -> None:
		"""Commit only a current opaque PyO3 reaction request, then reselect roots."""
		panel = self._panel
		tab = self._tab
		choices = self._choices
		if panel is None or tab is None or choices is None:
			return
		self._retire_if_stale()
		if self._panel is None:
			return
		reactants, products, arrow, conditions, pluses = panel.request()
		if not reactants or not products or arrow is None:
			panel.show_refusal(self.tr("Choose the required reactant, product, and arrow roles."))
			return
		member_ids = reactants + products + [arrow] + pluses + conditions
		try:
			tab.validate_reaction_authoring_choices(choices)
			request = tab.resolve_reaction_create(reactants, products, arrow, conditions, pluses)
			prepared = tab.prepare_session_operation_transition_v1(request)
			result = tab.commit_session_operation_transition_v1(prepared)
			created = tab.install_reaction_created_result(result)
		except ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabMutationPresentationError as exc:
			self._recover_accepted_presentation_failure(tab, exc.accepted_receipt, member_ids)
			return
		except ferrum_qt.ferrum.engine.OperationValidationError as exc:
			panel.show_refusal(str(exc))
			return
		except ferrum_qt.ferrum.engine.PreparedOperationError as exc:
			panel.show_refusal(str(exc))
			return
		except ferrum_qt.ferrum.engine.ReactionAuthoringChoicesError:
			self._restart_after_typed_refusal()
			return
		except ferrum_qt.ferrum.engine.ReactionGestureError as exc:
			self._handle_refusal(exc)
			return
		self.close()
		try:
			self._restore_selection(tab, member_ids)
		except (
			ferrum_qt.ferrum.engine.RenderInteractionError,
			ferrum_qt.ferrum.engine.RevisionConflictError,
		):
			self._window._replace_render_interaction_selection(None, tab)
			recovered = tab.refresh_authoritative()
			self._window.statusBar().showMessage(
				self.tr("Reaction {0} was created, but the authoritative display needs recovery.").format(
					created.reaction_id,
				) if not recovered else self.tr(
					"Reaction {0} was created. Refresh and select the reaction members again.",
				).format(created.reaction_id), 5000,
			)
			return
		self._window.statusBar().showMessage(
			self.tr("Created reaction {0}. {1} member roots are selected.").format(
				created.reaction_id, len(member_ids),
			), 5000,
		)
		self._window._refresh_actions()

	#============================================
	def _recover_accepted_presentation_failure(
			self, tab: object, result: object, member_ids: list[str],
			) -> None:
		"""Refresh after a Rust-accepted reaction could not install its first scene."""
		self.close()
		self._window._replace_render_interaction_selection(None, tab)
		recovered = tab.refresh_authoritative()
		if recovered:
			try:
				self._restore_selection(tab, member_ids)
			except (
				ferrum_qt.ferrum.engine.RenderInteractionError,
				ferrum_qt.ferrum.engine.RevisionConflictError,
			):
				self._window._replace_render_interaction_selection(None, tab)
				recovered = False
		self._window.statusBar().showMessage(
			self.tr("Reaction {0} was created, but the authoritative display needs recovery.").format(
				result.outcome.reaction_created.reaction_id,
			) if not recovered else self.tr(
				"Reaction {0} was created. Refresh and select the reaction members again.",
			).format(result.outcome.reaction_created.reaction_id), 5000,
		)
		self._window._refresh_actions()

	#============================================
	def _restore_selection(self, tab: object, identifiers: list[str]) -> None:
		"""Reacquire committed members using fresh Rust root queries only."""
		observation = tab.observe_direct_root_interaction()
		selection = None
		for identifier in identifiers:
			modifier = (
				ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
				if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
			)
			selection = tab.select_direct_roots(
				observation, selection,
				ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(identifier, modifier),
			)
		self._window._replace_render_interaction_selection(selection, tab)

	#============================================
	def _restart_after_typed_refusal(self) -> None:
		"""Retire only a Rust-declared invalid authoring observation."""
		tab = self._tab
		self.close()
		if tab is not None:
			self._window._replace_render_interaction_selection(None, tab)
		self._window.statusBar().showMessage(
			self.tr("Refresh and select the reaction members again."), 5000,
		)

	#============================================
	def _handle_refusal(self, exc: BaseException) -> None:
		"""Map frozen Rust categories to either restart or inline correction."""
		category = exc.category
		if category in {
			ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.stale_snapshot,
			ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.foreign_session,
			ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.replayed_gesture,
			ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.session_conflict,
			ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.missing_reaction,
			ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.membership_changed,
		}:
			self._restart_after_typed_refusal()
			return
		if self._panel is not None:
			if category in {
				ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.unrenderable_document,
				ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.render_preparation,
				ferrum_qt.ferrum.engine.ReactionRefusalCategoryV1.renderer_exclusion,
			}:
				self._panel.show_refusal(self.tr(
					"Choose renderable members. Rust rejected the current display preparation.",
				))
			else:
				self._panel.show_refusal(self.tr("Correct the selected reaction roles."))
