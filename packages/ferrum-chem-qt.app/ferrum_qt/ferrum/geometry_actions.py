"""Compose Ferrum root transforms and molecule repairs for the public window."""

# local repo modules
import ferrum_qt.ferrum.geometry_repair
import ferrum_qt.ferrum.top_level_transform


#============================================
class FerrumNativeGeometryActionsMixin(
		ferrum_qt.ferrum.top_level_transform.FerrumNativeTopLevelTransformMixin,
		ferrum_qt.ferrum.geometry_repair.FerrumNativeGeometryRepairWindowMixin,
		):
	"""Install and refresh independent Rust-owned geometry action families."""

	#============================================
	def _build_top_level_transform_actions(self, edit_menu: object) -> None:
		"""Build root transforms and the separate molecule Repair menu."""
		super()._build_top_level_transform_actions(edit_menu)
		self._build_geometry_repair_actions()

	#============================================
	def _refresh_top_level_transform_actions(
			self, tab: object, active: bool, pending: bool, busy: bool) -> None:
		"""Refresh both independent geometry action families."""
		super()._refresh_top_level_transform_actions(tab, active, pending, busy)
		self._refresh_geometry_repair_actions(tab, active, pending, busy)
