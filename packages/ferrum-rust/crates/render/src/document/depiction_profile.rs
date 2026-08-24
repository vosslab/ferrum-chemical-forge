//! Closed Ferrum-owned depiction policy applied to immutable document projections.

use crate::{
    AtomBondRenderRequest, AtomLabelFacts, AtomLabelFontProfile, AtomMarkRenderFacts,
    AtomMarkRenderKind, AtomNumberLabelFacts, AtomRenderTarget, BondRenderTarget, BondStyle,
    CompactGroupRenderPrimitiveV1, FerrumFontEnvironmentV1, FontFace, Paint, PositiveFinite,
    RenderPoint, RenderProvenance, RenderRevision, RenderTarget, Rgb24, TargetVisibility,
    VerifiedTelexGlyphMetrics, build_atom_bond_plan,
};
use ferrum_core::{BondOrder, BondStyle as DocumentBondStyle, Identifier, RecordId, RecordKind};
use ferrum_document_projection::{
    AtomMarkKindV1, AtomProjectionV1, BondEndpointKindV1, BondProjectionV1,
    DocumentHaworthPositionV1, DocumentObjectIdV1, DocumentProjectionV1, PresentationFontFaceV1,
    PresentationRootProjectionV1, ProjectionIssueCodeV1, Rgb24V1 as DocumentRgb24V1,
    TransparentOrRgb24V1, VisibilityV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{DocumentMoleculeRenderPlanV2, DocumentPlusRenderV1, DocumentTextRenderV1};

/// Closed schema identifier for the Ferrum V1 depiction profile.
pub const DEPICTION_PROFILE_SCHEMA_V1: &str = "ferrum-depiction-profile-v1";

/// Closed schema identifier for a V1 depiction-resolution response.
pub const DEPICTION_RESOLUTION_SCHEMA_V1: &str = "ferrum-depiction-resolution-v1";

const BUILTIN_BOND_LANE_SPACING: f64 = 6.0;
const BUILTIN_ATOM_NUMBER_FONT_SIZE: f64 = 9.0;
const BUILTIN_ATOM_NUMBER_OFFSET_X: f64 = 8.0;
const BUILTIN_ATOM_NUMBER_OFFSET_Y: f64 = -12.0;
const BUILTIN_ATOM_NUMBER_RGB: &str = "0000c8";
// CDML's historical `standard/bond@wedge-width` default is 5px.
const BUILTIN_DIRECT_GLYCOSIDIC_HAWORTH_WEDGE_WIDTH: f64 = 5.0;
const BUILTIN_BOND_WEDGE_WIDTH: f64 = 5.0;

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
                Ok(target) => bonds.push(target),
                Err(issue) => issues.push(issue),
            }
        }
        let request = AtomBondRenderRequest::new(
            RenderProvenance::new(
                RenderRevision::new(projection.revision())?,
                *projection.digest(),
            ),
            atoms,
            bonds,
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

fn resolve_atom(
    atom: &AtomProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<AtomRenderTarget, DepictionIssueV1> {
    let target = atom_target(atom)?;
    if atom.label_text().is_some() {
        return Err(issue(
            DepictionIssueCodeV1::UnsupportedRichLabel,
            atom.projection_key().as_str(),
            "V1 supports structured element, hydrogen, and charge labels only",
        ));
    }
    let font = resolved_font(
        projection,
        profile,
        atom.label_font(),
        atom.background_color().or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.area_color())
        }),
    )?;
    let element = atom.element().ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            atom.projection_key().as_str(),
            "atom element is absent",
        )
    })?;
    let charge = atom
        .formal_charge()
        .unwrap_or_default()
        .try_into()
        .map_err(|_| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                atom.projection_key().as_str(),
                "formal charge is outside V1 label range",
            )
        })?;
    let hydrogens = if effective_hydrogens(atom.hydrogens(), projection) {
        atom.explicit_hydrogens()
            .unwrap_or_default()
            .try_into()
            .map_err(|_| {
                issue(
                    DepictionIssueCodeV1::UnrenderableExplicitHydrogenCount,
                    atom.projection_key().as_str(),
                    "explicit hydrogen count is outside V1 label range",
                )
            })?
    } else {
        0
    };
    let label = AtomLabelFacts::new(element, charge, hydrogens).map_err(|error| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            atom.projection_key().as_str(),
            error.to_string(),
        )
    })?;
    let visibility = match atom.show() {
        Some(VisibilityV1::Disabled) => TargetVisibility::Hidden {
            reason: "author explicitly hid atom".to_owned(),
        },
        _ => TargetVisibility::Visible,
    };
    let number_label = match (atom.number(), atom.show_number()) {
        (Some(number), Some(VisibilityV1::Enabled)) => Some(
            AtomNumberLabelFacts::new(
                number,
                RenderPoint::new(BUILTIN_ATOM_NUMBER_OFFSET_X, BUILTIN_ATOM_NUMBER_OFFSET_Y)
                    .expect("built-in atom number offset is finite"),
                AtomLabelFontProfile::new(
                    font.face().clone(),
                    PositiveFinite::new(BUILTIN_ATOM_NUMBER_FONT_SIZE)
                        .expect("built-in atom number size is positive"),
                    Paint::rgb24(
                        Rgb24::new(BUILTIN_ATOM_NUMBER_RGB)
                            .expect("built-in atom number paint is valid RGB"),
                    ),
                ),
            )
            .map_err(|error| {
                issue(
                    DepictionIssueCodeV1::UnsupportedFeature,
                    atom.projection_key().as_str(),
                    error.to_string(),
                )
            })?,
        ),
        _ => None,
    };
    let marks = atom
        .marks()
        .iter()
        .map(|mark| {
            let radians = mark.angle_degrees().to_radians();
            AtomMarkRenderFacts::new(
                render_mark_kind(mark.kind()),
                RenderPoint::new(
                    mark.radial_offset() * radians.cos(),
                    mark.radial_offset() * radians.sin(),
                )?,
                mark.angle_degrees(),
                PositiveFinite::new(mark.size().value())?,
                mark.draw_circle(),
                PositiveFinite::new(mark.line_width().value())?,
                font.paint().clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                atom.projection_key().as_str(),
                error.to_string(),
            )
        })?;
    let target = AtomRenderTarget::new(
        target,
        RenderPoint::new(atom.position().x(), atom.position().y()).map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                atom.projection_key().as_str(),
                error.to_string(),
            )
        })?,
        label,
        visibility,
    )
    .map(|target| target.with_font_profile(font).with_marks(marks))
    .map_err(|error| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            atom.projection_key().as_str(),
            error.to_string(),
        )
    })?;
    Ok(match number_label {
        Some(number_label) => target.with_number_label(number_label),
        None => target,
    })
}

