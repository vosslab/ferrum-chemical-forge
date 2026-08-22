"""Command-line parser construction for the native-wheel builder."""

from __future__ import annotations

import argparse
from collections.abc import Callable
from pathlib import Path


def parser(
	build_handler: Callable[[argparse.Namespace], None],
	adapter_handler: Callable[[argparse.Namespace], None],
	self_test_handler: Callable[[argparse.Namespace], None],
	publication_validation_handler: Callable[[argparse.Namespace], None],
	publication_handler: Callable[[argparse.Namespace], None],
	artifact_result_handler: Callable[[argparse.Namespace], None],
	qt_worktree_closure_handler: Callable[[argparse.Namespace], None],
	qt_staging_handler: Callable[[argparse.Namespace], None],
	output_path: Callable[[str], Path],
	engine_bundle_path: Callable[[str], Path],
	archive_root_path: Callable[[str], Path],
) -> argparse.ArgumentParser:
	"""Create the native-wheel command parser from injected build operations."""
	result = argparse.ArgumentParser(description="Build Ferrum's native wheel.")
	subcommands = result.add_subparsers(dest="command", required=True)
	build = subcommands.add_parser("build", help="verify RDKit, source-build it, then build a wheel")
	build.add_argument("--output-root", required=True, type=output_path)
	build.add_argument(
		"--engine-bundle-dir",
		type=engine_bundle_path,
		help=(
			"optional fresh directory beneath --output-root for the same verified adapter's "
			"Ferrum CLI engine bundle"
		),
	)
	source = build.add_mutually_exclusive_group()
	source.add_argument(
		"--source-archive-root",
		type=archive_root_path,
		help="read-only directory containing every selected source archive",
	)
	source.add_argument(
		"--sealed-input-root",
		type=output_path,
		help="previous builder-validated native inputs copied into this fresh output root",
	)
	build.set_defaults(handler=build_handler)
	adapter = subcommands.add_parser(
		"adapter", help="build a replacement ABI-compatible adapter from sealed native inputs"
	)
	adapter.add_argument("--output-root", required=True, type=output_path)
	adapter.add_argument(
		"--rdkit-output-root",
		required=True,
		type=output_path,
		help=(
			"completed Ferrum native-build output root containing the private RDKit install "
			"and the selected Boost headers"
		),
	)
	adapter.set_defaults(handler=adapter_handler)
	self_test = subcommands.add_parser(
		"self-test", help="run deterministic native-wheel policy helper checks"
	)
	self_test.set_defaults(handler=self_test_handler)
	publication_validation = subcommands.add_parser(
		"validate-publication",
		help="verify copied publication evidence against staged and live Ferrum source closures",
	)
	publication_validation.add_argument("--staged-source-root", required=True, type=Path)
	publication_validation.add_argument("--worktree-source-root", required=True, type=Path)
	publication_validation.add_argument("--wheel", required=True, type=Path)
	publication_validation.add_argument("--receipt", required=True, type=Path)
	publication_validation.add_argument("--engine-bundle", required=True, type=Path)
	publication_validation.set_defaults(handler=publication_validation_handler)
	publish_publication = subcommands.add_parser(
		"publish-publication",
		help="validate copied evidence and atomically select one native publication",
	)
	publish_publication.add_argument("--candidate-root", required=True, type=Path)
	publish_publication.add_argument("--current-pointer", required=True, type=Path)
	publish_publication.add_argument("--staged-source-root", required=True, type=Path)
	publish_publication.add_argument("--worktree-source-root", required=True, type=Path)
	publish_publication.add_argument("--wheel", required=True, type=Path)
	publish_publication.add_argument("--receipt", required=True, type=Path)
	publish_publication.add_argument("--engine-bundle", required=True, type=Path)
	publish_publication.add_argument("--qt-wheel", required=True, type=Path)
	publish_publication.add_argument("--qt-source-root", required=True, type=Path)
	publish_publication.add_argument("--qt-source-closure", required=True, type=Path)
	publish_publication.add_argument("--qt-worktree-source-root", required=True, type=Path)
	publish_publication.add_argument("--qt-worktree-source-closure", required=True, type=Path)
	publish_publication.add_argument("--pair-receipt", required=True, type=Path)
	publish_publication.set_defaults(handler=publication_handler)
	artifact_result = subcommands.add_parser(
		"parse-artifact-result", help="validate one streamed native builder artifact result",
	)
	artifact_result.add_argument("--output-root", required=True, type=Path)
	artifact_result.set_defaults(handler=artifact_result_handler)
	qt_worktree_closure = subcommands.add_parser(
		"record-qt-worktree-source-closure",
		help="record the admitted Qt source closure before staging",
	)
	qt_worktree_closure.add_argument("--worktree-source-root", required=True, type=Path)
	qt_worktree_closure.add_argument("--closure-path", required=True, type=Path)
	qt_worktree_closure.set_defaults(handler=qt_worktree_closure_handler)
	qt_staging = subcommands.add_parser(
		"stage-qt-source-tree", help="stage exactly one admitted Qt source closure",
	)
	qt_staging.add_argument("--worktree-source-root", required=True, type=Path)
	qt_staging.add_argument("--destination", required=True, type=Path)
	qt_staging.add_argument("--closure-path", required=True, type=Path)
	qt_staging.add_argument("--admission-path", required=True, type=Path)
	qt_staging.set_defaults(handler=qt_staging_handler)
	return result
