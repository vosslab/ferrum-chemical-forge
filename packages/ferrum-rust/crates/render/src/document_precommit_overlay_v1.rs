//! Identifier-free precommit paint data for accepted direct-bond mutations.

use crate::{
    AcceptedRenderOverlayRequestV1, AcceptedRenderOverlayTargetKindV1, BatchSpace,
    DocumentRenderContentV1, DocumentRenderOutcomeV1, DocumentRenderPlanV1, RenderDisplayLayerV1,
    RenderError, RenderOp,
};
use ferrum_core::{Identifier, RecordId, RecordKind};
use std::collections::HashSet;

/// Immutable identifier-free paint data selected from one admitted document plan.
///
/// This value is intentionally inert: it carries only already-selected paint data
/// and cannot be used to recover source targets or affect document admission.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentPrecommitOverlayV1 {
    primitives: Vec<DocumentPrecommitPaintPrimitiveV1>,
}

impl DocumentPrecommitOverlayV1 {
    /// Return the immutable selected paint primitives in renderer draw order.
    #[must_use]
    pub fn primitives(&self) -> &[DocumentPrecommitPaintPrimitiveV1] {
        &self.primitives
    }
}

/// One identifier-free paint primitive copied from an admitted render batch.
///
/// This is intentionally the public observation boundary for a precommit
/// overlay: geometry and paint remain available to a renderer consumer, while
/// the source batch target and every document identifier are discarded.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentPrecommitPaintPrimitiveV1 {
    coordinate_space: BatchSpace,
    display_layer: RenderDisplayLayerV1,
    operation: RenderOp,
}

impl DocumentPrecommitPaintPrimitiveV1 {
    #[must_use]
    pub const fn coordinate_space(&self) -> &BatchSpace {
        &self.coordinate_space
    }

    #[must_use]
    pub const fn display_layer(&self) -> RenderDisplayLayerV1 {
        self.display_layer
    }

    #[must_use]
    pub const fn operation(&self) -> &RenderOp {
        &self.operation
    }
}

/// Construct identifier-free paint data for one renderer-admitted record selection.
pub(super) fn build_document_precommit_overlay_v1(
    plan: &DocumentRenderPlanV1,
    request: &AcceptedRenderOverlayRequestV1,
) -> Result<DocumentPrecommitOverlayV1, RenderError> {
    let mut requested = HashSet::new();
    for target in request.targets() {
        let identifier = Identifier::new(target.source_identifier().to_owned()).map_err(|_| {
            RenderError::InvalidRequest(
                "precommit overlay requires valid persistent target identifiers".to_owned(),
            )
        })?;
        let kind = match target.kind() {
            AcceptedRenderOverlayTargetKindV1::Atom => RecordKind::Atom,
            AcceptedRenderOverlayTargetKindV1::Bond => RecordKind::Bond,
        };
        requested.insert(RecordId::from_source(kind, &identifier));
    }
    let molecule = plan
        .outcomes()
        .iter()
        .find_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root)
                if matches!(root.content(), DocumentRenderContentV1::Molecule(_)) =>
            {
                let DocumentRenderContentV1::Molecule(molecule) = root.content() else {
                    unreachable!("molecule content was matched above");
                };
                molecule
                    .batches()
                    .iter()
                    .any(|batch| requested.contains(batch.target().record_id()))
                    .then_some(molecule)
            }
            _ => None,
        })
        .ok_or_else(|| {
            RenderError::InvalidRequest(
                "precommit overlay selection is absent from the admitted renderer plan".to_owned(),
            )
        })?;
    let batches = molecule
        .batches()
        .iter()
        .filter(|batch| requested.contains(batch.target().record_id()))
        .cloned()
        .collect::<Vec<_>>();
    if batches.len() != requested.len() {
        return Err(RenderError::InvalidRequest(
            "precommit overlay is missing an admitted selected target".to_owned(),
        ));
    }
    let primitives = batches
        .into_iter()
        .flat_map(|batch| {
            let coordinate_space = batch.coordinate_space().clone();
            let display_layer = batch.display_layer();
            batch
                .operations()
                .iter()
                .cloned()
                .map(move |operation| DocumentPrecommitPaintPrimitiveV1 {
                    coordinate_space: coordinate_space.clone(),
                    display_layer,
                    operation,
                })
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(DocumentPrecommitOverlayV1 { primitives })
}
