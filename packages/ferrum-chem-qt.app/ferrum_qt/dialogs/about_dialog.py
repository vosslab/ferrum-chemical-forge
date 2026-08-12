"""About dialog showing version and credits."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.versioning



#============================================
class AboutDialog(PySide6.QtWidgets.QDialog):
	"""About dialog with version, license, and credits.

	Displays the application name, version, description, technology
	stack, license, and links to the project repository.

	Args:
		parent: Optional parent widget.
	"""

	#============================================
	def __init__(self, parent: object | None = None) -> None:
		"""Initialize the about dialog.

		Args:
			parent: Optional parent widget.
		"""
		super().__init__(parent)
		self.setWindowTitle(self.tr("About Ferrum-Qt"))
		self.setFixedSize(420, 340)
		self._build_ui()

	#============================================
	def _build_ui(self) -> None:
		"""Build the about dialog layout."""
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		layout.setSpacing(8)

		# application name (large bold)
		name_label = PySide6.QtWidgets.QLabel(self.tr("Ferrum-Qt"))
		name_font = name_label.font()
		name_font.setPointSize(20)
		name_font.setBold(True)
		name_label.setFont(name_font)
		name_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(name_label)

		# version
		version_label = PySide6.QtWidgets.QLabel(
			self.tr("Version %1").arg(ferrum_qt.versioning.application_version())
		)
		version_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(version_label)

		# spacer
		layout.addSpacing(12)

		# description
		desc_label = PySide6.QtWidgets.QLabel(
			self.tr("2D molecular structure editor")
		)
		desc_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(desc_label)

		# technology
		tech_label = PySide6.QtWidgets.QLabel(
			self.tr("PySide6 frontend; Ferrum-Chem migration in progress")
		)
		tech_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(tech_label)

		# spacer
		layout.addSpacing(12)

		# license
		license_label = PySide6.QtWidgets.QLabel(
			self.tr("License: GNU Affero General Public License v3")
		)
		license_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(license_label)

		lineage_label = PySide6.QtWidgets.QLabel(
			self.tr("Continues the BKChem and OASA CDML lineage")
		)
		lineage_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(lineage_label)

		# link
		link_label = PySide6.QtWidgets.QLabel(
			'<a href="https://github.com/vosslab/ferrum-chemical-forge">'
			"github.com/vosslab/ferrum-chemical-forge</a>"
		)
		link_label.setOpenExternalLinks(True)
		link_label.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		layout.addWidget(link_label)

		# stretch to push OK button to the bottom
		layout.addStretch()

		# OK button
		button_box = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
		)
		button_box.accepted.connect(self.accept)
		layout.addWidget(button_box)

	#============================================
	@staticmethod
	def show_about(parent: object | None = None) -> None:
		"""Show the about dialog.

		Args:
			parent: Optional parent widget.
		"""
		dialog = AboutDialog(parent)
		dialog.exec()
