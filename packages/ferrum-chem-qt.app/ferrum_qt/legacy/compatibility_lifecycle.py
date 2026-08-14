"""Lifecycle capability retained by the migration-only session host."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


#============================================
def drain_pending_session_deletions(
		app: PySide6.QtWidgets.QApplication, target_window: object = None,
		max_passes: int = 4,
		) -> bool:
	"""Drain a caller-provided legacy session lifecycle capability."""
	if target_window is None:
		raise ValueError("A compatibility window is required to prove reaper completion")
	while target_window._retired_import_workers:
		loop = PySide6.QtCore.QEventLoop()
		target_window.worker_retirement_drained.connect(loop.quit)
		if target_window._retired_import_workers:
			loop.exec()
		try:
			target_window.worker_retirement_drained.disconnect(loop.quit)
		except (RuntimeError, TypeError):
			pass
	for _pass in range(max_passes):
		target_window._resolve_pending_session_graphics()
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		app.processEvents()
		if not target_window._pending_session_deletions:
			return True
	return False
