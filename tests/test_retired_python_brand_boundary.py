"""Keep retired Python product modules out of Ferrum release inputs."""

# Standard Library
import ast
import os
import re
import subprocess

# local repo modules
import file_utils


REPO_ROOT = file_utils.get_repo_root()
FORBIDDEN_MODULE_NAMES = frozenset({"oasa", "bkchem"})
IMMUTABLE_CDML_NAMESPACE = "http://www.freesoftware.fsf.org/bkchem/cdml"
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
def forbidden_terms(value: str) -> set[str]:
	"""
	Return retired product terms that occur as complete identifier tokens.

	Args:
		value: Identifier, path, import name, or manifest text to inspect.

	Returns:
		set[str]: Lowercase forbidden identifier tokens found in value.
	"""
	terms = {match.group(0).lower() for match in WORD_RE.finditer(value)}
	return terms & FORBIDDEN_MODULE_NAMES


#============================================
def imported_module_terms(source: str, rel: str) -> set[str]:
	"""
	Return forbidden roots imported by one Python source text.

	Args:
		source: Python source code.
		rel: Repository-relative path used in syntax diagnostics.

	Returns:
		set[str]: Retired module roots requested by import statements.

	Raises:
		SyntaxError: When the candidate Python source does not parse.
	"""
	tree = ast.parse(source, filename=rel)
	terms = set()
	for node in file_utils.iter_imports(tree):
		if isinstance(node, ast.Import):
			for alias in node.names:
				terms.update(forbidden_terms(alias.name))
		elif node.module is not None:
			terms.update(forbidden_terms(node.module))
	return terms


#============================================
def manifest_terms(source: str) -> set[str]:
	"""
	Return forbidden product terms from one dependency-manifest source.

	The immutable CDML namespace is explicitly removed before tokenization. It
	identifies the file format and is not a package or Python module reference.

	Args:
		source: Dependency-manifest text.

	Returns:
		set[str]: Retired product terms found outside the namespace allowlist.
	"""
	allowed_source = source.replace(IMMUTABLE_CDML_NAMESPACE, "")
	return forbidden_terms(allowed_source)


#============================================
def read_index_source(rel: str) -> str:
	"""
	Read one staged path so a release commit cannot retain deleted worktree code.

	Args:
		rel: Repository-relative path known to exist in the Git index.

	Returns:
		str: UTF-8 staged source text.

	Raises:
		AssertionError: When Git cannot provide the requested staged content.
	"""
	result = subprocess.run(
		["git", "show", f":{rel}"],
		capture_output=True,
		cwd=REPO_ROOT,
		text=True,
	)
	if result.returncode != 0:
		raise AssertionError(result.stderr.strip() or f"Cannot read staged path: {rel}")
	return result.stdout


#============================================
def staged_paths() -> list[str]:
	"""
	Return staged additions, copies, modifications, and renames for release scanning.

	Returns:
		list[str]: Sorted repository-relative paths with staged source content.

	Raises:
		AssertionError: When Git cannot list staged paths.
	"""
	result = subprocess.run(
		["git", "diff", "--cached", "--name-only", "-z", "--diff-filter=ACMR"],
		capture_output=True,
		cwd=REPO_ROOT,
		text=True,
	)
	if result.returncode != 0:
		raise AssertionError(result.stderr.strip() or "Cannot list staged release paths.")
	return sorted(path for path in result.stdout.split("\0") if path)


#============================================
def source_violations(source: str, rel: str, channel: str) -> list[str]:
	"""
	Return release-boundary violations for one selected source representation.

	Args:
		source: Worktree or staged source text.
		rel: Repository-relative source path.
		channel: Source representation label for diagnostics.

	Returns:
		list[str]: Formatted retired-import or manifest violations.
	"""
	if rel.endswith(".py"):
		path_terms = forbidden_terms(rel)
		if path_terms:
			return [f"{channel} Python module path names retired product: {sorted(path_terms)}"]
		terms = imported_module_terms(source, rel)
		if terms:
			return [f"{channel} Python import/module names retired product: {sorted(terms)}"]
		return []
	terms = manifest_terms(source)
	if terms:
		return [f"{channel} dependency manifest names retired product: {sorted(terms)}"]
	return []


#============================================
def collect_release_boundary_violations() -> dict[str, list[str]]:
	"""
	Scan worktree and staged release inputs for retired Python product names.

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
	for rel in staged_paths():
		if not is_scanned_path(rel):
			continue
		issues = source_violations(read_index_source(rel), rel, "staged")
		if issues:
			violations.setdefault(rel, []).extend(issues)
	return violations


#============================================
def test_retired_python_brand_boundary() -> None:
	"""Reject retired Python imports, module paths, and dependency-manifest names."""
	violations = collect_release_boundary_violations()
	assert not violations, f"Retired Python product boundary violations: {violations}"
