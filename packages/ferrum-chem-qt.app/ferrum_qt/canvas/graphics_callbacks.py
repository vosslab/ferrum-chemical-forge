"""Focused callback cleanup for disposable Qt graphics items."""

# PIP3 modules
import PySide6.QtWidgets


#============================================
def dispose_item_callbacks(item: PySide6.QtWidgets.QGraphicsItem) -> None:
	"""Release one item's projection and native callbacks before retirement."""
	first_error = None
	binding = getattr(item, "_projection_binding", None)
	if binding is not None:
		try:
			binding.dispose()
		except Exception as exc:
			first_error = exc
		finally:
			try:
				item._projection_binding = None
			except Exception as exc:
				if first_error is None:
					first_error = exc
	try:
		dispose = getattr(item, "dispose", None)
		if callable(dispose):
			dispose()
	except Exception as exc:
		if first_error is None:
			first_error = exc
	if first_error is not None:
		raise first_error
