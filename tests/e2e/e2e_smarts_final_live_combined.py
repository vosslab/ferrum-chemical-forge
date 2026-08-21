"""Run final installed-wheel SMARTS lifecycle evidence against one artifact pair."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import textwrap
import zipfile


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
QT_TESTS = REPO_ROOT / "packages" / "ferrum-chem-qt.app" / "tests"
API_TESTS = REPO_ROOT / "packages" / "ferrum-rust" / "crates" / "api" / "python" / "tests"
AMBIENT_RUNTIME_VARIABLES = (
	"DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH",
	"DYLD_FRAMEWORK_PATH",
	"DYLD_FALLBACK_FRAMEWORK_PATH",
	"PYTHONHOME",
	"PYTHONPATH",
)


#============================================
class FinalLiveCombinedE2eError(RuntimeError):
	"""Raised when the final sealed SMARTS lifecycle proof is incomplete."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return one immutable artifact digest."""
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


#============================================
def _run(*command: str, environment: dict[str, str], timeout: float | None = None) -> str:
	"""Run one child, retaining the exact failure output."""
	try:
		result = subprocess.run(command, env=environment, text=True,
			stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=timeout)
	except subprocess.TimeoutExpired as error:
		raise FinalLiveCombinedE2eError(
			"command timed out: %s\\nstdout:\\n%s\\nstderr:\\n%s" % (
				" ".join(command), error.stdout or "", error.stderr or "",
			),
		) from error
	if result.returncode:
		raise FinalLiveCombinedE2eError(
			"command failed (%d): %s\\nstdout:\\n%s\\nstderr:\\n%s" % (
				result.returncode, " ".join(command), result.stdout, result.stderr,
			),
		)
	return result.stdout


#============================================
def _require_regular_wheel(path: pathlib.Path) -> pathlib.Path:
	"""Resolve one regular wheel artifact without accepting a redirect."""
	if not path.is_file() or path.is_symlink() or path.suffix != ".whl":
		raise FinalLiveCombinedE2eError("wheel must be a regular .whl file: %s" % path)
	return path.resolve()


#============================================
def _cli_request(cdml: str, digest: str, query: dict[str, str], request_id: str) -> str:
	"""Return one bounded named-operation request envelope."""
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": {
			"kind": "document.molecule.smarts.query.v1",
			"document": {
				"cdml": cdml,
				"expected_revision": 0,
				"expected_digest_hex": digest,
			},
			"query": query,
			"limits": {"max_matches_per_molecule": 3, "max_total_matches": 3},
		},
	}, separators=(",", ":"))


#============================================
def _run_named_cli(
		cli: pathlib.Path, cdml: str, selected_id: str, environment: dict[str, str],
		directory: pathlib.Path,
		) -> dict[str, object]:
	"""Prove raw and durable-selected named CLI queries under the installed runtime."""
	digest = _sha256_bytes(cdml.encode("utf-8"))
	responses: dict[str, object] = {}
	for name, query in (
		("raw", {"kind": "smarts", "value": "[#6]"}),
		("selected", {"kind": "selected_molecule", "molecule_id": selected_id}),
	):
		request = directory / ("%s-request.json" % name)
		request.write_text(_cli_request(cdml, digest, query, "final-%s" % name), encoding="utf-8")
		output = _run(str(cli), "document", "command", "document.molecule.smarts.query.v1",
			str(request), environment=environment)
		try:
			response = json.loads(output)
		except json.JSONDecodeError as error:
			raise FinalLiveCombinedE2eError("named %s CLI emitted invalid JSON" % name) from error
		outcome = response.get("outcome", {}) if isinstance(response, dict) else {}
		if outcome.get("kind") != "document.molecule.smarts.query.v1":
			raise FinalLiveCombinedE2eError("named %s CLI did not succeed: %r" % (name, response))
		if outcome.get("query") != {
			"schema": "ferrum-document-molecule-smarts-query-v1",
			"traversal": {"kind": "complete"},
			"molecules": [{
				"source_order": 0,
				"match_count": 1,
				"completeness": "complete",
			}],
		}:
			raise FinalLiveCombinedE2eError(
				"named %s CLI did not return one complete source-ordered match: %r" % (
					name, outcome.get("query"),
				),
			)
		encoded = json.dumps(response, sort_keys=True)
		for forbidden in (cdml, selected_id, "[#6]", "receipt", "paint", "record_id", "graph_position"):
			if forbidden in encoded:
				raise FinalLiveCombinedE2eError("named %s CLI leaked %r" % (name, forbidden))
		responses[name] = outcome
	return responses


#============================================
def _sha256_bytes(value: bytes) -> str:
	"""Return the lowercase SHA-256 for a byte string."""
	return hashlib.sha256(value).hexdigest()


