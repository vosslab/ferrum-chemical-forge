"""Atomic OASA-free export from one detached Rust render observation."""

# Standard Library
import enum
import math
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtSvg
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection


_POINTS_PER_INCH = 72
_RGBA_BYTES_PER_PIXEL = 4
_MAX_QT_DIMENSION = 2**31 - 1


#============================================
class FerrumNativeSnapshotExportError(RuntimeError):
	"""One current Rust observation could not be published as an image."""


#============================================
class FerrumNativeSnapshotFormat(enum.Enum):
	"""Closed snapshot formats supported by the standalone native route."""

	SVG = "svg"
	PDF = "pdf"
	PNG = "png"


#============================================
class FerrumNativeSnapshotExportTabMixin:
	"""Build an unselected detached scene from one current Rust observation."""

	#============================================
	def build_snapshot_export_projection(
			self,
			) -> ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjection:
		"""Observe and validate the current revision without reusing screen items."""
		self._require_live()
		self._require_current_projection()
		snapshot = self.current_snapshot
		observation = self._session.observe_render(snapshot.revision)
		observed = observation.document.snapshot
		if observed.revision != snapshot.revision or observed.digest != snapshot.digest:
			raise FerrumNativeSnapshotExportError(
				"snapshot observation differs from the current Rust document",
			)
		import ferrum_chem
		projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
			observation, ferrum_chem.verified_telex_regular(),
		)
		if projection.revision != snapshot.revision or projection.digest != snapshot.digest:
			projection.dispose()
			raise FerrumNativeSnapshotExportError(
				"detached export projection differs from the current Rust document",
			)
		return projection


