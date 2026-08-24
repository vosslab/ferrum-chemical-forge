//! Protocol execution failure translation.

use super::*;

#[derive(Debug)]
pub(crate) struct ExecutionFailureV1 {
    pub(super) category: OperationProtocolErrorCategoryV1,
    pub(super) message: String,
    pub(super) resource_limit: Option<ProtocolResourceLimitRefusalV1>,
    pub(super) presentation_author_refusal: Option<PresentationAuthorRefusalV1>,
    pub(super) catalog_placement_refusal: Option<CatalogPlacementRefusalV1>,
    pub(super) reaction_refusal: Option<ReactionRefusalV1>,
}

impl ExecutionFailureV1 {
    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::InvalidRequest,
            message: message.into(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

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
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }
    pub(super) fn document_admission(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentAdmissionFailed,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(in crate::protocol) fn document_invalid(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentInvalid,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn render_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderUnsupported,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn render_failed(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::RenderFailed,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(in crate::protocol) fn chemistry_unavailable(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ChemistryUnavailable,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
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
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn conversion_unsupported(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ConversionUnsupported,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn coordinate(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::CoordinateGenerationFailed,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(in crate::protocol) fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            message: message.into(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(crate) fn oxidation_resource_limit(
        resource: ferrum_document::DocumentAtomOxidationResourceV1,
    ) -> Self {
        let reason = match resource {
            ferrum_document::DocumentAtomOxidationResourceV1::Atoms => {
                ProtocolResourceLimitReasonV1::OxidationRootAtomsExceeded
            }
            ferrum_document::DocumentAtomOxidationResourceV1::Bonds => {
                ProtocolResourceLimitReasonV1::OxidationRootBondsExceeded
            }
            ferrum_document::DocumentAtomOxidationResourceV1::Components => {
                ProtocolResourceLimitReasonV1::OxidationRootComponentsExceeded
            }
        };
        Self {
            category: OperationProtocolErrorCategoryV1::ResourceLimit,
            message: "selected oxidation root exceeds the supported resource bound".to_owned(),
            resource_limit: Some(ProtocolResourceLimitRefusalV1 {
                reason,
                recovery: ProtocolResourceLimitRecoveryV1::UseSmallerRoot,
            }),
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(crate) fn oxidation_refusal(
        refusal: ferrum_document::DocumentAtomOxidationRefusalV1,
    ) -> Self {
        let category = match refusal {
            ferrum_document::DocumentAtomOxidationRefusalV1::StaleObservation
            | ferrum_document::DocumentAtomOxidationRefusalV1::DigestMismatch => {
                OperationProtocolErrorCategoryV1::StaleDocument
            }
            ferrum_document::DocumentAtomOxidationRefusalV1::UnknownAtom => {
                OperationProtocolErrorCategoryV1::AtomNotFound
            }
            ferrum_document::DocumentAtomOxidationRefusalV1::UnknownDirectMolecule => {
                OperationProtocolErrorCategoryV1::MoleculeNotDirectRoot
            }
            ferrum_document::DocumentAtomOxidationRefusalV1::AtomNotInSelectedRoot => {
                OperationProtocolErrorCategoryV1::AtomNotInSelectedMolecule
            }
            ferrum_document::DocumentAtomOxidationRefusalV1::UnsupportedDocument => {
                OperationProtocolErrorCategoryV1::UnsupportedDocument
            }
            ferrum_document::DocumentAtomOxidationRefusalV1::DirectRootMismatch
            | ferrum_document::DocumentAtomOxidationRefusalV1::InvalidAuthenticatedGraph => {
                OperationProtocolErrorCategoryV1::InternalFailure
            }
        };
        let message = match category {
            OperationProtocolErrorCategoryV1::InternalFailure => {
                "oxidation observation could not complete".to_owned()
            }
            _ => refusal.to_string(),
        };
        Self {
            category,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(crate) fn hydrogen_materialization_refusal(
        refusal: ferrum_document::DocumentMoleculeHydrogenMaterializationRefusalV1,
    ) -> Self {
        let category = match refusal {
            ferrum_document::DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation
            | ferrum_document::DocumentMoleculeHydrogenMaterializationRefusalV1::DigestMismatch => {
                OperationProtocolErrorCategoryV1::StaleDocument
            }
            ferrum_document::DocumentMoleculeHydrogenMaterializationRefusalV1::UnknownDirectMolecule => {
                OperationProtocolErrorCategoryV1::MoleculeNotDirectRoot
            }
            ferrum_document::DocumentMoleculeHydrogenMaterializationRefusalV1::UnknownAnchorAtom => {
                OperationProtocolErrorCategoryV1::AtomNotFound
            }
            ferrum_document::DocumentMoleculeHydrogenMaterializationRefusalV1::AnchorNotInSelectedRoot => {
                OperationProtocolErrorCategoryV1::AtomNotInSelectedMolecule
            }
            _ => OperationProtocolErrorCategoryV1::InternalFailure,
        };
        let message = if matches!(category, OperationProtocolErrorCategoryV1::InternalFailure) {
            "hydrogen materialization could not complete".to_owned()
        } else {
            refusal.to_string()
        };
        Self {
            category,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(crate) fn internal(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::InternalFailure,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
        }
    }

    pub(super) fn presentation_author_refusal(
        authoring_kind: PresentationAuthoringKindV1,
        category: ProtocolPresentationAuthorCategoryV1,
        recovery: ProtocolPresentationAuthorRecoveryV1,
        message: String,
    ) -> Self {
        let error_category = match category {
            ProtocolPresentationAuthorCategoryV1::RenderPreparation => {
                OperationProtocolErrorCategoryV1::RenderFailed
            }
            ProtocolPresentationAuthorCategoryV1::Capacity => {
                OperationProtocolErrorCategoryV1::ResourceLimit
            }
            _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
        };
        Self {
            category: error_category,
            message,
            resource_limit: None,
            presentation_author_refusal: Some(PresentationAuthorRefusalV1 {
                authoring_kind,
                category,
                recovery,
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
            resource_limit: None,
            presentation_author_refusal: None,
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
            resource_limit: None,
            presentation_author_refusal: None,
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
