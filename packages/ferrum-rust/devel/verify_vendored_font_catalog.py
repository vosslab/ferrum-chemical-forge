#!/usr/bin/env python3
"""Verify every local font and license declared by the vendored font catalog."""

# Standard Library
import json
import hashlib
import pathlib


#============================================
def digest_file(path: pathlib.Path) -> str:
	"""Return the SHA-256 digest of one local regular file."""
	contents = path.read_bytes()
	digest = hashlib.sha256(contents).hexdigest()
	return digest


#============================================
def unique_json_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
	"""Build one JSON object while rejecting duplicate member names."""
	result = {}
	for key, value in pairs:
		if key in result:
			raise RuntimeError(f"font catalog repeats JSON member {key!r}")
		result[key] = value
	return result


#============================================
def require_object(value: object, label: str) -> dict[str, object]:
	"""Return one catalog object or raise a contextual failure."""
	if type(value) is not dict:
		raise RuntimeError(f"{label} must be a JSON object")
	return value


#============================================
def require_list(value: object, label: str) -> list[object]:
	"""Return one catalog list or raise a contextual failure."""
	if type(value) is not list:
		raise RuntimeError(f"{label} must be a JSON list")
	return value


#============================================
def require_exact_keys(value: dict[str, object], keys: set[str], label: str) -> None:
	"""Require one closed catalog object shape."""
	actual = set(value)
	if actual != keys:
		raise RuntimeError(f"{label} fields differ: expected {sorted(keys)}, found {sorted(actual)}")


#============================================
def resolved_catalog_path(root: pathlib.Path, relative: object, label: str) -> pathlib.Path:
	"""Resolve one catalog path while keeping it inside the render asset root."""
	if type(relative) is not str or not relative:
		raise RuntimeError(f"{label} must be a nonempty relative path")
	relative_path = pathlib.PurePosixPath(relative)
	if relative_path.is_absolute():
		raise RuntimeError(f"{label} must be relative")
	candidate = root / relative_path
	resolved = candidate.resolve(strict=True)
	if not resolved.is_relative_to(root.parent.resolve()):
		raise RuntimeError(f"{label} escapes the render asset root")
	if not candidate.is_file() or candidate.is_symlink():
		raise RuntimeError(f"{label} must name one ordinary local file")
	return candidate


#============================================
def verify_file(path: pathlib.Path, record: dict[str, object], label: str) -> None:
	"""Verify one catalog file against its declared byte length and digest."""
	declared_bytes = record["bytes"]
	declared_sha256 = record["sha256"]
	if type(declared_bytes) is not int or declared_bytes <= 0:
		raise RuntimeError(f"{label} bytes must be a positive integer")
	if type(declared_sha256) is not str or len(declared_sha256) != 64:
		raise RuntimeError(f"{label} SHA-256 must be one lowercase hexadecimal digest")
	if path.stat().st_size != declared_bytes or digest_file(path) != declared_sha256:
		raise RuntimeError(f"{label} does not match its catalog bytes and SHA-256")