#============================================
def _require_matching_native_closure(
		manifest: dict[str, object], native_wheel: pathlib.Path,
		installed_site_packages: pathlib.Path,
		) -> dict[str, str]:
	"""Require exact manifest, wheel archive, and installed native-library closure equality."""
	members = manifest.get("members")
	if not isinstance(members, list) or not members:
		raise FinalLiveCombinedE2eError("sealed engine manifest has no native closure")
	expected: dict[str, str] = {}
	for member in members:
		if not isinstance(member, dict) or set(member) != {"path", "sha256"}:
			raise FinalLiveCombinedE2eError("sealed engine manifest has an invalid closure member")
		name, digest = member["path"], member["sha256"]
		if (
			not isinstance(name, str) or pathlib.PurePosixPath(name).name != name
			or not isinstance(digest, str) or len(digest) != 64
			or name in expected
		):
			raise FinalLiveCombinedE2eError("sealed engine manifest has an invalid closure name")
		expected[name] = digest
	with zipfile.ZipFile(native_wheel) as archive:
		archived = {
			name.removeprefix(".dylibs/"): hashlib.sha256(archive.read(name)).hexdigest()
			for name in archive.namelist()
			if name.startswith(".dylibs/") and not name.endswith("/")
		}
	installed_dylibs = installed_site_packages / ".dylibs"
	installed = {
		path.name: _sha256(path)
		for path in installed_dylibs.iterdir()
		if path.is_file() and not path.is_symlink()
	}
	if set(expected) != set(archived) or set(expected) != set(installed):
		raise FinalLiveCombinedE2eError(
			"native closure differs across manifest, wheel archive, and installed wheel",
		)
	if expected != archived or expected != installed:
		raise FinalLiveCombinedE2eError(
			"native closure digest differs across manifest, wheel archive, and installed wheel",
		)
	return expected


