"""Rust-native direct-root Text editing through one closed Rust operation."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.rich_text_dialog


_STYLE_TO_DIALOG = {
	"subscript": "sub",
	"superscript": "sup",
}
_STYLE_TO_RUST = {
	"sub": "subscript",
	"sup": "superscript",
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeTextDialogModel:
	"""Rich Text values copied from one renderer-compatible Rust projection."""

	runs: tuple[tuple[str, tuple[str, ...]], ...]
	font_size: int
	color: str


#============================================
class FerrumNativeTextPropertiesMixin:
	"""Direct-root Text selection and mutation services for the native tab."""

	#============================================
	def has_one_selected_text(self) -> bool:
		"""Return whether current selection names one durable rendered Text."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "text"
			and selected[0].identifier is not None
		)

	#============================================
	def selected_text_projection(self) -> object:
		"""Return one selected frozen Rust Text projection for a native dialog."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "text")[0]
		if self._document_observation is None:
			raise RuntimeError("native tab has no installed document projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind == "text" and root.text.target.id == selected:
				if root.text.target.source_id is None:
					raise RuntimeError("selected Text has no durable authored source identifier")
				return root.text
		raise RuntimeError("selected Text is absent from the Rust projection")

	#============================================
	def apply_selected_text_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust Text patch while retaining durable selection."""
		if type(changes) is not tuple:
			raise TypeError("native Text properties require an exact change tuple")
		import ferrum_chem
		if any(type(change) is not ferrum_chem.DocumentTextPropertyChangeV1
				for change in changes):
			raise TypeError("native Text properties require exact frozen Ferrum changes")
		text = self.selected_text_projection()
		operation = ferrum_chem.DocumentOperationV1.set_text_properties(
			text.target.source_id, changes,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("text", text.target.id),))
		return result


#============================================
def dialog_model_from_projection(text: object) -> FerrumNativeTextDialogModel:
	"""Copy only Text facts the constrained Qt form represents without coercion."""
	import ferrum_chem
	if type(text) is not ferrum_chem.TextProjectionV1:
		raise TypeError("native Text properties require an exact Ferrum Text projection")
	if text.font.family is not None:
		raise ValueError(
			"selected Text uses a font family that the verified native renderer cannot preserve",
		)
	size = text.font.size
	if type(size) is not float or not size.is_integer() or not 4 <= size <= 144:
		raise ValueError("selected Rust Text font size is not representable by this dialog")
	if type(text.font.color) is not str:
		raise TypeError("selected Rust Text color must be a string")
	if type(text.runs) is not tuple or not text.runs:
		raise TypeError("selected Rust Text requires frozen nonempty runs")
	runs = []
	for run in text.runs:
		if type(run) is not ferrum_chem.PresentationTextRunV1:
			raise TypeError("selected Rust Text contains the wrong run DTO")
		if type(run.text) is not str or type(run.styles) is not tuple:
			raise TypeError("selected Rust Text run is not immutable plain data")
		if any(style not in _STYLE_TO_DIALOG for style in run.styles):
			raise ValueError(
				"selected Text uses bold or italic formatting without a verified native face",
			)
		runs.append((run.text, tuple(_STYLE_TO_DIALOG[style] for style in run.styles)))
	return FerrumNativeTextDialogModel(tuple(runs), int(size), text.font.color)


#============================================
def property_changes_from_dialog(
		model: FerrumNativeTextDialogModel,
		runs: tuple[tuple[str, tuple[str, ...]], ...],
		font_size: int, color: str,
		) -> tuple[object, ...]:
	"""Map changed representable dialog facts to frozen Rust property values."""
	if type(model) is not FerrumNativeTextDialogModel:
		raise TypeError("native Text changes require the closed dialog model")
	if type(runs) is not tuple:
		raise TypeError("native Text runs must be an exact tuple")
	if type(font_size) is not int or not 4 <= font_size <= 144:
		raise ValueError("native Text font size must be an integer from 4 through 144")
	if type(color) is not str:
		raise TypeError("native Text color must be a string")
	import ferrum_chem
	change_type = ferrum_chem.DocumentTextPropertyChangeV1
	changes = []
	if runs != model.runs:
		converted_runs = []
		for run in runs:
			if type(run) is not tuple or len(run) != 2:
				raise TypeError("native Text runs must contain exact text/style pairs")
			text, styles = run
			if type(text) is not str or type(styles) is not tuple:
				raise TypeError("native Text runs must contain plain immutable values")
			converted_styles = []
			for style in styles:
				style_name = _STYLE_TO_RUST.get(style)
				if style_name is None:
					raise ValueError("native Text editing supports baseline, subscript, and superscript")
				converted_styles.append(getattr(ferrum_chem.DocumentTextEditStyleV1, style_name))
			converted_runs.append(
				ferrum_chem.DocumentTextEditRunV1.create(text, tuple(converted_styles)),
			)
		changes.append(change_type.runs(tuple(converted_runs)))
	if font_size != model.font_size:
		changes.append(change_type.font_size(font_size))
	if color.lower() != model.color.lower():
		changes.append(change_type.color(color))
	return tuple(changes)


#============================================
def install_text_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one native Text action without adding dialog logic to the host."""
	action = PySide6.QtGui.QAction(window.tr("Edit Text Properties"), window)
	action.setToolTip(window.tr(
		"Edit one selected durable Text through one Rust-native operation",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_text_properties(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_text_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Make the action follow exact current durable Text selection."""
	action.setEnabled(
		tab is not None and active and not pending and not busy
		and tab.has_one_selected_text(),
	)


#============================================
def _on_edit_text_properties(window: object) -> None:
	"""Run one constrained form and submit only its explicit changed facts."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		model = dialog_model_from_projection(tab.selected_text_projection())
	except Exception as exc:
		window._refresh_actions()
		window._show_native_file_warning("Native Text Properties Unavailable", str(exc))
		return
	capabilities = ferrum_qt.dialogs.rich_text_dialog.RichTextDialogCapabilities(
		bold=False,
		italic=False,
		font_family=False,
		disabled_reason=(
			"The verified native renderer currently uses its regular Telex face only."
		),
	)
	dialog = ferrum_qt.dialogs.rich_text_dialog.RichTextDialog(
		model.runs, "Telex", model.font_size, model.color, window,
		capabilities=capabilities,
	)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		font = dialog.font_values()
		changes = property_changes_from_dialog(
			model, dialog.get_runs(), font["font_size"], font["font_color"],
		)
		if changes:
			tab.apply_selected_text_properties(changes)
	except Exception as exc:
		window._refresh_actions()
		window._show_native_file_warning("Native Text Properties Error", str(exc))
		return
	window.statusBar().showMessage(window.tr("Updated one Rust-native Text."), 5000)
	window._refresh_actions()
