"""Behavioral contract tests for build.sh local-runtime candidate ownership."""

import os
import shutil
import subprocess
from pathlib import Path


#============================================
def test_failed_candidate_build_preserves_the_last_sealed_runtime(tmp_path: Path) -> None:
	"""A post-native-stage failure discards only the unsealed candidate program."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "candidate-failure")
	assert completed.returncode != 0
	assert old_runtime.read_text(encoding="utf-8") == "sealed runtime\n"
	assert old_cli.read_text(encoding="utf-8") == "sealed CLI launcher\n"
	assert old_gui.read_text(encoding="utf-8") == "sealed GUI launcher\n"
	assert not list((repository / "build").glob(".ferrum-local-build-*"))
	assert not list((repository / "build/runtime").glob(".native-engine-*"))


#============================================
def test_failed_promoted_runtime_validation_restores_the_last_sealed_program(tmp_path: Path) -> None:
	"""A failed public receipt restores both prior launchers and the sealed runtime."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "final-validation-failure")
	assert completed.returncode != 0
	assert old_runtime.read_text(encoding="utf-8") == "sealed runtime\n"
	assert old_cli.read_text(encoding="utf-8") == "sealed CLI launcher\n"
	assert old_gui.read_text(encoding="utf-8") == "sealed GUI launcher\n"
	assert not list((repository / "build").glob(".ferrum-local-build-*"))
	assert not list((repository / "build").glob(".previous-local-build-*"))


#============================================
def test_failed_runtime_backup_preserves_the_last_sealed_program(tmp_path: Path) -> None:
	"""A failed runtime backup leaves the public sealed program runnable."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "backup-runtime-failure")
	assert "cannot save the existing local runtime" in completed.stderr
	assert _read_local_program(old_runtime, old_cli, old_gui) == (
		"sealed runtime\n", "sealed CLI launcher\n", "sealed GUI launcher\n",
	)


#============================================
def test_failed_launcher_backup_restores_the_last_sealed_program(tmp_path: Path) -> None:
	"""A failed launcher backup restores the runtime moved earlier in the transaction."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "backup-bin-failure")
	assert "restored the prior local program" in completed.stderr
	assert _read_local_program(old_runtime, old_cli, old_gui) == (
		"sealed runtime\n", "sealed CLI launcher\n", "sealed GUI launcher\n",
	)


#============================================
def test_failed_candidate_runtime_promotion_restores_the_last_sealed_program(
	tmp_path: Path,
) -> None:
	"""A candidate-runtime relocation failure restores the complete prior program."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "candidate-runtime-promotion-failure")
	assert completed.returncode != 0
	assert _read_local_program(old_runtime, old_cli, old_gui) == (
		"sealed runtime\n", "sealed CLI launcher\n", "sealed GUI launcher\n",
	)
	assert not list((repository / "build").glob(".ferrum-local-build-*"))
	assert not list((repository / "build").glob(".previous-local-build-*"))


#============================================
def test_failed_candidate_launcher_promotion_restores_the_last_sealed_program(
	tmp_path: Path,
) -> None:
	"""A candidate-launcher relocation failure restores the complete prior program."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "candidate-bin-promotion-failure")
	assert completed.returncode != 0
	assert _read_local_program(old_runtime, old_cli, old_gui) == (
		"sealed runtime\n", "sealed CLI launcher\n", "sealed GUI launcher\n",
	)
	assert not list((repository / "build").glob(".ferrum-local-build-*"))
	assert not list((repository / "build").glob(".previous-local-build-*"))


#============================================
def test_failed_recovery_retains_the_unrestored_sealed_component(tmp_path: Path) -> None:
	"""Recovery reports its retained sealed runtime after attempting every restoration."""
	repository = _write_fake_repository(tmp_path)
	old_runtime, old_cli, old_gui = _write_prior_local_program(repository)
	completed = _run_fake_build(repository, "restore-runtime-failure")
	retained_runtime = next((repository / "build").glob(".previous-local-build-*/runtime/python/known-good"))
	assert "local build recovery is incomplete" in completed.stderr
	assert str(retained_runtime.parents[2]) in completed.stderr
	assert retained_runtime.read_text(encoding="utf-8") == "sealed runtime\n"
	assert old_cli.read_text(encoding="utf-8") == "sealed CLI launcher\n"
	assert old_gui.read_text(encoding="utf-8") == "sealed GUI launcher\n"


#============================================
def _read_local_program(runtime: Path, cli: Path, gui: Path) -> tuple[str, str, str]:
	"""Read the three artifacts that make the public local program runnable."""
	return (
		runtime.read_text(encoding="utf-8"),
		cli.read_text(encoding="utf-8"),
		gui.read_text(encoding="utf-8"),
	)


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
	"""Run the copied shell lifecycle with real runtime-root containment validation."""
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
	"""Return the real module root containing the production containment function."""
	return Path(__file__).resolve().parents[1] / "packages/ferrum-rust"


#============================================
def _write_fake_repository(root: Path) -> Path:
	"""Create a no-network build harness around the production shell lifecycle."""
	repository = root / "repository"
	repository.mkdir()
	build_script = Path(__file__).resolve().parents[1] / "build.sh"
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
		"    final_failure = mode in {'final-validation-failure', 'restore-runtime-failure'}\n"
		"    if (mode == 'candidate-failure' and candidate) or (final_failure and not candidate): raise SystemExit(1)\n",
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
	(fake_bin / "mv").write_text(
		"#!/usr/bin/env bash\nset -euo pipefail\n"
		"source_path=$1\ndestination_path=$2\nmode=${FERRUM_FAKE_RECEIPT_MODE:?}\n"
		"repo=${FERRUM_FAKE_REPOSITORY:?}\n"
		"case \"${mode}:${source_path}:${destination_path}\" in\n"
		"  backup-runtime-failure:${repo}/build/runtime:${repo}/build/.previous-local-build-*/runtime) exit 19 ;;\n"
		"  backup-bin-failure:${repo}/build/bin:${repo}/build/.previous-local-build-*/bin) exit 20 ;;\n"
		"  candidate-runtime-promotion-failure:${repo}/build/.ferrum-local-build-*/runtime:${repo}/build/runtime) exit 22 ;;\n"
		"  candidate-bin-promotion-failure:${repo}/build/.ferrum-local-build-*/bin:${repo}/build/bin) exit 23 ;;\n"
		"  restore-runtime-failure:${repo}/build/.previous-local-build-*/runtime:${repo}/build/runtime) exit 21 ;;\n"
		"esac\nexec /bin/mv \"${source_path}\" \"${destination_path}\"\n",
		encoding="utf-8",
	)
	(fake_bin / "mv").chmod(0o755)
	return repository
