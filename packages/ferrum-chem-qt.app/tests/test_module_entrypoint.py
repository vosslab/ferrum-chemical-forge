"""Behavior checks for the ``python -m ferrum_qt`` entry point."""

# Standard Library
import pathlib
import subprocess
import sys


#============================================
def test_module_entrypoint_reports_the_ferrum_qt_version() -> None:
	"""The module entry point delegates to the branded command-line interface."""
	package_root = pathlib.Path(__file__).resolve().parents[1]
	result = subprocess.run(
		[sys.executable, "-B", "-m", "ferrum_qt", "--version"],
		cwd=package_root,
		check=False,
		capture_output=True,
		text=True,
	)

	assert result.returncode == 0
	assert result.stdout.startswith("Ferrum-Qt ")
	assert "BKChem" not in result.stdout
