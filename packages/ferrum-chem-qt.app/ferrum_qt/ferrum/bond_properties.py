"""Typed BondDialog adaptation for the Ferrum editor."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.bond_dialog


_PRESENTATION_DETAILS = {
	"normal_single": (1, "n"),
	"normal_double": (2, "n"),
	"normal_triple": (3, "n"),
	"solid_wedge": (1, "w"),
	"hashed_wedge": (1, "h"),
	"haworth_front": (1, "q"),
	"bold": (1, "b"),
	"dashed": (1, "d"),
	"wavy": (1, "s"),
}


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
	"""Return whether one current durable bond can open the Ferrum form."""
	if tab is None or tab.requires_refresh:
		return False
	try:
		return bool(tab.has_one_selected_bond())
	except (AttributeError, RuntimeError):
		return False


#============================================
def dialog_model_from_projection(bond: object) -> FerrumNativeBondDialogModel:
	"""Copy one exact Rust DTO into values faithfully representable by BondDialog.

	Rust owns the closed presentation: Qt derives its order/style controls exactly
	once for display and never accepts split transport facts.
	"""
	import ferrum_qt.ferrum.engine as engine
	if type(bond) is not engine.BondProjectionV1:
		raise TypeError("Ferrum bond properties require an exact Ferrum bond projection")
	order, bond_type = _dialog_presentation(bond.presentation, engine)
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
	"""Map dialog edits to closed Rust property changes without split style/order."""
	import ferrum_qt.ferrum.engine as engine
	if type(bond) is not engine.BondProjectionV1:
		raise TypeError("Ferrum bond properties require an exact Ferrum bond projection")
	if type(changes) is not tuple:
		raise TypeError("Ferrum bond property changes must be an exact tuple")
	final_presentation = _dialog_presentation(bond.presentation, engine)
	presentation_changed = False
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("Ferrum bond property changes must be exact field/value pairs")
		field, value = change
		if field == "presentation":
			final_presentation = _validate_dialog_presentation(value)
			presentation_changed = True
	effective_changes = changes + _inapplicable_field_clears(
		bond, changes, final_presentation, presentation_changed,
	)
	_validate_render_capabilities(effective_changes, final_presentation)
	return tuple(_property_change(change, engine) for change in effective_changes)


#============================================
def _dialog_presentation(value: object, extension: object) -> tuple[int, str]:
	"""Return one display pair from the one Rust-owned closed presentation fact."""
	if type(value) is not extension.DocumentBondPresentationV1:
		raise ValueError("selected Rust bond presentation is not representable by BondDialog")
	name = getattr(value, "name", None)
	if type(name) is not str or name not in _PRESENTATION_DETAILS:
		raise ValueError("selected Rust bond presentation is not representable by BondDialog")
	return _PRESENTATION_DETAILS[name]


#============================================
def _validate_dialog_presentation(value: object) -> tuple[int, str]:
	"""Validate the sole editable presentation field before converting it once."""
	if (
		type(value) is not tuple or len(value) != 2
		or type(value[0]) is not int or type(value[1]) is not str
	):
		raise ValueError("Ferrum BondDialog supplied an invalid bond presentation")
	order, bond_type = value
	if bond_type == "n" and order in (1, 2, 3):
		return order, bond_type
	if bond_type in ("w", "h", "q", "b", "d", "s") and order == 1:
		return order, bond_type
	raise ValueError("Ferrum BondDialog supplied an unsupported bond presentation")


#============================================
def _validate_render_capabilities(
		changes: tuple[tuple[str, object], ...], presentation: tuple[int, str],
		) -> None:
	"""Admit scalar changes only where the selected presentation uses them."""
	order, bond_type = presentation
	for field, value in changes:
		if field == "wedge_width" and bond_type not in ("w", "h"):
			if value is not None:
				raise ValueError("Choose a solid or hashed wedge before editing its wedge width.")
		if field == "center" and value is not None and (
			type(value) is not bool or bond_type != "n" or order != 2
	):
			raise ValueError("Ferrum BondDialog supports centering only for a normal double bond")
		if field == "bond_width" and value is not None and (
			bond_type != "n" or order not in (2, 3)
		):
			raise ValueError(
				"Ferrum BondDialog supports bond width only for a normal double or triple bond",
			)


#============================================
def _inapplicable_field_clears(
		bond: object, changes: tuple[tuple[str, object], ...],
		presentation: tuple[int, str], presentation_changed: bool,
		) -> tuple[tuple[str, None], ...]:
	"""Clear authored fields that the replacement presentation cannot retain.

	The Rust document preserves authored optional facts unless an explicit patch
	clears them.  A presentation replacement therefore owns the matching cleanup
	in the adapter, rather than relying on disabled controls or normalization.
	"""
	if not presentation_changed:
		return ()
	order, bond_type = presentation
	requested_fields = {field for field, _value in changes}
	clears = []
	if bond.center is not None and (bond_type != "n" or order != 2):
		if "center" not in requested_fields:
			clears.append(("center", None))
	if bond.bond_width is not None and (bond_type != "n" or order not in (2, 3)):
		if "bond_width" not in requested_fields:
			clears.append(("bond_width", None))
	if bond.wedge_width is not None and bond_type not in ("w", "h"):
		if "wedge_width" not in requested_fields:
			clears.append(("wedge_width", None))
	return tuple(clears)


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
def _property_change(change: tuple[str, object], extension: object) -> object:
	"""Convert one closed dialog fact without accepting legacy-shaped input."""
	field, value = change
	if field == "presentation":
		order, bond_type = _validate_dialog_presentation(value)
		name = next(
			name for name, detail in _PRESENTATION_DETAILS.items()
			if detail == (order, bond_type)
		)
		return extension.DocumentBondPropertyChangeV1.presentation(
			getattr(extension.DocumentBondPresentationV1, name),
		)
	if field == "center" and (value is None or type(value) is bool):
		return extension.DocumentBondPropertyChangeV1.center(value)
	if field == "line_width" and (value is None or type(value) is float):
		return extension.DocumentBondPropertyChangeV1.line_width(value)
	if field == "bond_width" and (value is None or type(value) is float):
		return extension.DocumentBondPropertyChangeV1.bond_width(value)
	if field == "wedge_width" and (value is None or type(value) is float):
		return extension.DocumentBondPropertyChangeV1.wedge_width(value)
	if field == "color" and (value is None or type(value) is str):
		return extension.DocumentBondPropertyChangeV1.color(value)
	raise ValueError("Ferrum BondDialog supplied an unsupported property change")


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
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	dialog = ferrum_qt.dialogs.bond_dialog.BondDialog(model, window)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = property_changes_from_dialog(bond, dialog.changes())
		if changes:
			tab.apply_selected_bond_properties(changes)
	except Exception as exc:
		_refresh_window_actions(window)
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Updated one bond."), 5000)
	_refresh_window_actions(window)


#============================================
def _refresh_window_actions(window: object) -> None:
	"""Refresh the action policy owned by either supported Ferrum host."""
	refresh_explicit = getattr(window, "_refresh_explicit_native_actions", None)
	if callable(refresh_explicit):
		refresh_explicit(window._active_native_tab())
		return
	window._refresh_actions()
