"""Typed BondDialog adaptation for the OASA-free Ferrum-native editor."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.bond_dialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeBondDialogModel:
	"""Scalar BondDialog inputs copied from one frozen Rust bond projection."""

	order: int
	type: str
	center: bool
	line_width: float
	bond_width: float
	wedge_width: float
	line_color: str


#============================================
def can_edit_selected_bond_properties(tab: object | None) -> bool:
	"""Return whether one current durable bond can open the native form."""
	if tab is None or tab.requires_refresh:
		return False
	try:
		return bool(tab.has_one_selected_bond())
	except (AttributeError, RuntimeError):
		return False


#============================================
def dialog_model_from_projection(bond: object) -> FerrumNativeBondDialogModel:
	"""Copy one exact Rust DTO into values faithfully representable by BondDialog.

	The dialog has no automatic/absent controls.  Display defaults are therefore
	used only for a comparison baseline: accepting without editing returns no
	changes and cannot author absent CDML facts.  A source fact that the visual
	form cannot show is rejected, rather than clamped or remapped.
	"""
	import ferrum_chem
	if type(bond) is not ferrum_chem.BondProjectionV1:
		raise TypeError("native bond properties require an exact Ferrum bond projection")
	order = _dialog_order(bond.order, ferrum_chem)
	bond_type = _dialog_style(bond.style, ferrum_chem)
	center = False if bond.center is None else bond.center
	line_width = _optional_width(bond.line_width, 2.0, 20.0, "line width")
	bond_width = _optional_width(bond.bond_width, 6.0, 40.0, "bond width")
	wedge_width = _optional_width(bond.wedge_width, 9.2, 40.0, "wedge width")
	line_color = "#000000" if bond.color is None else bond.color
	if type(center) is not bool:
		raise TypeError("selected Rust bond center fact must be a boolean")
	if type(line_color) is not str:
		raise TypeError("selected Rust bond color must be a string")
	return FerrumNativeBondDialogModel(
		order, bond_type, center, line_width, bond_width, wedge_width, line_color,
	)


#============================================
def property_changes_from_dialog(
		bond: object, changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map only visually supported BondDialog edits to frozen Rust changes."""
	import ferrum_chem
	if type(bond) is not ferrum_chem.BondProjectionV1:
		raise TypeError("native bond properties require an exact Ferrum bond projection")
	if type(changes) is not tuple:
		raise TypeError("native bond property changes must be an exact tuple")
	final_order = _dialog_order(bond.order, ferrum_chem)
	_dialog_style(bond.style, ferrum_chem)
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("native bond property changes must be exact field/value pairs")
		field, value = change
		if field == "order" and type(value) is int:
			if value not in (1, 2, 3):
				raise ValueError("native BondDialog supplied an unsupported property change")
			final_order = value
	_validate_render_capabilities(changes, final_order)
	converted: list[object] = []
	for change in changes:
		field, value = change
		converted.append(_property_change(field, value, ferrum_chem))
	return tuple(converted)


#============================================
def _validate_render_capabilities(
		changes: tuple[tuple[str, object], ...], final_order: int,
		) -> None:
	"""Reject form facts the current normal-bond renderer cannot show faithfully."""
	for field, value in changes:
		if field == "wedge_width":
			raise ValueError(
				"native BondDialog cannot apply wedge width until native wedge rendering is available",
			)
		if field == "center" and (type(value) is not bool or not value or final_order != 2):
			raise ValueError(
				"native BondDialog supports centering only for a normal double bond",
			)
		if field == "bond_width" and final_order not in (2, 3):
			raise ValueError(
				"native BondDialog supports bond width only for a normal double or triple bond",
			)


