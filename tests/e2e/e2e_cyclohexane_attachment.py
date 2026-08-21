"""Prove the installed native wheel drives one attached cyclohexane Qt workflow."""

from __future__ import annotations

# Standard-library imports.
import argparse
import hashlib
import json
import os
import pathlib
import re
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
class CyclohexaneAttachmentE2eError(RuntimeError):
	"""Raised when the installed-wheel C6 workflow has incomplete evidence."""


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
	"""Run one local child and preserve its diagnostics on failure."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise CyclohexaneAttachmentE2eError(
			"command failed (%d): %s\nstdout:\n%s\nstderr:\n%s" % (
				result.returncode, " ".join(command), result.stdout, result.stderr,
			),
		)
	return result.stdout


#============================================
def scrubbed_environment() -> dict[str, str]:
	"""Return an offscreen local environment without ambient native imports."""
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
			if re.fullmatch(r"ferrum_chem[^/]*\.so", name)
		]
		if len(members) != 1:
			raise CyclohexaneAttachmentE2eError(
				f"wheel must contain exactly one direct ferrum_chem extension, found {members!r}",
			)
		return hashlib.sha256(archive.read(members[0])).hexdigest()


CHILD_PROGRAM = r'''
import hashlib
import importlib.machinery
import json
import pathlib
import re
import sys

qt_root = pathlib.Path(sys.argv[1]).resolve()
expected_extension_digest = sys.argv[2]
sys.path.insert(0, str(qt_root))

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import ferrum_chem
import ferrum_qt
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window
import ferrum_qt.modes.base_mode


def fail(message):
	raise RuntimeError(message)


def trigger_visible_menu_action(window, label, app):
	"""Prove one visible menu QAction, then activate its real command contract."""
	def passive_phase_snapshot(action):
		"""Capture passive C6 ownership facts without settling or changing state."""
		return {
			"action_checked": action.isChecked(),
			"active_mode": enum_name(window._mode_manager.active_mode_id),
			"intent": intent_facts(window._line_gesture_intent),
			"trace": list(startup_trace["action_events"]),
		}

	def require_passive_phase(phase, action, before):
		"""Reject one menu-discovery phase that dispatched or armed C6."""
		after = passive_phase_snapshot(action)
		if (
			after["action_checked"]
			or after["active_mode"] is not None
			or after["intent"]["present"]
			or after["trace"] != before["trace"]
		):
			previous_trace = before["trace"]
			current_trace = after["trace"]
			trace_delta = (
				current_trace[len(previous_trace):]
				if current_trace[:len(previous_trace)] == previous_trace
				else {"before": previous_trace, "after": current_trace}
			)
			fail("passive menu phase changed C6 ownership: %s" % json.dumps({
				"event_delta": trace_delta,
				"phase": phase,
				"post_phase": after,
				"pre_phase": before,
			}, sort_keys=True))
		return after

	attach_action = window._attach_cyclohexane_ring_action
	before_menu_scan = passive_phase_snapshot(attach_action)
	matches = []
	for menu_action in window.menuBar().actions():
		menu = menu_action.menu()
		if menu is not None:
			matches.extend(
				(menu_action, menu, candidate)
				for candidate in menu.actions()
				if candidate.text().replace("&", "") == label
			)
	after_menu_traversal = require_passive_phase(
		"after_menu_traversal", attach_action, before_menu_scan,
	)
	if len(matches) != 1:
		fail("menu hierarchy must contain exactly one attach action: %s" % json.dumps({
			"label": label,
			"matches": len(matches),
		}, sort_keys=True))
	menu_action, menu, candidate = matches[0]
	action_group = candidate.actionGroup()
	after_action_group_lookup = require_passive_phase(
		"after_action_group_lookup", candidate, after_menu_traversal,
	)
	if (
		menu_action.text().replace("&", "") != "Edit"
		or candidate is not window._attach_cyclohexane_ring_action
		or candidate.text().replace("&", "") != "Attach Cyclohexane Ring"
		or not candidate.isVisible()
		or not candidate.isEnabled()
		or not candidate.isCheckable()
		or candidate.isChecked()
		or action_group is None
		or candidate not in action_group.actions()
	):
		fail("menu hierarchy does not own the expected attach QAction: %s" % json.dumps({
			"initial_action_state": initial_attach_action_state_facts(window, startup_trace),
			"candidate_checked": candidate.isChecked(),
			"candidate_checkable": candidate.isCheckable(),
			"candidate_enabled": candidate.isEnabled(),
			"candidate_is_window_attach_action": candidate is window._attach_cyclohexane_ring_action,
			"candidate_text": candidate.text(),
			"candidate_visible": candidate.isVisible(),
			"group_contains_candidate": action_group is not None and candidate in action_group.actions(),
			"group_present": action_group is not None,
			"menu_text": menu_action.text(),
		}, sort_keys=True))
	menu.popup(window.menuBar().mapToGlobal(window.menuBar().actionGeometry(menu_action).bottomLeft()))
	for _ in range(20):
		app.processEvents()
		if menu.isVisible():
			break
		PySide6.QtTest.QTest.qWait(10)
	if not menu.isVisible():
		fail("owning menu did not open before canonical QAction activation")
	menu.hide()
	for _ in range(20):
		app.processEvents()
		if not menu.isVisible():
			break
		PySide6.QtTest.QTest.qWait(10)
	if menu.isVisible():
		fail("owning menu did not close before canonical QAction activation")
	require_passive_phase("after_popup_close", candidate, after_action_group_lookup)
	candidate.trigger()
	dispatch_timer = PySide6.QtCore.QElapsedTimer()
	dispatch_timer.start()
	while True:
		app.processEvents()
		if candidate.isChecked() and window._line_gesture_intent is not None:
			break
		elapsed_ms = dispatch_timer.elapsed()
		if elapsed_ms >= 300:
			break
		PySide6.QtTest.QTest.qWait(min(10, 300 - elapsed_ms))
	if (
		not candidate.isChecked()
		or candidate is not window._attach_cyclohexane_ring_action
		or window._mode_manager.active_mode_id is not None
		or window._line_gesture_intent is None
		or window._line_gesture_intent.tool is not ferrum_qt.ferrum.line_tool_intent._NativeLineTool.ATTACH_CYCLOHEXANE_RING
	):
		active_popup = app.activePopupWidget()
		fail("shared attach QAction did not create its C6 line intent: %s" % json.dumps({
			"action_is_window_attach_action": candidate is window._attach_cyclohexane_ring_action,
			"action_checked": candidate.isChecked(),
			"active_mode": enum_name(window._mode_manager.active_mode_id),
			"active_popup_type": type(active_popup).__name__ if active_popup is not None else None,
			"active_popup_visible": active_popup.isVisible() if active_popup is not None else None,
			"elapsed_ms": dispatch_timer.elapsed(),
			"menu_visible": menu.isVisible(),
		}, sort_keys=True))


def graph_facts(tab, anchor):
	molecule = tab.current_document_observation().projection.molecules[0]
	atoms = {atom.source_id: atom for atom in molecule.atoms}
	if len(atoms) != 6 or set(atom.element for atom in atoms.values()) != {"C"}:
		fail("attachment did not produce six ordinary carbon atoms")
	if anchor not in atoms or len(molecule.bonds) != 6:
		fail("attachment did not preserve its anchor or produce six bonds")
	adjacency = {identifier: set() for identifier in atoms}
	for bond in molecule.bonds:
		if bond.source_type != "n1":
			fail("attachment produced a non-single bond")
		start = bond.start.source_id
		end = bond.end.source_id
		if start not in adjacency or end not in adjacency or start == end:
			fail("attachment produced an invalid bond endpoint")
		adjacency[start].add(end)
		adjacency[end].add(start)
	if any(len(neighbors) != 2 for neighbors in adjacency.values()):
		fail("attachment is not one shared-anchor C6 cycle")
	seen = {anchor}
	pending = [anchor]
	while pending:
		current = pending.pop()
		for neighbor in adjacency[current] - seen:
			seen.add(neighbor)
			pending.append(neighbor)
	if seen != set(atoms):
		fail("attachment cycle is not connected through the original anchor")
	return {"atom_count": len(atoms), "bond_count": len(molecule.bonds), "anchor_degree": len(adjacency[anchor])}


def source_isolation():
	line_tools = (qt_root / "ferrum_qt" / "ferrum" / "line_tools.py").resolve()
	controller = (qt_root / "ferrum_qt" / "ferrum" / "attached_cyclohexane_tab.py").resolve()
	public_tab_methods = (
		"begin_attached_cyclohexane", "preview_attached_cyclohexane",
		"commit_attached_cyclohexane", "cancel_attached_cyclohexane",
	)
	private_session_methods = (
		"_begin_attach_cyclohexane_v1", "_preview_attach_cyclohexane_v1",
		"_commit_attach_cyclohexane_v1", "_cancel_attach_cyclohexane_v1",
	)
	for path in (qt_root / "ferrum_qt").rglob("*.py"):
		source = path.read_text(encoding="utf-8")
		public_calls = tuple(
			method for method in public_tab_methods
			if re.search(r"\.%s\s*\(" % re.escape(method), source)
		)
		private_calls = tuple(
			method for method in private_session_methods
			if re.search(r"\.%s\s*\(" % re.escape(method), source)
		)
		if public_calls and path.resolve() != line_tools:
			fail("public C6 tab bridge escaped its line-tool owner: %s (%s)" % (
				path, ", ".join(public_calls),
			))
		if private_calls and path.resolve() != controller:
			fail("private C6 session bridge escaped its tab controller: %s (%s)" % (
				path, ", ".join(private_calls),
			))
	if pathlib.Path(ferrum_qt.__file__).resolve() != (qt_root / "ferrum_qt" / "__init__.py").resolve():
		fail("Qt package did not load from the current checkout")
	extension = pathlib.Path(ferrum_chem.__file__).resolve()
	if extension.parent != pathlib.Path(sys.prefix).resolve() / "lib" / "python3.12" / "site-packages":
		fail("native extension did not load from the isolated venv: %s" % extension)
	if not extension.name.endswith(tuple(importlib.machinery.EXTENSION_SUFFIXES)):
		fail("ferrum_chem is not the direct native extension")
	if hashlib.sha256(extension.read_bytes()).hexdigest() != expected_extension_digest:
		fail("installed native extension does not match the supplied wheel")
	return {"native": str(extension), "qt": str(pathlib.Path(ferrum_qt.__file__).resolve())}


def enum_name(value):
	"""Return one value-safe enum name without serializing Qt objects."""
	if value is None:
		return None
	return getattr(value, "name", type(value).__name__)


def focus_facts(app, window, viewport):
	"""Return the focus ownership facts relevant to one viewport drag."""
	focus = app.focusWidget()
	return {
		"focus_widget_type": type(focus).__name__ if focus is not None else None,
		"focus_is_viewport": focus is viewport,
		"viewport_enabled": viewport.isEnabled(),
		"viewport_focus_policy": enum_name(viewport.focusPolicy()),
		"viewport_has_focus": viewport.hasFocus(),
		"window_active": window.isActiveWindow(),
	}


def intent_facts(intent):
	"""Return presence-only C6 gesture facts without exposing receipt values."""
	if intent is None:
		return {"present": False}
	return {
		"tool": enum_name(getattr(intent, "tool", None)),
		"pending_present": getattr(intent, "attached_cyclohexane_pending", None) is not None,
		"present": True,
		"preview_present": getattr(intent, "preview", None) is not None,
		"start_atom_present": getattr(intent, "start_atom_id", None) is not None,
		"type": type(intent).__name__,
	}


def initial_attach_action_state_facts(window, startup_trace):
	"""Return passive startup ownership facts for an unexpectedly armed attach action."""
	registry = getattr(window, "_action_registry", None)
	registry_actions = getattr(registry, "_actions", {})
	if not isinstance(registry_actions, dict):
		registry_actions = {}
	checked_actions = [
		{
			"object_name": action.objectName(),
			"text": action.text().replace("&", ""),
		}
		for action in window.findChildren(PySide6.QtGui.QAction)
		if action.isCheckable() and action.isChecked()
	]
	checked_actions.sort(key=lambda action: (action["text"], action["object_name"]))
	return {
		"active_mode_id": enum_name(window._mode_manager.active_mode_id),
		"checked_actions": checked_actions,
		"line_intent": intent_facts(window._line_gesture_intent),
		"registry_ids": sorted(str(identifier) for identifier in registry_actions),
		"startup_trace": startup_trace,
	}


def observe_startup_mode(window, startup_trace, phase):
	"""Passively record one mode observation when startup mode ownership changes."""
	mode_name = enum_name(window._mode_manager.active_mode_id)
	observations = startup_trace["mode_observations"]
	if not observations or observations[-1]["active_mode_id"] != mode_name:
		observations.append({"active_mode_id": mode_name, "phase": phase})


def install_startup_trace(window):
	"""Observe the shared attach action without issuing a command or changing state."""
	startup_trace = {"action_events": [], "mode_observations": []}
	action = window._attach_cyclohexane_ring_action

	def record_action_event(signal_name, *arguments):
		startup_trace["action_events"].append({
			"checked": action.isChecked(),
			"signal": signal_name,
			"values": [argument for argument in arguments if isinstance(argument, bool)],
		})

	action.changed.connect(lambda: record_action_event("changed"))
	action.toggled.connect(lambda checked: record_action_event("toggled", checked))
	action.triggered.connect(lambda checked: record_action_event("triggered", checked))
	observe_startup_mode(window, startup_trace, "after_window_construction")
	return startup_trace


def require_initial_attach_mode_unarmed(window, startup_trace):
	"""Reject any hidden startup attach ownership before menu discovery or activation."""
	observe_startup_mode(window, startup_trace, "before_initial_unarmed_assertion")
	action = window._attach_cyclohexane_ring_action
	if (
		window._line_gesture_intent is not None
		or window._mode_manager.active_mode_id is not None
		or action.isChecked()
	):
		fail("startup attach mode must be unarmed: %s" % json.dumps(
			initial_attach_action_state_facts(window, startup_trace), sort_keys=True,
		))


def passive_pretrigger_phase_facts(window, startup_trace):
	"""Capture C6 ownership and signal facts without dispatching the action."""
	action = window._attach_cyclohexane_ring_action
	return {
		"action_checked": action.isChecked(),
		"active_mode": enum_name(window._mode_manager.active_mode_id),
		"intent": intent_facts(window._line_gesture_intent),
		"signal_trace": list(startup_trace["action_events"]),
	}


def require_passive_pretrigger_phase(phase, window, startup_trace, before):
	"""Reject pre-trigger setup that arms or signals the C6 action."""
	after = passive_pretrigger_phase_facts(window, startup_trace)
	previous_trace = before["signal_trace"]
	current_trace = after["signal_trace"]
	trace_delta = (
		current_trace[len(previous_trace):]
		if current_trace[:len(previous_trace)] == previous_trace
		else {"before": previous_trace, "after": current_trace}
	)
	if (
		after["action_checked"]
		or after["active_mode"] is not None
		or after["intent"]["present"]
		or trace_delta
	):
		fail("pre-trigger phase changed C6 ownership: %s" % json.dumps({
			"phase": phase,
			"signal_delta": trace_delta,
			"post_phase": after,
			"pre_phase": before,
		}, sort_keys=True))
	return after


def refusal_fact(request):
	"""Return a typed, value-safe summary of one recorded edit refusal."""
	return {
		"message_present": bool(getattr(request, "message", "")),
		"technical_detail_present": bool(getattr(request, "technical_detail", "")),
		"title_present": bool(getattr(request, "title", "")),
		"type": type(request).__name__,
	}


def attach_pending_debug_facts(window, intent, diagnostics):
	"""Return value-safe state facts when an attach drag has no preview."""
	action = window._attach_cyclohexane_ring_action
	return {
		"attach_action": {
			"checked": action.isChecked(),
			"enabled": action.isEnabled(),
			"visible": action.isVisible(),
		},
		"checked_actions": sorted(
			candidate.text().replace("&", "")
			for candidate in window.findChildren(PySide6.QtGui.QAction)
			if candidate.isCheckable() and candidate.isChecked()
		),
		"diagnostics": diagnostics,
		"intent": intent_facts(intent),
	}


def snapshot_facts(snapshot):
	"""Return the stable identity and lifecycle facts of one native snapshot."""
	return {
		"digest": snapshot.digest,
		"dirty": snapshot.is_dirty,
		"revision": snapshot.revision,
	}


def release_history_debug_facts(tab, window, refusals, diagnostics):
	"""Record authoritative release facts without attempting recovery or mutation."""
	try:
		native_snapshot = {"snapshot": snapshot_facts(tab._session.snapshot())}
	except Exception as error:
		native_snapshot = {
			"exception": {"class": type(error).__name__, "message": str(error)},
		}
	return {
		"active_intent": intent_facts(window._line_gesture_intent),
		"native_session_snapshot": native_snapshot,
		"refresh_reprojection": diagnostics["commit_refresh"],
		"refusal_recorder": {
			"count": len(refusals),
			"requests": [refusal_fact(request) for request in refusals],
		},
		"tab_current_snapshot": snapshot_facts(tab.current_snapshot),
	}


def ineligible_refusal_debug_facts(tab, refusal_before, window, refusals):
	"""Return redacted terminal facts for the intentional ineligible click."""
	action = window._attach_cyclohexane_ring_action
	return {
		"attach_action": {
			"checked": action.isChecked(),
			"intent": intent_facts(window._line_gesture_intent),
		},
		"refusal_recorder": {
			"count": len(refusals),
			"requests": [refusal_fact(request) for request in refusals],
		},
		"snapshots": {
			"after": snapshot_facts(tab.current_snapshot),
			"before": snapshot_facts(refusal_before),
		},
	}


app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
window = ferrum_qt.main_window.MainWindow(object())
startup_trace = install_startup_trace(window)
refusals = []


def record_edit_refusal(request):
	if type(request) is not ferrum_qt.dialogs.refusal_presenter.RefusalRequest:
		fail("edit refusal presenter received an untyped request")
	refusals.append(request)


window._show_edit_refusal = record_edit_refusal
window.resize(1400, 900)
tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
	"<cdml><molecule id='m'><atom id='anchor' name='C'><point x='10' y='20'/></atom></molecule></cdml>",
	"attached-cyclohexane.cdml",
)
bridge_diagnostics = {
	"begin": {"calls": 0, "exceptions": []},
	"commit_refresh": {"calls": 0, "exceptions": [], "succeeded": False},
	"preview": {"calls": 0, "exceptions": []},
	"refusals": refusals,
}


def wrap_bridge_call(name, original):
	"""Count one existing bridge call and retain a caught exception summary."""
	def wrapped(*arguments, **keywords):
		bridge_diagnostics[name]["calls"] += 1
		try:
			result = original(*arguments, **keywords)
			if "succeeded" in bridge_diagnostics[name]:
				bridge_diagnostics[name]["succeeded"] = True
			return result
		except Exception as error:
			bridge_diagnostics[name]["exceptions"].append({
				"class": type(error).__name__, "message": str(error),
			})
			raise
	return wrapped


tab.begin_attached_cyclohexane = wrap_bridge_call(
	"begin", tab.begin_attached_cyclohexane,
)
tab.preview_attached_cyclohexane = wrap_bridge_call(
	"preview", tab.preview_attached_cyclohexane,
)
tab.commit_attached_cyclohexane = wrap_bridge_call(
	"commit_refresh", tab.commit_attached_cyclohexane,
)
try:
	window._register_native_tab(tab, activate=True)
	window.show()
	app.processEvents()
	observe_startup_mode(window, startup_trace, "after_initial_event_delivery")
	if window._attach_cyclohexane_ring_action is window._insert_cyclohexane_ring_action:
		fail("attach action aliases the detached cyclohexane action")
	if not window._attach_cyclohexane_ring_action.isCheckable():
		fail("attach action is not checkable")
	require_initial_attach_mode_unarmed(window, startup_trace)
	pretrigger_phase = passive_pretrigger_phase_facts(window, startup_trace)
	before = tab.current_snapshot
	pretrigger_phase = require_passive_pretrigger_phase(
		"after_snapshot_read", window, startup_trace, pretrigger_phase,
	)
	anchor = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
	pretrigger_phase = require_passive_pretrigger_phase(
		"after_anchor_mapping", window, startup_trace, pretrigger_phase,
	)
	release = anchor + PySide6.QtCore.QPoint(80, 0)
	require_passive_pretrigger_phase(
		"after_release_coordinate", window, startup_trace, pretrigger_phase,
	)
	trigger_visible_menu_action(window, "Attach Cyclohexane Ring", app)
	bridge_diagnostics["focus"] = {
		"before_press": focus_facts(app, window, tab.view.viewport()),
	}
	PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor)
	bridge_diagnostics["focus"]["after_press"] = focus_facts(
		app, window, tab.view.viewport(),
	)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), release)
	app.processEvents()
	bridge_diagnostics["focus"]["after_move"] = focus_facts(
		app, window, tab.view.viewport(),
	)
	bridge_diagnostics["refusals"] = [refusal_fact(request) for request in refusals]
	intent = window._line_gesture_intent
	if intent is None or intent.attached_cyclohexane_pending is None or intent.preview is None:
		fail("attach drag did not receive a native pending preview: %s" % json.dumps(
			attach_pending_debug_facts(window, intent, bridge_diagnostics), sort_keys=True,
		))
	if tab.current_snapshot != before:
		fail("attachment preview mutated the document")
	PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, release)
	app.processEvents()
	if tab.current_snapshot.revision != before.revision + 1:
		fail("attachment did not create exactly one history transition: %s" % json.dumps(
			release_history_debug_facts(tab, window, refusals, bridge_diagnostics), sort_keys=True,
		))
	graph = graph_facts(tab, "anchor")
	attached_cdml = tab.current_snapshot.cdml
	if (
		window._undo_action.parent() is not window
		or window._undo_action.text().replace("&", "") != "Undo"
		or not window._undo_action.isEnabled()
	):
		fail("window-owned Undo QAction is not ready for the history route")
	window._undo_action.trigger()
	app.processEvents()
	undo = tab.current_snapshot
	if undo.cdml != before.cdml:
		fail("undo did not restore the pre-attachment document")
	if (
		window._attach_cyclohexane_ring_action.isChecked()
		or window._line_gesture_intent is not None
	):
		fail("undo did not retire the C6 tool state: %s" % json.dumps({
			"action_checked": window._attach_cyclohexane_ring_action.isChecked(),
			"intent": intent_facts(window._line_gesture_intent),
		}, sort_keys=True))
	if (
		window._redo_action.parent() is not window
		or window._redo_action.text().replace("&", "") != "Redo"
		or not window._redo_action.isEnabled()
	):
		fail("window-owned Redo QAction is not ready for the history route")
	window._redo_action.trigger()
	app.processEvents()
	redo = tab.current_snapshot
	if redo.cdml != attached_cdml:
		fail("redo did not restore the attached cycle")
	if (
		window._attach_cyclohexane_ring_action.isChecked()
		or window._line_gesture_intent is not None
	):
		fail("redo did not retire the C6 tool state: %s" % json.dumps({
			"action_checked": window._attach_cyclohexane_ring_action.isChecked(),
			"intent": intent_facts(window._line_gesture_intent),
		}, sort_keys=True))
	saved = pathlib.Path(sys.argv[3]) / "attached-cyclohexane.cdml"
	tab.save_atomic(saved)
	reopened = ferrum_chem.DocumentSession.load(saved.read_text(encoding="utf-8")).observe(0)
	if len(reopened.projection.molecules) != 1 or len(reopened.projection.molecules[0].atoms) != 6:
		fail("save/reopen did not preserve the attached cycle")
	trigger_visible_menu_action(window, "Attach Cyclohexane Ring", app)
	before_cancel = tab.current_snapshot
	if (
		not window._attach_cyclohexane_ring_action.isChecked()
		or window._line_gesture_intent is None
		or window._line_gesture_intent.tool is not ferrum_qt.ferrum.line_tool_intent._NativeLineTool.ATTACH_CYCLOHEXANE_RING
	):
		fail("canonical C6 rearm did not create its armed tool state: %s" % json.dumps({
			"action_checked": window._attach_cyclohexane_ring_action.isChecked(),
			"intent": intent_facts(window._line_gesture_intent),
		}, sort_keys=True))
	PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), release)
	app.processEvents()
	escape_intent = window._line_gesture_intent
	if (
		escape_intent is None
		or escape_intent.attached_cyclohexane_pending is None
		or escape_intent.preview is None
	):
		fail("Escape did not begin from a live native attach preview: %s" % json.dumps(
			attach_pending_debug_facts(window, escape_intent, bridge_diagnostics), sort_keys=True,
		))
	PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
	app.processEvents()
	if window._line_gesture_intent is not None or tab.current_snapshot != before_cancel:
		fail("Escape did not cancel without mutation")
	ineligible = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='n'><atom id='a' name='C'><point x='0' y='0'/></atom><atom id='b' name='C'><point x='20' y='0'/></atom><atom id='c' name='C'><point x='0' y='20'/></atom><atom id='d' name='C'><point x='-20' y='0'/></atom><bond id='ab' start='a' end='b' type='n1'/><bond id='ac' start='a' end='c' type='n1'/><bond id='ad' start='a' end='d' type='n1'/></molecule></cdml>",
		"ineligible-attachment.cdml",
	)
	window._register_native_tab(ineligible, activate=True)
	app.processEvents()
	refusal_before = ineligible.current_snapshot
	ineligible_anchor = ineligible.view.mapFromScene(PySide6.QtCore.QPointF(0.0, 0.0))
	trigger_visible_menu_action(window, "Attach Cyclohexane Ring", app)
	if not window._attach_cyclohexane_ring_action.isChecked():
		fail("ineligible-anchor refusal did not begin with attach mode armed")
	ineligible_release = ineligible_anchor + PySide6.QtCore.QPoint(12, 0)
	PySide6.QtTest.QTest.mousePress(ineligible.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, ineligible_anchor)
	PySide6.QtTest.QTest.mouseMove(ineligible.view.viewport(), ineligible_release)
	PySide6.QtTest.QTest.mouseRelease(ineligible.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, ineligible_release)
	app.processEvents()
	if (
		len(refusals) != 1
		or type(refusals[0]) is not ferrum_qt.dialogs.refusal_presenter.RefusalRequest
		or ineligible.current_snapshot != refusal_before
		or window._line_gesture_intent is not None
		or window._attach_cyclohexane_ring_action.isChecked()
	):
		fail("ineligible anchor refusal was not one typed terminal mutation-free refusal: %s" % json.dumps(
			ineligible_refusal_debug_facts(ineligible, refusal_before, window, refusals), sort_keys=True,
		))
	print(json.dumps({
		"schema": "ferrum-cyclohexane-attachment-e2e-v1",
		"graph": graph,
		"history": {"attach_revision": before.revision + 1, "undo_revision": undo.revision, "redo_revision": redo.revision},
		"isolation": source_isolation(),
		"status": "ok",
	}, sort_keys=True))
finally:
	window.close()
	window.deleteLater()
'''


#============================================
def main() -> int:
	"""Install one exact local wheel and execute the offscreen C6 workflow."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", required=True, type=pathlib.Path)
	parser.add_argument(
		"--receipt", type=pathlib.Path,
		default=pathlib.Path("/private/tmp/ferrum-cyclohexane-attachment-e2e-receipt.json"),
	)
	arguments = parser.parse_args()
	wheel = arguments.native_wheel.resolve()
	if not wheel.is_file() or wheel.is_symlink() or wheel.suffix != ".whl":
		raise CyclohexaneAttachmentE2eError(f"native wheel must be a regular .whl file: {wheel}")
	if not QT_ROOT.is_dir():
		raise CyclohexaneAttachmentE2eError(f"current checkout Qt root is missing: {QT_ROOT}")
	environment = scrubbed_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-cyclohexane-e2e-", dir="/private/tmp") as temporary:
		root = pathlib.Path(temporary)
		venv = root / "venv"
		run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		run(str(python), "-B", "-m", "pip", "install", "--ignore-installed", "--no-deps", str(wheel), environment=environment)
		child = root / "workflow.py"
		child_source = textwrap.dedent(CHILD_PROGRAM)
		compile(child_source, str(child), "exec")
		child.write_text(child_source, encoding="utf-8")
		output = run(
			str(python), "-I", "-B", str(child), str(QT_ROOT), extension_member_digest(wheel), str(root),
			environment=environment,
		)
		try:
			result = json.loads(output)
		except json.JSONDecodeError as error:
			raise CyclohexaneAttachmentE2eError("workflow did not emit one JSON result") from error
		if result.get("schema") != "ferrum-cyclohexane-attachment-e2e-v1" or result.get("status") != "ok":
			raise CyclohexaneAttachmentE2eError(f"workflow result is invalid: {result!r}")
	receipt = {
		"schema": "ferrum-cyclohexane-attachment-e2e-receipt-v1",
		"native_wheel": {"path": str(wheel), "sha256": sha256(wheel)},
		"workflow": result,
	}
	arguments.receipt.parent.mkdir(parents=True, exist_ok=True)
	arguments.receipt.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
	print(json.dumps(receipt, sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
