use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use ferrum_document::{
    DocumentSession, DocumentUserTemplateApplyErrorV1, DocumentUserTemplateResultV1,
    PresentationGesturePoint2V1, SessionOperation, SessionOperationResultV1, SessionOperationV1,
    apply_user_template_v1, prepare_user_template_v1,
};
use ferrum_domain::catalog_manifest_v1;
use ferrum_geometry::Point2;

use crate::types::TemplateCatalogSelectionV1;
use crate::user_directory::scan_user_directory;
use crate::{
    TEMPLATE_CATALOG_SCHEMA_V1, TemplateCatalogEntryV1, TemplateCatalogKeyV1,
    TemplateCatalogLimitsV1, TemplateCatalogProvenanceV1,
    TemplateCatalogRefusalCategoryV1 as Category, TemplateCatalogRefusalV1,
    TemplateCatalogSourceV1, TemplateCompatibilityV1, TemplateContentIdentityV1, TemplateFormatV1,
};

pub struct TemplateCatalogSnapshotV1 {
    schema: &'static str,
    catalog_version: String,
    identity: TemplateContentIdentityV1,
    limits: TemplateCatalogLimitsV1,
    entries: Vec<TemplateCatalogEntryV1>,
    refusals: Vec<TemplateCatalogRefusalV1>,
}
impl TemplateCatalogSnapshotV1 {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    #[must_use]
    pub fn catalog_version(&self) -> &str {
        &self.catalog_version
    }
    #[must_use]
    pub fn identity(&self) -> &TemplateContentIdentityV1 {
        &self.identity
    }
    /// Return the immutable resource contract used for this exact snapshot.
    #[must_use]
    pub const fn limits(&self) -> TemplateCatalogLimitsV1 {
        self.limits
    }
    #[must_use]
    pub fn entries(&self) -> &[TemplateCatalogEntryV1] {
        &self.entries
    }
    #[must_use]
    pub fn refusals(&self) -> &[TemplateCatalogRefusalV1] {
        &self.refusals
    }
    #[must_use]
    pub fn find_key_v1(&self, value: &str) -> Option<&TemplateCatalogKeyV1> {
        self.entries
            .iter()
            .find(|entry| entry.key.as_str() == value)
            .map(|entry| &entry.key)
    }
    fn selection(&self, key: &TemplateCatalogKeyV1) -> Option<&TemplateCatalogSelectionV1> {
        self.entries
            .iter()
            .find(|entry| entry.key == *key)
            .map(|entry| &entry.selection)
    }
}

