"""Guard replaced Ferrum capabilities from regaining legacy Python imports.

The migration remains runnable while OASA supplies capabilities that Rust has
not replaced.  Consequently this guard is deliberately capability-gated: a
milestone adds an owning capability only when its replacement has landed.
"""

import ast
from pathlib import Path

# local repo modules
import file_utils


# A capability is activated only for the production path where its complete
# Rust replacement has landed. M4d activates chemistry on the native route;
# the separately hosted legacy editor remains outside that bounded ownership.
ACTIVE_REPLACED_CAPABILITIES = frozenset({"chemistry"})

# Each capability owns the production paths where its legacy dependency would
# be architectural. The production selector below is intentionally positive:
# no tests, oracle, provenance material, fixtures, or OTHER_REPOS paths can
# enter this hygiene scan.
CAPABILITY_IMPORT_POLICIES = {
	"chemistry": {
		"paths": (
			"packages/ferrum-chem-qt.app/ferrum_qt/native/",
			"packages/ferrum-chem-qt.app/ferrum_qt/bridge/display_geometry.py",
			"packages/ferrum-chem-qt.app/ferrum_qt/bridge/insertion_placement.py",
		),
		"forbidden_roots": frozenset({"oasa"}),
	},
	"desktop_ui": {
		"paths": ("packages/ferrum-chem-qt.app/ferrum_qt/",),
		"forbidden_roots": frozenset({"tkinter", "Tkinter"}),
	},
}

# These locations may name historical origins but are categorically not
# production code. Keep the boundary visible as the policy expands.
PROVENANCE_ALLOWLIST = (
	"docs/",
	"tests/e2e/oracle/",
	"tests/fixtures/",
)


#============================================
def keep_governed_production_source(rel_path: str) -> bool:
	"""Select only Python sources in a capability-owned production package."""
	# The legacy reference is never Ferrum production code, even if it becomes
	# tracked for a future evidence snapshot.
	if rel_path.startswith("OTHER_REPOS/"):
		return False
	for allowed_prefix in PROVENANCE_ALLOWLIST:
		if rel_path.startswith(allowed_prefix):
			return False
	for policy in CAPABILITY_IMPORT_POLICIES.values():
		for production_prefix in policy["paths"]:
			if rel_path.startswith(production_prefix):
				return True
	return False


#============================================
def discover_governed_production_sources(repo_root: str | None = None) -> list[str]:
	"""Discover the only files this guard is allowed to inspect."""
	paths = file_utils.discover_files(
		extensions=(".py",),
		extra_filter=keep_governed_production_source,
		test_key="migration_import_exclusion",
		repo_root=repo_root,
	)
	return paths


#============================================
def import_roots(path: str) -> list[tuple[int, str]]:
	"""Return absolute import roots and their source lines for one module."""
	tree, error = file_utils.parse_source(path)
	if error is not None:
		raise RuntimeError(f"Cannot evaluate migration import policy: {path}: {error.msg}")
	roots = []
	for node in file_utils.iter_imports(tree):
		if isinstance(node, ast.Import):
			for alias in node.names:
				roots.append((node.lineno, alias.name.split(".", 1)[0]))
		if isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
			roots.append((node.lineno, node.module.split(".", 1)[0]))
	return roots


#============================================
def active_policy_violations(
	paths: list[str],
	active_capabilities: frozenset[str],
	repo_root: str,
) -> list[str]:
	"""Return legacy-import violations for the activated capability policies."""
	violations = []
	for capability in sorted(active_capabilities):
		policy = CAPABILITY_IMPORT_POLICIES[capability]
		for path in paths:
			rel_path = file_utils.rel_to_root(path, repo_root)
			if not any(rel_path.startswith(prefix) for prefix in policy["paths"]):
				continue
			for line_no, root in import_roots(path):
				if root in policy["forbidden_roots"]:
					violations.append(
						f"{rel_path}:{line_no}: {capability} must not import {root}"
					)
	violations.sort()
	return violations


#============================================
def find_active_capability_violations(repo_root: str | None = None) -> list[str]:
	"""Evaluate the currently replaced capabilities against Ferrum production."""
	if repo_root is None:
		repo_root = file_utils.get_repo_root()
	paths = discover_governed_production_sources(repo_root)
	violations = active_policy_violations(paths, ACTIVE_REPLACED_CAPABILITIES, repo_root)
	return violations


#============================================
def test_active_capability_imports_are_excluded() -> None:
	"""Every activated replacement path remains free of its legacy imports."""
	violations = find_active_capability_violations()
	assert not violations, "\n".join(violations)


#============================================
def test_seeded_oasa_import_is_rejected_after_chemistry_activation(tmp_path: Path) -> None:
	"""A chemistry replacement prevents OASA from returning in its owned path."""
	path = tmp_path / "packages/ferrum-chem-qt.app/ferrum_qt/native/chemistry.py"
	path.parent.mkdir(parents=True)
	path.write_text("import oasa\n", encoding="utf-8")
	violations = active_policy_violations([str(path)], frozenset({"chemistry"}), str(tmp_path))
	assert "chemistry must not import oasa" in violations[0]


#============================================
def test_seeded_tk_import_is_rejected_after_desktop_activation(tmp_path: Path) -> None:
	"""A desktop replacement prevents Tk from returning in its owned path."""
	path = tmp_path / "packages/ferrum-chem-qt.app/ferrum_qt/window.py"
	path.parent.mkdir(parents=True)
	path.write_text("from tkinter import Tk\n", encoding="utf-8")
	violations = active_policy_violations([str(path)], frozenset({"desktop_ui"}), str(tmp_path))
	assert "desktop_ui must not import tkinter" in violations[0]


#============================================
def test_other_repos_is_not_governed_production() -> None:
	"""The reference checkout is outside Ferrum's production import guard."""
	kept = keep_governed_production_source("OTHER_REPOS/bkchem-oasa/oasa/module.py")
	assert not kept
