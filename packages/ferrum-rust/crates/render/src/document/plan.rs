//! API-owned composition of one final observation into a whole-page render plan.

use std::collections::HashMap;
use std::hash::Hash;

use crate::{
    DocumentMoleculeRenderContentV1, DocumentRenderContentV1, DocumentRenderExclusionV1,
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, DocumentRenderRootV1, DocumentTextOpV1,
    GlyphBounds, RenderError, RenderProvenance, RenderRevision, RenderTarget, RenderViewportV1,
};
use ferrum_document_projection::{DocumentDirectRootKindV1, PresentationRootProjectionV1};
use thiserror::Error;

use crate::presentation::vector::lower_presentation_vector_v1;
use crate::{DepictionSuppressionV1, ResolvedDocumentRenderV1};

/// Compose one authoritative observation into a renderer-neutral page plan.
///
/// This in-process boundary owns the merge of direct document roots. It never reads
/// CDML, re-lays out text, or infers a visual replacement for an excluded root.
pub fn compose_document_render_plan_v1(
    observation: &ResolvedDocumentRenderV1,
) -> Result<DocumentRenderPlanV1, DocumentRenderPlanCompositionError> {
    if let Some(suppression) = observation.suppression() {
        return Err(DocumentRenderPlanCompositionError::Suppressed { suppression });
    }

    let projection = observation.projection();
    let page = projection.paper_layout().page();
    let viewport = page_viewport(
        page.scene_left(),
        page.scene_top(),
        page.scene_right(),
        page.scene_bottom(),
    )?;
    let provenance = RenderProvenance::new(
        RenderRevision::new(observation.projection().revision())?,
        *observation.projection().digest(),
    );

    let mut molecule_plans = HashMap::new();
    molecule_plans
        .try_reserve(observation.molecule_plans().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for entry in observation.molecule_plans() {
        insert_unique(
            &mut molecule_plans,
            entry.molecule().document_object_id().as_str(),
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
            entry.target().document_object_id().as_str(),
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
            entry.target().document_object_id().as_str(),
            entry,
            "Text render layouts",
        )?;
    }

    let mut molecule_roots = HashMap::new();
    molecule_roots
        .try_reserve(projection.molecules().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for molecule in projection.molecules() {
        insert_unique(
            &mut molecule_roots,
            molecule.document_object_id().as_str(),
            molecule,
            "authoritative molecule roots",
        )?;
    }

    let entries = projection.presentation_stack().entries();
    let mut presentation_roots = HashMap::new();
    presentation_roots
        .try_reserve(entries.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for entry in entries {
        let root = entry.root();
        insert_unique(
            &mut presentation_roots,
            root.target().document_object_id().as_str(),
            root,
            "presentation roots",
        )?;
    }

    let mut rejected_presentations = HashMap::new();
    rejected_presentations
        .try_reserve(projection.presentation_stack().issues().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for issue in projection.presentation_stack().issues() {
        insert_unique(
            &mut rejected_presentations,
            issue.target().document_object_id().as_str(),
            issue,
            "rejected presentation roots",
        )?;
    }
    if presentation_roots
        .keys()
        .any(|target| rejected_presentations.contains_key(target))
    {
        return Err(invalid(
            "presentation root and rejected presentation share a durable target",
        ));
    }
    let mut outcomes = Vec::new();
    outcomes
        .try_reserve(projection.direct_roots().len())
        .map_err(|_| RenderError::ResourceExhausted)?;

    for direct_root in projection.direct_roots() {
        let target = direct_root.document_object_id();
        let paint_order = direct_root.paint_order();
        let outcome = match direct_root.kind() {
            DocumentDirectRootKindV1::Molecule => {
                let Some(molecule) = molecule_roots.remove(target.as_str()) else {
                    return Err(invalid("direct molecule root has no molecule payload"));
                };
                let Some(entry) = molecule_plans.remove(target.as_str()) else {
                    return Err(invalid("direct molecule root has no render plan"));
                };
                if molecule.document_object_id() != target
                    || entry.molecule().document_object_id() != target
                    || entry.provenance() != provenance
                {
                    return Err(invalid(
                        "molecule render payload does not match its direct root",
                    ));
                }
                DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                    RenderTarget::document_object(target.clone()),
                    paint_order,
                    DocumentRenderContentV1::Molecule(DocumentMoleculeRenderContentV1::new(
                        entry.plan().clone(),
                        entry.member_issues().to_vec(),
                    )),
                ))
            }
            DocumentDirectRootKindV1::Presentation(kind) => {
                let Some(root) = presentation_roots.remove(target.as_str()) else {
                    return Err(invalid(
                        "direct presentation root has no presentation payload",
                    ));
                };
                if root.target().document_object_id() != target
                    || root.target().record_kind() != kind
                {
                    return Err(invalid(
                        "presentation payload kind does not match its direct root",
                    ));
                }
                presentation_outcome(root, paint_order, &mut plus_renders, &mut text_renders)?
            }
            DocumentDirectRootKindV1::RejectedPresentation(code) => {
                let Some(issue) = rejected_presentations.remove(target.as_str()) else {
                    return Err(invalid("rejected presentation root has no issue payload"));
                };
                if issue.target().document_object_id() != target || issue.code() != code {
                    return Err(invalid(
                        "rejected presentation issue does not match its direct root",
                    ));
                }
                DocumentRenderOutcomeV1::Exclusion(DocumentRenderExclusionV1::new(
                    RenderTarget::document_object(target.clone()),
                    paint_order,
                    format!("rejected_projection:{code:?}"),
                )?)
            }
        };
        outcomes.push(outcome);
    }
    if !molecule_roots.is_empty()
        || !molecule_plans.is_empty()
        || !presentation_roots.is_empty()
        || !plus_renders.is_empty()
        || !text_renders.is_empty()
        || !rejected_presentations.is_empty()
    {
        return Err(invalid(
            "render observation contains payload without a matching direct root",
        ));
    }
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

fn presentation_outcome(
    root: &PresentationRootProjectionV1,
    paint_order: u32,
    plus_renders: &mut HashMap<&str, &crate::DocumentPlusRenderV1>,
    text_renders: &mut HashMap<&str, &crate::DocumentTextRenderV1>,
) -> Result<DocumentRenderOutcomeV1, RenderError> {
    let target = root.target();
    let render_target = RenderTarget::document_object(target.document_object_id().clone());
    match root {
        PresentationRootProjectionV1::Plus { plus } => {
            let Some(render) = plus_renders.remove(target.document_object_id().as_str()) else {
                return Err(RenderError::InvalidRequest(
                    "direct Plus root has no verified layout".to_owned(),
                ));
            };
            if render.target() != plus.target() {
                return Err(RenderError::InvalidRequest(
                    "plus layout does not match its direct root".to_owned(),
                ));
            }
            let bounds = glyph_bounds(render.bounds())?;
            Ok(DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                render_target,
                paint_order,
                DocumentRenderContentV1::Text(DocumentTextOpV1::fixed(
                    render.anchor(),
                    render.operation().clone(),
                    bounds,
                    render.background().cloned(),
                )?),
            )))
        }
        PresentationRootProjectionV1::Text { text } => {
            let Some(render) = text_renders.remove(target.document_object_id().as_str()) else {
                return Err(RenderError::InvalidRequest(
                    "direct Text root has no verified layout".to_owned(),
                ));
            };
            if render.target() != text.target() {
                return Err(RenderError::InvalidRequest(
                    "Text layout does not match its direct root".to_owned(),
                ));
            }
            let bounds = glyph_bounds(render.bounds())?;
            Ok(DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                render_target,
                paint_order,
                DocumentRenderContentV1::Text(DocumentTextOpV1::presentation(
                    render.anchor(),
                    render.operation().clone(),
                    bounds,
                    render.background().cloned(),
                )?),
            )))
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
            Ok(DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                render_target,
                paint_order,
                DocumentRenderContentV1::Vector(lower_presentation_vector_v1(root)?),
            )))
        }
    }
}

fn insert_unique<'a, K, T>(
    values: &mut HashMap<K, &'a T>,
    key: K,
    value: &'a T,
    name: &str,
) -> Result<(), RenderError>
where
    K: Eq + Hash,
{
    if values.insert(key, value).is_some() {
        return Err(RenderError::InvalidRequest(format!(
            "{name} contain duplicate durable targets"
        )));
    }
    Ok(())
}

fn invalid(message: &str) -> DocumentRenderPlanCompositionError {
    DocumentRenderPlanCompositionError::Render(RenderError::InvalidRequest(message.to_owned()))
}
