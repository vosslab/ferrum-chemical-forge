"""Public neutral loader for Ferrum's packaged declarative resources."""

# PIP3 modules
import yaml

# local repo modules
import ferrum_qt.resource_paths


#============================================
class DeclarativeResourceError(ValueError):
	"""Report invalid Ferrum declarative UI resource data."""


#============================================
def load_packaged_yaml(resource_name: str) -> object:
	"""Load one packaged YAML document without importing presentation clients."""
	if type(resource_name) is not str or not resource_name:
		raise TypeError("Ferrum declarative resource names must be nonempty strings.")
	path = ferrum_qt.resource_paths.get_resource_path(resource_name)
	with open(path, "r", encoding="utf-8") as fh:
		return yaml.safe_load(fh)
