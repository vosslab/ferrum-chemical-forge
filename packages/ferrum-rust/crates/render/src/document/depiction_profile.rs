//! Closed Ferrum-owned depiction policy applied to immutable document projections.

use crate::{
    AtomBondRenderRequest, CompactGroupRenderPrimitiveV1, FerrumFontEnvironmentV1, Paint,
    PositiveFinite, RenderProvenance, RenderRevision, VerifiedTelexGlyphMetrics,
    build_atom_bond_plan,
};
use ferrum_document_projection::{
    DocumentProjectionV1, PresentationRootProjectionV1, ProjectionIssueCodeV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::depiction_profile_resolution::{
    apply_double_bond_carrier_marks, issue, positive, resolve_atom, resolve_bond,
    resolved_default_bond_lane_spacing, resolved_font, resolved_line_paint, resolved_line_width,
};
use crate::{DocumentMoleculeRenderPlanV2, DocumentPlusRenderV1, DocumentTextRenderV1};

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
    paint: Paint,
    line_width: PositiveFinite,
    wedge_width: PositiveFinite,
}

impl DirectGlycosidicHaworthStyleV1 {
    #[must_use]
    pub fn paint(&self) -> Paint {
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
    plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    issues: Vec<DepictionIssueV1>,
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
            plans: Vec<DocumentMoleculeRenderPlanV2>,
            plus_renders: Vec<DocumentPlusRenderV1>,
            text_renders: Vec<DocumentTextRenderV1>,
            issues: Vec<DepictionIssueV1>,
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
        let mut resolution = Self::new(
            wire.projection_revision,
            wire.projection_digest,
            wire.plans,
            wire.issues,
        );
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
    let environment = FerrumFontEnvironmentV1::load()?;
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)?;
    render_with_verified_telex_metrics(projection, profile, &metrics)
}

