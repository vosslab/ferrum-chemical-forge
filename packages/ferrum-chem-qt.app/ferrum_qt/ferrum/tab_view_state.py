"""Read-only display seams owned by one Ferrum document tab."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


#============================================
class FerrumNativeTabViewStateMixin:
	"""Expose projection-local view facts without publishing projection ownership."""

	#============================================
	@property
	def view(self) -> PySide6.QtWidgets.QGraphicsView:
		"""Return the tab-owned graphics view for projection-local interaction."""
		return self._view

	#============================================
	def document_content_bounds(self) -> PySide6.QtCore.QRectF | None:
		"""Return finite positive bounds of installed document roots, excluding paper."""
		projection = self._controller.projection
		if projection is None:
			return None
		bounds: PySide6.QtCore.QRectF | None = None
		for root in projection.roots:
			if root is projection.paper:
				continue
			root_bounds = PySide6.QtCore.QRectF(root.sceneBoundingRect())
			values = (
				float(root_bounds.left()), float(root_bounds.top()),
				float(root_bounds.right()), float(root_bounds.bottom()),
			)
			if not all(math.isfinite(value) for value in values):
				return None
			bounds = root_bounds if bounds is None else bounds.united(root_bounds)
		if bounds is None:
			return None
		values = (
			float(bounds.x()), float(bounds.y()),
			float(bounds.width()), float(bounds.height()),
		)
		if not all(math.isfinite(value) for value in values):
			return None
		if bounds.width() <= 0.0 or bounds.height() <= 0.0:
			return None
		return PySide6.QtCore.QRectF(bounds)
