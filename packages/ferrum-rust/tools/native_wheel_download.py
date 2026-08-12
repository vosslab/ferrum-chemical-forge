"""HTTPS-only source download validation for Ferrum native inputs."""

from __future__ import annotations

# Standard library imports.
import urllib.parse
import urllib.request


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
