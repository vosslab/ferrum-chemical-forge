"""Public immutable receipts for installed Ferrum documents."""

# Standard Library
import dataclasses


SCHEMA = "ferrum-qt-document-installation-v1"
INSTALLATION_KINDS = frozenset((
	"catalog_template",
	"smiles_import",
	"inchi_import",
	"molfile_import",
	"sdf_import",
	"peptide_template_import",
))
ACCESSIBLE_SUMMARIES = {
	"catalog_template": "Ferrum installed one catalog template.",
	"smiles_import": "Ferrum installed one SMILES molecule.",
	"inchi_import": "Ferrum installed one InChI molecule.",
	"molfile_import": "Ferrum installed one Molfile molecule.",
	"sdf_import": "Ferrum installed Ferrum SDF records.",
	"peptide_template_import": "Ferrum installed one peptide template molecule.",
}


#============================================
def accessible_summary_for_installation_kind(installation_kind: str,
		installed_record_count: int = 1) -> str:
	"""Return the closed assistive summary for one installed document."""
	if installation_kind not in INSTALLATION_KINDS:
		raise ValueError("Ferrum document installation requires a closed installation kind")
	if type(installed_record_count) is not int or installed_record_count < 1:
		raise ValueError("Ferrum document installation requires a positive record count")
	if installation_kind != "sdf_import":
		return ACCESSIBLE_SUMMARIES[installation_kind]
	return "Ferrum installed {0} SDF record{1}.".format(
		installed_record_count,
		"" if installed_record_count == 1 else "s",
	)


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumDocumentInstallationV1:
	"""One fully installed, current rendered Ferrum document state."""

	schema: str
	installation_kind: str
	source_revision: int
	source_digest_hex: str
	current_revision: int
	current_digest_hex: str
	installed_record_count: int
	accessible_summary: str

	#============================================
	def __post_init__(self) -> None:
		"""Reject values outside the compact public installation vocabulary."""
		if self.schema != SCHEMA:
			raise ValueError("Ferrum document installation requires its V1 schema")
		if self.installation_kind not in INSTALLATION_KINDS:
			raise ValueError("Ferrum document installation requires a closed installation kind")
		for revision in (self.source_revision, self.current_revision):
			if type(revision) is not int or revision < 0:
				raise TypeError("Ferrum document installation requires nonnegative revisions")
		for digest in (self.source_digest_hex, self.current_digest_hex):
			if type(digest) is not str or not digest:
				raise TypeError("Ferrum document installation requires document digests")
		if type(self.installed_record_count) is not int or self.installed_record_count < 1:
			raise ValueError("Ferrum document installation requires a positive record count")
		expected_summary = accessible_summary_for_installation_kind(
			self.installation_kind, self.installed_record_count,
		)
		if self.accessible_summary != expected_summary:
			raise ValueError("Ferrum document installation requires its closed assistive summary")
