# Standard Library
import ast
import pathlib

# local repo modules
import ferrum_qt


#============================================
def direct_ferrum_chem_importers() -> list[str]:
	"""Return Ferrum modules that import the compiled extension directly."""
	package_root = pathlib.Path(ferrum_qt.__file__).parent
	importers = []
	for path in sorted(package_root.rglob("*.py")):
		source = path.read_text(encoding="utf-8")
		tree = ast.parse(source, filename=str(path))
		for node in ast.walk(tree):
			if isinstance(node, ast.Import):
				if any(alias.name == "ferrum_chem" for alias in node.names):
					importers.append(path.relative_to(package_root).as_posix())
					break
			if isinstance(node, ast.ImportFrom) and node.module == "ferrum_chem":
				importers.append(path.relative_to(package_root).as_posix())
				break
	result = sorted(importers)
	return result


#============================================
def test_no_feature_module_directly_imports_the_extension() -> None:
	"""Keep features behind one lazy, reviewable Python boundary."""
	assert direct_ferrum_chem_importers() == []
	package_root = pathlib.Path(ferrum_qt.__file__).parent
	engine_source = (package_root / "ferrum" / "engine.py").read_text(
		encoding="utf-8",
	)
	assert 'import_module("ferrum_chem")' in engine_source