#============================================
def _run_live_reprojection_save_reopen(
		python: pathlib.Path, proof: pathlib.Path, save_path: pathlib.Path,
		environment: dict[str, str],
		) -> dict[str, object]:
	"""Exercise one installed tab through reprojection, Save As, and async Open."""
	script = proof / "live_reprojection_save_reopen.py"
	script.write_text(textwrap.dedent("""
		import faulthandler
		import json
		import pathlib
		import sys

		from PySide6 import QtCore, QtWidgets
		import ferrum_chem
		import ferrum_qt.ferrum.document_tab
		import ferrum_qt.main_window

		CDML = ('<cdml><molecule id="m"><atom id="a" name="C">'
			'<point x="1" y="2"/></atom></molecule></cdml>')

		def refuse(session, receipt):
			try:
				session._show_live_document_smarts_match_v1(receipt, 0)
			except ferrum_chem.LiveDocumentSmartsError as error:
				if str(error) != 'SMARTS query cannot continue':
					raise RuntimeError('old live receipt had an unexpected refusal') from error
				return
			raise RuntimeError('old live receipt remained redeemable')

		def run(session):
			try:
				run = session._run_live_document_smarts_query_v1('[#6]', 3, 3)
			except ferrum_chem.LiveDocumentSmartsError as error:
				raise RuntimeError(
					'live SMARTS query failed: %r/%r' % (error.category, error.reason),
				) from error
			if [(row.source_order, row.match_count) for row in run.molecules] != [(0, 1)]:
				raise RuntimeError('live SMARTS query did not return one source-ordered match')
			return run

		def reopen(window, path):
			completed = []
			loop = QtCore.QEventLoop()
			timeout = QtCore.QTimer()
			timeout.setSingleShot(True)
			def receive(completed_path, success):
				if pathlib.Path(completed_path) == path:
					completed.append(success)
					loop.quit()
			window.local_cdml_open_completed.connect(receive)
			timeout.timeout.connect(loop.quit)
			try:
				if not window.open_file_path(str(path)):
					raise RuntimeError('async native Open did not accept the saved document')
				timeout.start(10000)
				loop.exec()
				if completed != [True]:
					raise RuntimeError('async native Open did not complete successfully')
				return window._active_native_tab()
			finally:
				timeout.stop()
				window.local_cdml_open_completed.disconnect(receive)

		app = QtWidgets.QApplication.instance() or QtWidgets.QApplication([])
		window = ferrum_qt.main_window.MainWindow(object())
		tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(CDML, 'smarts-reopen.cdml')
		path = pathlib.Path(sys.argv[1])
		path.parent.mkdir(parents=True, exist_ok=True)
		if not path.parent.is_dir():
			raise RuntimeError('Save As proof destination parent is unavailable')
		try:
			print('registered', file=sys.stderr, flush=True)
			window._register_native_tab(tab, activate=True)
			window.show()
			app.processEvents()
			baseline = tab._session.observe_render(tab.current_snapshot.revision)
			if not tab._install_observation(baseline):
				raise RuntimeError('initial guarded render-plan publication was not presentable')
			print('baseline', file=sys.stderr, flush=True)
			first = run(tab._session)
			print('first-query', file=sys.stderr, flush=True)
			observation = tab._session.observe_render(tab.current_snapshot.revision)
			if not tab._install_observation(observation):
				raise RuntimeError('guarded observe_render did not retain a valid presentation')
			if tab._document_observation is None or tab.current_snapshot.revision != observation.document.snapshot.revision:
				raise RuntimeError('guarded observe_render did not install its current presentation')
			refuse(tab._session, first.receipt)
			print('reprojected', file=sys.stderr, flush=True)
			unread = run(tab._session)
			print('unread-query', file=sys.stderr, flush=True)
			def fail_save_refusal(request):
				raise RuntimeError('Save As refusal: %r' % (request,))
			window._show_refusal = fail_save_refusal
			print('save-start', file=sys.stderr, flush=True)
			faulthandler.enable(file=sys.stderr)
			faulthandler.dump_traceback_later(30, repeat=False, file=sys.stderr, exit=True)
			try:
				saved = window.save_active_to_path(str(path))
			finally:
				faulthandler.cancel_dump_traceback_later()
			print('save-returned', file=sys.stderr, flush=True)
			if not saved:
				raise RuntimeError('Save As did not publish the SMARTS-active document')
			refuse(tab._session, unread.receipt)
			print('saved', file=sys.stderr, flush=True)
			rust_reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding='utf-8'))
			rust_reopened._publish_live_render_plan_v1(rust_reopened.snapshot().revision)
			rust_run = run(rust_reopened)
			if not rust_reopened._show_live_document_smarts_match_v1(rust_run.receipt, 0).atom_bounds:
				raise RuntimeError('Rust reopen could not reveal a fresh SMARTS match')
			print('rust-reopened', file=sys.stderr, flush=True)
			reopened = reopen(window, path)
			print('gui-reopened', file=sys.stderr, flush=True)
			refuse(tab._session, unread.receipt)
			reopened_observation = reopened._session.observe_render(
				reopened.current_snapshot.revision,
			)
			if not reopened._install_observation(reopened_observation):
				raise RuntimeError('async GUI reopen did not install its guarded render plan')
			gui_run = run(reopened._session)
			if not reopened._session._show_live_document_smarts_match_v1(gui_run.receipt, 0).atom_bounds:
				raise RuntimeError('async GUI reopen could not reveal a fresh SMARTS match')
			print(json.dumps({'schema': 'ferrum-smarts-live-reprojection-save-reopen-v1', 'status': 'ok'}))
		finally:
			window.close()
			window.deleteLater()
	"""), encoding="utf-8")
	return json.loads(_run(str(python), "-I", "-B", str(script), str(save_path),
		environment=environment, timeout=45))


