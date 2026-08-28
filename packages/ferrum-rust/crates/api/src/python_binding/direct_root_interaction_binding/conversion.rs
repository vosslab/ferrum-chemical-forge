//! Conversion between validated Rust interaction values and opaque Python DTOs.

use super::{dto::*, error::RenderInteractionError, types::*};
use crate::{
    CommittedRenderInteractionTranslationV1, CommittedStructureDeletionV1,
    ReactionAuthoringChoiceV1, ReactionAuthoringExclusionV1, ReactionAuthoringObservationV1,
    RenderInteractionBoundsV1, RenderInteractionExclusionReasonV1, RenderInteractionExclusionV1,
    RenderInteractionObservationV1, RenderInteractionRootV1, RenderInteractionSelectionV1,
    RenderInteractionTranslationPreviewV1, StructureInteractionObservationV1,
    StructureInteractionSelectionV1, StructureInteractionTargetV1,
};
use ferrum_document::DocumentFenceV1;
use pyo3::prelude::*;

pub(super) fn root(py: Python<'_>, value: &RenderInteractionRootV1) -> PyResult<Py<PyRoot>> {
    Py::new(
        py,
        PyRoot {
            document_object_id: value.document_object_id().as_str().to_owned(),
            paint_order: value.paint_order(),
            kind: root_kind(value.kind()),
            bounds: bounds(value.bounds()),
        },
    )
}
pub(super) fn roots(
    py: Python<'_>,
    values: &[RenderInteractionRootV1],
) -> PyResult<Vec<Py<PyRoot>>> {
    values.iter().map(|value| root(py, value)).collect()
}
pub(super) fn bounds(value: RenderInteractionBoundsV1) -> PyBounds {
    PyBounds {
        left: value.left(),
        top: value.top(),
        right: value.right(),
        bottom: value.bottom(),
    }
}
pub(super) fn exclusion_reason(value: RenderInteractionExclusionReasonV1) -> PyExclusionReason {
    match value {
        RenderInteractionExclusionReasonV1::UnrenderableDepiction => {
            PyExclusionReason::UnrenderableDepiction
        }
        RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier => {
            PyExclusionReason::AmbiguousRootIdentifier
        }
        RenderInteractionExclusionReasonV1::DisplayOnly => PyExclusionReason::DisplayOnly,
    }
}
pub(super) fn exclusions(
    py: Python<'_>,
    values: &[RenderInteractionExclusionV1],
) -> PyResult<Vec<Py<PyExclusion>>> {
    values
        .iter()
        .map(|value| {
            Py::new(
                py,
                PyExclusion {
                    document_object_id: value.document_object_id().as_str().to_owned(),
                    reason: exclusion_reason(value.reason()),
                },
            )
        })
        .collect()
}
pub(super) fn reaction_authoring_observation(
    py: Python<'_>,
    value: &ReactionAuthoringObservationV1,
) -> PyResult<Py<PyReactionAuthoringObservation>> {
    let choices = value
        .choices()
        .iter()
        .map(|choice| reaction_choice(py, choice))
        .collect::<PyResult<_>>()?;
    let exclusions = value
        .exclusions()
        .iter()
        .map(|exclusion| reaction_exclusion(py, exclusion))
        .collect::<PyResult<_>>()?;
    Py::new(
        py,
        PyReactionAuthoringObservation {
            choices,
            exclusions,
        },
    )
}
pub(super) fn reaction_choice(
    py: Python<'_>,
    value: &ReactionAuthoringChoiceV1,
) -> PyResult<Py<PyReactionChoice>> {
    Py::new(
        py,
        PyReactionChoice {
            document_object_id: value.document_object_id().as_str().to_owned(),
            document_paint_order: value.paint_order(),
            kind: reaction_choice_kind(value.kind()),
            availability: reaction_choice_availability(value.availability()),
            label: value.label().to_owned(),
            bounds: bounds(value.bounds()),
        },
    )
}
pub(super) fn reaction_exclusion(
    py: Python<'_>,
    value: &ReactionAuthoringExclusionV1,
) -> PyResult<Py<PyReactionExclusion>> {
    Py::new(
        py,
        PyReactionExclusion {
            diagnostic_key: value.diagnostic_key().to_owned(),
            reason: reaction_exclusion_reason(value.reason()),
            recovery: reaction_exclusion_recovery(value.recovery()),
            label: value.label().to_owned(),
        },
    )
}
pub(super) fn observation(
    py: Python<'_>,
    value: RenderInteractionObservationV1,
) -> PyResult<PyObservation> {
    let fence = value.fence();
    let roots = roots(py, value.roots())?;
    let exclusions = exclusions(py, value.exclusions())?;
    let reaction_authoring = reaction_authoring_observation(py, value.reaction_authoring())?;
    Ok(PyObservation {
        value,
        roots,
        exclusions,
        reaction_authoring,
        revision: fence.revision(),
        digest: hex_digest(&fence.digest()),
    })
}
pub(super) fn selection(
    py: Python<'_>,
    value: RenderInteractionSelectionV1,
) -> PyResult<PySelection> {
    let roots = roots(py, value.roots())?;
    Ok(PySelection { value, roots })
}
pub(super) fn preview(
    py: Python<'_>,
    value: RenderInteractionTranslationPreviewV1,
) -> PyResult<PyPreview> {
    let bounds = value
        .bounds()
        .iter()
        .copied()
        .map(|value| Py::new(py, bounds(value)))
        .collect::<PyResult<_>>()?;
    Ok(PyPreview {
        dx: value.dx(),
        dy: value.dy(),
        bounds,
    })
}
pub(super) fn commit(
    py: Python<'_>,
    value: CommittedRenderInteractionTranslationV1,
) -> PyResult<PyCommit> {
    let selection = Py::new(
        py,
        PySelectionFacts {
            roots: roots(py, value.selection().roots())?,
        },
    )?;
    Ok(PyCommit {
        changed: value.changed(),
        result: value.result().clone().into(),
        selection,
    })
}
pub(super) fn structure_target(
    py: Python<'_>,
    value: &StructureInteractionTargetV1,
) -> PyResult<Py<PyStructureTarget>> {
    Py::new(
        py,
        PyStructureTarget {
            molecule_object_id: value.molecule_object_id().as_str().to_owned(),
            object_id: value.object_id().as_str().to_owned(),
            kind: structure_kind(value.kind()),
            bounds: bounds(value.bounds()),
        },
    )
}
pub(super) fn structure_targets(
    py: Python<'_>,
    values: &[StructureInteractionTargetV1],
) -> PyResult<Vec<Py<PyStructureTarget>>> {
    values
        .iter()
        .map(|value| structure_target(py, value))
        .collect()
}
pub(super) fn structure_observation(
    py: Python<'_>,
    value: StructureInteractionObservationV1,
) -> PyResult<PyStructureObservation> {
    let fence = value.fence();
    let targets = structure_targets(py, value.targets())?;
    Ok(PyStructureObservation {
        value,
        targets,
        revision: fence.revision(),
        digest: hex_digest(&fence.digest()),
    })
}
pub(super) fn structure_selection(
    py: Python<'_>,
    value: StructureInteractionSelectionV1,
) -> PyResult<PyStructureSelection> {
    let fence = value.fence();
    let targets = structure_targets(py, value.targets())?;
    Ok(PyStructureSelection {
        value,
        targets,
        revision: fence.revision(),
        digest: hex_digest(&fence.digest()),
    })
}
pub(super) fn structure_commit(value: CommittedStructureDeletionV1) -> PyStructureCommit {
    PyStructureCommit {
        result: value.result().clone().into(),
        removed_atom_count: value.removed_atom_count(),
        removed_bond_count: value.removed_bond_count(),
        removed_compact_group_count: value.removed_compact_group_count(),
    }
}
pub(super) fn fence(digest: &str, revision: u64) -> PyResult<DocumentFenceV1> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(RenderInteractionError::new_err(
            "expected digest must be exactly 64 lowercase hexadecimal characters",
        ));
    }
    let mut bytes = [0; 32];
    for (index, pair) in digest.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        bytes[index] = (hex(pair[0]) << 4) | hex(pair[1]);
    }
    Ok(DocumentFenceV1::new(revision, bytes))
}
const fn hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}
pub(super) fn hex_digest(value: &[u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}
