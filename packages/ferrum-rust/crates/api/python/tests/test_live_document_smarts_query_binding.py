"""Opaque installed-wheel contract for the private live-document SMARTS bridge."""

from __future__ import annotations

import json
import math
import os
import hashlib
import stat
from pathlib import Path
import re
import zipfile

import pytest


def _sealed_wheel_root() -> Path:
	configured = os.environ.get("FERRUM_SMARTS_SEALED_WHEEL_ROOT")
	if not configured:
		pytest.skip(
			"requires FERRUM_SMARTS_SEALED_WHEEL_ROOT naming a freshly installed sealed ABI-5 wheel",
			allow_module_level=True,
		)
	root = Path(configured).resolve()
	if not root.is_dir():
		raise RuntimeError(
			"FERRUM_SMARTS_SEALED_WHEEL_ROOT is not an installed sealed-wheel root"
		)
	return root


SEALED_WHEEL_ROOT = _sealed_wheel_root()


def _configured_digest(name: str) -> str:
	value = os.environ.get(name)
	if not value or not re.fullmatch(r"[0-9a-f]{64}", value):
		raise RuntimeError(
			f"{name} must name the immutable lowercase SHA-256 provenance for "
			"the configured sealed-wheel harness"
		)
	return value


def _configured_wheel() -> Path:
	configured = os.environ.get("FERRUM_SMARTS_SEALED_WHEEL_PATH")
	if not configured:
		raise RuntimeError(
			"FERRUM_SMARTS_SEALED_WHEEL_PATH is required when "
			"FERRUM_SMARTS_SEALED_WHEEL_ROOT is configured"
		)
	wheel = Path(configured).resolve()
	if not wheel.is_file() or wheel.is_symlink() or wheel.suffix != ".whl":
		raise RuntimeError("FERRUM_SMARTS_SEALED_WHEEL_PATH must be a regular wheel file")
	return wheel


def _sha256(path: Path) -> str:
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


def _archive_native_members(archive: zipfile.ZipFile) -> set[str]:
	"""Return the closed regular top-level `.dylibs` member set from one wheel."""
	member_names: set[str] = set()
	for info in archive.infolist():
		archive_name = info.filename
		if archive_name == ".dylibs/":
			continue
		if not archive_name.startswith(".dylibs/"):
			continue
		relative = archive_name.removeprefix(".dylibs/")
		# A wheel native closure has no nested paths, duplicate members, directories,
		# or links. Reject each form before set comparison can conceal it.
		if (
			not relative
			or "/" in relative
			or "\\" in relative
			or relative in {".", ".."}
			or info.is_dir()
			or stat.S_IFMT(info.external_attr >> 16) == stat.S_IFLNK
			or relative in member_names
		):
			raise RuntimeError(f"configured wheel has an unsafe native member: {archive_name}")
		member_names.add(relative)
	return member_names


def _installed_native_members(wheel_dylibs: Path) -> set[str]:
	"""Return the closed regular top-level `.dylibs` member set from one install."""
	if wheel_dylibs.is_symlink() or not wheel_dylibs.is_dir():
		raise RuntimeError("installed wheel has no regular native library closure")
	resolved_dylibs = wheel_dylibs.resolve()
	member_names: set[str] = set()
	for member in wheel_dylibs.iterdir():
		# Directory entries must be regular direct children. This refuses a nested
		# payload, a symlink escape, and every non-library extra before comparison.
		if (
			member.is_symlink()
			or not member.is_file()
			or not stat.S_ISREG(member.stat().st_mode)
			or member.resolve().parent != resolved_dylibs
		):
			raise RuntimeError(f"installed wheel has an unsafe native member: {member.name}")
		member_names.add(member.name)
	return member_names


def _require_exact_native_closure(
	archive: zipfile.ZipFile,
	wheel_dylibs: Path,
	expected_names: set[str],
) -> None:
	"""Require the wheel archive and installed package to equal the manifest closure."""
	archive_names = _archive_native_members(archive)
	installed_names = _installed_native_members(wheel_dylibs)
	if archive_names != expected_names:
		raise RuntimeError("configured wheel native closure differs from bundle manifest")
	if installed_names != expected_names:
		raise RuntimeError("installed wheel native closure differs from bundle manifest")