/// Build a fresh immutable snapshot.  The user source is one descriptor-relative scan;
/// later selection uses retained admitted plans, never names or paths (ASVS 2.1-2.3).
pub fn snapshot_template_catalog_v1(
    user_directory: Option<&Path>,
    limits: TemplateCatalogLimitsV1,
) -> Result<TemplateCatalogSnapshotV1, crate::TemplateCatalogErrorV1> {
    let manifest = catalog_manifest_v1();
    let mut entries = manifest
        .entries()
        .iter()
        .enumerate()
        .map(|(entry_order, entry)| {
            let key = entry.key().as_str();
            TemplateCatalogEntryV1 {
                key: TemplateCatalogKeyV1(key.to_owned()),
                identity: hash_identity(&format!(
                    "ferrum-authored-recipe:v1:key={key};semantic={}",
                    entry.recipe().canonical_descriptor()
                )),
                label: entry.label().to_owned(),
                search_terms: entry
                    .terms()
                    .iter()
                    .map(|term| (*term).to_owned())
                    .collect(),
                source: TemplateCatalogSourceV1::Shipped,
                family: Some(
                    match entry.family() {
                        ferrum_domain::CatalogFamilyV1::System => "system",
                        ferrum_domain::CatalogFamilyV1::Biomolecule => "biomolecule",
                    }
                    .to_owned(),
                ),
                family_label: Some(
                    match entry.family() {
                        ferrum_domain::CatalogFamilyV1::System => "System",
                        ferrum_domain::CatalogFamilyV1::Biomolecule => "Biomolecules",
                    }
                    .to_owned(),
                ),
                family_order: match entry.family() {
                    ferrum_domain::CatalogFamilyV1::System => 10,
                    ferrum_domain::CatalogFamilyV1::Biomolecule => 20,
                },
                category: Some(entry.category().key().to_owned()),
                category_label: Some(entry.category().label().to_owned()),
                category_order: entry.category().order() as usize,
                entry_order,
                provenance: TemplateCatalogProvenanceV1::new(
                    entry.provenance().source_kind().to_owned(),
                    entry.provenance().source_id().to_owned(),
                    Some(entry.provenance().license_spdx().to_owned()),
                    Some(entry.provenance().reviewed_on().to_owned()),
                    Some(entry.provenance().chemistry_scope().to_owned()),
                ),
                compatibility: TemplateCompatibilityV1::new(
                    "ferrum-authored-recipe-profile-v1",
                    TemplateFormatV1::FerrumAuthoredRecipe,
                ),
                selection: TemplateCatalogSelectionV1::Shipped(entry.key()),
                aliases: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    let mut refusals = Vec::new();
    if let Some(directory) = user_directory {
        let (files, mut scan_refusals) = scan_user_directory(
            directory,
            limits.max_file_bytes(),
            limits.max_entries(),
            limits.max_candidates(),
            limits.max_total_bytes(),
            limits.max_refusals(),
        )?;
        refusals.append(&mut scan_refusals);
        let mut dedup = BTreeMap::<String, usize>::new();
        for file in files {
            let identity = TemplateContentIdentityV1::sha256(file.digest.clone());
            let key = TemplateCatalogKeyV1(format!("user-template:v1:{}", file.digest));
            if let Some(existing) = dedup.get(identity.hex()).copied() {
                entries[existing].aliases.push(file.basename.clone());
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::DuplicateContent,
                    Some(file.basename),
                ));
                continue;
            }
            let text = match String::from_utf8(file.bytes) {
                Ok(value) => value,
                Err(_) => {
                    refusals.push(TemplateCatalogRefusalV1::new(
                        Category::Utf8Invalid,
                        Some(file.basename),
                    ));
                    continue;
                }
            };
            let plan = match prepare_user_template_v1(&text) {
                Ok(value) => value,
                Err(_) => {
                    refusals.push(TemplateCatalogRefusalV1::new(
                        Category::DocumentAdmission,
                        Some(file.basename),
                    ));
                    continue;
                }
            };
            let index = entries.len();
            dedup.insert(identity.hex().to_owned(), index);
            let label = plan
                .display_name()
                .filter(|name| !name.trim().is_empty())
                .map_or_else(
                    || user_filename_stem(&file.basename).to_owned(),
                    str::to_owned,
                );
            entries.push(TemplateCatalogEntryV1 {
                key,
                identity,
                label: label.clone(),
                search_terms: vec![label, file.basename.clone()],
                source: TemplateCatalogSourceV1::UserDirectory,
                family: Some("user".to_owned()),
                family_label: Some("My templates".to_owned()),
                family_order: usize::MAX,
                category: Some("user_templates".to_owned()),
                category_label: Some("User templates".to_owned()),
                category_order: usize::MAX,
                entry_order: entries.len(),
                provenance: TemplateCatalogProvenanceV1::new(
                    "configured_user_directory".to_owned(),
                    "configured_user_directory".to_owned(),
                    None,
                    None,
                    None,
                ),
                compatibility: TemplateCompatibilityV1::new(
                    "ferrum-document-user-template-profile-v1",
                    TemplateFormatV1::Cdml,
                ),
                selection: TemplateCatalogSelectionV1::User(plan),
                aliases: vec![file.basename],
            });
        }
    }
    let shipped = manifest.entries().len();
    entries[shipped..].sort_by(|left, right| {
        left.key
            .as_str()
            .as_bytes()
            .cmp(right.key.as_str().as_bytes())
    });
    for (entry_order, entry) in entries.iter_mut().enumerate() {
        entry.entry_order = entry_order;
    }
    bound_refusals(&mut refusals, limits.max_refusals());
    let snapshot_material =
        snapshot_identity_material(&entries, &refusals, manifest.catalog_version(), limits);
    Ok(TemplateCatalogSnapshotV1 {
        schema: TEMPLATE_CATALOG_SCHEMA_V1,
        catalog_version: manifest.catalog_version().to_owned(),
        identity: hash_identity(&snapshot_material),
        limits,
        entries,
        refusals,
    })
}
fn hash_identity(value: &str) -> TemplateContentIdentityV1 {
    TemplateContentIdentityV1::sha256(hex_digest(Sha256::digest(value.as_bytes()).as_slice()))
}
fn user_filename_stem(basename: &str) -> &str {
    basename.strip_suffix(".cdml").unwrap_or(basename)
}
fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(HEX[(byte >> 4) as usize] as char);
        value.push(HEX[(byte & 15) as usize] as char);
    }
    value
}

