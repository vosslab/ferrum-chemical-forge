//! Closed Ferrum-owned depiction policy applied to immutable document projections.

use crate::{
    AtomBondRenderRequest, CompactGroupRenderPrimitiveV1, FerrumFontEnvironment, PositiveFinite,
    RenderPaintV3, RenderProvenance, RenderRevision, VerifiedMoleculeLabelGlyphMetrics,
};
use ferrum_document_projection::{
    DocumentObjectIdV1, DocumentProjectionV1, MoleculeProjectionV1, PresentationRootProjectionV1,
    ProjectionIssueCodeV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::depiction_profile_resolution::{
    apply_double_bond_carrier_marks, positive, resolve_atom, resolve_bond,
    resolve_normal_single_clip_policy, resolved_default_bond_lane_spacing, resolved_font,
    resolved_line_paint, resolved_line_width,
};
use crate::{DocumentMoleculeRenderPlanV4, DocumentPlusRenderV1, DocumentTextRenderV1};

/// Closed schema identifier for the Ferrum V1 depiction profile.
pub const DEPICTION_PROFILE_SCHEMA_V1: &str = "ferrum-depiction-profile-v1";

/// Closed schema identifier for a V1 depiction-resolution response.
pub const DEPICTION_RESOLUTION_SCHEMA_V1: &str = "ferrum-depiction-resolution-v1";

const BUILTIN_DIRECT_GLYCOSIDIC_HAWORTH_WEDGE_WIDTH: f64 = 5.0;
/// Rust-owned, non-serializable presentation policy for the first Ferrum profile.
///
/// The private representation deliberately prevents a frontend or decoded input from
/// manufacturing a variant with system fonts, semantic paint, or device defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepictionProfileV1 {
    private: (),
}

/// Closed direct-Haworth paint facts resolved from one accepted projection.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthStyleV1 {
    paint: RenderPaintV3,
    line_width: PositiveFinite,
    wedge_width: PositiveFinite,
}

impl DirectGlycosidicHaworthStyleV1 {
    #[must_use]
    pub fn paint(&self) -> RenderPaintV3 {
        self.paint.clone()
    }

    #[must_use]
    pub const fn line_width(&self) -> PositiveFinite {
        self.line_width
    }

    #[must_use]
    pub const fn wedge_width(&self) -> PositiveFinite {
        self.wedge_width
    }
}

/// Resolve document-standard style facts for the closed direct-Haworth route.
pub fn resolve_direct_glycosidic_haworth_style_v1(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<DirectGlycosidicHaworthStyleV1, DepictionIssueV1> {
    Ok(DirectGlycosidicHaworthStyleV1 {
        paint: resolved_line_paint(projection, profile)?,
        line_width: resolved_line_width(projection, profile)?,
        wedge_width: positive(
            projection
                .drawing_standard()
                .and_then(|standard| standard.wedge_width())
                .map_or(BUILTIN_DIRECT_GLYCOSIDIC_HAWORTH_WEDGE_WIDTH, |value| {
                    value.value()
                }),
        )?,
    })
}

/// Resolve one attached compact-group pose from the projection/profile boundary.
///
/// This is the sole public admission path. It resolves document depiction,
/// opens the verified resource, and applies the final normal-single clipping
/// policy before returning an anchor that can commit without a later geometry
/// disagreement.
pub fn resolve_attached_compact_group_pose_v2(
    projection: &DocumentProjectionV1,
    atom: &ferrum_document_projection::AtomProjectionV1,
    profile: &DepictionProfileV1,
    catalog_key: ferrum_document_model::CompactGroupCatalogKeyV1,
    raw_release: crate::RenderPoint,
) -> Result<crate::ResolvedAttachedCompactGroupPoseV1, crate::AttachedCompactGroupPoseErrorV2> {
    let facts =
        super::depiction_profile_resolution::resolve_attached_compact_group_anchor_render_facts(
            projection, atom, profile,
        )
        .map_err(|issue| crate::AttachedCompactGroupPoseErrorV2::Depiction {
            detail: issue.detail().to_owned(),
        })?;
    let environment = FerrumFontEnvironment::load().map_err(|error| {
        crate::AttachedCompactGroupPoseErrorV2::FontResource {
            detail: error.to_string(),
        }
    })?;
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment).map_err(|error| {
        crate::AttachedCompactGroupPoseErrorV2::FontResource {
            detail: error.to_string(),
        }
    })?;
    crate::attached_compact_group_pose::resolve_attached_compact_group_pose(
        &facts,
        catalog_key,
        raw_release,
        &metrics,
    )
}

