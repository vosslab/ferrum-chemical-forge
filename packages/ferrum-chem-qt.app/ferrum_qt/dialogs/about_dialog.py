"""About dialog for the Ferrum desktop application."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.versioning
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog


#============================================
class AboutDialog(FerrumAccessibleDialog):
	"""Show Ferrum identity, implementation boundary, licenses, and project link."""

	#============================================
	def __init__(
			self, parent: PySide6.QtWidgets.QWidget | None = None,
			) -> None:
		"""Build the small read-only application-information dialog."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("About Ferrum"))
		self.setMinimumSize(440, 320)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		layout.setSpacing(8)

		name = PySide6.QtWidgets.QLabel(self.tr("Ferrum"), self)
		font = name.font()
		font.setPointSize(20)
		font.setBold(True)
		name.setFont(font)
		name.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		name.setAccessibleName(self.tr("Ferrum application name"))
		layout.addWidget(name)

		version = PySide6.QtWidgets.QLabel(
			self.tr("Version {0}").format(ferrum_qt.versioning.application_version()),
			self,
		)
		version.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		version.setAccessibleName(self.tr("Ferrum version"))
		layout.addWidget(version)

		description = PySide6.QtWidgets.QLabel(
			self.tr("A 2D molecular structure editor with a Rust chemistry engine."),
			self,
		)
		description.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		description.setWordWrap(True)
		layout.addWidget(description)

		license_text = PySide6.QtWidgets.QLabel(
			self.tr(
				"Desktop: GNU AGPL v3 only\n"
				"Ferrum-Chem engine: GNU LGPL v3 only",
			),
			self,
		)
		license_text.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		license_text.setAccessibleName(self.tr("Ferrum license summary"))
		layout.addWidget(license_text)

		project_link = PySide6.QtWidgets.QLabel(
			'<a href="https://github.com/vosslab/ferrum-chemical-forge">'
			"github.com/vosslab/ferrum-chemical-forge</a>",
			self,
		)
		project_link.setOpenExternalLinks(True)
		project_link.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignCenter)
		project_link.setAccessibleName(self.tr("Ferrum project website"))
		project_link.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		layout.addWidget(project_link)
		layout.addStretch()

		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
			parent=self,
		)
		buttons.accepted.connect(self.accept)
		layout.addWidget(buttons)
		self.setTabOrder(
			project_link,
			buttons.button(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok),
		)

	#============================================
	@staticmethod
	def show_about(parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Open one modal Ferrum information dialog."""
		AboutDialog(parent).exec()