fn snapshot_identity_material(
    entries: &[TemplateCatalogEntryV1],
    refusals: &[TemplateCatalogRefusalV1],
    version: &str,
    limits: TemplateCatalogLimitsV1,
) -> String {
    let mut material = format!(
        "schema={TEMPLATE_CATALOG_SCHEMA_V1}\nversion={version}\nlimits={}|{}|{}|{}|{}\n",
        limits.max_entries(),
        limits.max_candidates(),
        limits.max_refusals(),
        limits.max_file_bytes(),
        limits.max_total_bytes()
    );
    for entry in entries {
        material.push_str(&format!(
            "entry|{}|{}|{}|{}|{:?}|{:?}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            entry.key.as_str(),
            entry.identity.hex(),
            entry.label,
            entry.search_terms.join("\u{1f}"),
            entry.source,
            entry.compatibility.format(),
            entry.compatibility.profile(),
            entry.family.as_deref().unwrap_or(""),
            entry.family_label.as_deref().unwrap_or(""),
            entry.family_order,
            entry.category.as_deref().unwrap_or(""),
            entry.category_label.as_deref().unwrap_or(""),
            entry.category_order,
            entry.entry_order,
            provenance_identity_material(&entry.provenance),
        ));
        for alias in &entry.aliases {
            material.push_str(alias);
            material.push('\u{1f}');
        }
        material.push('\n');
    }
    for refusal in refusals {
        material.push_str(&format!(
            "refusal|{:?}|{:?}|{:?}|{}\n",
            refusal.category(),
            refusal.recovery(),
            refusal.basename(),
            refusal.occurrences()
        ));
    }
    material
}
fn bound_refusals(refusals: &mut Vec<TemplateCatalogRefusalV1>, maximum: usize) {
    let detailed = maximum.saturating_sub(1);
    let mut suppressed = 0u64;
    if refusals.len() > detailed {
        for refusal in refusals.drain(detailed..) {
            suppressed = suppressed.saturating_add(refusal.occurrences());
        }
    }
    if suppressed > 0 && maximum > 0 {
        refusals.push(TemplateCatalogRefusalV1::aggregate_limit_exceeded(
            suppressed,
        ));
    }
}
fn provenance_identity_material(provenance: &TemplateCatalogProvenanceV1) -> String {
    format!(
        "kind={}|id={}|license={:?}|reviewed={:?}|scope={:?}",
        provenance.source_kind(),
        provenance.source_id(),
        provenance.license_spdx(),
        provenance.reviewed_on(),
        provenance.chemistry_scope()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_identity_material_includes_observable_refusals() {
        let limits = TemplateCatalogLimitsV1::new(3, 5, 8);
        let clean = snapshot_identity_material(&[], &[], "v", limits);
        let refused = snapshot_identity_material(
            &[],
            &[TemplateCatalogRefusalV1::new(
                Category::Utf8Invalid,
                Some("bad.cdml".to_owned()),
            )],
            "v",
            limits,
        );
        assert_ne!(hash_identity(&clean).hex(), hash_identity(&refused).hex());
    }
    #[test]
    fn snapshot_identity_material_includes_approved_limits() {
        let small =
            snapshot_identity_material(&[], &[], "v", TemplateCatalogLimitsV1::new(3, 5, 8));
        let larger =
            snapshot_identity_material(&[], &[], "v", TemplateCatalogLimitsV1::new(3, 6, 8));
        assert_ne!(hash_identity(&small).hex(), hash_identity(&larger).hex());
    }
    #[test]
    fn entry_ordinals_are_assigned_after_final_ordering() {
        let mut entries = Vec::new();
        let snapshot = snapshot_template_catalog_v1(None, TemplateCatalogLimitsV1::new(3, 5, 8))
            .expect("shipped catalog should be closed");
        entries.extend(
            snapshot
                .entries()
                .iter()
                .map(TemplateCatalogEntryV1::entry_order),
        );
        assert!(entries.windows(2).all(|pair| pair[0] < pair[1]));
    }
    #[test]
    fn shipped_search_vocabulary_survives_catalog_projection() {
        let snapshot =
            snapshot_template_catalog_v1(None, TemplateCatalogLimitsV1::product_default())
                .expect("shipped catalog should be closed");
        assert!(snapshot.entries().iter().any(|entry| {
            entry.source() == TemplateCatalogSourceV1::Shipped
                && entry.search_terms().iter().any(|term| term == "sulfur")
        }));
    }
    #[test]
    fn provenance_identity_material_commits_kind_and_chemistry_scope() {
        let original = TemplateCatalogProvenanceV1::new(
            "local".to_owned(),
            "source".to_owned(),
            None,
            None,
            Some("scope-a".to_owned()),
        );
        let changed_kind = TemplateCatalogProvenanceV1::new(
            "curated".to_owned(),
            "source".to_owned(),
            None,
            None,
            Some("scope-a".to_owned()),
        );
        let changed_scope = TemplateCatalogProvenanceV1::new(
            "local".to_owned(),
            "source".to_owned(),
            None,
            None,
            Some("scope-b".to_owned()),
        );
        assert_ne!(
            provenance_identity_material(&original),
            provenance_identity_material(&changed_kind)
        );
        assert_ne!(
            provenance_identity_material(&original),
            provenance_identity_material(&changed_scope)
        );
    }
}

pub enum TemplateCatalogPlacementResultV1 {
    Shipped(SessionOperationResultV1),
    User(DocumentUserTemplateResultV1),
}
impl TemplateCatalogPlacementResultV1 {
    #[must_use]
    pub const fn source(&self) -> TemplateCatalogSourceV1 {
        match self {
            Self::Shipped(_) => TemplateCatalogSourceV1::Shipped,
            Self::User(_) => TemplateCatalogSourceV1::UserDirectory,
        }
    }
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        match self {
            Self::Shipped(value) => value,
            Self::User(value) => value.operation_result(),
        }
    }
    #[must_use]
    pub fn inserted_molecule(
        &self,
    ) -> Option<&ferrum_document::DocumentUserTemplateInsertedMoleculeV1> {
        match self {
            Self::Shipped(_) => None,
            Self::User(value) => Some(value.inserted_molecule()),
        }
    }
}
#[derive(Debug, Error)]
pub enum TemplateCatalogApplyErrorV1 {
    #[error("template catalog selection was not found in this snapshot")]
    SelectionNotFound,
    #[error("template catalog point is invalid")]
    InvalidPoint,
    #[error(transparent)]
    Shipped(#[from] ferrum_catalog_placement::CatalogPlacementErrorV1),
    #[error(transparent)]
    User(#[from] DocumentUserTemplateApplyErrorV1),
    #[error(transparent)]
    Session(#[from] ferrum_document::DocumentSessionError),
}
/// Apply exactly one snapshot-owned choice through the existing revision/digest fence.
pub fn apply_template_catalog_entry_v1(
    session: &mut DocumentSession,
    snapshot: &TemplateCatalogSnapshotV1,
    key: &TemplateCatalogKeyV1,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    x: f64,
    y: f64,
) -> Result<TemplateCatalogPlacementResultV1, TemplateCatalogApplyErrorV1> {
    let current = session.snapshot()?;
    if current.revision() != expected_revision || current.digest() != expected_digest {
        return Err(TemplateCatalogApplyErrorV1::Session(
            ferrum_document::DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: current.revision(),
            },
        ));
    }
    match snapshot
        .selection(key)
        .ok_or(TemplateCatalogApplyErrorV1::SelectionNotFound)?
    {
        TemplateCatalogSelectionV1::Shipped(key) => {
            let anchor = PresentationGesturePoint2V1::new(x, y)
                .map_err(|_| TemplateCatalogApplyErrorV1::InvalidPoint)?;
            let request = ferrum_catalog_placement::resolve_catalog_molecule_placement_v1(
                key.as_str(),
                anchor,
            )?;
            Ok(TemplateCatalogPlacementResultV1::Shipped(
                session.apply_document_operation_v1(
                    expected_revision,
                    SessionOperation::V1(SessionOperationV1::PlaceCatalogMoleculeV1(request)),
                )?,
            ))
        }
        TemplateCatalogSelectionV1::User(plan) => {
            let anchor =
                Point2::new(x, y).map_err(|_| TemplateCatalogApplyErrorV1::InvalidPoint)?;
            Ok(TemplateCatalogPlacementResultV1::User(
                apply_user_template_v1(session, expected_revision, expected_digest, plan, anchor)?,
            ))
        }
    }
}
