//! API-owned composition of one final observation into a whole-page render plan.

use std::collections::{HashMap, HashSet};

use crate::{
    DocumentRenderContentV1, DocumentRenderExclusionV1, DocumentRenderIdentityV1,
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, DocumentRenderRootV1, DocumentTextOpV1,
    GlyphBounds, RenderError, RenderProvenance, RenderRevision, RenderViewportV1,
};
use ferrum_document::{MoleculeProjectionV1, PresentationRootProjectionV1, PresentationTargetV1};
use thiserror::Error;

use crate::presentation::vector::lower_presentation_vector_v1;
use crate::{DepictionSuppressionV1, RenderObservationV1};

/// Compose one authoritative observation into a renderer-neutral page plan.
///
/// This in-process boundary owns the merge of direct document roots. It never reads
/// CDML, re-lays out text, or infers a visual replacement for an excluded root.
pub fn compose_document_render_plan_v1(
    observation: &RenderObservationV1,
) -> Result<DocumentRenderPlanV1, DocumentRenderPlanCompositionError> {
    if let Some(suppression) = observation.suppression() {
        return Err(DocumentRenderPlanCompositionError::Suppressed { suppression });
    }

    let projection = observation.document().projection();
    let page = projection.paper_layout().page();
    let viewport = page_viewport(
        page.scene_left(),
        page.scene_top(),
        page.scene_right(),
        page.scene_bottom(),
    )?;
    let provenance = RenderProvenance::new(
        RenderRevision::new(observation.document().snapshot().revision())?,
        *observation.document().snapshot().digest(),
    );

    let mut molecule_plans = HashMap::new();
    molecule_plans
        .try_reserve(observation.molecule_plans().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for entry in observation.molecule_plans() {
        insert_unique(
            &mut molecule_plans,
            entry.molecule().source_order(),
            entry,
            "molecule render plans",
        )?;
    }

    let mut plus_renders = HashMap::new();
    plus_renders
        .try_reserve(observation.plus_renders().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for entry in observation.plus_renders() {
        insert_unique(
            &mut plus_renders,
            entry.target().source_order(),
            entry,
            "plus render layouts",
        )?;
    }

    let mut text_renders = HashMap::new();
    text_renders
        .try_reserve(observation.text_renders().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for entry in observation.text_renders() {
        insert_unique(
            &mut text_renders,
            entry.target().source_order(),
            entry,
            "Text render layouts",
        )?;
    }

    let roots = projection.presentation_stack().roots();
    let mut retained_targets = HashSet::new();
    retained_targets
        .try_reserve(roots.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for root in roots {
        if !retained_targets.insert(root.target().projection_key().as_str()) {
            return Err(invalid(
                "presentation roots contain a duplicate projection key",
            ));
        }
    }

    let mut issues_by_target = HashMap::new();
    issues_by_target
        .try_reserve(observation.issues().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for issue in observation.issues() {
        issues_by_target.entry(issue.target()).or_insert(issue);
    }

    let mut outcomes = Vec::new();
    outcomes
        .try_reserve(
            projection.molecules().len()
                + roots.len()
                + projection.presentation_stack().issues().len(),
        )
        .map_err(|_| RenderError::ResourceExhausted)?;
    let mut identities = HashSet::new();
    identities
        .try_reserve(outcomes.capacity())
        .map_err(|_| RenderError::ResourceExhausted)?;
    let mut source_orders = HashSet::new();
    source_orders
        .try_reserve(outcomes.capacity())
        .map_err(|_| RenderError::ResourceExhausted)?;

    for molecule in projection.molecules() {
        let Some(entry) = molecule_plans.remove(&molecule.source_order()) else {
            return Err(invalid("authoritative molecule root has no render plan"));
        };
        let identity = molecule_identity(molecule)?;
        if entry.molecule().id() != molecule.id().map(|id| id.as_str())
            || entry.molecule().projection_key() != molecule.projection_key().as_str()
            || entry.molecule().source_order() != molecule.source_order()
            || entry.provenance() != provenance
        {
            return Err(invalid(
                "molecule render plan does not match its authoritative root",
            ));
        }
        insert_outcome(
            &mut outcomes,
            &mut identities,
            &mut source_orders,
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                molecule.source_order(),
                identity,
                DocumentRenderContentV1::Molecule(entry.plan().clone()),
            )),
        )?;
    }
    if !molecule_plans.is_empty() {
        return Err(invalid(
            "render observation contains a molecule plan without an authoritative root",
        ));
    }

    for root in roots {
        let target = root.target();
        let identity = target_identity(target)?;
        let outcome = match root {
            PresentationRootProjectionV1::Plus { plus } => {
                if let Some(render) = plus_renders.remove(&target.source_order()) {
                    if render.target() != plus.target() {
                        return Err(invalid("plus layout does not match its authoritative root"));
                    }
                    let bounds = glyph_bounds(render.bounds())?;
                    DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                        target.source_order(),
                        identity,
                        DocumentRenderContentV1::Text(DocumentTextOpV1::fixed(
                            render.anchor(),
                            render.operation().clone(),
                            bounds,
                            render.background().cloned(),
                        )?),
                    ))
                } else {
                    profile_exclusion(
                        target,
                        issues_by_target.get(target.projection_key().as_str()),
                    )?
                }
            }
            PresentationRootProjectionV1::Text { text } => {
                if let Some(render) = text_renders.remove(&target.source_order()) {
                    if render.target() != text.target() {
                        return Err(invalid("Text layout does not match its authoritative root"));
                    }
                    let bounds = glyph_bounds(render.bounds())?;
                    DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                        target.source_order(),
                        identity,
                        DocumentRenderContentV1::Text(DocumentTextOpV1::presentation(
                            render.anchor(),
                            render.operation().clone(),
                            bounds,
                            render.background().cloned(),
                        )?),
                    ))
                } else {
                    profile_exclusion(
                        target,
                        issues_by_target.get(target.projection_key().as_str()),
                    )?
                }
            }
            PresentationRootProjectionV1::Arrow { .. }
            | PresentationRootProjectionV1::Polyline { .. }
            | PresentationRootProjectionV1::Wavy { .. }
            | PresentationRootProjectionV1::RoundBracket { .. }
            | PresentationRootProjectionV1::Rectangle { .. }
            | PresentationRootProjectionV1::Square { .. }
            | PresentationRootProjectionV1::Oval { .. }
            | PresentationRootProjectionV1::Circle { .. }
            | PresentationRootProjectionV1::Polygon { .. } => {
                DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                    target.source_order(),
                    identity,
                    DocumentRenderContentV1::Vector(lower_presentation_vector_v1(root)?),
                ))
            }
        };
        insert_outcome(&mut outcomes, &mut identities, &mut source_orders, outcome)?;
    }
    if !plus_renders.is_empty() || !text_renders.is_empty() {
        return Err(invalid(
            "render observation contains a presentation layout without an authoritative root",
        ));
    }

    let mut rejected_projection_targets = HashSet::new();
    rejected_projection_targets
        .try_reserve(projection.presentation_stack().issues().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for issue in projection.presentation_stack().issues() {
        let target = issue.target();
        if retained_targets.contains(target.projection_key().as_str()) {
            continue;
        }
        if !rejected_projection_targets.insert(target.projection_key().as_str()) {
            continue;
        }
        let identity = target_identity(target)?;
        insert_outcome(
            &mut outcomes,
            &mut identities,
            &mut source_orders,
            DocumentRenderOutcomeV1::Exclusion(DocumentRenderExclusionV1::new(
                target.source_order(),
                identity,
                format!("rejected_projection:{:?}", issue.code()),
            )?),
        )?;
    }

    outcomes.sort_unstable_by_key(DocumentRenderOutcomeV1::source_order);
    Ok(DocumentRenderPlanV1::new(provenance, viewport, outcomes)?)
}

