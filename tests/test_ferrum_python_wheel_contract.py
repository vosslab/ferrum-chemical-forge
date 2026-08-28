"""Lock the dedicated Rust ownership boundary for Ferrum's Maturin wheel."""

# Standard Library
import os
import tomllib

# local repo modules
import file_utils


REPO_ROOT = file_utils.get_repo_root()
PYPROJECT_PATH = os.path.join(
	REPO_ROOT,
	"packages/ferrum-rust/crates/api/python/pyproject.toml",
)
EXTENSION_MANIFEST_PATH = os.path.join(
	REPO_ROOT,
	"packages/ferrum-rust/crates/api-python/Cargo.toml",
)


#============================================
def load_toml(path: str) -> dict:
	"""Return one checked-in TOML document."""
	with open(path, "rb") as handle:
		return tomllib.load(handle)


#============================================
def test_maturin_project_selects_the_dedicated_extension_crate() -> None:
	"""Keep ordinary Rust linking separate from the CPython wheel owner."""
	project = load_toml(PYPROJECT_PATH)
	maturin = project["tool"]["maturin"]
	extension = load_toml(EXTENSION_MANIFEST_PATH)

	assert maturin == {
		"module-name": "ferrum_chem",
		"bindings": "pyo3",
		"manifest-path": "../../api-python/Cargo.toml",
		"strip": False,
	}
	assert extension["package"]["name"] == "ferrum-api-python"
	assert extension["lib"] == {
		"name": "ferrum_chem",
		"crate-type": ["cdylib"],
	}
