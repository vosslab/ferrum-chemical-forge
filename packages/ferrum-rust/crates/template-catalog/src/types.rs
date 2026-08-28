use ferrum_document::DocumentUserTemplatePlanV1;
use ferrum_domain::CatalogKeyV1;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TemplateCatalogKeyV1(pub(crate) String);
impl TemplateCatalogKeyV1 {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateContentIdentityV1 {
    hex: String,
}
impl TemplateContentIdentityV1 {
    pub(crate) fn sha256(hex: String) -> Self {
        Self { hex }
    }
    #[must_use]
    pub fn algorithm(&self) -> &'static str {
        "sha256"
    }
    #[must_use]
    pub fn hex(&self) -> &str {
        &self.hex
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateCatalogSourceV1 {
    Shipped,
    UserDirectory,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateFormatV1 {
    FerrumAuthoredRecipe,
    Cdml,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateCompatibilityV1 {
    profile: &'static str,
    format: TemplateFormatV1,
}
impl TemplateCompatibilityV1 {
    pub(crate) const fn new(profile: &'static str, format: TemplateFormatV1) -> Self {
        Self { profile, format }
    }
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }
    #[must_use]
    pub const fn format(&self) -> TemplateFormatV1 {
        self.format
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateCatalogProvenanceV1 {
    source_kind: String,
    source_id: String,
    license_spdx: Option<String>,
    reviewed_on: Option<String>,
    chemistry_scope: Option<String>,
}
impl TemplateCatalogProvenanceV1 {
    pub(crate) fn new(
        source_kind: String,
        source_id: String,
        license_spdx: Option<String>,
        reviewed_on: Option<String>,
        chemistry_scope: Option<String>,
    ) -> Self {
        Self {
            source_kind,
            source_id,
            license_spdx,
            reviewed_on,
            chemistry_scope,
        }
    }
    #[must_use]
    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub fn license_spdx(&self) -> Option<&str> {
        self.license_spdx.as_deref()
    }
    #[must_use]
    pub fn reviewed_on(&self) -> Option<&str> {
        self.reviewed_on.as_deref()
    }
    #[must_use]
    pub fn chemistry_scope(&self) -> Option<&str> {
        self.chemistry_scope.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateCatalogLimitsV1 {
    max_entries: usize,
    max_candidates: usize,
    max_refusals: usize,
    max_file_bytes: usize,
    max_total_bytes: usize,
}
impl TemplateCatalogLimitsV1 {
    #[must_use]
    pub const fn new(max_entries: usize, max_file_bytes: usize, max_total_bytes: usize) -> Self {
        let candidate_product = max_entries.saturating_mul(4);
        let max_candidates = if candidate_product > max_entries {
            candidate_product
        } else {
            max_entries
        };
        let refusal_product = max_entries.saturating_mul(2);
        let max_refusals = if refusal_product > 1 {
            refusal_product
        } else {
            1
        };
        Self::with_scan_limits(
            max_entries,
            max_candidates,
            max_refusals,
            max_file_bytes,
            max_total_bytes,
        )
    }
    /// Construct an explicit bounded directory-scan contract.
    #[must_use]
    pub const fn with_scan_limits(
        max_entries: usize,
        max_candidates: usize,
        max_refusals: usize,
        max_file_bytes: usize,
        max_total_bytes: usize,
    ) -> Self {
        Self {
            max_entries,
            max_candidates,
            max_refusals,
            max_file_bytes,
            max_total_bytes,
        }
    }
    #[must_use]
    pub const fn product_default() -> Self {
        Self::new(
            256,
            ferrum_document::document_user_template_budget_v1().max_utf8_bytes,
            64 * 1024 * 1024,
        )
    }
    #[must_use]
    pub const fn max_entries(&self) -> usize {
        self.max_entries
    }
    #[must_use]
    pub const fn max_file_bytes(&self) -> usize {
        self.max_file_bytes
    }
    #[must_use]
    pub const fn max_candidates(&self) -> usize {
        self.max_candidates
    }
    #[must_use]
    pub const fn max_refusals(&self) -> usize {
        self.max_refusals
    }
    #[must_use]
    pub const fn max_total_bytes(&self) -> usize {
        self.max_total_bytes
    }
}

pub(crate) enum TemplateCatalogSelectionV1 {
    Shipped(CatalogKeyV1),
    User(DocumentUserTemplatePlanV1),
}
pub struct TemplateCatalogEntryV1 {
    pub(crate) key: TemplateCatalogKeyV1,
    pub(crate) identity: TemplateContentIdentityV1,
    pub(crate) label: String,
    pub(crate) search_terms: Vec<String>,
    pub(crate) source: TemplateCatalogSourceV1,
    pub(crate) family: Option<String>,
    pub(crate) family_label: Option<String>,
    pub(crate) family_order: usize,
    pub(crate) category: Option<String>,
    pub(crate) category_label: Option<String>,
    pub(crate) category_order: usize,
    pub(crate) entry_order: usize,
    pub(crate) provenance: TemplateCatalogProvenanceV1,
    pub(crate) compatibility: TemplateCompatibilityV1,
    pub(crate) selection: TemplateCatalogSelectionV1,
    pub(crate) aliases: Vec<String>,
}
impl TemplateCatalogEntryV1 {
    #[must_use]
    pub fn key(&self) -> &TemplateCatalogKeyV1 {
        &self.key
    }
    #[must_use]
    pub fn content_identity(&self) -> &TemplateContentIdentityV1 {
        &self.identity
    }
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Return bounded Rust-issued search vocabulary without exposing content.
    #[must_use]
    pub fn search_terms(&self) -> &[String] {
        &self.search_terms
    }
    #[must_use]
    pub const fn source(&self) -> TemplateCatalogSourceV1 {
        self.source
    }
    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }
    #[must_use]
    pub fn family_label(&self) -> Option<&str> {
        self.family_label.as_deref()
    }
    #[must_use]
    pub const fn family_order(&self) -> usize {
        self.family_order
    }
    #[must_use]
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
    #[must_use]
    pub fn category_label(&self) -> Option<&str> {
        self.category_label.as_deref()
    }
    pub const fn category_order(&self) -> usize {
        self.category_order
    }
    pub const fn entry_order(&self) -> usize {
        self.entry_order
    }
    #[must_use]
    pub fn provenance(&self) -> &TemplateCatalogProvenanceV1 {
        &self.provenance
    }
    #[must_use]
    pub fn compatibility(&self) -> &TemplateCompatibilityV1 {
        &self.compatibility
    }
    #[must_use]
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }
}
