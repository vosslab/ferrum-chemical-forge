#!/usr/bin/env python3
"""Exercise local-build ownership cleanup through the supported shell lifecycle."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


#============================================
class BuildCleanupE2eError(RuntimeError):
	"""Report one failed local-build ownership outcome."""


#============================================
def main() -> int:
	"""Prove a failed candidate preserves the sealed program and cleanup is complete."""
	with tempfile.TemporaryDirectory(prefix="ferrum-local-build-cleanup-") as directory:
		repository = _write_fake_repository(Path(directory))
		old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
		_write_obsolete_build_owned_outputs(repository)
		failed_candidate = _run_fake_build(repository, "candidate-failure")
		if failed_candidate.returncode == 0:
			raise BuildCleanupE2eError("a rejected candidate reported local-build success")
		if _read_local_program(old_runtime, old_cli, old_gui) != (
			"sealed runtime\n", "sealed CLI launcher\n", "sealed GUI launcher\n",
		):
			raise BuildCleanupE2eError("a rejected candidate changed the sealed local program")
		_require_no_owned_transients(repository)

		successful_candidate = _run_fake_build(repository, "success")
		if successful_candidate.returncode != 0:
			raise BuildCleanupE2eError(
			f"the selected local build failed: {successful_candidate.stderr.strip()}"
		)
		_require_selected_local_program(repository)
		_require_no_owned_transients(repository)
	print('{"schema":"ferrum-local-build-cleanup-e2e-v1","status":"ok"}')
	return 0


#============================================
def _read_local_program(runtime: Path, cli: Path, gui: Path) -> tuple[str, str, str]:
	"""Read the three artifacts that make the prior public local program runnable."""
	return (
		runtime.read_text(encoding="utf-8"),
		cli.read_text(encoding="utf-8"),
		gui.read_text(encoding="utf-8"),
	)


#============================================
def _require_no_owned_transients(repository: Path) -> None:
	"""Require compiler, candidate, retired, and stale owned work to be absent."""
	build_root = repository / "build"
	owned_paths = (
		repository / "output_native_wheel",
		build_root / ".cargo-target",
		build_root / ".ferrum-local-build-interrupted",
		build_root / "runtime/.native-engine-interrupted",
		repository / "packages/ferrum-rust/target",
	)
	remaining = [str(path.relative_to(repository)) for path in owned_paths if path.exists()]
	remaining.extend(
		str(path.relative_to(repository))
		for pattern in (".ferrum-local-build-*", ".previous-local-build-*")
		for path in build_root.glob(pattern)
	)
	if remaining:
		raise BuildCleanupE2eError(f"owned build work survived: {', '.join(sorted(remaining))}")


#============================================
def _require_selected_local_program(repository: Path) -> None:
	"""Require the candidate promoted to the public CLI, GUI, and Python runtime."""
	build_root = repository / "build"
	required = (
		build_root / "bin/ferrum",
		build_root / "bin/ferrum-qt",
		build_root / "runtime/python/ferrum_chem.fake",
		build_root / "runtime/python/.dylibs/libferrum_chem.dylib",
		build_root / "runtime/engine-v1/ferrum-engine-bundle-v1.json",
	)
	missing = [str(path.relative_to(repository)) for path in required if not path.is_file()]
	if missing:
		raise BuildCleanupE2eError(f"selected local program is incomplete: {', '.join(missing)}")


#============================================
def _write_obsolete_build_owned_outputs(repository: Path) -> None:
	"""Seed interrupted compiler and staging outputs owned by the local build."""
	build_root = repository / "build"
	for path in (
		repository / "output_native_wheel",
		build_root / ".cargo-target",
		build_root / ".ferrum-local-build-interrupted",
		build_root / "runtime/.native-engine-interrupted",
		repository / "packages/ferrum-rust/target",
	):
		path.mkdir(parents=True)
		(path / "stale").write_text("obsolete build output\n", encoding="utf-8")


#============================================
def _write_prior_local_program(repository: Path) -> tuple[Path, Path, Path]:
	"""Seed one sealed local program that a failed replacement must preserve."""
	old_runtime = repository / "build/runtime/python/known-good"
	old_runtime.parent.mkdir(parents=True)
	old_runtime.write_text("sealed runtime\n", encoding="utf-8")
	old_cli = repository / "build/bin/ferrum"
	old_cli.parent.mkdir()
	old_cli.write_text("sealed CLI launcher\n", encoding="utf-8")
	old_gui = repository / "build/bin/ferrum-qt"
	old_gui.write_text("sealed GUI launcher\n", encoding="utf-8")
	return old_runtime, old_cli, old_gui


#============================================
def _run_fake_build(repository: Path, receipt_mode: str) -> subprocess.CompletedProcess[str]:
	"""Run the real local build lifecycle against its inline fake tool boundary."""
	environment = os.environ | {
		"FERRUM_FAKE_RECEIPT_MODE": receipt_mode,
		"FERRUM_FAKE_REPOSITORY": str(repository),
		"FERRUM_LOCAL_RUNTIME_SOURCE_ROOT": str(_rust_source_root()),
		"PATH": f"{repository / 'fake-bin'}:/usr/bin:/bin",
		"PYTHONDONTWRITEBYTECODE": "1",
	}
	return subprocess.run(
		(str(repository / "build.sh"),), cwd=repository, env=environment,
		check=False, capture_output=True, text=True,
	)


#============================================
def _rust_source_root() -> Path:
	"""Return the production containment-module root needed by the copied lifecycle."""
	return Path(__file__).resolve().parents[2] / "packages/ferrum-rust"


#============================================
def _write_fake_repository(root: Path) -> Path:
	"""Create a no-network fake Cargo repository around the real build lifecycle."""
	repository = root / "repository"
	repository.mkdir()
	build_script = Path(__file__).resolve().parents[2] / "build.sh"
	shutil.copy2(build_script, repository / "build.sh")
	(repository / "build.sh").chmod(0o755)
	(repository / "source_me.sh").write_text(
		"export PYTHONUNBUFFERED=1\nexport PYTHONDONTWRITEBYTECODE=1\n", encoding="utf-8"
	)
	rust_root = repository / "packages/ferrum-rust"
	engine_lib = rust_root / "engine_lib"
	engine_lib.mkdir(parents=True)
	(engine_lib / "local_runtime_launcher.py").write_text(
		"import argparse\nimport os\nfrom pathlib import Path\n"
		"parser = argparse.ArgumentParser()\nparser.add_argument('--write-gui', action='store_true')\n"
		"parser.add_argument('--launcher-path', type=Path, required=True)\nargs = parser.parse_args()\n"
		"args.launcher_path.write_text('#!/usr/bin/env bash\\n', encoding='utf-8')\n"
		"os.chmod(args.launcher_path, 0o755)\n",
		encoding="utf-8",
	)
	(rust_root / "local_engine_builder.py").write_text(
		"import argparse\nimport os\nimport sys\nfrom pathlib import Path\n"
		"sys.path.insert(0, os.environ['FERRUM_LOCAL_RUNTIME_SOURCE_ROOT'])\n"
		"from engine_lib import local_runtime\n"
		"local_runtime.REPO_ROOT = Path(os.environ['FERRUM_FAKE_REPOSITORY'])\n"
		"parser = argparse.ArgumentParser()\nparser.add_argument('--runtime-root', type=Path, required=True)\n"
		"root = local_runtime.runtime_root_path(str(parser.parse_args().runtime_root))\n"
		"(root / '.dylibs').mkdir(parents=True)\n"
		"(root / '.dylibs/libferrum_chem.dylib').write_bytes(b'candidate adapter')\n"
		"bundle = root.parent / 'engine-v1'\nbundle.mkdir()\n"
		"(bundle / 'ferrum-engine-bundle-v1.json').write_text('{}', encoding='utf-8')\n",
		encoding="utf-8",
	)
	(rust_root / "local_runtime_receipt.py").write_text(
		"import argparse\nimport os\nfrom pathlib import Path\n"
		"parser = argparse.ArgumentParser()\nparser.add_argument('command')\n"
		"parser.add_argument('--runtime-root', type=Path, required=True)\nargs = parser.parse_args()\n"
		"if args.command == 'extension-path': print(args.runtime_root / 'ferrum_chem.fake')\n"
		"elif args.command == 'validate':\n"
		"    mode = os.environ['FERRUM_FAKE_RECEIPT_MODE']\n"
		"    candidate = '.ferrum-local-build-' in str(args.runtime_root)\n"
		"    if mode == 'candidate-failure' and candidate: raise SystemExit(1)\n",
		encoding="utf-8",
	)
	fake_bin = repository / "fake-bin"
	fake_bin.mkdir()
	(fake_bin / "cargo").write_text(
		"#!/usr/bin/env bash\nset -euo pipefail\n"
		"mkdir -p \"${CARGO_TARGET_DIR}/release\"\n"
		"printf 'cli' >\"${CARGO_TARGET_DIR}/release/ferrum\"\n"
		"printf 'extension' >\"${CARGO_TARGET_DIR}/release/libferrum_chem.dylib\"\n",
		encoding="utf-8",
	)
	(fake_bin / "cargo").chmod(0o755)
	return repository


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except BuildCleanupE2eError as error:
		print(f"local build cleanup E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