#============================================
def main() -> int:
	"""Install current wheels and execute final M4b lifecycle receipts."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", required=True, type=pathlib.Path)
	parser.add_argument("--qt-wheel", required=True, type=pathlib.Path)
	parser.add_argument("--bundle", required=True, type=pathlib.Path)
	parser.add_argument("--cli", required=True, type=pathlib.Path)
	arguments = parser.parse_args()
	native_wheel = _require_regular_wheel(arguments.native_wheel)
	qt_wheel = _require_regular_wheel(arguments.qt_wheel)
	bundle = arguments.bundle.resolve()
	manifest_path = bundle / "ferrum-engine-bundle-v1.json"
	if not manifest_path.is_file() or manifest_path.is_symlink():
		raise FinalLiveCombinedE2eError("sealed engine manifest is missing")
	manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
	if manifest.get("schema") != "ferrum-engine-bundle-v1" or manifest.get("adapter_abi_version") != 5:
		raise FinalLiveCombinedE2eError("engine bundle is not sealed ABI-5")
	adapter = bundle / str(manifest.get("adapter", ""))
	if not adapter.is_file() or adapter.is_symlink():
		raise FinalLiveCombinedE2eError("sealed engine adapter is missing")
	cli = arguments.cli.resolve()
	if not cli.is_file() or cli.is_symlink():
		raise FinalLiveCombinedE2eError("CLI must be a regular executable")
	environment = os.environ.copy()
	for variable in AMBIENT_RUNTIME_VARIABLES:
		environment.pop(variable, None)
	environment.update({"PYTHONDONTWRITEBYTECODE": "1", "QT_QPA_PLATFORM": "offscreen"})
	with tempfile.TemporaryDirectory(
			prefix="ferrum-smarts-final-live-", dir="/private/tmp",
		) as temporary:
		root = pathlib.Path(temporary)
		venv = root / "venv"
		_run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(str(python), "-B", "-m", "pip", "install", "--force-reinstall", "--no-deps",
			str(native_wheel), str(qt_wheel), environment=environment)
		site_packages = pathlib.Path(_run(str(python), "-I", "-B", "-c",
			"import site; print(site.getsitepackages()[0])", environment=environment).strip())
		installed_adapter = site_packages / ".dylibs" / "libferrum_chem.dylib"
		if not installed_adapter.is_file() or installed_adapter.is_symlink():
			raise FinalLiveCombinedE2eError("installed wheel has no regular native adapter")
		native_closure = _require_matching_native_closure(manifest, native_wheel, site_packages)
		proof = root / "proof"
		proof.mkdir()
		save_directory = root / "save-output"
		save_directory.mkdir()
		runtime_home = root / "runtime-home"
		runtime_home.mkdir()
		proof_environment = environment.copy()
		proof_environment["HOME"] = str(runtime_home)
		_run(str(cli), "engine", "install", str(bundle), environment=proof_environment)
		if _run(str(cli), "engine", "status", environment=proof_environment).strip() != "ready":
			raise FinalLiveCombinedE2eError("isolated shared engine bundle is not ready")
		for source in (
			QT_TESTS / "e2e_native_smarts_live_bridge.py",
			QT_TESTS / "test_ferrum_native_document_tab.py",
			QT_TESTS / "e2e_bracket_authoring.py",
			API_TESTS / "selected_token_packaged_e2e.py",
		):
			shutil.copy2(source, proof / source.name)
		proof_environment.update({
			"FERRUM_SMARTS_QT_SEALED_WHEEL_ROOT": str(site_packages),
			"FERRUM_SMARTS_QT_NATIVE_WHEEL": str(native_wheel),
			"FERRUM_SMARTS_QT_NATIVE_WHEEL_SHA256": _sha256(native_wheel),
			"FERRUM_SMARTS_QT_WHEEL": str(qt_wheel),
			"FERRUM_SMARTS_QT_WHEEL_SHA256": _sha256(qt_wheel),
			"FERRUM_SMARTS_QT_SOURCE_TEST_PATH": str(
				proof / "test_ferrum_native_document_tab.py",
			),
		})
		imports = json.loads(_run(str(python), "-I", "-B", "-c",
			"import json,pathlib,sys,ferrum_chem,ferrum_qt; r=pathlib.Path(sys.prefix); print(json.dumps({'native':str(pathlib.Path(ferrum_chem.__file__).resolve()),'qt':str(pathlib.Path(ferrum_qt.__file__).resolve()),'prefix':str(r)}))",
			environment=proof_environment))
		if not all(str(site_packages) in imports[key] for key in ("native", "qt")):
			raise FinalLiveCombinedE2eError("installed import proof escaped the isolated wheel root")
		selected_output = _run(str(python), "-I", "-B", str(proof / "selected_token_packaged_e2e.py"),
			str(site_packages), str(installed_adapter), environment=proof_environment)
		bridge_output = _run(str(python), "-I", "-B", str(proof / "e2e_native_smarts_live_bridge.py"),
			"--native-wheel", str(native_wheel), "--qt-wheel", str(qt_wheel), environment=proof_environment)
		bracket_output = _run(str(python), "-I", "-B", str(proof / "e2e_bracket_authoring.py"), environment=proof_environment)
		live_reopen_output = _run_live_reprojection_save_reopen(
			python, proof, save_directory / "smarts-live-reopen.cdml", proof_environment,
		)
		cdml = '<cdml><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/></atom></molecule></cdml>'
		selected_id = _run(str(python), "-I", "-B", "-c",
			"import ferrum_chem; s=ferrum_chem.DocumentSession.load(%r); print(s.observe(0).projection.molecules[0].id)" % cdml,
			environment=proof_environment).strip()
		cli_outcomes = _run_named_cli(
			cli, cdml, selected_id, proof_environment, proof,
		)
		print(json.dumps({
			"schema": "ferrum-smarts-final-live-combined-e2e-v1",
			"native_wheel": {"sha256": _sha256(native_wheel), "path": str(native_wheel)},
			"qt_wheel": {"sha256": _sha256(qt_wheel), "path": str(qt_wheel)},
			"bundle_manifest_sha256": _sha256(manifest_path),
			"native_closure": native_closure,
			"installed_imports": imports,
			"selected_token": "passed" if not selected_output else "passed",
			"qt_multirun": json.loads(bridge_output),
			"save_as_async_reopen": json.loads(bracket_output),
			"smarts_live_reprojection_save_reopen": live_reopen_output,
			"named_cli": cli_outcomes,
		}, sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
