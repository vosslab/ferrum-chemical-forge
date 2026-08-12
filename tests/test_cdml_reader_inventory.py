"""Boundary guard: CDML is recognized only by Ferrum's document crate.

A CDML reader is a Rust source file that names the CDML namespace URI, because
reading CDML means recognizing that namespace. M8 retired the disposable M2 corpus
loader; the comparison example now consumes the document crate's typed projection.
Any additional reader is an accidental competing authority.
"""

# local repo modules
import file_utils

CDML_NAMESPACE = "http://www.freesoftware.fsf.org/bkchem/cdml"

# Repo-relative POSIX paths allowed to name the CDML namespace URI.
ALLOWED_CDML_READERS = frozenset({
	"packages/ferrum-rust/crates/document/src/lib.rs",
})


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
def find_cdml_readers() -> set[str]:
	"""
	Return every workspace Rust source that names the CDML namespace URI.

	Returns:
		set[str]: Repo-relative POSIX paths of the current CDML readers.
	"""
	rust_sources = file_utils.discover_files(
		extensions=(".rs",),
		extra_filter=keep_rust_workspace_source,
		test_key="cdml_reader_inventory",
	)
	readers = set()
	for source_path in rust_sources:
		with open(source_path, "r", encoding="utf-8") as source_handle:
			source_text = source_handle.read()
		if CDML_NAMESPACE in source_text:
			readers.add(file_utils.rel_to_root(source_path))
	return readers


#============================================
def test_cdml_readers_match_allowlist() -> None:
	"""Every CDML reader in the Rust workspace is one the repo decided to have."""
	readers = find_cdml_readers()
	message = f"CDML readers changed: {sorted(readers)}"
	assert readers == set(ALLOWED_CDML_READERS), message