fn render_mark_kind(kind: AtomMarkKindV1) -> AtomMarkRenderKind {
    match kind {
        AtomMarkKindV1::Plus => AtomMarkRenderKind::Plus,
        AtomMarkKindV1::Minus => AtomMarkRenderKind::Minus,
        AtomMarkKindV1::Radical => AtomMarkRenderKind::Radical,
        AtomMarkKindV1::Biradical => AtomMarkRenderKind::Biradical,
        AtomMarkKindV1::Electronpair => AtomMarkRenderKind::Electronpair,
        AtomMarkKindV1::DottedElectronpair => AtomMarkRenderKind::DottedElectronpair,
        AtomMarkKindV1::PzOrbital => AtomMarkRenderKind::PzOrbital,
    }
}

fn resolve_bond(
    bond: &BondProjectionV1,
    endpoints: &std::collections::HashMap<DocumentObjectIdV1, RenderTarget>,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<BondRenderTarget, DepictionIssueV1> {
    let target = bond_target(bond)?;
    let first = endpoint_record(bond.start(), endpoints, bond.projection_key().as_str())?;
    let second = endpoint_record(bond.end(), endpoints, bond.projection_key().as_str())?;
    if let Some(value) = bond.bond_width().filter(|value| value.value() < 0.0) {
        // A negative CDML bond_width selects an uncentered double-bond lane side.
        // This profile cannot lower that direction yet.  Keep the authoritative
        // signed fact in the projection and make the durable bond target a plan
        // issue rather than erasing it during positive-scalar resolution.
        return BondRenderTarget::new(
            target,
            first,
            second,
            BondStyle::Unsupported {
                detail: format!(
                    "unsupported signed bond lane placement: bond_width={} requires an uncentered direction-aware double-bond renderer",
                    value.value(),
                ),
            },
            TargetVisibility::Visible,
        )
        .map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                bond.projection_key().as_str(),
                error.to_string(),
            )
        });
    }
    let width = resolved_bond_width(bond, projection, profile)?;
    let lane_spacing = resolved_bond_lane_spacing(bond, projection, profile)?;
    let paint = resolved_bond_paint(bond, projection, profile)?;
    let wedge_width = resolved_bond_wedge_width(bond, projection, profile)?;
    let style = render_bond_style(bond);
    BondRenderTarget::new(target, first, second, style, TargetVisibility::Visible)
        .map(|target| target.with_appearance(width, lane_spacing, wedge_width, paint))
        .map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                bond.projection_key().as_str(),
                error.to_string(),
            )
        })
}

