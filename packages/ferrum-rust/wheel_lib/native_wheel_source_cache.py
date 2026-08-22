"""Profile-owned source-archive cache helpers for Ferrum native builds."""

from __future__ import annotations

from pathlib import Path


class NativeSourceCacheError(ValueError):
	"""The generated source-archive cache violates its fixed ownership contract."""


#============================================
def managed_source_archive_cache_root(repo_root: Path, profile: object) -> Path:
	"""Return the generated cache location owned by one native capability profile."""
	return repo_root / "build" / "native-source-archives" / profile.name


#============================================
def provision_managed_source_archive_cache(
		repo_root: Path, profile: object, verified_archive: object,
		download_verified_archive: object,
		) -> Path:
	"""Return a complete verified cache, provisioning only missing archives."""
	cache_root = managed_source_archive_cache_root(repo_root, profile)
	if cache_root.is_symlink():
		raise NativeSourceCacheError(
			f"managed native archive cache must not be a symbolic link: {cache_root}"
		)
	if cache_root.exists() and not cache_root.is_dir():
		raise NativeSourceCacheError(f"managed native archive cache must be a directory: {cache_root}")
	physical_repo_root = repo_root.resolve()
	physical_cache_root = cache_root.resolve()
	if not physical_cache_root.is_relative_to(physical_repo_root):
		raise NativeSourceCacheError(
			f"managed native archive cache resolves outside the repository: {cache_root}"
		)
	if "OTHER_REPOS" in physical_cache_root.parts:
		raise NativeSourceCacheError(
			f"managed native archive cache must not resolve into OTHER_REPOS: {cache_root}"
		)
	cache_root.mkdir(parents=True, exist_ok=True)
	for source in (profile.rdkit, *profile.dependencies):
		destination = cache_root / source.archive_filename
		if destination.exists() or destination.is_symlink():
			verified_archive(destination, source.sha256, source.name)
		else:
			download_verified_archive(destination, source.url, source.sha256, source.name)
	return cache_root
