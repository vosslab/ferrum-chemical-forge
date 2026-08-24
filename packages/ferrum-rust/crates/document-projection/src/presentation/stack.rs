//! Immutable presentation-stack projection values.

#[path = "stack_model.rs"]
mod stack_model;
#[cfg(test)]
#[path = "stack_tests.rs"]
mod stack_tests;
#[path = "stack_wire.rs"]
mod stack_wire;

pub use stack_model::{
    PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PolylinePathV1, PolylineProjectionV1,
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationRecordKindV1, PresentationRootProjectionV1, PresentationStackProjectionV1,
    PresentationStackProjectionV1Error, PresentationStrokeV1, PresentationTargetV1,
};