SEALED_WHEEL = _configured_wheel()
SEALED_WHEEL_SHA256 = _configured_digest("FERRUM_SMARTS_SEALED_WHEEL_SHA256")
SEALED_BUNDLE_MANIFEST_SHA256 = _configured_digest(
	"FERRUM_SMARTS_SEALED_BUNDLE_MANIFEST_SHA256"
)

import ferrum_chem


def _require_fresh_abi5_bundle() -> None:
	extension_path = Path(ferrum_chem.__file__).resolve()
	if SEALED_WHEEL_ROOT not in extension_path.parents:
		raise RuntimeError(
			"ferrum_chem was not imported from FERRUM_SMARTS_SEALED_WHEEL_ROOT; "
			"source fallback is forbidden"
		)
	if _sha256(SEALED_WHEEL) != SEALED_WHEEL_SHA256:
		raise RuntimeError("configured sealed wheel does not match its expected SHA-256")
	bundle_candidates = (
		extension_path.parent / "ferrum-engine-bundle",
		extension_path.parent.parent / "ferrum-engine-bundle",
	)
	manifests = [
		candidate / "ferrum-engine-bundle-v1.json"
		for candidate in bundle_candidates
		if (candidate / "ferrum-engine-bundle-v1.json").is_file()
	]
	if len(manifests) != 1:
		raise RuntimeError("installed wheel must contain exactly one sealed engine-bundle manifest")
	manifest_path = manifests[0]
	bundle = manifest_path.parent
	if manifest_path.is_symlink() or bundle.is_symlink() or not stat.S_ISREG(manifest_path.stat().st_mode):
		raise RuntimeError("installed sealed bundle manifest must be a regular non-symlink file")
	if _sha256(manifest_path) != SEALED_BUNDLE_MANIFEST_SHA256:
		raise RuntimeError("installed sealed bundle manifest does not match its expected SHA-256")
	manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
	if set(manifest) != {"schema", "target", "adapter_abi_version", "adapter", "members"}:
		raise RuntimeError("installed sealed bundle manifest has an unexpected shape")
	if (
		manifest["schema"] != "ferrum-engine-bundle-v1"
		or manifest["adapter_abi_version"] != 5
		or manifest["adapter"] != "libferrum_chem.dylib"
		or not isinstance(manifest["members"], list)
	):
		raise RuntimeError("installed sealed bundle is not the required ABI-5 SMARTS closure")
	members = manifest["members"]
	if not members or not all(isinstance(member, dict) and set(member) == {"path", "sha256"} for member in members):
		raise RuntimeError("installed sealed bundle members have an unexpected shape")
	expected_members = {"ferrum-engine-bundle-v1.json"}
	for member in members:
		name, digest = member["path"], member["sha256"]
		if (
			not isinstance(name, str)
			or Path(name).name != name
			or not name
			or not isinstance(digest, str)
			or not re.fullmatch(r"[0-9a-f]{64}", digest)
			or name in expected_members
		):
			raise RuntimeError("installed sealed bundle has an unsafe member record")
		expected_members.add(name)
		member_path = bundle / name
		if member_path.is_symlink() or not member_path.is_file() or not stat.S_ISREG(member_path.stat().st_mode):
			raise RuntimeError(f"installed sealed bundle member is not regular: {name}")
		if _sha256(member_path) != digest:
			raise RuntimeError(f"installed sealed bundle member digest mismatch: {name}")
	actual_members = {entry.name for entry in bundle.iterdir()}
	if actual_members != expected_members:
		raise RuntimeError("installed sealed bundle has missing or extra members")
	adapter = bundle / "libferrum_chem.dylib"
	try:
		import ctypes
		library = ctypes.CDLL(str(adapter))
		library.ferrum_chem_abi_version.restype = ctypes.c_uint32
		library.ferrum_chem_capabilities_v1.restype = ctypes.c_uint64
	except (AttributeError, OSError) as error:
		raise RuntimeError("installed adapter lacks the required ABI-5 symbols") from error
	if library.ferrum_chem_abi_version() != 5 or not (library.ferrum_chem_capabilities_v1() & 0x1000):
		raise RuntimeError("installed adapter does not expose ABI-5 SMARTS-match capability")
	try:
		extension_member = extension_path.relative_to(SEALED_WHEEL_ROOT).as_posix()
	except ValueError as error:
		raise RuntimeError("installed wheel paths escape the configured wheel root") from error
	wheel_dylibs = SEALED_WHEEL_ROOT / ".dylibs"
	with zipfile.ZipFile(SEALED_WHEEL) as archive:
		expected_native_names = expected_members - {"ferrum-engine-bundle-v1.json"}
		_require_exact_native_closure(archive, wheel_dylibs, expected_native_names)
		required_archive_names = {extension_member}
		required_archive_names.update(f".dylibs/{name}" for name in expected_native_names)
		for archive_name in required_archive_names:
			try:
				info = archive.getinfo(archive_name)
			except KeyError as error:
				raise RuntimeError(
				"configured wheel does not contain the imported extension and native closure"
			) from error
			if stat.S_IFMT(info.external_attr >> 16) == stat.S_IFLNK:
				raise RuntimeError(f"configured wheel contains a symbolic native member: {archive_name}")
			installed = SEALED_WHEEL_ROOT / archive_name
			if installed.is_symlink() or not installed.is_file() or not stat.S_ISREG(installed.stat().st_mode):
				raise RuntimeError(f"installed wheel provenance member is not regular: {archive_name}")
			if hashlib.sha256(archive.read(archive_name)).hexdigest() != _sha256(installed):
				raise RuntimeError(f"configured wheel differs from installed provenance member: {archive_name}")
			if archive_name.startswith(".dylibs/"):
				bundle_member = bundle / Path(archive_name).name
				if _sha256(bundle_member) != _sha256(installed):
					raise RuntimeError(f"configured wheel differs from supplied bundle member: {archive_name}")


