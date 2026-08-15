"""Typed ArrowDialog adaptation for the Rust-native Ferrum editor."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.arrow_dialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeArrowDialogModel:
	"""Normal non-spline Arrow values copied from one frozen Rust projection."""

	start_head: bool
	end_head: bool
	line_width: float
	color: str


#============================================
def dialog_model_from_projection(arrow: object) -> FerrumNativeArrowDialogModel:
	"""Copy only Arrow facts the current native renderer and form can preserve."""
	import ferrum_chem
	if type(arrow) is not ferrum_chem.ArrowProjectionV1:
		raise TypeError("native Arrow properties require an exact Ferrum Arrow projection")
	if type(arrow.start_head) is not bool or type(arrow.end_head) is not bool:
		raise TypeError("selected Rust Arrow head facts must be exact booleans")
	width = arrow.stroke.width
	if (
		type(width) is not float
		or width < 0.5
		or width > 20.0
		or not (width * 10.0).is_integer()
	):
		raise ValueError("selected Rust Arrow width is not representable by ArrowDialog")
	color = arrow.stroke.color
	if type(color) is not str:
		raise TypeError("selected Rust Arrow color must be a string")
	return FerrumNativeArrowDialogModel(
		arrow.start_head, arrow.end_head, width, color,
	)


#============================================
def property_changes_from_dialog(
		changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map only faithfully rendered ArrowDialog edits to frozen Rust changes."""
	import ferrum_chem
	if type(changes) is not tuple:
		raise TypeError("native Arrow property changes must be an exact tuple")
	converted: list[object] = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("native Arrow property changes must be exact field/value pairs")
		field, value = change
		if field == "start_head" and type(value) is bool:
			converted.append(ferrum_chem.DocumentArrowPropertyChangeV1.start_head(value))
		elif field == "end_head" and type(value) is bool:
			converted.append(ferrum_chem.DocumentArrowPropertyChangeV1.end_head(value))
		elif field == "line_width" and type(value) is float:
			converted.append(ferrum_chem.DocumentArrowPropertyChangeV1.line_width(value))
		elif field == "color" and type(value) is str:
			converted.append(ferrum_chem.DocumentArrowPropertyChangeV1.color(value))
		elif field == "spline":
			raise ValueError(
				"native ArrowDialog cannot edit spline state until spline rendering is available",
			)
		else:
			raise ValueError("native ArrowDialog supplied an unsupported property change")
	return tuple(converted)


#============================================
def install_arrow_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one native Arrow action without adding dialog logic to the host."""
	action = PySide6.QtGui.QAction(window.tr("Edit Arrow Properties"), window)
	action.setToolTip(window.tr(
		"Edit one selected normal Arrow through one Rust-native operation",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_arrow_properties(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_arrow_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Make the action follow exact current durable Arrow selection."""
	action.setEnabled(
		active and not pending and not busy and tab.has_one_selected_arrow(),
	)


#============================================
def _on_edit_arrow_properties(window: object) -> None:
	"""Run one detached normal-Arrow form and submit only changed fields."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		arrow = tab.selected_arrow_projection()
		model = dialog_model_from_projection(arrow)
	except Exception as exc:
		window._refresh_actions()
		window._show_native_file_warning("Native Arrow Properties Unavailable", str(exc))
		return
	dialog = ferrum_qt.dialogs.arrow_dialog.ArrowDialog(
		window,
		start_head=model.start_head,
		end_head=model.end_head,
		line_width=model.line_width,
		spline=False,
		color=model.color,
	)
	dialog.set_spline_editable(False, window.tr(
		"Spline editing is unavailable until the native spline renderer is complete",
	))
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = property_changes_from_dialog(dialog.changes())
		tab.apply_selected_arrow_properties(changes)
	except Exception as exc:
		window._refresh_actions()
		window._show_native_file_warning("Native Arrow Properties Error", str(exc))
		return
	window.statusBar().showMessage(window.tr("Updated one Rust-native Arrow."), 5000)
	window._refresh_actions()