#============================================
def _dialog_order(value: object, ferrum_chem: object) -> int:
	"""Return an editable normal covalent order without synthesizing a fallback."""
	if type(value) is not ferrum_chem.DocumentBondOrderV1:
		raise ValueError("selected Rust bond order is not representable by BondDialog")
	if value is ferrum_chem.DocumentBondOrderV1.single:
		return 1
	if value is ferrum_chem.DocumentBondOrderV1.double:
		return 2
	if value is ferrum_chem.DocumentBondOrderV1.triple:
		return 3
	raise ValueError("selected Rust bond order is not representable by BondDialog")


#============================================
def _dialog_style(value: object, ferrum_chem: object) -> str:
	"""Allow only the style that the current native renderer can retain visibly."""
	if value is not ferrum_chem.DocumentBondStyleV1.normal:
		raise ValueError(
			"native BondDialog currently supports normal bond style only; "
			"the selected style has no native renderer support",
		)
	return "n"


#============================================
def _optional_width(value: object, default: float, maximum: float, label: str) -> float:
	"""Return a width only when the matching Qt spin box can preserve it exactly."""
	if value is None:
		return default
	if (
		type(value) is not float
		or value < 0.1
		or value > maximum
		or not (value * 10.0).is_integer()
	):
		raise ValueError(f"selected Rust bond {label} is not representable by BondDialog")
	return value


#============================================
def _property_change(field: object, value: object, ferrum_chem: object) -> object:
	"""Convert one closed BondDialog field without accepting legacy-shaped input."""
	if field == "order" and type(value) is int:
		orders = ferrum_chem.DocumentBondOrderV1
		if value == 1:
			return ferrum_chem.DocumentBondPropertyChangeV1.order(orders.single)
		if value == 2:
			return ferrum_chem.DocumentBondPropertyChangeV1.order(orders.double)
		if value == 3:
			return ferrum_chem.DocumentBondPropertyChangeV1.order(orders.triple)
	if field == "type" and type(value) is str:
		styles = ferrum_chem.DocumentBondStyleV1
		if value == "n":
			return ferrum_chem.DocumentBondPropertyChangeV1.style(styles.normal)
		raise ValueError(
			"native BondDialog currently supports normal bond style only; "
			"choose Normal before submitting",
		)
	if field == "center" and type(value) is bool:
		return ferrum_chem.DocumentBondPropertyChangeV1.center(value)
	if field == "line_width" and type(value) is float:
		return ferrum_chem.DocumentBondPropertyChangeV1.line_width(value)
	if field == "bond_width" and type(value) is float:
		return ferrum_chem.DocumentBondPropertyChangeV1.bond_width(value)
	if field == "wedge_width" and type(value) is float:
		return ferrum_chem.DocumentBondPropertyChangeV1.wedge_width(value)
	if field == "color" and type(value) is str:
		return ferrum_chem.DocumentBondPropertyChangeV1.color(value)
	raise ValueError("native BondDialog supplied an unsupported property change")


#============================================
def run_bond_properties_dialog(window: object) -> None:
	"""Run one visual bond form while the Rust session owns durable state."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		bond = tab.selected_bond_projection()
		model = dialog_model_from_projection(bond)
	except Exception as exc:
		_refresh_window_actions(window)
		window._show_native_file_warning("Native Bond Properties Unavailable", str(exc))
		return
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(
		model,
		window,
		ferrum_qt.dialogs.bond_dialog.NATIVE_RENDER_CAPABILITIES,
	)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = property_changes_from_dialog(bond, dialog.changes())
		tab.apply_selected_bond_properties(changes)
	except Exception as exc:
		_refresh_window_actions(window)
		window._show_native_file_warning("Native Bond Properties Error", str(exc))
		return
	window.statusBar().showMessage(window.tr("Updated one Rust-native bond."), 5000)
	_refresh_window_actions(window)


#============================================
def _refresh_window_actions(window: object) -> None:
	"""Refresh the action policy owned by either supported native host."""
	refresh_explicit = getattr(window, "_refresh_explicit_native_actions", None)
	if callable(refresh_explicit):
		refresh_explicit(window._active_native_tab())
		return
	window._refresh_actions()
