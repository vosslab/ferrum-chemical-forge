//! Protocol execution failure translation.

use super::*;

#[derive(Debug)]
pub(crate) struct ExecutionFailureV1 {
    pub(super) category: OperationProtocolErrorCategoryV1,
    pub(super) message: String,
    pub(super) presentation_vector_refusal: Option<PresentationVectorRefusalV1>,
    pub(super) catalog_placement_refusal: Option<CatalogPlacementRefusalV1>,
    pub(super) reaction_refusal: Option<ReactionRefusalV1>,
}

impl ExecutionFailureV1 {
    pub(super) fn interchange_import_refusal(refusal: crate::InterchangeImportRefusalV1) -> Self {
        let category = match refusal.category() {
            crate::InterchangeImportRefusalCategoryV1::ConversionFailed => {
                OperationProtocolErrorCategoryV1::ConversionFailed
            }
            crate::InterchangeImportRefusalCategoryV1::ConversionUnsupported => {
                OperationProtocolErrorCategoryV1::ConversionUnsupported
            }
            crate::InterchangeImportRefusalCategoryV1::ResourceLimit => {
                OperationProtocolErrorCategoryV1::ResourceLimit
            }
            crate::InterchangeImportRefusalCategoryV1::DocumentAdmissionFailed
            | crate::InterchangeImportRefusalCategoryV1::StaleDocument => {
                OperationProtocolErrorCategoryV1::DocumentAdmissionFailed
            }
            crate::InterchangeImportRefusalCategoryV1::ChemistryUnavailable => {
                OperationProtocolErrorCategoryV1::ChemistryUnavailable
            }
        };
        Self {
            category,
            message: format!("interchange_import_refused:{:?}", refusal.reason()),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }
    pub(super) fn document_admission(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentAdmissionFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(in crate::protocol) fn document_invalid(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentInvalid,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn render_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderUnsupported,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn render_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(in crate::protocol) fn chemistry_unavailable(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ChemistryUnavailable,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn chemistry_runtime_unavailable() -> Self {
        Self::chemistry_unavailable("Ferrum chemistry runtime is unavailable".to_owned())
    }

    pub(super) fn conversion_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn conversion_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionUnsupported,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn coordinate(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::CoordinateGenerationFailed,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(in crate::protocol) fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            message: message.into(),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn internal(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::InternalFailure,
            message,
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn vector_refusal(error: PresentationVectorGestureErrorV1) -> Self {
        Self {
            category: match error.category() {
                PresentationVectorGestureCategoryV1::RenderPreparation => {
                    OperationProtocolErrorCategoryV1::RenderFailed
                }
                PresentationVectorGestureCategoryV1::ResourceExhausted => {
                    OperationProtocolErrorCategoryV1::ResourceLimit
                }
                _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
            },
            message: error.to_string(),
            presentation_vector_refusal: Some(PresentationVectorRefusalV1 {
                category: vector_category(error.category()),
                recovery: vector_recovery(error.recovery()),
            }),
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn catalog_refusal(error: CatalogPlacementErrorV2) -> Self {
        Self {
            category: match error.category() {
                CatalogPlacementCategoryV2::RenderPreparation => {
                    OperationProtocolErrorCategoryV1::RenderFailed
                }
                _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
            },
            message: error.to_string(),
            presentation_vector_refusal: None,
            catalog_placement_refusal: Some(CatalogPlacementRefusalV1 {
                category: catalog_category(error.category()),
                recovery: catalog_recovery(error.recovery()),
            }),
            reaction_refusal: None,
        }
    }

    pub(super) fn reaction_refusal(error: ReactionGestureErrorV1) -> Self {
        Self {
            category: match error.category() {
                ReactionGestureCategoryV1::UnrenderableDocument
                | ReactionGestureCategoryV1::RenderPreparation => {
                    OperationProtocolErrorCategoryV1::RenderFailed
                }
                _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
            },
            message: error.to_string(),
            presentation_vector_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: Some(ReactionRefusalV1 {
                category: reaction_category(error.category()),
                recovery: reaction_recovery(error.recovery()),
            }),
        }
    }
}

pub(super) fn reaction_category(
    value: ReactionGestureCategoryV1,
) -> ProtocolReactionRefusalCategoryV1 {
    match value {
        ReactionGestureCategoryV1::StaleSnapshot => {
            ProtocolReactionRefusalCategoryV1::StaleSnapshot
        }
        ReactionGestureCategoryV1::ForeignSession => {
            ProtocolReactionRefusalCategoryV1::ForeignSession
        }
        ReactionGestureCategoryV1::ReplayedGesture => {
            ProtocolReactionRefusalCategoryV1::ReplayedGesture
        }
        ReactionGestureCategoryV1::InvalidRequest => {
            ProtocolReactionRefusalCategoryV1::InvalidRequest
        }
        ReactionGestureCategoryV1::MissingTarget => {
            ProtocolReactionRefusalCategoryV1::MissingTarget
        }
        ReactionGestureCategoryV1::WrongTargetKind => {
            ProtocolReactionRefusalCategoryV1::WrongTargetKind
        }
        ReactionGestureCategoryV1::DuplicateTarget => {
            ProtocolReactionRefusalCategoryV1::DuplicateTarget
        }
        ReactionGestureCategoryV1::CrossReactionReuse => {
            ProtocolReactionRefusalCategoryV1::CrossReactionReuse
        }
        ReactionGestureCategoryV1::UnrenderableDocument => {
            ProtocolReactionRefusalCategoryV1::UnrenderableDocument
        }
        ReactionGestureCategoryV1::RenderPreparation => {
            ProtocolReactionRefusalCategoryV1::RenderPreparation
        }
        ReactionGestureCategoryV1::SessionConflict => {
            ProtocolReactionRefusalCategoryV1::SessionConflict
        }
        ReactionGestureCategoryV1::MissingReaction => {
            ProtocolReactionRefusalCategoryV1::MissingReaction
        }
        ReactionGestureCategoryV1::LegacyDefinitionNotEditable => {
            ProtocolReactionRefusalCategoryV1::LegacyDefinitionNotEditable
        }
        ReactionGestureCategoryV1::MembershipChanged => {
            ProtocolReactionRefusalCategoryV1::MembershipChanged
        }
        ReactionGestureCategoryV1::RendererExclusion => {
            ProtocolReactionRefusalCategoryV1::RendererExclusion
        }
        _ => unreachable!("a new reaction category requires protocol mapping"),
    }
}

pub(super) fn reaction_recovery(
    value: ReactionGestureRecoveryV1,
) -> ProtocolReactionRefusalRecoveryV1 {
    match value {
        ReactionGestureRecoveryV1::RefreshAndRestart => {
            ProtocolReactionRefusalRecoveryV1::RefreshAndRestart
        }
        ReactionGestureRecoveryV1::CorrectSelectors => {
            ProtocolReactionRefusalRecoveryV1::CorrectSelectors
        }
        ReactionGestureRecoveryV1::ChooseRenderableMembers => {
            ProtocolReactionRefusalRecoveryV1::ChooseRenderableMembers
        }
        ReactionGestureRecoveryV1::RepairLegacyDefinition => {
            ProtocolReactionRefusalRecoveryV1::RepairLegacyDefinition
        }
        _ => unreachable!("a new reaction recovery requires protocol mapping"),
    }
}

pub(super) fn catalog_category(
    value: CatalogPlacementCategoryV2,
) -> ProtocolCatalogPlacementCategoryV1 {
    match value {
        CatalogPlacementCategoryV2::UnknownKey => ProtocolCatalogPlacementCategoryV1::UnknownKey,
        CatalogPlacementCategoryV2::StaleSnapshot => {
            ProtocolCatalogPlacementCategoryV1::StaleSnapshot
        }
        CatalogPlacementCategoryV2::ForeignSession => {
            ProtocolCatalogPlacementCategoryV1::ForeignSession
        }
        CatalogPlacementCategoryV2::MismatchedPreview => {
            ProtocolCatalogPlacementCategoryV1::MismatchedPreview
        }
        CatalogPlacementCategoryV2::ReplayedGesture => {
            ProtocolCatalogPlacementCategoryV1::ReplayedGesture
        }
        CatalogPlacementCategoryV2::InvalidPoint => {
            ProtocolCatalogPlacementCategoryV1::InvalidPoint
        }
        CatalogPlacementCategoryV2::RenderPreparation => {
            ProtocolCatalogPlacementCategoryV1::RenderPreparation
        }
        CatalogPlacementCategoryV2::SessionConflict => {
            ProtocolCatalogPlacementCategoryV1::SessionConflict
        }
    }
}

pub(super) fn catalog_recovery(
    value: CatalogPlacementRecoveryV2,
) -> ProtocolCatalogPlacementRecoveryV1 {
    match value {
        CatalogPlacementRecoveryV2::ChooseCatalogEntry => {
            ProtocolCatalogPlacementRecoveryV1::ChooseCatalogEntry
        }
        CatalogPlacementRecoveryV2::RefreshAndRestart => {
            ProtocolCatalogPlacementRecoveryV1::RefreshAndRestart
        }
        CatalogPlacementRecoveryV2::DocumentUnchanged => {
            ProtocolCatalogPlacementRecoveryV1::DocumentUnchanged
        }
    }
}

pub(super) fn vector_category(
    value: PresentationVectorGestureCategoryV1,
) -> ProtocolPresentationVectorGestureCategoryV1 {
    match value {
        PresentationVectorGestureCategoryV1::StaleSnapshot => {
            ProtocolPresentationVectorGestureCategoryV1::StaleSnapshot
        }
        PresentationVectorGestureCategoryV1::ForeignSession => {
            ProtocolPresentationVectorGestureCategoryV1::ForeignSession
        }
        PresentationVectorGestureCategoryV1::MismatchedPreview => {
            ProtocolPresentationVectorGestureCategoryV1::MismatchedPreview
        }
        PresentationVectorGestureCategoryV1::ReplayedGesture => {
            ProtocolPresentationVectorGestureCategoryV1::ReplayedGesture
        }
        PresentationVectorGestureCategoryV1::InvalidPoint => {
            ProtocolPresentationVectorGestureCategoryV1::InvalidPoint
        }
        PresentationVectorGestureCategoryV1::DegenerateGeometry => {
            ProtocolPresentationVectorGestureCategoryV1::DegenerateGeometry
        }
        PresentationVectorGestureCategoryV1::UnsupportedKind => {
            ProtocolPresentationVectorGestureCategoryV1::UnsupportedKind
        }
        PresentationVectorGestureCategoryV1::UnrenderableStandard => {
            ProtocolPresentationVectorGestureCategoryV1::UnrenderableStandard
        }
        PresentationVectorGestureCategoryV1::RenderPreparation => {
            ProtocolPresentationVectorGestureCategoryV1::RenderPreparation
        }
        PresentationVectorGestureCategoryV1::SessionConflict => {
            ProtocolPresentationVectorGestureCategoryV1::SessionConflict
        }
        PresentationVectorGestureCategoryV1::ResourceExhausted => {
            ProtocolPresentationVectorGestureCategoryV1::ResourceExhausted
        }
        _ => unreachable!("new vector category requires protocol mapping"),
    }
}

pub(super) fn vector_recovery(
    value: PresentationVectorGestureRecoveryV1,
) -> ProtocolPresentationVectorGestureRecoveryV1 {
    match value {
        PresentationVectorGestureRecoveryV1::DocumentUnchanged => {
            ProtocolPresentationVectorGestureRecoveryV1::DocumentUnchanged
        }
        PresentationVectorGestureRecoveryV1::RefreshAndRestart => {
            ProtocolPresentationVectorGestureRecoveryV1::RefreshAndRestart
        }
        PresentationVectorGestureRecoveryV1::ChangeGeometry => {
            ProtocolPresentationVectorGestureRecoveryV1::ChangeGeometry
        }
        PresentationVectorGestureRecoveryV1::ChooseSupportedAppearance => {
            ProtocolPresentationVectorGestureRecoveryV1::ChooseSupportedAppearance
        }
        PresentationVectorGestureRecoveryV1::ReduceRequest => {
            ProtocolPresentationVectorGestureRecoveryV1::ReduceRequest
        }
        _ => unreachable!("new vector recovery requires protocol mapping"),
    }
}
