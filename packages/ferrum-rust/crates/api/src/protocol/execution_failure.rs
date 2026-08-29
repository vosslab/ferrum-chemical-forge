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
    pub(super) compact_group_materialization_refusal: Option<CompactGroupMaterializationRefusalV1>,
    pub(super) compact_group_attachment_refusal: Option<CompactGroupAttachmentRefusalV1>,
    pub(super) document_molecule_export_refusal: Option<DocumentMoleculeExportRefusalV1>,
}

impl ExecutionFailureV1 {
    /// Build the only public selected-root export refusal envelope fact.
    ///
    /// The ordinary error category is derived once here so neither protocol
    /// callers nor CLI presentation need to parse diagnostics.
    pub(crate) fn document_molecule_export_refusal(
        refusal: DocumentMoleculeExportRefusalV1,
    ) -> Self {
        let category = match refusal.category {
            ProtocolDocumentMoleculeExportCategoryV1::SnapshotNotAdmitted => {
                OperationProtocolErrorCategoryV1::DocumentAdmissionFailed
            }
            ProtocolDocumentMoleculeExportCategoryV1::UnknownOrNonDirectRoot => {
                OperationProtocolErrorCategoryV1::DocumentInvalid
            }
            ProtocolDocumentMoleculeExportCategoryV1::RepresentationUnsupported => {
                OperationProtocolErrorCategoryV1::RenderUnsupported
            }
            ProtocolDocumentMoleculeExportCategoryV1::ChemistryUnavailable => {
                OperationProtocolErrorCategoryV1::ChemistryUnavailable
            }
            ProtocolDocumentMoleculeExportCategoryV1::OutputLimitExceeded => {
                OperationProtocolErrorCategoryV1::ResourceLimit
            }
        };
        Self {
            category,
            message: "selected molecule export refused".to_owned(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: Some(refusal),
        }
    }
    pub(crate) fn compact_group_materialization_refusal(
        refusal: CompactGroupMaterializationRefusalV1,
    ) -> Self {
        let category = match refusal.category {
            ProtocolCompactGroupMaterializationCategoryV1::StaleDocumentFence => {
                OperationProtocolErrorCategoryV1::StaleDocument
            }
            ProtocolCompactGroupMaterializationCategoryV1::UnknownOrForeignTarget
            | ProtocolCompactGroupMaterializationCategoryV1::IneligibleTarget
            | ProtocolCompactGroupMaterializationCategoryV1::SessionConflictOrConsumedPreparation => {
                OperationProtocolErrorCategoryV1::DocumentInvalid
            }
            ProtocolCompactGroupMaterializationCategoryV1::RendererPreparationRefusal => {
                OperationProtocolErrorCategoryV1::RenderUnsupported
            }
        };
        Self {
            category,
            message: "compact-group materialization refused".to_owned(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
            compact_group_materialization_refusal: Some(refusal),
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
        }
    }
    pub(crate) fn compact_group_attachment_refusal(
        refusal: CompactGroupAttachmentRefusalV1,
    ) -> Self {
        let category = match refusal.category {
            ProtocolCompactGroupAttachmentCategoryV1::StaleDocumentFence => {
                OperationProtocolErrorCategoryV1::StaleDocument
            }
            ProtocolCompactGroupAttachmentCategoryV1::UnknownTarget => {
                OperationProtocolErrorCategoryV1::AtomNotFound
            }
            ProtocolCompactGroupAttachmentCategoryV1::ForeignTarget => {
                OperationProtocolErrorCategoryV1::AtomNotInSelectedMolecule
            }
            ProtocolCompactGroupAttachmentCategoryV1::InvalidRelease
            | ProtocolCompactGroupAttachmentCategoryV1::CandidateAdmission => {
                OperationProtocolErrorCategoryV1::DocumentInvalid
            }
            ProtocolCompactGroupAttachmentCategoryV1::RendererAdmission => {
                OperationProtocolErrorCategoryV1::RenderUnsupported
            }
            ProtocolCompactGroupAttachmentCategoryV1::SessionConflict => {
                OperationProtocolErrorCategoryV1::InternalFailure
            }
        };
        Self {
            category,
            message: "compact-group attachment refused".to_owned(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: Some(refusal),
            document_molecule_export_refusal: None,
        }
    }
    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::InvalidRequest,
            message: message.into(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
        }
    }
    pub(crate) fn document_admission(message: String) -> Self {
        Self {
            category: OperationProtocolErrorCategoryV1::DocumentAdmissionFailed,
            message,
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: None,
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
        }
    }

    pub(super) fn catalog_refusal(error: CatalogPlacementErrorV1) -> Self {
        Self {
            category: match error.category() {
                CatalogPlacementCategoryV1::RenderPreparation => {
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
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
        }
    }

    /// Translate one durable reaction-command refusal without exposing source details.
    pub(super) fn reaction_authoring_command_refusal(
        error: ferrum_document::ReactionAuthoringCommandRefusalV1,
    ) -> Self {
        match error {
            ferrum_document::ReactionAuthoringCommandRefusalV1::InvalidMembers(error) => {
                Self::reaction_operation_refusal(error)
            }
            ferrum_document::ReactionAuthoringCommandRefusalV1::InvalidSelection(error) => {
                Self::reaction_selection_refusal(error)
            }
            ferrum_document::ReactionAuthoringCommandRefusalV1::ForeignSession => {
                Self::reaction_failure(
                    ProtocolReactionRefusalCategoryV1::ForeignSession,
                    ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
                )
            }
            ferrum_document::ReactionAuthoringCommandRefusalV1::Consumed => Self::reaction_failure(
                ProtocolReactionRefusalCategoryV1::Consumed,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::ReactionAuthoringCommandRefusalV1::StaleRevision
            | ferrum_document::ReactionAuthoringCommandRefusalV1::StaleDigest => {
                Self::reaction_failure(
                    ProtocolReactionRefusalCategoryV1::StaleSnapshot,
                    ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
                )
            }
        }
    }

    /// Translate one durable reaction-selection refusal without exposing source details.
    pub(super) fn reaction_selection_refusal(
        error: ferrum_document::ReactionMemberSelectionRefusalV1,
    ) -> Self {
        let (category, recovery) = match error {
            ferrum_document::ReactionMemberSelectionRefusalV1::InvalidIdentity(_) => (
                ProtocolReactionRefusalCategoryV1::InvalidRequest,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::ReactionMemberSelectionRefusalV1::UnknownReaction
            | ferrum_document::ReactionMemberSelectionRefusalV1::UnresolvedReaction => (
                ProtocolReactionRefusalCategoryV1::MissingReaction,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::ReactionMemberSelectionRefusalV1::WrongReactionKind => (
                ProtocolReactionRefusalCategoryV1::WrongTargetKind,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionMemberSelectionRefusalV1::UnresolvedMember => (
                ProtocolReactionRefusalCategoryV1::MissingTarget,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionMemberSelectionRefusalV1::ForeignSession => (
                ProtocolReactionRefusalCategoryV1::ForeignSession,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::ReactionMemberSelectionRefusalV1::StaleRevision
            | ferrum_document::ReactionMemberSelectionRefusalV1::StaleDigest => (
                ProtocolReactionRefusalCategoryV1::StaleSnapshot,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::ReactionMemberSelectionRefusalV1::MembershipMismatch => (
                ProtocolReactionRefusalCategoryV1::MembershipChanged,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
        };
        Self::reaction_failure(category, recovery)
    }

    /// Translate a reaction target-contract refusal without exposing member IDs.
    pub(super) fn reaction_operation_refusal(
        error: ferrum_document::ReactionOperationRefusalV1,
    ) -> Self {
        let (category, recovery) = match error {
            ferrum_document::ReactionOperationRefusalV1::MissingRequiredMembers
            | ferrum_document::ReactionOperationRefusalV1::EmptyMemberIdentifier => (
                ProtocolReactionRefusalCategoryV1::InvalidRequest,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionOperationRefusalV1::DuplicateMember => (
                ProtocolReactionRefusalCategoryV1::DuplicateTarget,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionOperationRefusalV1::MissingMember => (
                ProtocolReactionRefusalCategoryV1::MissingTarget,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionOperationRefusalV1::WrongMemberKind => (
                ProtocolReactionRefusalCategoryV1::WrongTargetKind,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionOperationRefusalV1::CrossReactionReuse => (
                ProtocolReactionRefusalCategoryV1::CrossReactionReuse,
                ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            ),
            ferrum_document::ReactionOperationRefusalV1::InvalidDefinition => (
                ProtocolReactionRefusalCategoryV1::MissingReaction,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
        };
        Self::reaction_failure(category, recovery)
    }

    /// Translate generic renderer-admitted transition refusals for a reaction operation.
    pub(super) fn reaction_transition_refusal(
        error: ferrum_document::AdmittedSessionTransitionRefusalV1,
    ) -> Self {
        let (category, recovery) = match error {
            ferrum_document::AdmittedSessionTransitionRefusalV1::ForeignSession => (
                ProtocolReactionRefusalCategoryV1::ForeignSession,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::AdmittedSessionTransitionRefusalV1::Consumed => (
                ProtocolReactionRefusalCategoryV1::Consumed,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::AdmittedSessionTransitionRefusalV1::StaleSnapshot => (
                ProtocolReactionRefusalCategoryV1::StaleSnapshot,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
            ferrum_document::AdmittedSessionTransitionRefusalV1::RendererAdmission => (
                ProtocolReactionRefusalCategoryV1::RenderPreparation,
                ProtocolReactionRefusalRecoveryV1::ChooseRenderableMembers,
            ),
            ferrum_document::AdmittedSessionTransitionRefusalV1::ProvisionalCapability => (
                ProtocolReactionRefusalCategoryV1::SessionConflict,
                ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            ),
        };
        Self::reaction_failure(category, recovery)
    }

    /// Return the closed protocol representation for malformed public reaction input.
    pub(super) fn reaction_invalid_request() -> Self {
        Self::reaction_failure(
            ProtocolReactionRefusalCategoryV1::InvalidRequest,
            ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
        )
    }

    fn reaction_failure(
        category: ProtocolReactionRefusalCategoryV1,
        recovery: ProtocolReactionRefusalRecoveryV1,
    ) -> Self {
        let error_category = match category {
            ProtocolReactionRefusalCategoryV1::UnrenderableDocument
            | ProtocolReactionRefusalCategoryV1::RenderPreparation
            | ProtocolReactionRefusalCategoryV1::RendererExclusion => {
                OperationProtocolErrorCategoryV1::RenderFailed
            }
            _ => OperationProtocolErrorCategoryV1::DocumentInvalid,
        };
        Self {
            category: error_category,
            message: "reaction command refused".to_owned(),
            resource_limit: None,
            presentation_author_refusal: None,
            catalog_placement_refusal: None,
            reaction_refusal: Some(ReactionRefusalV1 { category, recovery }),
            compact_group_materialization_refusal: None,
            compact_group_attachment_refusal: None,
            document_molecule_export_refusal: None,
        }
    }
}

pub(super) fn catalog_category(
    value: CatalogPlacementCategoryV1,
) -> ProtocolCatalogPlacementCategoryV1 {
    match value {
        CatalogPlacementCategoryV1::UnknownKey => ProtocolCatalogPlacementCategoryV1::UnknownKey,
        CatalogPlacementCategoryV1::StaleSnapshot => {
            ProtocolCatalogPlacementCategoryV1::StaleSnapshot
        }
        CatalogPlacementCategoryV1::ForeignSession => {
            ProtocolCatalogPlacementCategoryV1::ForeignSession
        }
        CatalogPlacementCategoryV1::MismatchedPreview => {
            ProtocolCatalogPlacementCategoryV1::MismatchedPreview
        }
        CatalogPlacementCategoryV1::Consumed => ProtocolCatalogPlacementCategoryV1::Consumed,
        CatalogPlacementCategoryV1::InvalidPoint => {
            ProtocolCatalogPlacementCategoryV1::InvalidPoint
        }
        CatalogPlacementCategoryV1::RenderPreparation => {
            ProtocolCatalogPlacementCategoryV1::RenderPreparation
        }
        CatalogPlacementCategoryV1::SessionConflict => {
            ProtocolCatalogPlacementCategoryV1::SessionConflict
        }
    }
}

pub(super) fn catalog_recovery(
    value: CatalogPlacementRecoveryV1,
) -> ProtocolCatalogPlacementRecoveryV1 {
    match value {
        CatalogPlacementRecoveryV1::ChooseCatalogEntry => {
            ProtocolCatalogPlacementRecoveryV1::ChooseCatalogEntry
        }
        CatalogPlacementRecoveryV1::RefreshAndRestart => {
            ProtocolCatalogPlacementRecoveryV1::RefreshAndRestart
        }
        CatalogPlacementRecoveryV1::DocumentUnchanged => {
            ProtocolCatalogPlacementRecoveryV1::DocumentUnchanged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_molecule_export_refusal_derives_the_one_public_category_mapping() {
        let cases = [
            (
                ProtocolDocumentMoleculeExportCategoryV1::SnapshotNotAdmitted,
                OperationProtocolErrorCategoryV1::DocumentAdmissionFailed,
            ),
            (
                ProtocolDocumentMoleculeExportCategoryV1::UnknownOrNonDirectRoot,
                OperationProtocolErrorCategoryV1::DocumentInvalid,
            ),
            (
                ProtocolDocumentMoleculeExportCategoryV1::RepresentationUnsupported,
                OperationProtocolErrorCategoryV1::RenderUnsupported,
            ),
            (
                ProtocolDocumentMoleculeExportCategoryV1::ChemistryUnavailable,
                OperationProtocolErrorCategoryV1::ChemistryUnavailable,
            ),
            (
                ProtocolDocumentMoleculeExportCategoryV1::OutputLimitExceeded,
                OperationProtocolErrorCategoryV1::ResourceLimit,
            ),
        ];

        for (export_category, ordinary_category) in cases {
            let refusal = DocumentMoleculeExportRefusalV1 {
                category: export_category,
                recovery: ProtocolDocumentMoleculeExportRecoveryV1::SelectSmallerRoot,
            };
            let failure = ExecutionFailureV1::document_molecule_export_refusal(refusal);
            assert_eq!(failure.category, ordinary_category);
            assert_eq!(failure.document_molecule_export_refusal, Some(refusal));
            assert_eq!(failure.message, "selected molecule export refused");
        }
    }
    #[test]
    fn durable_reaction_command_refusals_keep_the_closed_protocol_category() {
        let failure = ExecutionFailureV1::reaction_authoring_command_refusal(
            ferrum_document::ReactionAuthoringCommandRefusalV1::InvalidMembers(
                ferrum_document::ReactionOperationRefusalV1::DuplicateMember,
            ),
        );
        assert_eq!(
            failure.reaction_refusal,
            Some(ReactionRefusalV1 {
                category: ProtocolReactionRefusalCategoryV1::DuplicateTarget,
                recovery: ProtocolReactionRefusalRecoveryV1::CorrectSelectors,
            })
        );
        assert_eq!(failure.message, "reaction command refused");
    }

    #[test]
    fn invalid_durable_reaction_identity_keeps_the_public_refresh_contract() {
        let failure = ExecutionFailureV1::reaction_selection_refusal(
            ferrum_document::ReactionMemberSelectionRefusalV1::InvalidIdentity(
                ferrum_document::ProjectionError::InvalidValue {
                    context: "durable reaction binding".to_owned(),
                    field: "object:id",
                    value: "malformed retained identity".to_owned(),
                },
            ),
        );

        assert_eq!(
            failure.category,
            OperationProtocolErrorCategoryV1::DocumentInvalid
        );
        assert_eq!(
            failure.reaction_refusal,
            Some(ReactionRefusalV1 {
                category: ProtocolReactionRefusalCategoryV1::InvalidRequest,
                recovery: ProtocolReactionRefusalRecoveryV1::RefreshAndRestart,
            })
        );
    }

    #[test]
    fn renderer_admission_uses_render_preparation_recovery() {
        let failure = ExecutionFailureV1::reaction_transition_refusal(
            ferrum_document::AdmittedSessionTransitionRefusalV1::RendererAdmission,
        );
        assert_eq!(
            failure.reaction_refusal,
            Some(ReactionRefusalV1 {
                category: ProtocolReactionRefusalCategoryV1::RenderPreparation,
                recovery: ProtocolReactionRefusalRecoveryV1::ChooseRenderableMembers,
            })
        );
        assert_eq!(
            failure.category,
            OperationProtocolErrorCategoryV1::RenderFailed
        );
    }
}