/// Lower with explicit metrics for deterministic crate-local behavior tests only.
fn render_with_verified_telex_metrics(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
    metrics: &VerifiedTelexGlyphMetrics,
) -> Result<DepictionResolutionV1, DepictionError> {
    let mut plans = Vec::new();
    let mut plus_renders = Vec::new();
    let mut text_renders = Vec::new();
    let mut issues = Vec::new();
    let invalid_presentation = projection
        .issues()
        .iter()
        .filter(|issue| issue.code() == ProjectionIssueCodeV1::InvalidPresentationFact)
        .map(|issue| {
            DepictionIssueV1::new(
                DepictionIssueCodeV1::InvalidPresentationFact,
                issue.path(),
                issue.detail(),
            )
        })
        .collect::<Vec<_>>();
    if !invalid_presentation.is_empty() {
        let mut resolution = DepictionResolutionV1::new(
            projection.revision(),
            *projection.digest(),
            plans,
            invalid_presentation,
        );
        resolution.suppression = Some(DepictionSuppressionV1::InvalidPresentationFacts);
        return Ok(resolution);
    }
    for molecule in projection.molecules() {
        let mut atoms = Vec::new();
        let mut endpoint_targets = std::collections::HashMap::new();
        for atom in molecule.atoms() {
            match resolve_atom(atom, projection, profile) {
                Ok(target) => {
                    if let Some(id) = atom.id() {
                        endpoint_targets.insert(id.clone(), target.target().clone());
                    }
                    atoms.push(target);
                }
                Err(issue) => issues.push(issue),
            }
        }
        let font = match resolved_font(projection, profile, None, None) {
            Ok(value) => value,
            Err(issue) => {
                issues.push(issue);
                continue;
            }
        };
        let line_width = match resolved_line_width(projection, profile) {
            Ok(value) => value,
            Err(issue) => {
                issues.push(issue);
                continue;
            }
        };
        let line_paint = match resolved_line_paint(projection, profile) {
            Ok(value) => value,
            Err(issue) => {
                issues.push(issue);
                continue;
            }
        };
        let bond_lane_spacing = match resolved_default_bond_lane_spacing(projection, profile) {
            Ok(value) => value,
            Err(issue) => {
                issues.push(issue);
                continue;
            }
        };
        let mut compact_group_primitives = Vec::new();
        compact_group_primitives
            .try_reserve(molecule.compact_groups().len())
            .map_err(|_| crate::RenderError::ResourceExhausted)?;
        for group in molecule.compact_groups() {
            compact_group_primitives.push(CompactGroupRenderPrimitiveV1::from_projection(
                group,
                metrics,
                line_paint.clone(),
            )?);
        }
        endpoint_targets.extend(
            compact_group_primitives
                .iter()
                .zip(molecule.compact_groups())
                .map(|(primitive, group)| (group.id().clone(), primitive.target().clone())),
        );
        let mut bonds = Vec::new();
        for bond in molecule.bonds() {
            match resolve_bond(bond, &endpoint_targets, projection, profile) {
                Ok(target) => bonds.push((bond, target)),
                Err(issue) => issues.push(issue),
            }
        }
        if let Err(issue) =
            apply_double_bond_carrier_marks(&mut bonds, molecule.double_bond_carrier_marks())
        {
            issues.push(issue);
        }
        let request = AtomBondRenderRequest::new(
            RenderProvenance::new(
                RenderRevision::new(projection.revision())?,
                *projection.digest(),
            ),
            atoms,
            bonds.into_iter().map(|(_, target)| target).collect(),
            font,
            line_width,
            bond_lane_spacing,
            line_paint.clone(),
        )?
        .with_compact_group_endpoints(
            compact_group_primitives
                .iter()
                .map(CompactGroupRenderPrimitiveV1::bond_endpoint)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let base_plan = build_atom_bond_plan(&request, metrics)?;
        let mut batches = base_plan.batches().to_vec();
        batches.extend(
            compact_group_primitives
                .iter()
                .map(|group| group.batch().clone()),
        );
        batches.sort_by_key(|batch| batch.target().source_order());
        let plan = crate::MoleculeRenderPlan::new(
            base_plan.provenance(),
            batches,
            base_plan.issues().to_vec(),
        )?;
        plans.push(DocumentMoleculeRenderPlanV2::from_projection(
            molecule,
            plan,
            compact_group_primitives,
        ));
    }
    for root in projection.presentation_stack().roots() {
        match root {
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
                    issues.push(issue(
                        DepictionIssueCodeV1::UnsupportedTextStyle,
                        text.target().projection_key().as_str(),
                        "bold and italic Text require verified font faces not present in V1",
                    ));
                    continue;
                }
                match DocumentTextRenderV1::from_projection(text, metrics) {
                    Ok(render) => text_renders.push(render),
                    Err(error) => issues.push(issue(
                        DepictionIssueCodeV1::UnsupportedFeature,
                        text.target().projection_key().as_str(),
                        error.to_string(),
                    )),
                }
            }
            _ => {}
        }
    }
    let mut resolution =
        DepictionResolutionV1::new(projection.revision(), *projection.digest(), plans, issues);
    resolution.plus_renders = plus_renders;
    resolution.text_renders = text_renders;
    Ok(resolution)
}

impl DepictionResolutionV1 {
    /// Construct a response from exactly one immutable document projection provenance.
    #[must_use]
    pub fn new(
        projection_revision: u64,
        projection_digest: [u8; 32],
        plans: Vec<DocumentMoleculeRenderPlanV2>,
        issues: Vec<DepictionIssueV1>,
    ) -> Self {
        Self {
            schema: DEPICTION_RESOLUTION_SCHEMA_V1,
            profile: DEPICTION_PROFILE_SCHEMA_V1,
            projection_revision,
            projection_digest,
            plans,
            plus_renders: Vec::new(),
            text_renders: Vec::new(),
            issues,
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
    pub fn plans(&self) -> &[DocumentMoleculeRenderPlanV2] {
        &self.plans
    }
    /// Return exact verified-Telex layouts for supported direct-root plus signs.
    #[must_use]
    pub fn plus_renders(&self) -> &[DocumentPlusRenderV1] {
        &self.plus_renders
    }

    /// Return verified-Telex direct-root Text layouts in source order.
    #[must_use]
    pub fn text_renders(&self) -> &[DocumentTextRenderV1] {
        &self.text_renders
    }
    /// Return named exclusions that have no renderer fallback.
    #[must_use]
    pub fn issues(&self) -> &[DepictionIssueV1] {
        &self.issues
    }
    /// Return the typed whole-projection suppression, when malformed facts prevent plans.
    #[must_use]
    pub const fn suppression(&self) -> Option<DepictionSuppressionV1> {
        self.suppression
    }
}
