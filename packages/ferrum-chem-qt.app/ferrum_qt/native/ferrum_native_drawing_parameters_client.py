"""Shared Qt clients for application-owned next-drawing preferences."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_drawing_parameters


#============================================
class FerrumNativeDrawingParametersClient(PySide6.QtWidgets.QWidget):
	"""Edit the one shared next-drawing model without owning document state."""

	#============================================
	def __init__(self,
			drawing_parameters: (
				ferrum_qt.native.ferrum_native_drawing_parameters.
				FerrumNativeDrawingParameters
			), cancel_action: PySide6.QtGui.QAction | None = None, *,
			compact: bool = False,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Project the shared choices through a compact accessible form."""
		super().__init__(parent)
		self._drawing_parameters = drawing_parameters
		self._cancel_action = cancel_action
		self.setAccessibleName(self.tr("Next Drawing"))
		self.setAccessibleDescription(self.tr(
			"Choose the element, order, and presentation for the next native drawing operation.",
		))
		layout: PySide6.QtWidgets.QBoxLayout | PySide6.QtWidgets.QFormLayout
		layout = (
			PySide6.QtWidgets.QFormLayout(self)
			if compact else PySide6.QtWidgets.QHBoxLayout(self)
		)
		layout.setContentsMargins(2, 0, 2, 0)
		layout.setSpacing(3)
		element_label = PySide6.QtWidgets.QLabel(self.tr("Next atom:"), self)
		self.element_combo = PySide6.QtWidgets.QComboBox(self)
		self.element_combo.setEditable(True)
		self.element_combo.addItems(
			ferrum_qt.native.ferrum_native_drawing_parameters.common_elements(),
		)
		element_label.setBuddy(self.element_combo)
		self.element_combo.setAccessibleName(self.tr("Next atom"))
		self.element_combo.setAccessibleDescription(self.tr(
			"Element used by Add Atom and by a Draw Bond drag ending in empty space. "
			"Enter letters for a valid atom or pseudo-atom name.",
		))
		self._element_editor = self.element_combo.lineEdit()
		validator = PySide6.QtGui.QRegularExpressionValidator(
			PySide6.QtCore.QRegularExpression("[A-Za-z]+"), self.element_combo,
		)
		self._element_editor.setValidator(validator)
		self._element_editor.installEventFilter(self)
		application = PySide6.QtWidgets.QApplication.instance()
		if application is not None:
			application.installEventFilter(self)
		self._element_editor.editingFinished.connect(self._commit_element)
		order_label = PySide6.QtWidgets.QLabel(self.tr("Next bond:"), self)
		self.order_combo = PySide6.QtWidgets.QComboBox(self)
		for label, order_name in (
			(self.tr("Single"), "single"),
			(self.tr("Double"), "double"),
			(self.tr("Triple"), "triple"),
		):
			self.order_combo.addItem(label, order_name)
		order_label.setBuddy(self.order_combo)
		self.order_combo.setAccessibleName(self.tr("Next bond"))
		self.order_combo.currentIndexChanged.connect(self._commit_order)
		presentation_label = PySide6.QtWidgets.QLabel(self.tr("Next presentation:"), self)
		self.presentation_combo = PySide6.QtWidgets.QComboBox(self)
		for label, presentation_name in (
			(self.tr("Normal"), "normal"),
			(self.tr("Solid wedge from start atom"), "solid_wedge"),
			(self.tr("Hashed wedge from start atom"), "hashed_wedge"),
		):
			self.presentation_combo.addItem(label, presentation_name)
		presentation_label.setBuddy(self.presentation_combo)
		self.presentation_combo.setAccessibleName(self.tr("Next presentation"))
		self.presentation_combo.setAccessibleDescription(self.tr(
			"Choose Normal, a solid wedge, or a hashed wedge. Directed wedges run from "
			"the gesture start atom at the tip to its end atom at the wide base.",
		))
		self.presentation_combo.currentIndexChanged.connect(self._commit_presentation)
		if compact:
			layout.addRow(element_label, self.element_combo)
			layout.addRow(order_label, self.order_combo)
			layout.addRow(presentation_label, self.presentation_combo)
		else:
			layout.addWidget(element_label)
			layout.addWidget(self.element_combo)
			layout.addWidget(order_label)
			layout.addWidget(self.order_combo)
			layout.addWidget(presentation_label)
			layout.addWidget(self.presentation_combo)
		self._drawing_parameters.changed.connect(self._project_parameters)
		self._project_parameters()

	#============================================
	def _project_parameters(self) -> None:
		"""Reflect a shared choice without feeding it back as a new edit."""
		snapshot = self._drawing_parameters.snapshot()
		element_blocker = PySide6.QtCore.QSignalBlocker(self.element_combo)
		order_blocker = PySide6.QtCore.QSignalBlocker(self.order_combo)
		presentation_blocker = PySide6.QtCore.QSignalBlocker(self.presentation_combo)
		self.element_combo.setCurrentText(snapshot.element)
		self._element_editor.setText(snapshot.element)
		self.order_combo.setCurrentIndex(self.order_combo.findData(snapshot.order_name))
		self.presentation_combo.setCurrentIndex(
			self.presentation_combo.findData(snapshot.presentation_name),
		)
		self.order_combo.setEnabled(snapshot.presentation_name == "normal")
		self.order_combo.setAccessibleDescription(self.tr(
			"Bond order used by a normal Draw Bond gesture. Directed wedges use Single.",
		))
		del element_blocker
		del order_blocker
		del presentation_blocker

	#============================================
	def _commit_element(self) -> None:
		"""Keep the last valid atom spelling active after a completed edit."""
		if self._drawing_parameters.set_element(self.element_combo.currentText()):
			self._restore_element()
			return
		self.element_combo.setToolTip(self.tr(
			"Use letters for an atom or pseudo-atom name; the previous valid value remains active.",
		))
		self._element_editor.setAccessibleDescription(self.tr(
			"Use letters for an atom or pseudo-atom name; the previous valid value remains active.",
		))
		self._restore_element(clear_feedback=False)

	#============================================
	def _restore_element(self, clear_feedback: bool = True) -> None:
		"""Show the last effective atom spelling after local editor recovery."""
		element = self._drawing_parameters.snapshot().element
		self.element_combo.setCurrentText(element)
		self._element_editor.setText(element)
		if clear_feedback:
			self.element_combo.setToolTip("")
			self._element_editor.setAccessibleDescription("")

	#============================================
	def _commit_order(self, index: int) -> None:
		"""Store only a selected closed bond-order preference."""
		if self._drawing_parameters.set_order_name(self.order_combo.itemData(index)):
			return
		snapshot = self._drawing_parameters.snapshot()
		self.order_combo.setCurrentIndex(self.order_combo.findData(snapshot.order_name))

	#============================================
	def _commit_presentation(self, index: int) -> None:
		"""Store one closed next-bond depiction without touching a document."""
		if self._drawing_parameters.set_presentation_name(
				self.presentation_combo.itemData(index)):
			return
		snapshot = self._drawing_parameters.snapshot()
		self.presentation_combo.setCurrentIndex(
			self.presentation_combo.findData(snapshot.presentation_name),
		)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Restore focused text before the shared Escape cancellation reaches the host."""
		if not isinstance(event, PySide6.QtGui.QKeyEvent):
			return False
		if (
			event.type() in (PySide6.QtCore.QEvent.Type.ShortcutOverride,
				PySide6.QtCore.QEvent.Type.KeyPress)
			and event.key() == PySide6.QtCore.Qt.Key.Key_Escape
			and self._element_editor.hasFocus()
		):
			self._restore_element()
			if self._cancel_action is not None and self._cancel_action.isEnabled():
				PySide6.QtCore.QTimer.singleShot(0, self._cancel_action.trigger)
			return True
		return super().eventFilter(watched, event)


#============================================
class FerrumNativeDrawingParametersDialog(PySide6.QtWidgets.QDialog):
	"""Provide the menu and toolbar action client for compact native windows."""

	#============================================
	def __init__(self,
			drawing_parameters: (
				ferrum_qt.native.ferrum_native_drawing_parameters.
				FerrumNativeDrawingParameters
			), cancel_action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build a compact model-view dialog with no document ownership."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Next Drawing"))
		self.setAccessibleName(self.tr("Next Drawing"))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		self.client = FerrumNativeDrawingParametersClient(
			drawing_parameters, cancel_action, compact=True, parent=self,
		)
		layout.addWidget(self.client)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close, self,
		)
		buttons.rejected.connect(self.reject)
		buttons.accepted.connect(self.accept)
		layout.addWidget(buttons)


#============================================
def show_native_drawing_parameters_dialog(
		window: PySide6.QtWidgets.QMainWindow,
		drawing_parameters: (
			ferrum_qt.native.ferrum_native_drawing_parameters.
			FerrumNativeDrawingParameters
		), cancel_action: PySide6.QtGui.QAction) -> None:
	"""Show the compact standard-action client for shared drawing preferences."""
	dialog = FerrumNativeDrawingParametersDialog(
		drawing_parameters, cancel_action, window,
	)
	dialog.exec()
