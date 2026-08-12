"""Boundary guard: only the document crate owns CDML namespace recognition.

Rust tests necessarily contain CDML format fixtures. They do not recognize CDML in
the shipped application, so this check separates test modules from production
sources before locating the sole namespace authority. Command and integration crates
consume the document API; they must not define a second namespace reader.
"""

# Standard Library
import re

# local repo modules
import file_utils

CDML_NAMESPACE = "http://www.freesoftware.fsf.org/bkchem/cdml"

# The descriptive production module that defines the crate-private namespace value.
CDML_NAMESPACE_AUTHORITY = (
	"packages/ferrum-rust/crates/document/src/identity_index.rs"
)


#============================================
def keep_rust_workspace_source(rel: str) -> bool:
	"""
	Select Rust sources inside the workspace, leaving build output alone.

	Args:
		rel: Repo-relative POSIX path supplied by file_utils.discover_files.

	Returns:
		bool: True for a workspace Rust source outside any target/ directory.
	"""
	in_workspace = rel.startswith("packages/ferrum-rust/")
	in_build_output = "/target/" in rel
	kept = in_workspace and not in_build_output
	return kept


#============================================
def is_rust_test_source(rel: str) -> bool:
	"""
	Return whether a Rust file exists only to exercise production behavior.

	Args:
		rel: Repo-relative POSIX Rust source path.

	Returns:
		bool: True for integration tests and descriptive Rust test modules.
	"""
	path_parts = rel.split("/")
	filename = path_parts[-1]
	is_test_directory = "tests" in path_parts
	is_test_module = filename == "tests.rs" or filename.endswith("_tests.rs")
	is_test_source = is_test_directory or is_test_module
	return is_test_source


#============================================
def find_cdml_namespace_literals() -> set[str]:
	"""
	Return production Rust modules that spell the CDML namespace URI directly.

	Returns:
		set[str]: Repo-relative paths of production namespace-literal modules.
	"""
	rust_sources = file_utils.discover_files(
		extensions=(".rs",),
		extra_filter=keep_rust_workspace_source,
		test_key="cdml_reader_inventory",
	)
	# A just-created authoritative module can exist before the human records it in
	# the repository index. Include that required path explicitly so a crate-root
	# split fails closed during local fast-lane validation as well as after review.
	repo_root = file_utils.get_repo_root()
	authority_path = f"{repo_root}/{CDML_NAMESPACE_AUTHORITY}"
	if authority_path not in rust_sources:
		rust_sources.append(authority_path)
	readers = set()
	for source_path in rust_sources:
		relative_path = file_utils.rel_to_root(source_path)
		if is_rust_test_source(relative_path):
			continue
		with open(source_path, "r", encoding="utf-8") as source_handle:
			source_text = source_handle.read()
		if CDML_NAMESPACE in source_text:
			readers.add(relative_path)
	return readers


#============================================
def test_cdml_namespace_has_one_production_authority() -> None:
	"""Production namespace recognition stays in the document identity module."""
	authorities = find_cdml_namespace_literals()
	message = f"CDML namespace authority changed: {sorted(authorities)}"
	assert authorities == {CDML_NAMESPACE_AUTHORITY}, message


#============================================
def test_cdml_namespace_authority_is_a_document_constant() -> None:
	"""The sole spelling remains a crate-private document namespace declaration."""
	repo_root = file_utils.get_repo_root()
	authority_path = f"{repo_root}/{CDML_NAMESPACE_AUTHORITY}"
	with open(authority_path, "r", encoding="utf-8") as authority_handle:
		authority_source = authority_handle.read()
	declaration = re.compile(
		r'^pub\(crate\) const CDML_NAMESPACE: &str = "'
		+ re.escape(CDML_NAMESPACE)
		+ r'";$',
		re.MULTILINE,
	)
	assert declaration.search(authority_source), "CDML namespace constant moved or changed visibility"
