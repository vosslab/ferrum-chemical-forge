#!/usr/bin/env python3
"""Exercise local-build ownership cleanup through the supported shell lifecycle."""

import os
import json
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile


#============================================
class BuildCleanupE2eError(RuntimeError):
	"""Report one failed local-build ownership outcome."""


#============================================
def main() -> int:
	"""Prove local builds retire obsolete outputs and preserve current leased programs."""
	with tempfile.TemporaryDirectory(prefix="ferrum-local-build-cleanup-") as directory:
		repository = _write_fake_repository(Path(directory))
		_write_obsolete_direct_layout(repository)
		_write_obsolete_build_owned_outputs(repository)
		_verify_build_lock_recovery(repository)
		_require_obsolete_direct_layout_retired(repository)
		_require_selected_local_program(repository)
		_require_gui_launcher_handoff(repository)
		_require_cli_launcher_argv_handoff(repository)
		selected_before_rejection = (repository / "build/current").resolve()
		immutable_native_temp = _write_immutable_native_engine_temp(repository)
		malformed_programs = _write_malformed_noncurrent_programs(repository)
		unrelated_programs_content = _write_unrelated_programs_content(repository)
		failed_candidate = _run_fake_build(repository, "candidate-failure")
		if failed_candidate.returncode == 0:
			raise BuildCleanupE2eError("a rejected candidate reported local-build success")
		if (repository / "build/current").resolve() != selected_before_rejection:
			raise BuildCleanupE2eError("a rejected candidate changed the selected leased program")
		if any(program.exists() for program in malformed_programs):
			raise BuildCleanupE2eError("a malformed non-current local program survived cleanup")
		if not immutable_native_temp.is_file():
			raise BuildCleanupE2eError("cleanup mutated the selected immutable program root")
		if not unrelated_programs_content.is_file():
			raise BuildCleanupE2eError("cleanup removed unrelated build/programs content")
		_require_no_owned_transients(repository)

		old_program = (repository / "build/current").resolve()
		lease_holder = _start_runtime_lease_holder(repository, old_program / "bin/ferrum")
		successful_candidate = _run_fake_build(repository, "success")
		if successful_candidate.returncode != 0:
			_release_runtime_lease_holder(lease_holder)
			raise BuildCleanupE2eError(
				f"the selected local build failed: {successful_candidate.stderr.strip()}"
		)
		_require_selected_local_program(repository)
		_require_no_owned_transients(repository)
		if not old_program.is_dir():
			_release_runtime_lease_holder(lease_holder)
			raise BuildCleanupE2eError("a lease-held superseded local program was removed")
		_release_runtime_lease_holder(lease_holder)
		cleanup_candidate = _run_fake_build(repository, "success")
		if cleanup_candidate.returncode != 0:
			raise BuildCleanupE2eError(
				f"the inactive-program cleanup build failed: {cleanup_candidate.stderr.strip()}"
			)
		if old_program.exists():
			raise BuildCleanupE2eError("an inactive superseded local program survived cleanup")

		_current_program_recovers_after_promoted_root_kill(repository)

		pointer_failure = _run_fake_build(repository, "success", fail_after_pointer_stage=True)
		if pointer_failure.returncode == 0:
			raise BuildCleanupE2eError("pointer-stage interruption reported local-build success")
		if list((repository / "build").glob(".current-next-*")):
			raise BuildCleanupE2eError("pointer-stage interruption retained owned staging state")
		recovery = _run_fake_build(repository, "success")
		if recovery.returncode != 0:
			raise BuildCleanupE2eError(
				f"a build could not recover after pointer-stage interruption: {recovery.stderr.strip()}"
			)
	print('{"schema":"ferrum-local-build-cleanup-e2e-v1","status":"ok"}')
	return 0


#============================================
def _start_runtime_lease_holder(
	repository: Path, launcher: Path,
) -> subprocess.Popen[str]:
	"""Start the generated CLI wrapper and wait for its executable handoff."""
	environment = os.environ | {"FERRUM_LEASE_READY": "1"}
	holder = subprocess.Popen(
		(str(launcher),), cwd=repository, env=environment, stdin=subprocess.PIPE,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
	)
	if holder.stdout is None or holder.stdout.readline().strip() != "runtime-lease-ready":
		_release_runtime_lease_holder(holder)
		raise BuildCleanupE2eError("the generated runtime lease holder did not start")
	return holder