fn render_bond_style(bond: &BondProjectionV1) -> BondStyle {
    match (bond.order(), bond.style(), bond.haworth_position()) {
        (Some(BondOrder::Single), Some(DocumentBondStyle::Normal), _) => BondStyle::NormalSingle,
        (Some(BondOrder::Double), Some(DocumentBondStyle::Normal), _) => BondStyle::Double,
        (Some(BondOrder::Triple), Some(DocumentBondStyle::Normal), _) => BondStyle::Triple,
        (
            Some(BondOrder::Single),
            Some(DocumentBondStyle::Wedge),
            Some(DocumentHaworthPositionV1::Front),
        ) => BondStyle::HaworthFrontWedge,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Wedge), _) => BondStyle::SolidWedge,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Hashed), _) => BondStyle::HashedWedge,
        (
            Some(BondOrder::Single),
            Some(DocumentBondStyle::HaworthFront),
            Some(DocumentHaworthPositionV1::Front),
        ) => BondStyle::HaworthFrontStroke,
        (order, style, position) => BondStyle::Unsupported {
            detail: format!(
                "unsupported CDML bond type {:?}: order={order:?}, style={style:?}, haworth_position={position:?}",
                bond.source_type(),
            ),
        },
    }
}

fn effective_hydrogens(local: Option<VisibilityV1>, projection: &DocumentProjectionV1) -> bool {
    local.or_else(|| {
        projection
            .drawing_standard()
            .and_then(|standard| standard.show_hydrogens())
    }) == Some(VisibilityV1::Enabled)
}

fn atom_target(atom: &AtomProjectionV1) -> Result<RenderTarget, DepictionIssueV1> {
    record_target(
        atom.id(),
        atom.source_id(),
        atom.source_order(),
        RecordKind::Atom,
        atom.projection_key().as_str(),
    )
}

fn bond_target(bond: &BondProjectionV1) -> Result<RenderTarget, DepictionIssueV1> {
    record_target(
        bond.id(),
        bond.source_id(),
        bond.source_order(),
        RecordKind::Bond,
        bond.projection_key().as_str(),
    )
}

fn record_target(
    durable: Option<&ferrum_document_projection::DocumentObjectIdV1>,
    source_id: Option<&str>,
    source_order: u32,
    kind: RecordKind,
    local: &str,
) -> Result<RenderTarget, DepictionIssueV1> {
    let Some(_durable) = durable else {
        return Err(issue(
            DepictionIssueCodeV1::NonDurableTarget,
            local,
            "rendering requires an authored durable source ID",
        ));
    };
    let source_id = source_id.ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            local,
            "durable rendering target has no source identifier",
        )
    })?;
    let identifier = Identifier::new(source_id).map_err(|error| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            local,
            error.to_string(),
        )
    })?;
    Ok(RenderTarget::new(
        RecordId::from_source(kind, &identifier),
        source_order,
    ))
}

