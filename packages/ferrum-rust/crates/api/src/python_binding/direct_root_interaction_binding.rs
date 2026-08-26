//! Opaque PyO3 facade for render-evidence-backed direct-root interaction.
//!
//! The public Python module remains a single contract.  Private submodules
//! separate input construction, opaque state, session operations, conversion,
//! and failure classification so each responsibility can evolve independently.

mod conversion;
mod dto;
mod error;
mod query;
mod session;
mod types;

use pyo3::prelude::*;

use dto::*;
use error::*;
use query::*;
use types::*;

#[cfg(test)]
pub(crate) use dto::test_selection_from_value_v1;
pub(crate) use dto::{
    PySelection, SelectedDirectRootV1, selected_direct_root_from_value_v1, selection_value_v1,
};

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "RenderInteractionError",
        module.py().get_type::<RenderInteractionError>(),
    )?;
    module.add_class::<PyCategory>()?;
    module.add_class::<PyRecovery>()?;
    module.add_class::<PyExclusionReason>()?;
    module.add_class::<PyModifier>()?;
    module.add_class::<PyAxis>()?;
    module.add_class::<PyGridSnapPolicy>()?;
    module.add_class::<PyStructureTargetKind>()?;
    module.add_class::<PyRootKind>()?;
    module.add_class::<PyReactionChoiceKind>()?;
    module.add_class::<PyReactionChoiceAvailability>()?;
    module.add_class::<PyReactionExclusionReason>()?;
    module.add_class::<PyReactionExclusionRecovery>()?;
    module.add_class::<PyQuery>()?;
    module.add_class::<PyStructureQuery>()?;
    module.add_class::<PySnap>()?;
    module.add_class::<PyBounds>()?;
    module.add_class::<PyRoot>()?;
    module.add_class::<PyExclusion>()?;
    module.add_class::<PyReactionChoice>()?;
    module.add_class::<PyReactionExclusion>()?;
    module.add_class::<PyReactionAuthoringObservation>()?;
    module.add_class::<PyStructureTarget>()?;
    module.add_class::<PyObservation>()?;
    module.add_class::<PySelection>()?;
    module.add_class::<PyStructureObservation>()?;
    module.add_class::<PyStructureSelection>()?;
    module.add_class::<PyStructureCommit>()?;
    module.add_class::<PyGesture>()?;
    module.add_class::<PyPreview>()?;
    module.add_class::<PySelectionFacts>()?;
    module.add_class::<PyCommit>()?;
    Ok(())
}
