"""HTTPS-only source download validation for Ferrum native inputs."""

from __future__ import annotations

# Standard library imports.
import urllib.parse
import urllib.request
import shutil
import stat
import tarfile
import zipfile
from pathlib import Path


class ArchiveExtractionError(RuntimeError):
	"""A verified native source archive could not be safely unpacked."""


def validated_https_url(url: str, label: str) -> str:
	"""Accept one credential-free HTTPS URL before any request can use it."""
	parsed_url = urllib.parse.urlsplit(url)
	if parsed_url.scheme != "https" or not parsed_url.hostname:
		raise ValueError(f"{label} URL must use HTTPS with a host: {url}")
	if parsed_url.username or parsed_url.password or parsed_url.fragment:
		raise ValueError(f"{label} URL must not contain credentials or a fragment")
	return url


class HttpsOnlyRedirectHandler(urllib.request.HTTPRedirectHandler):
	"""Reject every unsafe redirect before urllib constructs its next request."""

	def redirect_request(
		self,
		request: urllib.request.Request,
		file_pointer: object,
		code: int,
		message: str,
		headers: object,
		new_url: str,
	) -> urllib.request.Request | None:
		validated_https_url(new_url, "redirect")
		return super().redirect_request(request, file_pointer, code, message, headers, new_url)


#============================================
def safe_extract(archive: Path, destination: Path) -> Path:
	"""Extract one tar archive after rejecting traversal and duplicate entries."""
	with tarfile.open(archive, "r:gz") as contents:
		members = contents.getmembers()
		seen = set()
		for member in members:
			member_path = (destination / member.name).resolve()
			if not member_path.is_relative_to(destination.resolve()):
				raise ArchiveExtractionError(
					f"RDKit archive contains an unsafe path: {member.name}"
				)
			if member_path in seen:
				raise ArchiveExtractionError(
					f"RDKit archive contains a duplicate path: {member.name}"
				)
			seen.add(member_path)
		contents.extractall(destination, members, filter="data")
	children = [path for path in destination.iterdir() if path.is_dir()]
	if len(children) != 1:
		raise ArchiveExtractionError(
			"verified source archive must extract one top-level directory"
		)
	return children[0]


#============================================
def safe_extract_zip(archive: Path, destination: Path) -> Path:
	"""Extract one ZIP archive after rejecting unsafe member structure."""
	with zipfile.ZipFile(archive) as contents:
		safe_extract_zip_members(contents, destination)
	children = [path for path in destination.iterdir() if path.is_dir()]
	if len(children) != 1:
		raise ArchiveExtractionError(
			"verified source archive must extract one top-level directory"
		)
	return children[0]


#============================================
def safe_extract_zip_members(contents: zipfile.ZipFile, destination: Path) -> None:
	"""Extract regular ZIP members without traversal, links, or duplicate targets."""
	root = destination.resolve()
	seen = set()
	for member in contents.infolist():
		target = (destination / member.filename).resolve()
		if not target.is_relative_to(root):
			raise ArchiveExtractionError(
				f"verified archive contains an unsafe path: {member.filename}"
			)
		if target in seen:
			raise ArchiveExtractionError(
				f"verified archive contains a duplicate path: {member.filename}"
			)
		seen.add(target)
		mode = member.external_attr >> 16
		file_type = stat.S_IFMT(mode)
		if member.is_dir():
			if file_type not in (0, stat.S_IFDIR):
				raise ArchiveExtractionError(
					f"verified archive contains an invalid directory: {member.filename}"
				)
			target.mkdir(parents=True, exist_ok=True)
			continue
		if file_type not in (0, stat.S_IFREG):
			raise ArchiveExtractionError(
				f"verified archive contains a non-regular file: {member.filename}"
			)
		target.parent.mkdir(parents=True, exist_ok=True)
		with contents.open(member) as source, target.open("xb") as output:
			shutil.copyfileobj(source, output)
		permissions = mode & 0o777
		if permissions:
			target.chmod(permissions)
