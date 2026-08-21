"""Prove the installed native wheel opens one CML document through the Qt queue."""

from __future__ import annotations

# Standard-library imports.
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import textwrap
import zipfile


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
QT_ROOT = REPO_ROOT / "packages" / "ferrum-chem-qt.app"
AMBIENT_RUNTIME_VARIABLES = (
	"DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH",
	"DYLD_FRAMEWORK_PATH",
	"DYLD_FALLBACK_FRAMEWORK_PATH",
	"PYTHONHOME",
	"PYTHONPATH",
)


#============================================
class CmlQtNewDocumentE2eError(RuntimeError):
	"""Raised when the isolated CML Qt route contradicts its contract."""


#============================================
def sha256(path: pathlib.Path) -> str:
	"""Return the immutable digest for one regular artifact."""
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


#============================================
def run(*command: str, environment: dict[str, str]) -> str:
	"""Run one local child and retain actionable diagnostics on failure."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise CmlQtNewDocumentE2eError(
			"command failed (%d): %s\nstdout:\n%s\nstderr:\n%s" % (
				result.returncode, " ".join(command), result.stdout, result.stderr,
			),
		)
	return result.stdout


#============================================
def scrubbed_environment() -> dict[str, str]:
	"""Return an offscreen environment without ambient native import paths."""
	environment = os.environ.copy()
	for variable in AMBIENT_RUNTIME_VARIABLES:
		environment.pop(variable, None)
	environment.update({"PYTHONDONTWRITEBYTECODE": "1", "QT_QPA_PLATFORM": "offscreen"})
	return environment


#============================================
def extension_member_digest(wheel: pathlib.Path) -> str:
	"""Read the one direct native extension digest from the supplied wheel."""
	with zipfile.ZipFile(wheel) as archive:
		members = [
			name for name in archive.namelist()
			if name.startswith("ferrum_chem") and name.endswith(".so") and "/" not in name
		]
		if len(members) != 1:
			raise CmlQtNewDocumentE2eError(
				f"wheel must contain exactly one direct ferrum_chem extension, found {members!r}",
			)
		return hashlib.sha256(archive.read(members[0])).hexdigest()


CHILD_PROGRAM = r'''
import hashlib
import importlib.machinery
import json
import pathlib
import sys

qt_root = pathlib.Path(sys.argv[1]).resolve()
expected_extension_digest = sys.argv[2]
work_root = pathlib.Path(sys.argv[3]).resolve()
sys.path.insert(0, str(qt_root))

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem
import ferrum_qt
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


def fail(message):
	raise RuntimeError(message)


def safe_path_label(path):
	"""Return a fixed label for one temporary fixture path."""
	if pathlib.Path(path) == valid_path:
		return "valid"
	if pathlib.Path(path) == invalid_path:
		return "invalid"
	return "unexpected"


def queue_state(window):
	"""Return value-safe public queue and tab facts."""
	return {
		"pending": bool(window.has_pending_local_cdml_open()),
		"tab_count": window.centralWidget().count(),
	}


def wait_for_queue(window, start):
	"""Drive exactly one public local-open batch to its public completion."""
	completed = []
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)

	def receive(success):
		completed.append(bool(success))
		loop.quit()

	window.local_cdml_open_queue_drained.connect(receive)
	timeout.timeout.connect(loop.quit)
	try:
		PySide6.QtCore.QTimer.singleShot(0, start)
		timeout.start(10000)
		loop.exec()
	finally:
		timeout.stop()
		window.local_cdml_open_queue_drained.disconnect(receive)
		timeout.timeout.disconnect(loop.quit)
	if len(completed) != 1:
		fail("local Open queue did not emit exactly one completion: %r" % completed)
	if window.has_pending_local_cdml_open():
		fail("local Open queue retained a pending request after completion")
	return completed[0]


def current_tab(window):
	"""Return the active native tab from the ordinary central-widget tree."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		fail("ordinary Ferrum window has no tab widget")
	tab = tabs.currentWidget()
	if not isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab):
		fail("ordinary Ferrum window has no active native document tab")
	return tab


