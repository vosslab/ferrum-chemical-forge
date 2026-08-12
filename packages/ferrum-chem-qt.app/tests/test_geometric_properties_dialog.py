"""Behavior tests for detached geometric appearance intent."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.geometric_properties_dialog


#============================================
def test_fillable_dialog_returns_only_changed_plain_geometry_values(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A dialog keeps provisional geometry local and returns accepted intent."""
	dialog = ferrum_qt.dialogs.geometric_properties_dialog.GeometricPropertiesDialog(
		"Rectangle", 1.0, "#112233", "#AABBCC", True,
	)
	width = dialog.findChild(PySide6.QtWidgets.QDoubleSpinBox)
	fill = dialog.findChild(PySide6.QtWidgets.QCheckBox)
	if width is None or fill is None:
		raise AssertionError("Geometric dialog did not provide editable controls")
	width.setValue(2.5)
	fill.setChecked(False)

	assert dialog.changes() == (("line_width", 2.5), ("area_color", None))


#============================================
def test_line_only_dialog_omits_fill_intent(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A line-only editor exposes no shape-fill operation to its caller."""
	dialog = ferrum_qt.dialogs.geometric_properties_dialog.GeometricPropertiesDialog(
		"Polyline", 1.0, "#112233", None, False,
	)

	assert "area_color" not in dialog.get_values()