impl DepictionProfileV1 {
    /// Return the sole Ferrum V1 product profile.
    #[must_use]
    pub const fn ferrum_default() -> Self {
        Self { private: () }
    }

    /// Return the immutable provenance identifier recorded in every resolution.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        DEPICTION_PROFILE_SCHEMA_V1
    }
}

/// Stable profile-resolution diagnostic categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepictionIssueCodeV1 {
    /// The projection record lacks an authored durable source identifier.
    NonDurableTarget,
    /// A recognized presentation fact was malformed and cannot use a fallback.
    InvalidPresentationFact,
    /// An authored font family has no verified Ferrum V1 resource.
    UnsupportedAuthoredFontFamily,
    /// Authored rich text exceeds V1's structured chemical-label grammar.
    UnsupportedRichLabel,
    /// A standalone Text style has no verified Ferrum V1 face.
    UnsupportedTextStyle,
    /// A source visibility request cannot be represented by this profile.
    InvalidVisibility,
    /// The explicit hydrogen count exceeds the V1 structured-label range.
    UnrenderableExplicitHydrogenCount,
    /// A source feature cannot be lowered by the V1 molecule profile.
    UnsupportedFeature,
}

/// One closed depiction diagnostic owned by an exact durable molecule member.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoleculeMemberDepictionIssueV1 {
    target: DocumentObjectIdV1,
    code: DepictionIssueCodeV1,
    detail: String,
}

impl MoleculeMemberDepictionIssueV1 {
    /// Build one diagnostic for an atom, bond, or compact group in its owner molecule.
    #[must_use]
    pub fn new(
        target: DocumentObjectIdV1,
        code: DepictionIssueCodeV1,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            target,
            code,
            detail: detail.into(),
        }
    }

    /// Return the exact durable member that owns this diagnostic.
    #[must_use]
    pub const fn target(&self) -> &DocumentObjectIdV1 {
        &self.target
    }

    /// Return the stable issue category.
    #[must_use]
    pub const fn code(&self) -> DepictionIssueCodeV1 {
        self.code
    }

    /// Return the bounded actionable explanation.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// One named target exclusion from an otherwise immutable depiction response.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DepictionIssueV1 {
    code: DepictionIssueCodeV1,
    target: String,
    detail: String,
}

impl DepictionIssueV1 {
    /// Build an actionable profile diagnostic with a projection-local target key.
    #[must_use]
    pub fn new(
        code: DepictionIssueCodeV1,
        target: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            target: target.into(),
            detail: detail.into(),
        }
    }

    /// Return the stable issue category.
    #[must_use]
    pub const fn code(&self) -> DepictionIssueCodeV1 {
        self.code
    }

    /// Return the durable or explicitly projection-local target key.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Return the actionable explanation.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// A revision- and digest-bound depiction response with no frontend defaults.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DepictionResolutionV1 {
    schema: &'static str,
    profile: &'static str,
    projection_revision: u64,
    projection_digest: [u8; 32],
    plans: Vec<DocumentMoleculeRenderPlanV4>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

/// A typed whole-projection suppression that prevents malformed facts using defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepictionSuppressionV1 {
    /// One or more recognized presentation facts was malformed.
    InvalidPresentationFacts,
}