#============================================
def _release_runtime_lease_holder(holder: subprocess.Popen[str]) -> None:
	"""Release the generated CLI process and its inherited shared lease."""
	if holder.poll() is None and holder.stdin is not None:
		holder.stdin.write("release\n")
		holder.stdin.flush()
	if holder.poll() is None:
		holder.wait(timeout=5)


#============================================
def _current_program_recovers_after_promoted_root_kill(repository: Path) -> None:
	"""Require locked startup to remove a SIGKILL orphan before another promotion."""
	current_before_kill = (repository / "build/current").resolve()
	environment = _fake_build_environment(repository, "success")
	environment["FERRUM_BUILD_KILL_AFTER_PROGRAM_RENAME"] = "1"
	crashed_build = subprocess.Popen(
		(str(repository / "build.sh"),), cwd=repository, env=environment,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True,
	)
	if crashed_build.stderr is None or crashed_build.stderr.readline().strip() != "program-rename-ready":
		_stop_process_group(crashed_build, signal.SIGTERM)
		raise BuildCleanupE2eError("the crash injection missed program-root promotion")
	_stop_process_group(crashed_build, signal.SIGKILL)
	orphans = [
		program for program in (repository / "build/programs").glob("program-*")
		if program.resolve() != current_before_kill
	]
	if not orphans:
		raise BuildCleanupE2eError("SIGKILL did not leave a promoted-root orphan")

	recovery = _run_fake_build(repository, "candidate-failure")
	if recovery.returncode == 0:
		raise BuildCleanupE2eError("crash recovery unexpectedly promoted a rejected candidate")
	if (repository / "build/current").resolve() != current_before_kill:
		raise BuildCleanupE2eError("crash recovery changed the prior selected local program")
	if any(program.exists() for program in orphans):
		raise BuildCleanupE2eError("locked startup retained the unreachable promoted-root orphan")


#============================================
def _verify_build_lock_recovery(repository: Path) -> None:
	"""Require an inode-held lock to refuse contenders until its holder exits."""
	_lock_metadata_is_diagnostic_only(repository)

	marker = repository / "lock-ready"
	environment = _fake_build_environment(repository, "lock-hold")
	environment["FERRUM_FAKE_LOCK_READY"] = str(marker)
	live_build = subprocess.Popen(
		(str(repository / "build.sh"),), cwd=repository, env=environment,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True,
	)
	if live_build.stderr is None or live_build.stderr.readline().strip() != "lock-ready":
		_stop_lock_owner_only(live_build)
		raise BuildCleanupE2eError("the live lock owner did not reach its guarded build phase")
	competing_build = _run_fake_build(repository, "success")
	if competing_build.returncode == 0 or "another ./build.sh invocation owns" not in competing_build.stderr:
		_stop_lock_owner_only(live_build)
		raise BuildCleanupE2eError(
			"a matching live build lock did not refuse the competitor: "
			f"{competing_build.stderr.strip()}"
		)
	_stop_lock_owner_only(live_build)
	if not (repository / "build/.build.lock").is_file():
		raise BuildCleanupE2eError("the stable local-build lock inode was not retained")
	replacement = _run_fake_build(repository, "success")
	if replacement.returncode != 0:
		raise BuildCleanupE2eError(
			"a contender could not acquire the released local-build lock: "
			f"{replacement.stderr.strip()}"
		)
	marker.with_name("lock-release").write_text("release\n", encoding="utf-8")


#============================================
def _stop_process_group(live_build: subprocess.Popen[str], termination_signal: signal.Signals) -> None:
	"""Stop the build shell and its compiler children in their session-owned group."""
	os.killpg(live_build.pid, termination_signal)
	live_build.wait(timeout=5)


#============================================
def _stop_lock_owner_only(live_build: subprocess.Popen[str]) -> None:
	"""Interrupt only the lock owner; the synthetic compiler stays alive by handshake."""
	os.kill(live_build.pid, signal.SIGTERM)
	live_build.wait(timeout=5)


#============================================
def _lock_metadata_is_diagnostic_only(repository: Path) -> None:
	"""Require unrelated historical owner text to have no ownership effect."""
	metadata = repository / "build/.build.lock.owner"
	metadata.parent.mkdir(parents=True, exist_ok=True)
	metadata.write_text("interrupted owner metadata\n", encoding="utf-8")
	recovery = _run_fake_build(repository, "success")
	if recovery.returncode != 0:
		raise BuildCleanupE2eError(
			"diagnostic lock metadata prevented a replacement local build: "
			f"{recovery.stderr.strip()}"
		)


