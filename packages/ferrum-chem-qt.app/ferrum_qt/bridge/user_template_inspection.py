"""Qt-free bridge for Rust-owned saved user-template admission."""


class UserTemplateInspectionError(ValueError):
	"""Raised when Rust rejects one serialized user template."""


#============================================
def inspect_user_template_display_name(template_cdml: str) -> str | None:
	"""Return Rust's optional display name for one eligible template.

	Args:
		template_cdml: Exact complete CDML text saved in one user template.

	Returns:
		A stripped nonblank molecule name, or ``None`` when it is absent.

	Raises:
		UserTemplateInspectionError: If Rust rejects the template CDML.
	"""
	return prepare_user_template(template_cdml).display_name


#============================================
def prepare_user_template(template_cdml: str) -> object:
	"""Return one immutable Rust plan for catalog display and later placement."""
	try:
		import ferrum_qt.ferrum.engine as engine
		return engine.prepare_user_template_v1(template_cdml)
	except Exception as error:
		raise UserTemplateInspectionError(str(error)) from error