fn endpoint_record(
    endpoint: &ferrum_document_projection::BondEndpointV1,
    endpoints: &std::collections::HashMap<DocumentObjectIdV1, RenderTarget>,
    local: &str,
) -> Result<RecordId, DepictionIssueV1> {
    let kind = match endpoint.kind() {
        BondEndpointKindV1::Atom => RecordKind::Atom,
        BondEndpointKindV1::Group => RecordKind::Group,
        _ => {
            return Err(issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                local,
                "bond endpoint is not a renderable atom or compact group",
            ));
        }
    };
    let object_id = endpoint.object_id().ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            local,
            "bond endpoint has no durable document identity",
        )
    })?;
    endpoints
        .get(object_id)
        .filter(|target| target.record_id().kind() == kind)
        .map(|target| target.record_id().clone())
        .ok_or_else(|| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                local,
                "bond endpoint has no renderable durable atom or compact group",
            )
        })
}

fn resolved_font(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
    local: Option<&ferrum_document_projection::FontFactsV1>,
    label_mask: Option<&TransparentOrRgb24V1>,
) -> Result<AtomLabelFontProfile, DepictionIssueV1> {
    let family = local.and_then(|font| font.family()).or_else(|| {
        projection
            .drawing_standard()
            .and_then(|standard| standard.font_family())
    });
    if family.is_some_and(|value| PresentationFontFaceV1::from_cdml_family(value).is_none()) {
        return Err(issue(
            DepictionIssueCodeV1::UnsupportedAuthoredFontFamily,
            "document",
            "V1 has only the verified ferrum-telex-regular-v1 resource",
        ));
    }
    let size = local
        .and_then(|font| font.size())
        .or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.font_size())
        })
        .map(|value| value.value())
        .unwrap_or(12.0);
    let paint = local
        .and_then(|font| font.color())
        .or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.line_color())
        })
        .map(rgb_paint)
        .unwrap_or_else(|| paint("000000"));
    let mut font = AtomLabelFontProfile::new(FontFace::telex_regular(), positive(size)?, paint);
    if let Some(TransparentOrRgb24V1::Rgb24(mask)) = label_mask {
        font = font.with_label_mask(rgb_paint(mask));
    }
    let _ = profile;
    Ok(font)
}

fn resolved_line_width(
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    positive(
        projection
            .drawing_standard()
            .and_then(|standard| standard.line_width())
            .map_or(1.0, |value| value.value()),
    )
}
fn resolved_line_paint(
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<Paint, DepictionIssueV1> {
    Ok(projection
        .drawing_standard()
        .and_then(|standard| standard.line_color())
        .map(rgb_paint)
        .unwrap_or_else(|| paint("000000")))
}
fn resolved_bond_width(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    bond.line_width()
        .map(|value| positive(value.value()))
        .unwrap_or_else(|| resolved_line_width(projection, profile))
}
fn resolved_default_bond_lane_spacing(
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    positive(
        projection
            .drawing_standard()
            .and_then(|standard| standard.bond_width())
            .map_or(BUILTIN_BOND_LANE_SPACING, |value| value.value()),
    )
}
fn resolved_bond_lane_spacing(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    bond.bond_width()
        .map(|value| positive(value.value()))
        .unwrap_or_else(|| resolved_default_bond_lane_spacing(projection, profile))
}
fn resolved_bond_wedge_width(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    positive(
        bond.wedge_width()
            .map(|value| value.value())
            .or_else(|| {
                projection
                    .drawing_standard()
                    .and_then(|standard| standard.wedge_width())
                    .map(|value| value.value())
            })
            .unwrap_or(BUILTIN_BOND_WEDGE_WIDTH),
    )
}
fn resolved_bond_paint(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<Paint, DepictionIssueV1> {
    Ok(bond
        .color()
        .map(rgb_paint)
        .unwrap_or(resolved_line_paint(projection, profile)?))
}
fn positive(value: f64) -> Result<PositiveFinite, DepictionIssueV1> {
    PositiveFinite::new(value).map_err(|error| {
        issue(
            DepictionIssueCodeV1::InvalidPresentationFact,
            "document",
            error.to_string(),
        )
    })
}
fn rgb_paint(value: &DocumentRgb24V1) -> Paint {
    paint(&value.as_str()[1..])
}
fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("validated profile RGB"))
}
fn issue(
    code: DepictionIssueCodeV1,
    target: impl Into<String>,
    detail: impl Into<String>,
) -> DepictionIssueV1 {
    DepictionIssueV1::new(code, target, detail)
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