#============================================
def _require_no_owned_transients(repository: Path) -> None:
	"""Require compiler, candidate, retired, and stale owned work to be absent."""
	build_root = repository / "build"
	owned_paths = (
		repository / "output_native_wheel",
		build_root / ".cargo-target",
		build_root / ".ferrum-local-build-interrupted",
		repository / "packages/ferrum-rust/target",
	)
	remaining = [str(path.relative_to(repository)) for path in owned_paths if path.exists()]
	remaining.extend(
		str(path.relative_to(repository))
		for path in (build_root / "programs").glob(".staging-*")
	)
	remaining.extend(
		str(path.relative_to(repository))
		for path in build_root.glob(".cargo-target-*")
	)
	if remaining:
		raise BuildCleanupE2eError(f"owned build work survived: {', '.join(sorted(remaining))}")


#============================================
def _require_selected_local_program(repository: Path) -> None:
	"""Require every stable path to resolve inside one selected immutable root."""
	build_root = repository / "build"
	current = build_root / "current"
	if not current.is_symlink():
		raise BuildCleanupE2eError("selected local program lacks its current pointer")
	selected_root = current.resolve()
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
	for path in required:
		if not path.resolve().is_relative_to(selected_root):
			raise BuildCleanupE2eError(
				f"stable local path escapes its selected program: {path.relative_to(repository)}"
			)
	if not (build_root / "bin").is_symlink() or not (build_root / "runtime").is_symlink():
		raise BuildCleanupE2eError("stable local paths are not current-pointer links")


#============================================
def _require_gui_launcher_handoff(repository: Path) -> None:
	"""Require the stable Qt payload to preserve source-owned module provenance."""
	launch = subprocess.run(
		(str(repository / "build/bin/ferrum-qt"),), cwd=repository,
		env=_fake_build_environment(repository, "success"), check=False,
		capture_output=True, text=True,
	)
	if launch.returncode != 0:
		raise BuildCleanupE2eError(
			"the stable Qt launcher did not reach its selected runtime payload: "
			f"{launch.stderr.strip()}"
		)
	try:
		provenance = json.loads(launch.stdout)
	except json.JSONDecodeError as error:
		raise BuildCleanupE2eError(
			"the generated Qt payload did not report module provenance"
		) from error
	source_repository = repository.resolve()
	qt_source_root = source_repository / "packages/ferrum-chem-qt.app"
	runtime_root = source_repository / "build/current/runtime/python"
	caller_entries = (
		repository / "caller-python-one",
		repository / "caller-python-two",
	)
	if provenance.get("ferrum_qt") != str((qt_source_root / "ferrum_qt/__init__.py").resolve()):
		raise BuildCleanupE2eError("the generated Qt payload did not import repository ferrum_qt")
	paths = provenance.get("sys_path")
	if not isinstance(paths, list) or not all(isinstance(path, str) for path in paths):
		raise BuildCleanupE2eError("the generated Qt payload did not report Python search paths")
	expected_order = (qt_source_root.resolve(), runtime_root, *caller_entries)
	try:
		path_indexes = tuple(paths.index(str(path)) for path in expected_order)
	except ValueError as error:
		raise BuildCleanupE2eError(
			"the generated Qt payload lost a source-owned Python search path: "
			f"expected {[str(path) for path in expected_order]}, reported {paths}"
		) from error
	if path_indexes != tuple(sorted(path_indexes)):
		raise BuildCleanupE2eError(
			"the generated Qt payload changed source_me Python path precedence"
		)


#============================================
def _require_cli_launcher_argv_handoff(repository: Path) -> None:
	"""Require the stable CLI wrapper to forward literal caller arguments."""
	launch = subprocess.run(
		(str(repository / "build/bin/ferrum"), "--help"), cwd=repository,
		env=_fake_build_environment(repository, "success"), check=False,
		capture_output=True, text=True,
	)
	if launch.returncode != 0 or launch.stdout.strip() != "fake-cli-help":
		raise BuildCleanupE2eError(
			"the stable CLI launcher did not forward --help to its payload: "
			f"{launch.stderr.strip()}"
		)