_require_fresh_abi5_bundle()


def test_configured_invalid_sealed_wheel_root_fails(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
	"""A supplied but missing harness root is a provenance failure, never a skip."""
	missing_root = tmp_path / "missing-installed-wheel"
	monkeypatch.setenv("FERRUM_SMARTS_SEALED_WHEEL_ROOT", str(missing_root))
	with pytest.raises(RuntimeError, match="^FERRUM_SMARTS_SEALED_WHEEL_ROOT"):
		_sealed_wheel_root()


@pytest.mark.parametrize("extra_location", ["archive", "installed"])
def test_native_closure_rejects_extra_wheel_or_installed_member(
	tmp_path: Path,
	extra_location: str,
) -> None:
	"""The sealed manifest names every native library, so extra members are forbidden."""
	wheel_dylibs = tmp_path / ".dylibs"
	wheel_dylibs.mkdir()
	declared = wheel_dylibs / "libdeclared.dylib"
	declared.write_bytes(b"declared")
	if extra_location == "installed":
		(wheel_dylibs / "libextra.dylib").write_bytes(b"extra")
	wheel = tmp_path / "fixture.whl"
	with zipfile.ZipFile(wheel, "w") as archive:
		archive.writestr(".dylibs/libdeclared.dylib", b"declared")
		if extra_location == "archive":
			archive.writestr(".dylibs/libextra.dylib", b"extra")
	with zipfile.ZipFile(wheel) as archive:
		with pytest.raises(RuntimeError, match="native closure differs from bundle manifest"):
			_require_exact_native_closure(archive, wheel_dylibs, {"libdeclared.dylib"})


@pytest.mark.parametrize("missing_location", ["archive", "installed"])
def test_native_closure_rejects_missing_wheel_or_installed_member(
	tmp_path: Path,
	missing_location: str,
) -> None:
	"""A manifest-declared native member may not be omitted on either side."""
	wheel_dylibs = tmp_path / ".dylibs"
	wheel_dylibs.mkdir()
	if missing_location != "installed":
		(wheel_dylibs / "libdeclared.dylib").write_bytes(b"declared")
	wheel = tmp_path / "fixture.whl"
	with zipfile.ZipFile(wheel, "w") as archive:
		if missing_location != "archive":
			archive.writestr(".dylibs/libdeclared.dylib", b"declared")
	with zipfile.ZipFile(wheel) as archive:
		with pytest.raises(RuntimeError, match="native closure differs from bundle manifest"):
			_require_exact_native_closure(archive, wheel_dylibs, {"libdeclared.dylib"})


def test_native_closure_rejects_nested_archive_member_and_installed_symlink_escape(
	tmp_path: Path,
) -> None:
	"""Native closure entries must be direct regular files, never paths or links."""
	wheel_dylibs = tmp_path / ".dylibs"
	wheel_dylibs.mkdir()
	declared = wheel_dylibs / "libdeclared.dylib"
	declared.write_bytes(b"declared")
	nested_wheel = tmp_path / "nested.whl"
	with zipfile.ZipFile(nested_wheel, "w") as archive:
		archive.writestr(".dylibs/libdeclared.dylib", b"declared")
		archive.writestr(".dylibs/nested/libescape.dylib", b"escape")
	with zipfile.ZipFile(nested_wheel) as archive:
		with pytest.raises(RuntimeError, match="unsafe native member"):
			_require_exact_native_closure(archive, wheel_dylibs, {"libdeclared.dylib"})

	linked_wheel = tmp_path / "linked.whl"
	with zipfile.ZipFile(linked_wheel, "w") as archive:
		archive.writestr(".dylibs/libdeclared.dylib", b"declared")
	(wheel_dylibs / "libescape.dylib").symlink_to(declared)
	with zipfile.ZipFile(linked_wheel) as archive:
		with pytest.raises(RuntimeError, match="unsafe native member"):
			_require_exact_native_closure(archive, wheel_dylibs, {"libdeclared.dylib"})


SOURCE = ('<cdml><molecule id="m"><atom id="a" name="C">'
	'<point x="1" y="2"/></atom></molecule></cdml>')


def _session() -> ferrum_chem.DocumentSession:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	session._publish_live_render_plan_v1(session.snapshot().revision)
	return session


def _unavailable(call: object) -> None:
	with pytest.raises(RuntimeError, match="^SMARTS match is unavailable$"):
		call()


def test_live_smarts_receipts_are_opaque_one_use_and_bound_to_session_epoch_and_lifecycle() -> None:
	first, second = _session(), _session()
	run = first._run_live_document_smarts_query_v1("[#6]", 1, 1)
	assert run.traversal == "complete"
	assert [(item.source_order, item.match_count, item.completeness) for item in run.molecules] == [(0, 1, "complete")]
	receipt = run.receipt
	for name in ("issuer", "key", "query", "rows", "graph", "record_id"):
		assert not hasattr(receipt, name)
		with pytest.raises((AttributeError, TypeError)):
			setattr(receipt, name, "forged")
	for secret in (SOURCE, "[#6]", "issuer", "key", "libferrum_chem", ".dylibs"):
		assert secret not in repr(receipt) and secret not in str(receipt)
	_unavailable(lambda: second._show_live_document_smarts_match_v1(receipt, 0))
	paint = first._show_live_document_smarts_match_v1(receipt, 0)
	assert tuple(paint.atom_bounds) == ((-7.0, -6.0, 9.0, 10.0),)
	assert all(math.isfinite(value) for bounds in paint.atom_bounds for value in bounds)
	_unavailable(lambda: first._show_live_document_smarts_match_v1(receipt, 0))

	stale = first._run_live_document_smarts_query_v1("[#6]", 1, 1).receipt
	first._run_live_document_smarts_query_v1("[#6]", 1, 1)
	_unavailable(lambda: first._show_live_document_smarts_match_v1(stale, 0))
	retired = first._run_live_document_smarts_query_v1("[#6]", 1, 1).receipt
	first._publish_live_render_plan_v1(first.snapshot().revision)
	_unavailable(lambda: first._show_live_document_smarts_match_v1(retired, 0))


def test_live_smarts_receipt_only_retirement_preserves_plan_and_refuses_old_receipts() -> None:
	"""Query cleanup revokes capabilities without making an unchanged drawing unqueryable."""
	session = _session()
	old_raw = session._run_live_document_smarts_query_v1("[#6]", 1, 1).receipt
	session._retire_live_document_smarts_receipts_v1()
	_unavailable(lambda: session._show_live_document_smarts_match_v1(old_raw, 0))
	fresh_raw = session._run_live_document_smarts_query_v1("[#6]", 1, 1).receipt
	assert tuple(session._show_live_document_smarts_match_v1(fresh_raw, 0).atom_bounds) == (
		(-7.0, -6.0, 9.0, 10.0),
	)

	_, old_selected_token = _selected_query_token(session)
	old_selected = session._run_live_document_smarts_query_v1(old_selected_token, 1, 1).receipt
	session._retire_live_document_smarts_receipts_v1()
	_unavailable(lambda: session._show_live_document_smarts_match_v1(old_selected, 0))
	_, fresh_selected_token = _selected_query_token(session)
	fresh_selected = session._run_live_document_smarts_query_v1(fresh_selected_token, 1, 1).receipt
	assert tuple(session._show_live_document_smarts_match_v1(fresh_selected, 0).atom_bounds) == (
		(-7.0, -6.0, 9.0, 10.0),
	)

	session._retire_live_document_smarts_query_v1()
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		session._run_live_document_smarts_query_v1("[#6]", 1, 1)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.plan_not_published
	retired = first._run_live_document_smarts_query_v1("[#6]", 1, 1).receipt
	first._retire_live_document_smarts_query_v1()
	_unavailable(lambda: first._show_live_document_smarts_match_v1(retired, 0))


def _selected_query_token(session: ferrum_chem.DocumentSession) -> tuple[object, object]:
	"""Capture one private SMARTS token without exposing its generic root facts."""
	snapshot = session.snapshot()
	observation = session.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
	selection = session.select_render_interaction_roots_v1(
		observation, None, ferrum_chem.RenderInteractionQueryV1.root("m"),
	)
	return selection, session._capture_live_document_smarts_selected_query_v1(selection)


def test_live_smarts_selected_tokens_refuse_before_native_dispatch() -> None:
	"""Selection failures are closed without consulting the native matcher."""
	first, second = _session(), _session()
	selection, selected = _selected_query_token(first)
	for secret in (SOURCE, "m", "[#6]", "issuer", "selection"):
		assert secret not in repr(selected) and secret not in str(selected)
	with pytest.raises((TypeError, AttributeError)):
		json.dumps(selected)

	extension = Path(ferrum_chem.__file__).resolve()
	manifests = list(extension.parent.glob("ferrum-engine-bundle/ferrum-engine-bundle-v1.json"))
	if not manifests:
		manifests = list(extension.parent.parent.glob("ferrum-engine-bundle/ferrum-engine-bundle-v1.json"))
	assert len(manifests) == 1
	adapter = manifests[0].parent / "libferrum_chem.dylib"
	disabled = adapter.with_name(adapter.name + ".selected-token-e2e-disabled")
	assert adapter.is_file() and not adapter.is_symlink() and not disabled.exists()
	adapter.rename(disabled)
	try:
		with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
			second._run_live_document_smarts_query_v1(selected, 1, 1)
		assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.foreign_selection
		assert caught.value.category != ferrum_chem.LiveDocumentSmartsCategoryV1.unavailable

		gesture = first.begin_render_interaction_translation_v1(
			selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
		)
		preview = first.preview_render_interaction_translation_v1(gesture, 3.0, 2.0)
		first.commit_render_interaction_translation_v1(gesture, preview)
		with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
			first._run_live_document_smarts_query_v1(selected, 1, 1)
		assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.stale_selection
		assert caught.value.category != ferrum_chem.LiveDocumentSmartsCategoryV1.unavailable

		multi = ferrum_chem.DocumentSession.load(
			'<cdml><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/>'
			'</atom></molecule><molecule id="n"><atom id="b" name="C"><point x="8" y="2"/>'
			'</atom></molecule></cdml>',
		)
		multi._publish_live_render_plan_v1(multi.snapshot().revision)
		snapshot = multi.snapshot()
		observation = multi.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
		selected_m = multi.select_render_interaction_roots_v1(
			observation, None, ferrum_chem.RenderInteractionQueryV1.root("m"),
		)
		selected_both = multi.select_render_interaction_roots_v1(
			observation, selected_m, ferrum_chem.RenderInteractionQueryV1.root(
				"n", ferrum_chem.RenderInteractionModifierV1.toggle,
			),
		)
		multiple = multi._capture_live_document_smarts_selected_query_v1(selected_both)
		with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
			multi._run_live_document_smarts_query_v1(multiple, 1, 1)
		assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.selected_root_multiple
		assert caught.value.category != ferrum_chem.LiveDocumentSmartsCategoryV1.unavailable

		text = ferrum_chem.DocumentSession.load(
			'<cdml><text id="t"><point x="2" y="2"/><ftext>note</ftext></text></cdml>',
		)
		text._publish_live_render_plan_v1(text.snapshot().revision)
		snapshot = text.snapshot()
		observation = text.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
		text_selection = text.select_render_interaction_roots_v1(
			observation, None, ferrum_chem.RenderInteractionQueryV1.root("t"),
		)
		non_molecule = text._capture_live_document_smarts_selected_query_v1(text_selection)
		with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
			text._run_live_document_smarts_query_v1(non_molecule, 1, 1)
		assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.selected_source_not_molecule
		assert caught.value.category != ferrum_chem.LiveDocumentSmartsCategoryV1.unavailable

	finally:
		disabled.rename(adapter)


def test_live_smarts_selected_query_token_is_opaque_and_not_a_display_selection() -> None:
	session = _session()
	selection, token = _selected_query_token(session)
	for name in ("issuer", "selection", "roots", "identifier", "graph", "query", "__dict__"):
		assert not hasattr(token, name)
		with pytest.raises((AttributeError, TypeError)):
			setattr(token, name, "forged")
	for secret in (SOURCE, "m", "[#6]", "issuer", "selection"):
		assert secret not in repr(token) and secret not in str(token)
	run = session._run_live_document_smarts_query_v1(token, 1, 1)
	assert [(item.source_order, item.match_count) for item in run.molecules] == [(0, 1)]
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		session._run_live_document_smarts_query_v1(selection, 1, 1)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.selected_root_empty


def test_live_smarts_selected_receipt_redeems_rows_once_and_refuses_foreign_or_stale() -> None:
	"""Selected-query receipts preserve usable rows until their first successful paint."""
	cdml = (
		'<cdml><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/>'
		'</atom></molecule><molecule id="n"><atom id="b" name="C"><point x="8" y="2"/>'
		'</atom></molecule></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(cdml)
	session._publish_live_render_plan_v1(session.snapshot().revision)
	_, token = _selected_query_token(session)
	run = session._run_live_document_smarts_query_v1(token, 1, 2)
	assert [(item.source_order, item.match_count) for item in run.molecules] == [(0, 1), (1, 1)]
	assert tuple(session._show_live_document_smarts_match_v1(run.receipt, 0).atom_bounds) == (
		(-7.0, -6.0, 9.0, 10.0),
	)
	assert tuple(session._show_live_document_smarts_match_v1(run.receipt, 1).atom_bounds) == (
		(0.0, -6.0, 16.0, 10.0),
	)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		session._show_live_document_smarts_match_v1(run.receipt, 0)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.receipt_unavailable

	foreign = ferrum_chem.DocumentSession.load(cdml)
	foreign._publish_live_render_plan_v1(foreign.snapshot().revision)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		foreign._show_live_document_smarts_match_v1(run.receipt, 0)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.receipt_unavailable

	stale = session._run_live_document_smarts_query_v1(token, 1, 2)
	session._publish_live_render_plan_v1(session.snapshot().revision)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		session._show_live_document_smarts_match_v1(stale.receipt, 0)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.receipt_unavailable

	mutation = ferrum_chem.DocumentSession.load(cdml)
	mutation._publish_live_render_plan_v1(mutation.snapshot().revision)
	selection, mutation_token = _selected_query_token(mutation)
	pending = mutation._run_live_document_smarts_query_v1(mutation_token, 1, 2)
	gesture = mutation.begin_render_interaction_translation_v1(
		selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	preview = mutation.preview_render_interaction_translation_v1(gesture, 3.0, 2.0)
	mutation.commit_render_interaction_translation_v1(gesture, preview)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		mutation._show_live_document_smarts_match_v1(pending.receipt, 0)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.stale_document


def test_live_smarts_selected_query_tokens_reject_foreign_stale_and_multiple_roots() -> None:
	first, second = _session(), _session()
	_, foreign = _selected_query_token(first)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		second._run_live_document_smarts_query_v1(foreign, 1, 1)
	assert (
		caught.value.category,
		caught.value.reason,
		caught.value.recovery,
	) == (
		ferrum_chem.LiveDocumentSmartsCategoryV1.refused,
		ferrum_chem.LiveDocumentSmartsReasonV1.foreign_selection,
		ferrum_chem.LiveDocumentSmartsRecoveryV1.select_one_molecule,
	)

	selection, stale = _selected_query_token(first)
	gesture = first.begin_render_interaction_translation_v1(
		selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	preview = first.preview_render_interaction_translation_v1(gesture, 3.0, 2.0)
	first.commit_render_interaction_translation_v1(gesture, preview)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		first._run_live_document_smarts_query_v1(stale, 1, 1)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.stale_selection

	multi = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/>'
		'</atom></molecule><molecule id="n"><atom id="b" name="C"><point x="8" y="2"/>'
		'</atom></molecule></cdml>',
	)
	multi._publish_live_render_plan_v1(multi.snapshot().revision)
	snapshot = multi.snapshot()
	observation = multi.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
	selected_m = multi.select_render_interaction_roots_v1(
		observation, None, ferrum_chem.RenderInteractionQueryV1.root("m"),
	)
	selected_both = multi.select_render_interaction_roots_v1(
		observation, selected_m, ferrum_chem.RenderInteractionQueryV1.root(
			"n", ferrum_chem.RenderInteractionModifierV1.toggle,
		),
	)
	multiple = multi._capture_live_document_smarts_selected_query_v1(selected_both)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		multi._run_live_document_smarts_query_v1(multiple, 1, 1)
	assert caught.value.reason == ferrum_chem.LiveDocumentSmartsReasonV1.selected_root_multiple
def test_live_smarts_selected_query_joins_authored_source_identity() -> None:
    session = ferrum_chem.DocumentSession.load(
        '<cdml><molecule id="carbon-root"><atom id="carbon-atom" name="C"><point x="1" y="2"/>'
        '</atom></molecule><molecule id="selected-oxygen"><atom id="oxygen-atom" name="O"><point x="8" y="2"/>'
        '</atom></molecule></cdml>',
    )
    session._publish_live_render_plan_v1(session.snapshot().revision)
    snapshot = session.snapshot()
    observation = session.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
    selection = session.select_render_interaction_roots_v1(
        observation,
        None,
        ferrum_chem.RenderInteractionQueryV1.root("selected-oxygen"),
    )
    token = session._capture_live_document_smarts_selected_query_v1(selection)

    run = session._run_live_document_smarts_query_v1(token, 1, 1)
    assert [(item.source_order, item.match_count) for item in run.molecules] == [(1, 1)]
    for item in run.molecules:
        for name in ("source_id", "durable_id", "identifier", "graph", "query"):
            assert not hasattr(item, name)
        for secret in ("selected-oxygen", "oxygen-atom", "carbon-root", "carbon-atom", "[#8]"):
            assert secret not in repr(item) and secret not in str(item)
