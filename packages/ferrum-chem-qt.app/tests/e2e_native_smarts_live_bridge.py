"""Prove a current Qt wheel drives the installed ABI-5 live SMARTS bridge."""

# Standard Library
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile


APP_ROOT = pathlib.Path(__file__).resolve().parents[1]
TEST_PATHS = (
	APP_ROOT / "tests" / "test_ferrum_native_document_tab.py",
	APP_ROOT / "tests" / "test_smarts_selected_root_capture.py",
)


#============================================
class NativeSmartsLiveBridgeE2eError(RuntimeError):
	"""Raised when the sealed Qt/live-SMARTS route contradicts its contract."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return the immutable artifact digest for one regular file."""
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one isolated proof process and retain diagnostics on failure."""
	result = subprocess.run(command, env=environment, text=True,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
	if result.returncode:
		raise NativeSmartsLiveBridgeE2eError(
			"command failed (%d): %s\\n%s" % (
				result.returncode, " ".join(command), result.stderr.strip(),
			),
		)
	return result.stdout


#============================================
def _probe() -> dict[str, object]:
	"""Run only the real live-bridge Qt test from the installed wheel pair."""
	test_paths = tuple(pathlib.Path(path) for path in json.loads(
		os.environ["FERRUM_SMARTS_QT_TEST_PATHS"],
	))
	result = subprocess.run((sys.executable, "-B", "-m", "pytest", *map(str, test_paths),
		"-k", (
			"sealed_live_bridge_multiple_rows_replay_and_restore_failure_retire or "
			"canvas_click_consumes_generic_selection_into_opaque_token"
		), "-q"),
		env=os.environ.copy(), text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
		check=False)
	if result.returncode:
		raise NativeSmartsLiveBridgeE2eError(result.stdout + result.stderr)
	if "2 passed" not in result.stdout:
		raise NativeSmartsLiveBridgeE2eError("sealed live-bridge/capture proofs did not execute exactly once")
	return {"schema": "ferrum-native-smarts-live-bridge-e2e-v1", "passed": True}


#============================================
def main() -> int:
	"""Install immutable wheel artifacts and execute the isolated Qt proof."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", type=pathlib.Path)
	parser.add_argument("--qt-wheel", type=pathlib.Path)
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		print(json.dumps(_probe(), sort_keys=True))
		return 0
	if not arguments.native_wheel or not arguments.qt_wheel:
		raise NativeSmartsLiveBridgeE2eError("both immutable wheel artifacts are required")
	for artifact in (arguments.native_wheel, arguments.qt_wheel):
		if not artifact.is_file() or artifact.is_symlink() or artifact.suffix != ".whl":
			raise NativeSmartsLiveBridgeE2eError("wheel artifacts must be regular .whl files")
	environment = os.environ.copy()
	environment.pop("PYTHONPATH", None)
	environment.update({"PYTHONDONTWRITEBYTECODE": "1", "QT_QPA_PLATFORM": "offscreen"})
	with tempfile.TemporaryDirectory(prefix="ferrum-smarts-live-bridge-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(str(python), "-B", "-m", "pip", "install", "--force-reinstall", "--no-deps", str(arguments.native_wheel.resolve()), environment=environment)
		_run(str(python), "-B", "-m", "pip", "install", "--force-reinstall", "--no-deps", str(arguments.qt_wheel.resolve()), environment=environment)
		site_packages = pathlib.Path(_run(str(python), "-B", "-c", "import site; print(site.getsitepackages()[0])", environment=environment).strip())
		source_test_paths = tuple(path.resolve() for path in TEST_PATHS)
		if any(not path.is_file() for path in source_test_paths):
			raise NativeSmartsLiveBridgeE2eError("a live-bridge source test is missing")
		proof_tests = venv / "proof-tests"
		proof_tests.mkdir()
		for source_test_path in source_test_paths:
			shutil.copy2(source_test_path, proof_tests / source_test_path.name)
		(proof_tests / "conftest.py").write_text(
			"import os\n"
			"os.environ.setdefault('QT_QPA_PLATFORM', 'offscreen')\n"
			"import pytest\n"
			"from PySide6.QtWidgets import QApplication\n"
			"@pytest.fixture(scope='session')\n"
			"def qapp():\n"
			"    app = QApplication.instance() or QApplication([])\n"
			"    yield app\n",
			encoding="utf-8",
		)
		environment.update({
			"FERRUM_SMARTS_QT_SEALED_WHEEL_ROOT": str(site_packages),
			"FERRUM_SMARTS_QT_NATIVE_WHEEL": str(arguments.native_wheel.resolve()),
			"FERRUM_SMARTS_QT_NATIVE_WHEEL_SHA256": _sha256(arguments.native_wheel),
			"FERRUM_SMARTS_QT_WHEEL": str(arguments.qt_wheel.resolve()),
			"FERRUM_SMARTS_QT_WHEEL_SHA256": _sha256(arguments.qt_wheel),
			"FERRUM_SMARTS_QT_TEST_PATHS": json.dumps([
				str(proof_tests / source_test_path.name)
				for source_test_path in source_test_paths
			]),
		})
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe", environment=environment)
	print(output.strip())
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
