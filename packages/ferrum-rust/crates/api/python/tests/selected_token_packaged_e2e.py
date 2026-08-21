"""Isolated forced-native-admission proof for live SMARTS selected tokens.

Run with ``python3 -I selected_token_packaged_e2e.py INSTALL_ROOT ADAPTER``.
The caller must validate wheel and bundle provenance before invocation.
"""

from __future__ import annotations

import json
import pickle
import sys
from pathlib import Path


def main() -> None:
	if len(sys.argv) != 3:
		raise SystemExit("usage: selected_token_packaged_e2e.py INSTALL_ROOT ADAPTER")
	install_root = Path(sys.argv[1]).resolve()
	adapter = Path(sys.argv[2]).resolve()
	if not install_root.is_dir() or not adapter.is_file() or adapter.is_symlink():
		raise SystemExit("isolated selected-token harness requires regular wheel members")
	disabled = adapter.with_name(adapter.name + ".selected-token-e2e-disabled")
	if disabled.exists():
		raise SystemExit("isolated selected-token harness disabled path already exists")
	sys.path.insert(0, str(install_root))
	import ferrum_chem

	def session(cdml: str):
		value = ferrum_chem.DocumentSession.load(cdml)
		value._publish_live_render_plan_v1(value.snapshot().revision)
		return value

	def selected(value, identifier: str):
		snapshot = value.snapshot()
		observation = value.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
		selection = value.select_render_interaction_roots_v1(
			observation, None, ferrum_chem.RenderInteractionQueryV1.root(identifier),
		)
		return selection, value._capture_live_document_smarts_selected_query_v1(selection)

	def expect_selection_failure(value, token, reason) -> None:
		try:
			value._run_live_document_smarts_query_v1(token, 1, 1)
		except ferrum_chem.LiveDocumentSmartsError as error:
			if error.reason != reason:
				raise RuntimeError(f"wrong selection failure: {error.reason!r}")
			if error.category == ferrum_chem.LiveDocumentSmartsCategoryV1.unavailable:
				raise RuntimeError("selection failure deferred to unavailable native runtime")
			return
		raise RuntimeError("selection failure unexpectedly reached native matching")

	source = '<cdml><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/></atom></molecule></cdml>'
	first, second = session(source), session(source)
	display_selection, token = selected(first, "m")
	for attribute in ("issuer", "selection", "roots", "identifier", "graph", "query", "__dict__"):
		if hasattr(token, attribute):
			raise RuntimeError(f"opaque selected token exposes {attribute}")
	for secret in (source, "<cdml", "[#6]", "issuer=", "selection="):
		if secret in repr(token) or secret in str(token):
			raise RuntimeError("opaque selected token repr leaks private material")
	for serializer in (lambda: json.dumps(token), lambda: pickle.dumps(token)):
		try:
			serializer()
		except (TypeError, pickle.PicklingError):
			pass
		else:
			raise RuntimeError("opaque selected token unexpectedly serializes")
	try:
		positive = first._run_live_document_smarts_query_v1(token, 1, 1)
	except ferrum_chem.LiveDocumentSmartsError as error:
		raise RuntimeError(
			"selected token native query failed: %r/%r" % (error.category, error.reason),
		) from error
	if positive.traversal != "complete" or len(positive.molecules) != 1:
		raise RuntimeError("selected token did not produce the expected native result")
	matched = positive.molecules[0]
	if (matched.source_order, matched.match_count, matched.completeness) != (0, 1, "complete"):
		raise RuntimeError("selected token result contradicts the current molecule")
	paint = first._show_live_document_smarts_match_v1(positive.receipt, 0)
	if len(paint.atom_bounds) != 1:
		raise RuntimeError("selected-token row zero did not issue one atom paint")
	try:
		first._show_live_document_smarts_match_v1(positive.receipt, 0)
	except ferrum_chem.LiveDocumentSmartsError as error:
		if error.reason != ferrum_chem.LiveDocumentSmartsReasonV1.receipt_unavailable:
			raise RuntimeError(f"selected-token replay returned {error.reason!r}") from error
	else:
		raise RuntimeError("selected-token row zero replay unexpectedly succeeded")
	try:
		second._show_live_document_smarts_match_v1(positive.receipt, 0)
	except ferrum_chem.LiveDocumentSmartsError as error:
		if error.reason != ferrum_chem.LiveDocumentSmartsReasonV1.receipt_unavailable:
			raise RuntimeError(f"selected-token foreign receipt returned {error.reason!r}") from error
	else:
		raise RuntimeError("selected-token foreign receipt unexpectedly succeeded")

	mutating = session(source)
	mutation_selection, mutation_token = selected(mutating, "m")
	pending = mutating._run_live_document_smarts_query_v1(mutation_token, 1, 1)
	gesture = mutating.begin_render_interaction_translation_v1(
		mutation_selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	preview = mutating.preview_render_interaction_translation_v1(gesture, 3.0, 2.0)
	mutating.commit_render_interaction_translation_v1(gesture, preview)
	try:
		mutating._show_live_document_smarts_match_v1(pending.receipt, 0)
	except ferrum_chem.LiveDocumentSmartsError as error:
		if error.reason != ferrum_chem.LiveDocumentSmartsReasonV1.stale_document:
			raise RuntimeError(f"selected-token stale receipt returned {error.reason!r}") from error
	else:
		raise RuntimeError("selected-token stale receipt unexpectedly succeeded")

	adapter.rename(disabled)
	try:
		expect_selection_failure(
			second, token, ferrum_chem.LiveDocumentSmartsReasonV1.foreign_selection,
		)
		expect_selection_failure(
			first, display_selection, ferrum_chem.LiveDocumentSmartsReasonV1.selected_root_empty,
		)
		gesture = first.begin_render_interaction_translation_v1(
			display_selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
		)
		preview = first.preview_render_interaction_translation_v1(gesture, 3.0, 2.0)
		first.commit_render_interaction_translation_v1(gesture, preview)
		expect_selection_failure(
			first, token, ferrum_chem.LiveDocumentSmartsReasonV1.stale_selection,
		)

		multi = session('<cdml><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/></atom></molecule><molecule id="n"><atom id="b" name="C"><point x="8" y="2"/></atom></molecule></cdml>')
		snapshot = multi.snapshot()
		observation = multi.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
		one = multi.select_render_interaction_roots_v1(
			observation, None, ferrum_chem.RenderInteractionQueryV1.root("m"),
		)
		both = multi.select_render_interaction_roots_v1(
			observation, one, ferrum_chem.RenderInteractionQueryV1.root(
				"n", ferrum_chem.RenderInteractionModifierV1.toggle,
			),
		)
		expect_selection_failure(
			multi,
			multi._capture_live_document_smarts_selected_query_v1(both),
			ferrum_chem.LiveDocumentSmartsReasonV1.selected_root_multiple,
		)

		text = session('<cdml><text id="t"><point x="2" y="2"/><ftext>note</ftext></text></cdml>')
		_, non_molecule = selected(text, "t")
		expect_selection_failure(
			text, non_molecule, ferrum_chem.LiveDocumentSmartsReasonV1.selected_source_not_molecule,
		)

		unavailable = session(source)
		try:
			unavailable._run_live_document_smarts_query_v1("[#6]", 1, 1)
		except ferrum_chem.LiveDocumentSmartsError as error:
			if (error.category, error.reason) != (
				ferrum_chem.LiveDocumentSmartsCategoryV1.unavailable,
				ferrum_chem.LiveDocumentSmartsReasonV1.native_runtime_unavailable,
			):
				raise RuntimeError("forced native runtime failure leaked or changed category")
		else:
			raise RuntimeError("disabled packaged adapter unexpectedly matched")
	finally:
		disabled.rename(adapter)


if __name__ == "__main__":
	main()