#============================================
def _write_malformed_noncurrent_programs(repository: Path) -> tuple[Path, ...]:
	"""Seed missing, non-regular, and unreadable leases for owned program roots."""
	programs_root = repository / "build/programs"
	missing_lease = programs_root / "program-missing-lease"
	nonregular_lease = programs_root / "program-nonregular-lease"
	unreadable_lease = programs_root / "program-unreadable-lease"
	for program in (missing_lease, nonregular_lease, unreadable_lease):
		(program / "bin").mkdir(parents=True)
		(program / "bin/ferrum.program").write_text("legacy payload\n", encoding="utf-8")
	(nonregular_lease / ".ferrum-runtime.lease").mkdir()
	lease = unreadable_lease / ".ferrum-runtime.lease"
	lease.touch()
	lease.chmod(0)
	if os.access(lease, os.R_OK):
		raise BuildCleanupE2eError("the platform cannot create an unreadable lease fixture")
	return missing_lease, nonregular_lease, unreadable_lease


#============================================
def _write_immutable_native_engine_temp(repository: Path) -> Path:
	"""Seed content that proves stable runtime paths never mutate a published root."""
	path = repository / "build/current/runtime/.native-engine-immutable"
	path.mkdir()
	sentinel = path / "sentinel"
	sentinel.write_text("published program content\n", encoding="utf-8")
	return sentinel


#============================================
def _write_unrelated_programs_content(repository: Path) -> Path:
	"""Seed content beside owned roots that local-build cleanup must ignore."""
	path = repository / "build/programs/unrelated-directory/sentinel"
	path.parent.mkdir()
	path.write_text("not an owned program root\n", encoding="utf-8")
	return path


#============================================
def _write_obsolete_build_owned_outputs(repository: Path) -> None:
	"""Seed interrupted compiler and staging outputs owned by the local build."""
	build_root = repository / "build"
	for path in (
		repository / "output_native_wheel",
		build_root / ".cargo-target",
		build_root / ".ferrum-local-build-interrupted",
		repository / "packages/ferrum-rust/target",
	):
		path.mkdir(parents=True)
		(path / "stale").write_text("obsolete build output\n", encoding="utf-8")


#============================================
def _write_obsolete_direct_layout(repository: Path) -> None:
	"""Seed disposable pre-current local build artifacts without a runtime lease."""
	legacy_runtime = repository / "build/runtime/python/known-good"
	legacy_runtime.parent.mkdir(parents=True, exist_ok=True)
	legacy_runtime.write_text("obsolete direct runtime\n", encoding="utf-8")
	legacy_cli = repository / "build/bin/ferrum"
	legacy_cli.parent.mkdir(exist_ok=True)
	legacy_cli.write_text("obsolete direct CLI launcher\n", encoding="utf-8")
	legacy_gui = repository / "build/bin/ferrum-qt"
	legacy_gui.write_text("obsolete direct GUI launcher\n", encoding="utf-8")


#============================================
def _require_obsolete_direct_layout_retired(repository: Path) -> None:
	"""Require obsolete direct outputs to be replaced by a fresh leased program."""
	build_root = repository / "build"
	if (build_root / "runtime/python/known-good").exists():
		raise BuildCleanupE2eError("obsolete direct runtime survived local-build staging")
	for launcher in (build_root / "bin/ferrum", build_root / "bin/ferrum-qt"):
		if "obsolete direct" in launcher.read_text(encoding="utf-8"):
			raise BuildCleanupE2eError("obsolete direct launcher survived local-build staging")
	selected_root = (build_root / "current").resolve()
	if not (selected_root / ".ferrum-runtime.lease").is_file():
		raise BuildCleanupE2eError("fresh local program lacks its runtime lease")


#============================================
def _run_fake_build(
	repository: Path, receipt_mode: str, *, fail_after_pointer_stage: bool = False,
) -> subprocess.CompletedProcess[str]:
	"""Run the real local build lifecycle against its inline fake tool boundary."""
	return subprocess.run(
		(str(repository / "build.sh"),), cwd=repository,
		env=_fake_build_environment(repository, receipt_mode, fail_after_pointer_stage),
		check=False, capture_output=True, text=True,
	)


