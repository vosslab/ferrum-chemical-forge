"""Ferrum QObject terminal lifecycle helpers."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import shiboken6


#============================================
def delete_qobject_and_wait(
		app: PySide6.QtWidgets.QApplication,
		target: PySide6.QtCore.QObject,
		max_passes: int = 4,
		) -> bool:
	"""Queue one QObject deletion and prove its destroyed signal was delivered."""
	if not shiboken6.isValid(target):
		raise RuntimeError("Cannot retire an already-retired QObject")
	destroyed = []

	#============================================
	def record_destroyed(*_args: object) -> None:
		"""Record either PySide6 destroyed-signal signature."""
		destroyed.append(True)

	target.destroyed.connect(record_destroyed)
	target.deleteLater()
	for _pass in range(max_passes):
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		app.processEvents()
		if destroyed:
			return True
	return False