/// Failure while composing a page plan from a final render observation.
#[derive(Debug, Error)]
pub enum DocumentRenderPlanCompositionError {
    /// The observation intentionally contains no render payload because presentation facts are invalid.
    #[error("document render composition was suppressed: {suppression:?}")]
    Suppressed { suppression: DepictionSuppressionV1 },
    /// Observation facts or checked render-model construction were inconsistent.
    #[error(transparent)]
    Render(#[from] RenderError),
}

fn page_viewport(
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
) -> Result<RenderViewportV1, RenderError> {
    let width = right - left;
    let height = bottom - top;
    if !width.is_finite() || !height.is_finite() {
        return Err(RenderError::InvalidRequest(
            "document paper page extents overflowed while composing the viewport".to_owned(),
        ));
    }
    RenderViewportV1::new(left, top, width, height)
}

fn glyph_bounds(bounds: crate::PresentationTextBoundsV1) -> Result<GlyphBounds, RenderError> {
    GlyphBounds::new(bounds.left(), bounds.top(), bounds.right(), bounds.bottom())
}

fn molecule_identity(
    molecule: &MoleculeProjectionV1,
) -> Result<DocumentRenderIdentityV1, RenderError> {
    match molecule.id() {
        Some(id) => DocumentRenderIdentityV1::durable(id.as_str()),
        None => DocumentRenderIdentityV1::projection_local(molecule.projection_key().as_str()),
    }
}

fn target_identity(target: &PresentationTargetV1) -> Result<DocumentRenderIdentityV1, RenderError> {
    match target.id() {
        Some(id) => DocumentRenderIdentityV1::durable(id.as_str()),
        None => DocumentRenderIdentityV1::projection_local(target.projection_key().as_str()),
    }
}

fn profile_exclusion(
    target: &PresentationTargetV1,
    issue: Option<&&crate::DepictionIssueV1>,
) -> Result<DocumentRenderOutcomeV1, RenderError> {
    let Some(issue) = issue else {
        return Err(RenderError::InvalidRequest(
            "retained Plus or Text root has no verified layout or profile exclusion".to_owned(),
        ));
    };
    Ok(DocumentRenderOutcomeV1::Exclusion(
        DocumentRenderExclusionV1::new(
            target.source_order(),
            target_identity(target)?,
            format!("profile_excluded:{:?}", issue.code()),
        )?,
    ))
}

fn insert_unique<'a, T>(
    values: &mut HashMap<u32, &'a T>,
    source_order: u32,
    value: &'a T,
    name: &str,
) -> Result<(), RenderError> {
    if values.insert(source_order, value).is_some() {
        return Err(RenderError::InvalidRequest(format!(
            "{name} contain duplicate source orders"
        )));
    }
    Ok(())
}

fn insert_outcome(
    outcomes: &mut Vec<DocumentRenderOutcomeV1>,
    identities: &mut HashSet<DocumentRenderIdentityV1>,
    source_orders: &mut HashSet<u32>,
    outcome: DocumentRenderOutcomeV1,
) -> Result<(), RenderError> {
    if !identities.insert(outcome.identity().clone()) {
        return Err(RenderError::InvalidRequest(
            "document roots and exclusions contain duplicate identities".to_owned(),
        ));
    }
    if !source_orders.insert(outcome.source_order()) {
        return Err(RenderError::InvalidRequest(
            "document roots and exclusions contain duplicate source orders".to_owned(),
        ));
    }
    outcomes.push(outcome);
    Ok(())
}

fn invalid(message: &str) -> DocumentRenderPlanCompositionError {
    DocumentRenderPlanCompositionError::Render(RenderError::InvalidRequest(message.to_owned()))
}