#============================================
def _fake_build_environment(
	repository: Path, receipt_mode: str, fail_after_pointer_stage: bool = False,
) -> dict[str, str]:
	"""Return the controlled process environment for one copied build lifecycle."""
	return os.environ | {
		"FERRUM_FAKE_RECEIPT_MODE": receipt_mode,
		"FERRUM_FAKE_REPOSITORY": str(repository),
		"FERRUM_LOCAL_RUNTIME_SOURCE_ROOT": str(_rust_source_root()),
		"PATH": f"{repository / 'fake-bin'}:/usr/bin:/bin:/usr/sbin",
		"PYTHONPATH": ":".join((
			str(repository / "caller-python-one"),
			str(repository / "caller-python-two"),
		)),
		"PYTHONDONTWRITEBYTECODE": "1",
		"FERRUM_BUILD_FAIL_AFTER_POINTER_STAGE": "1" if fail_after_pointer_stage else "",
	}


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
	for caller_entry in ("caller-python-one", "caller-python-two"):
		(repository / caller_entry).mkdir()
	(repository / "source_me.sh").write_text(
		"FERRUM_CALLER_PYTHONPATH=\"${PYTHONPATH-}\"\n"
		"export PYTHONPATH=\"${BASH_SOURCE[0]%/*}/packages/ferrum-chem-qt.app:"
		"${BASH_SOURCE[0]%/*}/build/current/runtime/python"
		"${FERRUM_CALLER_PYTHONPATH:+:${FERRUM_CALLER_PYTHONPATH}}\"\n"
		"export PYTHONUNBUFFERED=1\nexport PYTHONDONTWRITEBYTECODE=1\n"
		"export FERRUM_FAKE_SOURCE_ME_READY=1\n", encoding="utf-8"
	)
	rust_root = repository / "packages/ferrum-rust"
	engine_lib = rust_root / "engine_lib"
	engine_lib.mkdir(parents=True)
	shutil.copy2(_rust_source_root() / "engine_lib/local_runtime_launcher.py",
		engine_lib / "local_runtime_launcher.py")
	qt_package = repository / "packages/ferrum-chem-qt.app/ferrum_qt"
	qt_package.mkdir(parents=True)
	(qt_package / "__init__.py").write_text("", encoding="utf-8")
	(qt_package / "__main__.py").write_text(
		"import json\nimport sys\nfrom pathlib import Path\n"
		"import ferrum_qt\n"
		"print(json.dumps({'ferrum_qt': str(Path(ferrum_qt.__file__).resolve()), "
		"'sys_path': sys.path}))\n", encoding="utf-8"
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
		"    if os.environ.get('FERRUM_FAKE_SOURCE_ME_READY') != '1': raise SystemExit(2)\n"
		"    mode = os.environ['FERRUM_FAKE_RECEIPT_MODE']\n"
		"    candidate = '/.staging-' in str(args.runtime_root)\n"
		"    if mode == 'candidate-failure' and candidate: raise SystemExit(1)\n",
		encoding="utf-8",
	)
	fake_bin = repository / "fake-bin"
	fake_bin.mkdir()
	(fake_bin / "cargo").write_text(
		"#!/usr/bin/env bash\nset -euo pipefail\n"
		"if [[ \"${FERRUM_FAKE_RECEIPT_MODE}\" == \"lock-hold\" ]]; then\n"
		"  : \"${FERRUM_FAKE_LOCK_READY:?}\"\n"
		"  : >\"${FERRUM_FAKE_LOCK_READY}\"\n"
		"  printf 'lock-ready\\n' >&2\n"
		"  while [[ ! -e \"${FERRUM_FAKE_LOCK_READY%/*}/lock-release\" ]]; do sleep 0.05; done\n"
		"fi\n"
		"mkdir -p \"${CARGO_TARGET_DIR}/release\"\n"
		"printf '%s\\n' '#!/usr/bin/env bash' 'set -euo pipefail' "
		"'if [[ -n \"${FERRUM_LEASE_READY:-}\" ]]; then' "
		"'  printf \"runtime-lease-ready\\n\"' '  read -r _' 'fi' "
		"'if [[ \"${1:-}\" == \"--help\" ]]; then' "
		"'  printf \"fake-cli-help\\n\"' 'fi' "
		">\"${CARGO_TARGET_DIR}/release/ferrum\"\n"
		"chmod 755 \"${CARGO_TARGET_DIR}/release/ferrum\"\n"
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
