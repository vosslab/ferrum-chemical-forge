"""Typed PlusDialog adaptation for the Ferrum editor."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.plus_dialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativePlusDialogModel:
	"""Scalar PlusDialog values copied from one frozen Rust projection."""

	font_size: int
	color: str


#============================================
def dialog_model_from_projection(plus: object) -> FerrumNativePlusDialogModel:
	"""Copy only Plus facts the integer Qt form can represent without coercion."""
	import ferrum_qt.ferrum.engine as engine
	if type(plus) is not engine.PlusProjectionV1:
		raise TypeError("Ferrum Plus properties require an exact Ferrum Plus projection")
	size = plus.font.size
	if type(size) is not float or not size.is_integer() or not 4 <= size <= 144:
		raise ValueError("selected Rust Plus font size is not representable by PlusDialog")
	color = plus.font.color
	if type(color) is not str:
		raise TypeError("selected Rust Plus color must be a string")
	return FerrumNativePlusDialogModel(int(size), color)


#============================================
def property_changes_from_dialog(
		changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map only PlusDialog's explicit edits to frozen Rust change values."""
	import ferrum_qt.ferrum.engine as engine
	if type(changes) is not tuple:
		raise TypeError("Ferrum Plus property changes must be an exact tuple")
	converted = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("Ferrum Plus property changes must be exact field/value pairs")
		field, value = change
		if field == "font_size" and type(value) is int:
			converted.append(engine.DocumentPlusPropertyChangeV1.font_size(value))
		elif field == "color" and type(value) is str:
			converted.append(engine.DocumentPlusPropertyChangeV1.color(value))
		else:
			raise ValueError("Ferrum PlusDialog supplied an unsupported property change")
	return tuple(converted)


#============================================
def install_plus_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one Ferrum Plus action without adding dialog logic to the host."""
	action = PySide6.QtGui.QAction(window.tr("Edit Plus Properties"), window)
	action.setToolTip(window.tr(
		"Edit one selected durable Plus through one operation",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_plus_properties(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_plus_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Make the action follow exact current durable Plus selection."""
	action.setEnabled(
		active and not pending and not busy and tab.has_one_selected_plus(),
	)


#============================================
def _on_edit_plus_properties(window: object) -> None:
	"""Run one detached visual form and submit only its closed changed fields."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		plus = tab.selected_plus_projection()
		model = dialog_model_from_projection(plus)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	dialog = ferrum_qt.dialogs.plus_dialog.PlusDialog(
		model.font_size, model.color, window,
	)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = property_changes_from_dialog(dialog.changes())
		tab.apply_selected_plus_properties(changes)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Updated one Ferrum Plus."), 5000)
	window._refresh_actions()