#============================================
def verify_family(
		family_value: object,
		font_root: pathlib.Path,
		family_ids: set[str],
		asset_ids: set[str],
		asset_files: set[str],
		) -> int:
	"""Verify one family, its license, and every declared font binary."""
	family = require_object(family_value, "font family")
	require_exact_keys(
		family,
		{
			"id", "display_name", "version", "source", "revision", "license",
			"license_file", "license_sha256", "asset_count", "assets",
		},
		"font family",
	)
	family_id = family["id"]
	if type(family_id) is not str or family_id in family_ids:
		raise RuntimeError("font catalog has an invalid or duplicate family ID")
	family_ids.add(family_id)
	label = f"font family {family_id!r}"
	if type(family["source"]) is not str or not family["source"].startswith("https://github.com/"):
		raise RuntimeError(f"{label} must identify one upstream GitHub repository")
	if type(family["revision"]) is not str or len(family["revision"]) != 40:
		raise RuntimeError(f"{label} must identify one full upstream Git revision")
	if family["license"] != "OFL-1.1":
		raise RuntimeError(f"{label} must declare OFL-1.1")
	license_path = resolved_catalog_path(font_root, family["license_file"], f"{label} license")
	license_record = {"bytes": license_path.stat().st_size, "sha256": family["license_sha256"]}
	verify_file(license_path, license_record, f"{label} license")
	assets = require_list(family["assets"], f"{label} assets")
	if family["asset_count"] != len(assets):
		raise RuntimeError(f"{label} asset count differs from its asset rows")
	distribution: dict[tuple[object, object], int] = {}
	for asset_value in assets:
		asset = require_object(asset_value, f"{label} asset")
		require_exact_keys(
			asset,
			{"id", "file", "upstream_file", "container", "kind", "bytes", "sha256"},
			f"{label} asset",
		)
		asset_id = asset["id"]
		asset_file = asset["file"]
		if type(asset_id) is not str or asset_id in asset_ids:
			raise RuntimeError(f"{label} has an invalid or duplicate asset ID")
		if type(asset_file) is not str or asset_file in asset_files:
			raise RuntimeError(f"{label} has an invalid or duplicate asset file")
		path = resolved_catalog_path(font_root, asset_file, f"{label} asset {asset_id!r}")
		verify_file(path, asset, f"{label} asset {asset_id!r}")
		container = asset["container"]
		kind = asset["kind"]
		if container != path.suffix.removeprefix(".") or kind not in {"static", "variable"}:
			raise RuntimeError(f"{label} asset {asset_id!r} has invalid container or kind metadata")
		distribution[(container, kind)] = distribution.get((container, kind), 0) + 1
		upstream_file = asset["upstream_file"]
		upstream_path = pathlib.PurePosixPath(upstream_file) if type(upstream_file) is str else None
		if upstream_path is None or upstream_path.is_absolute() or ".." in upstream_path.parts:
			raise RuntimeError(f"{label} asset {asset_id!r} has an invalid upstream path")
		asset_ids.add(asset_id)
		asset_files.add(asset_file)
	expected_distribution = {
		("otf", "static"): 14,
		("ttf", "static"): 14,
		("ttf", "variable"): 2,
		("woff2", "static"): 14,
		("woff2", "variable"): 2,
	}
	if distribution != expected_distribution:
		raise RuntimeError(f"{label} does not contain every approved distribution form")
	return len(assets)


#============================================
def main() -> int:
	"""Verify the complete offline catalog and print one developer receipt."""
	workspace = pathlib.Path(__file__).resolve().parent.parent
	font_root = workspace / "crates/render/assets/fonts"
	catalog_path = font_root / "catalog.json"
	with catalog_path.open(encoding="utf-8") as handle:
		catalog_value = json.load(handle, object_pairs_hook=unique_json_object)
	catalog = require_object(catalog_value, "font catalog")
	require_exact_keys(
		catalog,
		{"schema", "default_roles", "asset_count", "families"},
		"font catalog",
	)
	if catalog["schema"] != "ferrum-vendored-font-catalog":
		raise RuntimeError("font catalog has an unknown schema")
	family_ids: set[str] = set()
	asset_ids: set[str] = set()
	asset_files: set[str] = set()
	families = require_list(catalog["families"], "font catalog families")
	asset_count = 0
	for family in families:
		asset_count += verify_family(family, font_root, family_ids, asset_ids, asset_files)
	if catalog["asset_count"] != asset_count:
		raise RuntimeError("font catalog asset count differs from its family rows")
	default_roles = require_object(catalog["default_roles"], "font catalog default roles")
	if set(default_roles) != {"molecule_label"}:
		raise RuntimeError("font catalog must declare only the current molecule-label default role")
	if any(asset_id not in asset_ids for asset_id in default_roles.values()):
		raise RuntimeError("font catalog default roles must reference declared asset IDs")
	actual_files = {
		path.relative_to(font_root).as_posix()
		for path in font_root.rglob("*")
		if path.is_file() and path.suffix.lower() in {".otf", ".ttf", ".woff2"}
	}
	if actual_files != asset_files:
		raise RuntimeError("font catalog rows differ from the local vendored font inventory")
	receipt = {
		"schema": "ferrum-vendored-font-catalog-verification",
		"status": "ok",
		"families": len(families),
		"assets": asset_count,
		"default_roles": default_roles,
	}
	print(json.dumps(receipt, sort_keys=True, separators=(",", ":")))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