def source_isolation():
	"""Prove the Qt checkout and native extension came from their required roots."""
	if pathlib.Path(ferrum_qt.__file__).resolve() != qt_root / "ferrum_qt" / "__init__.py":
		fail("Qt package did not load from the current checkout")
	extension = pathlib.Path(ferrum_chem.__file__).resolve()
	if extension.parent != pathlib.Path(sys.prefix).resolve() / "lib" / "python3.12" / "site-packages":
		fail("native extension did not load from the isolated venv: %s" % extension)
	if not extension.name.endswith(tuple(importlib.machinery.EXTENSION_SUFFIXES)):
		fail("ferrum_chem is not the direct native extension")
	if hashlib.sha256(extension.read_bytes()).hexdigest() != expected_extension_digest:
		fail("installed native extension does not match the supplied wheel")
	return {"native": str(extension), "qt": str(pathlib.Path(ferrum_qt.__file__).resolve())}


valid_path = work_root / "accepted.cml"
invalid_path = work_root / "refused.cml"
valid_path.write_text(
	'<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule id="cml-molecule"><atomArray><atom id="cml-carbon" elementType="C" x2="0" y2="0"/></atomArray></molecule></cml>',
	encoding="utf-8",
)
invalid_path.write_text("<cml xmlns=\"urn:unsupported\"/>", encoding="utf-8")
app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
window = ferrum_qt.main_window.MainWindow(object())
bootstrap = current_tab(window)
open_action = next(
	action
	for action in window.findChildren(PySide6.QtGui.QAction)
	if action.text() == "Open"
)
original_get_open_file_name = PySide6.QtWidgets.QFileDialog.getOpenFileName
refusals = []
diagnostics = {"completed": [], "queue_starts": [], "refusals": []}


def record_refusal(request):
	"""Record the ordinary typed request without showing a modal dialog."""
	if type(request) is not ferrum_qt.dialogs.refusal_presenter.RefusalRequest:
		fail("Open refusal was not a typed RefusalRequest")
	refusals.append(request)


def refusal_facts(request):
	"""Return typed, redacted presentation facts for one refusal request."""
	presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(request)
	return {
		"request_type": type(request).__name__,
		"title": presentation.title,
		"has_technical_details": presentation.technical_details is not None,
		"ordinary_text_redacted": "urn:unsupported" not in presentation.ordinary_text().lower(),
		"state": queue_state(window),
	}


window._show_edit_refusal = record_refusal
def record_completed(path, success):
	"""Record public completion facts without retaining file paths or CML."""
	diagnostics["completed"].append({
		"path": safe_path_label(path),
		"success": bool(success),
		"state": queue_state(window),
	})


window.local_cdml_open_completed.connect(record_completed)


def scheduled_open(path, label):
	"""Return a queued public start callback and retain its accepted result."""
	def start():
		accepted = bool(window.open_file_path(str(path)))
		diagnostics["queue_starts"].append({
			"path": label,
			"accepted": accepted,
			"state": queue_state(window),
		})
	return start