impl<'de> Deserialize<'de> for DepictionResolutionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireResolution {
            schema: String,
            profile: String,
            projection_revision: u64,
            projection_digest: [u8; 32],
            plans: Vec<DocumentMoleculeRenderPlanV4>,
            plus_renders: Vec<DocumentPlusRenderV1>,
            text_renders: Vec<DocumentTextRenderV1>,
            suppression: Option<DepictionSuppressionV1>,
        }
        let wire = WireResolution::deserialize(deserializer)?;
        if wire.schema != DEPICTION_RESOLUTION_SCHEMA_V1
            || wire.profile != DEPICTION_PROFILE_SCHEMA_V1
        {
            return Err(serde::de::Error::custom(
                "unknown Ferrum depiction-resolution schema or profile",
            ));
        }
        let mut resolution =
            Self::new(wire.projection_revision, wire.projection_digest, wire.plans);
        resolution.plus_renders = wire.plus_renders;
        resolution.text_renders = wire.text_renders;
        resolution.suppression = wire.suppression;
        Ok(resolution)
    }
}

/// Failure while constructing a complete Ferrum-owned depiction operation.
#[derive(Debug, Error)]
pub enum DepictionError {
    /// A lower-level render invariant rejected an otherwise resolved request.
    #[error("could not lower resolved Ferrum depiction: {0}")]
    Render(#[from] crate::RenderError),
}

/// Resolve immutable CDML facts with the closed Ferrum profile and lower them.
///
/// Invalid and unsupported authored facts become named issues. They never become a
/// toolkit fallback or a synthetic durable identity.
pub fn render_document_projection_v1(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<DepictionResolutionV1, DepictionError> {
    let environment = FerrumFontEnvironment::load()?;
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)?;
    render_with_verified_molecule_label_metrics(projection, profile, &metrics)
}

/// Lower with explicit metrics for deterministic crate-local behavior tests only.
fn render_with_verified_molecule_label_metrics(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
    metrics: &VerifiedMoleculeLabelGlyphMetrics,
) -> Result<DepictionResolutionV1, DepictionError> {
    let mut plans = Vec::new();
    let mut plus_renders = Vec::new();
    let mut text_renders = Vec::new();
    let invalid_presentation = projection
        .issues()
        .iter()
        .filter(|issue| issue.code() == ProjectionIssueCodeV1::InvalidPresentationFact)
        .collect::<Vec<_>>();
    if !invalid_presentation.is_empty() {
        let mut resolution =
            DepictionResolutionV1::new(projection.revision(), *projection.digest(), plans);
        resolution.suppression = Some(DepictionSuppressionV1::InvalidPresentationFacts);
        return Ok(resolution);
    }
    for molecule in projection.molecules() {
        let owner_molecule_object_id = molecule.document_object_id();
        let mut member_issues = Vec::new();
        let mut atoms = Vec::new();
        let mut endpoint_targets = std::collections::HashMap::new();
        for atom in molecule.atoms() {
            match resolve_atom(atom, owner_molecule_object_id, projection, profile) {
                Ok((target, record_id)) => {
                    endpoint_targets.insert(atom.document_object_id().clone(), record_id);
                    atoms.push(target);
                }
                Err(issue) => member_issues.push(member_issue(atom.document_object_id(), issue)),
            }
        }
        let font = match resolved_font(projection, profile, None, None) {
            Ok(value) => value,
            Err(issue) => {
                return Err(DepictionError::Render(crate::RenderError::InvalidRequest(
                    issue.detail().to_owned(),
                )));
            }
        };
        let line_width = match resolved_line_width(projection, profile) {
            Ok(value) => value,
            Err(issue) => {
                return Err(DepictionError::Render(crate::RenderError::InvalidRequest(
                    issue.detail().to_owned(),
                )));
            }
        };
        let line_paint = match resolved_line_paint(projection, profile) {
            Ok(value) => value,
            Err(issue) => {
                return Err(DepictionError::Render(crate::RenderError::InvalidRequest(
                    issue.detail().to_owned(),
                )));
            }
        };
        let bond_lane_spacing = match resolved_default_bond_lane_spacing(projection, profile) {
            Ok(value) => value,
            Err(issue) => {
                return Err(DepictionError::Render(crate::RenderError::InvalidRequest(
                    issue.detail().to_owned(),
                )));
            }
        };
        let mut compact_group_primitives = Vec::new();
        compact_group_primitives
            .try_reserve(molecule.compact_groups().len())
            .map_err(|_| crate::RenderError::ResourceExhausted)?;
        for group in molecule.compact_groups() {
            match CompactGroupRenderPrimitiveV1::from_projection(
                group,
                owner_molecule_object_id,
                metrics,
                line_paint.clone(),
            ) {
                Ok(primitive) => compact_group_primitives.push(primitive),
                Err(error) => member_issues.push(MoleculeMemberDepictionIssueV1::new(
                    group.id().clone(),
                    DepictionIssueCodeV1::UnsupportedFeature,
                    error.to_string(),
                )),
            }
        }
        endpoint_targets.extend(
            compact_group_primitives
                .iter()
                .zip(molecule.compact_groups())
                .map(|(primitive, group)| {
                    let endpoint = primitive
                        .bond_endpoint()
                        .expect("compact group primitive has a bond endpoint");
                    (group.id().clone(), endpoint.context().record_id().clone())
                }),
        );
        let mut bonds = Vec::new();
        for bond in molecule.bonds() {
            match resolve_bond(
                bond,
                owner_molecule_object_id,
                &endpoint_targets,
                projection,
                profile,
            ) {
                Ok(target) => bonds.push((bond, target)),
                Err(issue) => member_issues.push(member_issue(bond.document_object_id(), issue)),
            }
        }
        if let Err(issue) =
            apply_double_bond_carrier_marks(&mut bonds, molecule.double_bond_carrier_marks())
        {
            let target = molecule
                .double_bond_carrier_marks()
                .first()
                .map(|mark| mark.carrier_bond().clone())
                .ok_or_else(|| {
                    DepictionError::Render(crate::RenderError::InvalidRequest(
                        issue.detail().to_owned(),
                    ))
                })?;
            member_issues.push(MoleculeMemberDepictionIssueV1::new(
                target,
                issue.code(),
                issue.detail(),
            ));
        }
        let normal_single_clip_policy = resolve_normal_single_clip_policy(
            line_width,
            font.size(),
            owner_molecule_object_id.as_str(),
        )
        .map_err(|issue| crate::RenderError::InvalidRequest(issue.detail().to_owned()))?;
        let request = AtomBondRenderRequest::new_with_normal_single_clip_policy(
            RenderProvenance::new(
                RenderRevision::new(projection.revision())?,
                *projection.digest(),
            ),
            atoms,
            bonds.into_iter().map(|(_, target)| target).collect(),
            font,
            line_width,
            bond_lane_spacing,
            normal_single_clip_policy,
            line_paint.clone(),
        )?
        .with_compact_group_endpoints(
            compact_group_primitives
                .iter()
                .map(CompactGroupRenderPrimitiveV1::bond_endpoint)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let base_plan = crate::atom_bond::build_atom_bond_plan(&request, metrics)?;
        let mut batches = base_plan.batches().to_vec();
        batches.extend(
            compact_group_primitives
                .iter()
                .map(|group| group.batch().clone()),
        );
        batches.sort_by_key(crate::RenderBatchV4::paint_order);
        let plan = crate::MoleculeRenderPlanV4::new(
            base_plan.provenance(),
            batches,
            base_plan.issues().to_vec(),
        )?;
        plans.push(DocumentMoleculeRenderPlanV4::from_document_object_id(
            owner_molecule_object_id.clone(),
            plan,
            compact_group_primitives,
            molecule_member_ids(molecule),
            member_issues,
        )?);
    }
    for entry in projection.presentation_stack().entries() {
        match entry.root() {
            PresentationRootProjectionV1::Plus { plus } => {
                plus_renders.push(DocumentPlusRenderV1::from_projection(plus, metrics)?);
            }
            PresentationRootProjectionV1::Text { text } => {
                if text.runs().iter().any(|run| {
                    run.styles().iter().any(|style| {
                        matches!(
                            style,
                            ferrum_document_projection::PresentationTextStyleV1::Bold
                                | ferrum_document_projection::PresentationTextStyleV1::Italic
                        )
                    })
                }) {
                    return Err(DepictionError::Render(crate::RenderError::InvalidRequest(
                        "bold and italic Text require verified font faces not present in V1"
                            .to_owned(),
                    )));
                }
                match DocumentTextRenderV1::from_projection(text, metrics) {
                    Ok(render) => text_renders.push(render),
                    Err(error) => {
                        return Err(DepictionError::Render(crate::RenderError::InvalidRequest(
                            error.to_string(),
                        )));
                    }
                }
            }
            _ => {}
        }
    }
    let mut resolution =
        DepictionResolutionV1::new(projection.revision(), *projection.digest(), plans);
    resolution.plus_renders = plus_renders;
    resolution.text_renders = text_renders;
    Ok(resolution)
}

fn member_issue(
    target: &DocumentObjectIdV1,
    issue: DepictionIssueV1,
) -> MoleculeMemberDepictionIssueV1 {
    MoleculeMemberDepictionIssueV1::new(target.clone(), issue.code(), issue.detail())
}

fn molecule_member_ids(molecule: &MoleculeProjectionV1) -> Vec<DocumentObjectIdV1> {
    molecule
        .atoms()
        .iter()
        .map(|atom| atom.document_object_id().clone())
        .chain(
            molecule
                .bonds()
                .iter()
                .map(|bond| bond.document_object_id().clone()),
        )
        .chain(
            molecule
                .compact_groups()
                .iter()
                .map(|group| group.id().clone()),
        )
        .collect()
}

impl DepictionResolutionV1 {
    /// Construct a response from exactly one immutable document projection provenance.
    #[must_use]
    pub fn new(
        projection_revision: u64,
        projection_digest: [u8; 32],
        plans: Vec<DocumentMoleculeRenderPlanV4>,
    ) -> Self {
        Self {
            schema: DEPICTION_RESOLUTION_SCHEMA_V1,
            profile: DEPICTION_PROFILE_SCHEMA_V1,
            projection_revision,
            projection_digest,
            plans,
            plus_renders: Vec::new(),
            text_renders: Vec::new(),
            suppression: None,
        }
    }

    /// Return the exact source projection revision, including initial revision zero.
    #[must_use]
    pub const fn projection_revision(&self) -> u64 {
        self.projection_revision
    }
    /// Return the exact source projection digest.
    #[must_use]
    pub const fn projection_digest(&self) -> &[u8; 32] {
        &self.projection_digest
    }
    /// Return complete per-molecule plans.
    #[must_use]
    pub fn plans(&self) -> &[DocumentMoleculeRenderPlanV4] {
        &self.plans
    }
    /// Return exact verified-Atkinson Hyperlegible Next layouts for supported direct-root plus signs.
    #[must_use]
    pub fn plus_renders(&self) -> &[DocumentPlusRenderV1] {
        &self.plus_renders
    }

    /// Return verified-Atkinson Hyperlegible Next direct-root Text layouts in source order.
    #[must_use]
    pub fn text_renders(&self) -> &[DocumentTextRenderV1] {
        &self.text_renders
    }
    /// Return the typed whole-projection suppression, when malformed facts prevent plans.
    #[must_use]
    pub const fn suppression(&self) -> Option<DepictionSuppressionV1> {
        self.suppression
    }
}