#============================================
class FerrumNativeSnapshotExportWindowMixin:
	"""Expose explicit vector and bounded raster snapshot actions."""

	#============================================
	def _build_snapshot_export_actions(self, file_menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add one closed export submenu to the standalone native window."""
		menu = file_menu.addMenu(self.tr("Export Rust Snapshot"))
		self._snapshot_export_actions = {}
		for export_format, label in (
			(FerrumNativeSnapshotFormat.SVG, "Export SVG..."),
			(FerrumNativeSnapshotFormat.PDF, "Export PDF..."),
			(FerrumNativeSnapshotFormat.PNG, "Export PNG at 72 DPI..."),
		):
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.triggered.connect(
				lambda _checked=False, selected=export_format:
				self._choose_snapshot_export(selected),
			)
			menu.addAction(action)
			self._snapshot_export_actions[export_format] = action

	#============================================
	def _refresh_snapshot_export_actions(
			self, active: bool, pending: bool, busy: bool) -> None:
		"""Disable snapshot export whenever the displayed authority is not current."""
		available = active and not pending and not busy
		for action in self._snapshot_export_actions.values():
			action.setEnabled(available)

	#============================================
	def _choose_snapshot_export(self, export_format: FerrumNativeSnapshotFormat) -> None:
		"""Choose one destination and publish the current detached observation."""
		filters = {
			FerrumNativeSnapshotFormat.SVG: self.tr("Scalable Vector Graphics (*.svg)"),
			FerrumNativeSnapshotFormat.PDF: self.tr("Portable Document Format (*.pdf)"),
			FerrumNativeSnapshotFormat.PNG: self.tr("Portable Network Graphics (*.png)"),
		}
		path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export Rust Snapshot"), "", filters[export_format],
		)[0]
		if path:
			self.export_active_snapshot(path, export_format)

	#============================================
	def export_active_snapshot(
			self, path: str, export_format: FerrumNativeSnapshotFormat) -> bool:
		"""Publish one exact active snapshot or report an actionable failure."""
		if type(path) is not str or type(export_format) is not FerrumNativeSnapshotFormat:
			raise TypeError("native snapshot export requires an exact path and format")
		tab = self._active_native_tab()
		if tab is None:
			return False
		try:
			projection = tab.build_snapshot_export_projection()
			export_snapshot_projection(projection, pathlib.Path(path), export_format)
		except Exception as error:
			self._show_native_file_warning("Rust Snapshot Export Failed", str(error))
			return False
		self.statusBar().showMessage(
			self.tr("Exported the current Rust snapshot to {0}").format(path), 5000,
		)
		return True


#============================================
def export_snapshot_projection(
		projection: ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjection,
		path: pathlib.Path, export_format: FerrumNativeSnapshotFormat) -> None:
	"""Consume one detached projection and atomically publish its exact format."""
	if type(projection) is not ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjection:
		raise TypeError("snapshot export requires an exact Ferrum render projection")
	try:
		_validate_export_request(path, export_format)
		rectangle = _scene_rectangle(projection)
		if export_format is FerrumNativeSnapshotFormat.SVG:
			_write_svg(projection.scene, rectangle, path)
		elif export_format is FerrumNativeSnapshotFormat.PDF:
			_write_pdf(projection.scene, rectangle, path)
		else:
			_write_png(projection.scene, rectangle, path)
	finally:
		projection.dispose()


#============================================
def _validate_export_request(
		path: pathlib.Path, export_format: FerrumNativeSnapshotFormat) -> None:
	"""Reject ambiguous formats and unsafe existing destinations before painting."""
	if not isinstance(path, pathlib.Path):
		raise TypeError("snapshot export path must be a pathlib.Path")
	if type(export_format) is not FerrumNativeSnapshotFormat:
		raise TypeError("snapshot export requires an exact closed format")
	if path.suffix.lower() != f".{export_format.value}":
		raise FerrumNativeSnapshotExportError(
			f"snapshot destination must end in .{export_format.value}",
		)
	if path.is_symlink():
		raise FerrumNativeSnapshotExportError(
			"snapshot destination must not be a symbolic link",
		)
	if path.exists() and not path.is_file():
		raise FerrumNativeSnapshotExportError(
			"snapshot destination must be a regular file",
		)


#============================================
def _scene_rectangle(
		projection: ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjection,
		) -> PySide6.QtCore.QRectF:
	"""Return one positive finite backend-issued paper rectangle."""
	rectangle = PySide6.QtCore.QRectF(projection.scene.sceneRect())
	values = (
		float(rectangle.x()), float(rectangle.y()),
		float(rectangle.width()), float(rectangle.height()),
	)
	if not all(math.isfinite(value) for value in values):
		raise FerrumNativeSnapshotExportError("snapshot paper rectangle is nonfinite")
	if rectangle.width() <= 0.0 or rectangle.height() <= 0.0:
		raise FerrumNativeSnapshotExportError("snapshot paper rectangle must be positive")
	return rectangle


#============================================
def _open_atomic_device(path: pathlib.Path) -> PySide6.QtCore.QSaveFile:
	"""Open one Qt atomic publisher without following an existing target symlink."""
	device = PySide6.QtCore.QSaveFile(str(path))
	if not device.open(PySide6.QtCore.QIODevice.OpenModeFlag.WriteOnly):
		raise FerrumNativeSnapshotExportError(device.errorString())
	return device


#============================================
def _commit_atomic_device(device: PySide6.QtCore.QSaveFile) -> None:
	"""Commit one fully painted temporary file or surface its exact Qt error."""
	if not device.commit():
		raise FerrumNativeSnapshotExportError(device.errorString())


#============================================
def _paint_scene(scene: PySide6.QtWidgets.QGraphicsScene,
		paint_device: PySide6.QtGui.QPaintDevice,
		target: PySide6.QtCore.QRectF, source: PySide6.QtCore.QRectF) -> None:
	"""Paint one detached scene without consulting a view or screen selection."""
	painter = PySide6.QtGui.QPainter()
	if not painter.begin(paint_device):
		raise FerrumNativeSnapshotExportError("Qt could not start snapshot painting")
	try:
		scene.render(
			painter, target, source,
			PySide6.QtCore.Qt.AspectRatioMode.IgnoreAspectRatio,
		)
	finally:
		painter.end()


#============================================
def _integer_page_size(rectangle: PySide6.QtCore.QRectF) -> PySide6.QtCore.QSize:
	"""Resolve vector/raster device dimensions at the 72-point document scale."""
	width = math.ceil(rectangle.width())
	height = math.ceil(rectangle.height())
	if width > _MAX_QT_DIMENSION or height > _MAX_QT_DIMENSION:
		raise FerrumNativeSnapshotExportError(
			"snapshot paper dimensions exceed Qt integer representation",
		)
	return PySide6.QtCore.QSize(width, height)


#============================================
def _write_svg(scene: PySide6.QtWidgets.QGraphicsScene,
		rectangle: PySide6.QtCore.QRectF, path: pathlib.Path) -> None:
	"""Publish one vector snapshot with the exact scene rectangle as its view box."""
	device = _open_atomic_device(path)
	try:
		generator = PySide6.QtSvg.QSvgGenerator()
		generator.setOutputDevice(device)
		generator.setSize(_integer_page_size(rectangle))
		generator.setViewBox(rectangle)
		generator.setTitle("Ferrum Rust snapshot")
		generator.setDescription("Rendered from one immutable Ferrum observation")
		_paint_scene(scene, generator, rectangle, rectangle)
		del generator
		_commit_atomic_device(device)
	except Exception:
		device.cancelWriting()
		raise


#============================================
def _write_pdf(scene: PySide6.QtWidgets.QGraphicsScene,
		rectangle: PySide6.QtCore.QRectF, path: pathlib.Path) -> None:
	"""Publish one vector PDF whose page is the backend-issued paper size."""
	device = _open_atomic_device(path)
	try:
		writer = PySide6.QtGui.QPdfWriter(device)
		writer.setCreator("Ferrum-Qt")
		writer.setTitle("Ferrum Rust snapshot")
		writer.setResolution(_POINTS_PER_INCH)
		page_size = PySide6.QtGui.QPageSize(
			PySide6.QtCore.QSizeF(rectangle.width(), rectangle.height()),
			PySide6.QtGui.QPageSize.Unit.Point,
			"Ferrum document",
			PySide6.QtGui.QPageSize.SizeMatchPolicy.ExactMatch,
		)
		if not page_size.isValid() or not writer.setPageSize(page_size):
			raise FerrumNativeSnapshotExportError("Qt rejected the Rust paper size")
		if not writer.setPageMargins(
				PySide6.QtCore.QMarginsF(), PySide6.QtGui.QPageLayout.Unit.Point,
				):
			raise FerrumNativeSnapshotExportError("Qt rejected zero PDF page margins")
		target = PySide6.QtCore.QRectF(0.0, 0.0, writer.width(), writer.height())
		_paint_scene(scene, writer, target, rectangle)
		del writer
		_commit_atomic_device(device)
	except Exception:
		device.cancelWriting()
		raise


#============================================
def _write_png(scene: PySide6.QtWidgets.QGraphicsScene,
		rectangle: PySide6.QtCore.QRectF, path: pathlib.Path) -> None:
	"""Publish one 72-DPI raster within Qt's configured image allocation limit."""
	size = _integer_page_size(rectangle)
	allocation_mib = PySide6.QtGui.QImageReader.allocationLimit()
	if allocation_mib <= 0:
		raise FerrumNativeSnapshotExportError(
			"PNG export requires a positive Qt image allocation limit",
		)
	required = size.width() * size.height() * _RGBA_BYTES_PER_PIXEL
	if required > allocation_mib * 1024 * 1024:
		raise FerrumNativeSnapshotExportError(
			"PNG export exceeds Qt's configured image allocation limit",
		)
	image = PySide6.QtGui.QImage(
		size, PySide6.QtGui.QImage.Format.Format_ARGB32_Premultiplied,
	)
	if image.isNull():
		raise FerrumNativeSnapshotExportError("Qt could not allocate the PNG snapshot")
	image.fill(PySide6.QtCore.Qt.GlobalColor.transparent)
	dots_per_meter = round(_POINTS_PER_INCH / 0.0254)
	image.setDotsPerMeterX(dots_per_meter)
	image.setDotsPerMeterY(dots_per_meter)
	_paint_scene(
		scene, image,
		PySide6.QtCore.QRectF(0.0, 0.0, size.width(), size.height()), rectangle,
	)
	device = _open_atomic_device(path)
	try:
		writer = PySide6.QtGui.QImageWriter(device, b"png")
		if not writer.write(image):
			raise FerrumNativeSnapshotExportError(writer.errorString())
		del writer
		_commit_atomic_device(device)
	except Exception:
		device.cancelWriting()
		raise