try:
	PySide6.QtWidgets.QFileDialog.getOpenFileName = (
		lambda *_args: (str(valid_path), "CML (*.cml)")
	)
	before_tabs = window.centralWidget().count()
	valid_completed = wait_for_queue(window, open_action.trigger)
	if not valid_completed:
		fail("valid CML did not complete successfully; diagnostics=%s" % json.dumps({
			"completed": diagnostics["completed"],
			"queue_starts": diagnostics["queue_starts"],
			"refusals": [refusal_facts(request) for request in refusals],
			"state": queue_state(window),
		}, sort_keys=True))
	tab = current_tab(window)
	if (
		window.centralWidget().count() != before_tabs + 1
		or bootstrap.is_disposed
		or tab is bootstrap
		or tab.file_path is not None
		or tab.is_dirty
		or tab.title != valid_path.name
	):
		fail("interactive valid CML did not retain bootstrap and install one clean new tab")
	projection = tab.current_document_observation().projection
	if len(projection.molecules) != 1:
		fail("valid CML did not render one molecule")
	atoms = projection.molecules[0].atoms
	if len(atoms) != 1 or atoms[0].element != "C":
		fail("valid CML did not render one carbon atom")
	before_invalid_tabs = window.centralWidget().count()
	if wait_for_queue(window, scheduled_open(invalid_path, "invalid")):
		fail("invalid CML completed successfully")
	if window.centralWidget().count() != before_invalid_tabs or current_tab(window) is not tab:
		fail("invalid CML changed the installed document tabs")
	if len(refusals) != 1:
		fail("invalid CML did not emit exactly one typed refusal")
	presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(refusals[0])
	if (
		presentation.title != "Could Not Open Document"
		or presentation.technical_details is None
		or "urn:unsupported" in presentation.ordinary_text().lower()
	):
		fail("invalid CML refusal was not typed and redacted")
	diagnostics["refusals"].append(refusal_facts(refusals[0]))
	print(json.dumps({
		"schema": "ferrum-cml-qt-new-document-e2e-v1",
		"isolation": source_isolation(),
		"status": "ok",
		"tab_count": before_invalid_tabs,
	}, sort_keys=True))
finally:
	PySide6.QtWidgets.QFileDialog.getOpenFileName = original_get_open_file_name
	window.local_cdml_open_completed.disconnect(record_completed)
	window.close()
	window.deleteLater()
'''


#============================================
def main() -> int:
	"""Install one exact local wheel and execute the isolated Qt CML workflow."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", required=True, type=pathlib.Path)
	parser.add_argument(
		"--receipt", type=pathlib.Path,
		default=pathlib.Path("/private/tmp/ferrum-cml-qt-new-document-root-e2e.json"),
	)
	arguments = parser.parse_args()
	wheel = arguments.native_wheel.resolve()
	if not wheel.is_file() or wheel.is_symlink() or wheel.suffix != ".whl":
		raise CmlQtNewDocumentE2eError(f"native wheel must be a regular .whl file: {wheel}")
	if not QT_ROOT.is_dir():
		raise CmlQtNewDocumentE2eError(f"current checkout Qt root is missing: {QT_ROOT}")
	environment = scrubbed_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-cml-qt-e2e-", dir="/private/tmp") as temporary:
		root = pathlib.Path(temporary)
		venv = root / "venv"
		run(
			sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv),
			environment=environment,
		)
		python = venv / "bin" / "python"
		run(
			str(python), "-B", "-m", "pip", "install", "--ignore-installed", "--no-deps",
			str(wheel), environment=environment,
		)
		child = root / "workflow.py"
		child_source = textwrap.dedent(CHILD_PROGRAM)
		compile(child_source, str(child), "exec")
		child.write_text(child_source, encoding="utf-8")
		output = run(
			str(python), "-I", "-B", str(child), str(QT_ROOT), extension_member_digest(wheel),
			str(root), environment=environment,
		)
		try:
			result = json.loads(output)
		except json.JSONDecodeError as error:
			raise CmlQtNewDocumentE2eError("workflow did not emit one JSON result") from error
		if result.get("schema") != "ferrum-cml-qt-new-document-e2e-v1" or result.get("status") != "ok":
			raise CmlQtNewDocumentE2eError(f"workflow result is invalid: {result!r}")
	receipt = {
		"schema": "ferrum-cml-qt-new-document-e2e-receipt-v1",
		"native_wheel": {"path": str(wheel), "sha256": sha256(wheel)},
		"workflow": result,
	}
	arguments.receipt.parent.mkdir(parents=True, exist_ok=True)
	arguments.receipt.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
	print(json.dumps(receipt, sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
