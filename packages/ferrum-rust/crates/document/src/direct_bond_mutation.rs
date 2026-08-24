//! Public semantic endpoint values for native direct-bond authoring.

use crate::DirectBondPoint2V1;

/// Explicitly resolved input for native, noninteractive direct-bond mutation.
///
/// Pointer probing, viewport transforms, render plans, and admission proofs are
/// deliberately outside this semantic input boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum DirectBondEndpointIntent {
    ExistingAtom { atom: crate::DocumentObjectIdV1 },
    NewAtomAt { raw_point: DirectBondPoint2V1 },
}
