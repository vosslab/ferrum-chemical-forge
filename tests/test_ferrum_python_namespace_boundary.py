"""Keep Ferrum release inputs within the Ferrum Python namespace boundary."""

# Standard Library
import ast
import os
import re

# local repo modules
import file_utils


REPO_ROOT = file_utils.get_repo_root()
EXTERNAL_PRODUCT_MODULE_NAMES = frozenset({"oasa", "bkchem"})
IMMUTABLE_CDML_NAMESPACE = "urn:ferrum:cdml"
ARCHIVAL_PROVENANCE_PREFIXES = ("OTHER_REPOS/",)
MANIFEST_FILENAMES = frozenset({
	"cargo.lock",
	"cargo.toml",
	"pipfile",
	"poetry.lock",
	"pyproject.toml",
	"requirements-dev.txt",
	"requirements.txt",
	"setup.cfg",
	"setup.py",
})
WORD_RE = re.compile(r"[A-Za-z0-9]+")


#============================================
def is_archival_provenance_path(rel: str) -> bool:
	"""
	Return whether a path belongs to the read-only provenance reference tree.

	Args:
		rel: Repository-relative POSIX path.

	Returns:
		bool: True only for explicitly isolated archival provenance paths.
	"""
	return rel.startswith(ARCHIVAL_PROVENANCE_PREFIXES)


#============================================
def is_dependency_manifest(rel: str) -> bool:
	"""
	Return whether one path is a shipping dependency-manifest format.

	Args:
		rel: Repository-relative POSIX path.

	Returns:
		bool: True when the filename is a supported dependency manifest.
	"""
	filename = os.path.basename(rel).lower()
	if filename in MANIFEST_FILENAMES:
		return True
	return filename.startswith("pip_requirements") and filename.endswith(".txt")


#============================================
def is_scanned_path(rel: str) -> bool:
	"""
	Select live Python and dependency-manifest release inputs.

	Args:
		rel: Repository-relative POSIX path.

	Returns:
		bool: True for a non-archival Python source or dependency manifest.
	"""
	if is_archival_provenance_path(rel):
		return False
	return rel.endswith(".py") or is_dependency_manifest(rel)


#============================================
def external_product_terms(value: str) -> set[str]:
	"""
	Return external product terms that occur as complete identifier tokens.

	Args:
		value: Identifier, path, import name, or manifest text to inspect.

	Returns:
		set[str]: Lowercase external product identifier tokens found in value.
	"""
	terms = {match.group(0).lower() for match in WORD_RE.finditer(value)}
	return terms & EXTERNAL_PRODUCT_MODULE_NAMES


#============================================
def imported_module_terms(source: str, rel: str) -> set[str]:
	"""
	Return external product roots imported by one Python source text.

	Args:
		source: Python source code.
		rel: Repository-relative path used in syntax diagnostics.

	Returns:
		set[str]: External product module roots requested by import statements.

	Raises:
		SyntaxError: When the candidate Python source does not parse.
	"""
	tree = ast.parse(source, filename=rel)
	terms = set()
	for node in file_utils.iter_imports(tree):
		if isinstance(node, ast.Import):
			for alias in node.names:
				terms.update(external_product_terms(alias.name))
		elif node.module is not None:
			terms.update(external_product_terms(node.module))
	return terms


#============================================
def manifest_terms(source: str) -> set[str]:
	"""
	Return external product terms from one dependency-manifest source.

	The immutable CDML namespace is explicitly removed before tokenization. It
	identifies the file format and is not a package or Python module reference.

	Args:
		source: Dependency-manifest text.

	Returns:
		set[str]: External product terms found outside the namespace allowlist.
	"""
	allowed_source = source.replace(IMMUTABLE_CDML_NAMESPACE, "")
	return external_product_terms(allowed_source)


#============================================
#============================================
def source_violations(source: str, rel: str, channel: str) -> list[str]:
	"""
	Return release-boundary violations for one selected source representation.

	Args:
		source: Worktree or staged source text.
		rel: Repository-relative source path.
		channel: Source representation label for diagnostics.

	Returns:
		list[str]: Formatted Ferrum namespace-boundary violations.
	"""
	if rel.endswith(".py"):
		path_terms = external_product_terms(rel)
		if path_terms:
			return [f"{channel} Ferrum release input path names external product module: {sorted(path_terms)}"]
		terms = imported_module_terms(source, rel)
		if terms:
			return [f"{channel} Ferrum release input imports external product module: {sorted(terms)}"]
		return []
	terms = manifest_terms(source)
	if terms:
		return [f"{channel} Ferrum dependency manifest names external product module: {sorted(terms)}"]
	return []


#============================================
def collect_release_boundary_violations() -> dict[str, list[str]]:
	"""
	Scan tracked worktree release inputs for external product module names.

	Returns:
		dict[str, list[str]]: Violations keyed by repository-relative path.
	"""
	violations = {}
	tracked_paths = file_utils.list_tracked_files(REPO_ROOT)
	for rel in tracked_paths:
		if not is_scanned_path(rel):
			continue
		path = os.path.join(REPO_ROOT, rel)
		if not os.path.isfile(path):
			continue
		with open(path, "r", encoding="utf-8") as handle:
			issues = source_violations(handle.read(), rel, "worktree")
		if issues:
			violations[rel] = issues
	return violations


#============================================
def test_ferrum_python_release_inputs_use_ferrum_namespace() -> None:
	"""Require release inputs to use the Ferrum Python namespace."""
	violations = collect_release_boundary_violations()
	assert not violations, f"Ferrum Python namespace boundary violations: {violations}"
