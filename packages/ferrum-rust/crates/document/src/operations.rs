//! Product-level document operations that do not require a delivery adapter.

use ferrum_geometry::Point2;
use thiserror::Error;

use crate::{
    DocumentClipboardCutErrorV1, DocumentClipboardCutPlanV1, DocumentClipboardPasteErrorV1,
    DocumentClipboardPastePlanV1, DocumentClipboardPasteResultV1, DocumentClipboardSelectionV1,
    DocumentMoleculeInspectionErrorV1, DocumentObjectIdV1, DocumentSession, DocumentSessionError,
    DocumentUserTemplateErrorV1, DocumentUserTemplatePlanV1, DocumentUserTemplateResultV1,
    PersistentId, PreparedLinearFormConvertResultV1, SessionDocumentObservationV1,
    SessionOperation, SessionOperationResultV1, SessionOperationV1, TopLevelRootSelectorV1,
    TopLevelTranslationAnchorV1, XmlInputBudgetV1, direct_projection_molecule_v1,
    prepare_document_clipboard_cut_v1, prepare_document_clipboard_paste_v1,
    prepare_document_user_template_v1, verify_molecule_observation_v1,
};

pub const DOCUMENT_CLIPBOARD_PASTE_PROFILE_V1: &str = "ferrum-document-clipboard-paste-profile-v1";
pub const DOCUMENT_CLIPBOARD_PASTE_TRANSLATION_V1: (f64, f64) = (20.0, 20.0);
pub const DOCUMENT_USER_TEMPLATE_PROFILE_V1: &str = "ferrum-document-user-template-profile-v1";

#[must_use]
pub const fn document_operation_budget_v1() -> XmlInputBudgetV1 {
    XmlInputBudgetV1 {
        max_utf8_bytes: 16 * 1024 * 1024,
        max_elements: 262_144,
        max_depth: 64,
        max_attributes: 1_048_576,
        max_text_bytes: 8 * 1024 * 1024,
    }
}

#[must_use]
pub const fn document_clipboard_paste_budget_v1() -> XmlInputBudgetV1 {
    document_operation_budget_v1()
}

pub fn prepare_clipboard_cut_v1(
    observation: &SessionDocumentObservationV1,
    selection: DocumentClipboardSelectionV1,
) -> Result<DocumentClipboardCutPlanV1, DocumentClipboardCutErrorV1> {
    prepare_document_clipboard_cut_v1(observation, selection)
}

#[derive(Debug, Error)]
pub enum DocumentClipboardCutApplyErrorV1 {
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

pub fn apply_clipboard_cut_v1(
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    plan: &DocumentClipboardCutPlanV1,
) -> Result<SessionOperationResultV1, DocumentClipboardCutApplyErrorV1> {
    session
        .cut_document_clipboard_v1(expected_revision, expected_digest, plan)
        .map_err(Into::into)
}

pub fn prepare_clipboard_paste_v1(
    source: &str,
) -> Result<DocumentClipboardPastePlanV1, DocumentClipboardPasteErrorV1> {
    prepare_document_clipboard_paste_v1(source, document_clipboard_paste_budget_v1())
}

#[derive(Debug, Error)]
pub enum DocumentClipboardPasteApplyErrorV1 {
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

pub fn apply_clipboard_paste_v1(
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    plan: &DocumentClipboardPastePlanV1,
) -> Result<DocumentClipboardPasteResultV1, DocumentClipboardPasteApplyErrorV1> {
    let (dx, dy) = DOCUMENT_CLIPBOARD_PASTE_TRANSLATION_V1;
    session
        .paste_document_clipboard_v1(expected_revision, expected_digest, plan, dx, dy)
        .map_err(Into::into)
}

#[must_use]
pub const fn document_user_template_budget_v1() -> XmlInputBudgetV1 {
    document_operation_budget_v1()
}

pub fn prepare_user_template_v1(
    source: &str,
) -> Result<DocumentUserTemplatePlanV1, DocumentUserTemplateErrorV1> {
    prepare_document_user_template_v1(source, document_user_template_budget_v1())
}

#[derive(Debug, Error)]
pub enum DocumentUserTemplateApplyErrorV1 {
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

pub fn apply_user_template_v1(
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    plan: &DocumentUserTemplatePlanV1,
    anchor: Point2,
) -> Result<DocumentUserTemplateResultV1, DocumentUserTemplateApplyErrorV1> {
    session
        .insert_document_user_template_v1(expected_revision, expected_digest, plan, anchor)
        .map_err(Into::into)
}

pub fn observe_top_level_translation_anchor_v1(
    session: &DocumentSession,
    expected_revision: u64,
    targets: Vec<TopLevelRootSelectorV1>,
) -> Result<TopLevelTranslationAnchorV1, DocumentSessionError> {
    session.observe_top_level_translation_anchor_v1(expected_revision, targets)
}

/// Immutable exact intent for one direct-root authored-name replacement or clear.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeNameRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    name: String,
}

impl DocumentMoleculeNameRequestV1 {
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        name: String,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            name,
        }
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Error)]
pub enum DocumentMoleculeNameErrorV1 {
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Replace or clear one exact direct-root molecule name in the authoritative session.
pub fn set_document_molecule_name_v1(
    session: &mut DocumentSession,
    request: DocumentMoleculeNameRequestV1,
) -> Result<SessionOperationResultV1, DocumentMoleculeNameErrorV1> {
    let observation = session.observe(request.expected_revision)?;
    verify_molecule_observation_v1(
        &observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;
    let name = (!request.name.is_empty()).then_some(request.name);
    session
        .submit(
            request.expected_revision,
            SessionOperation::V1(SessionOperationV1::SetMoleculeName {
                molecule_id: request.molecule_id,
                name,
            }),
        )
        .map_err(Into::into)
}

/// Immutable exact intent for one direct-root linear-form conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentLinearFormRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    selected_atom_ids: Vec<PersistentId>,
}

impl DocumentLinearFormRequestV1 {
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        selected_atom_ids: Vec<PersistentId>,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            selected_atom_ids,
        }
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub fn selected_atom_ids(&self) -> &[PersistentId] {
        &self.selected_atom_ids
    }
}

#[derive(Debug)]
pub enum DocumentLinearFormResultV1 {
    Changed(SessionOperationResultV1),
    NoChange(SessionOperationResultV1),
}
impl DocumentLinearFormResultV1 {
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        match self {
            Self::Changed(result) | Self::NoChange(result) => result,
        }
    }
    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        match self {
            Self::Changed(result) | Self::NoChange(result) => result,
        }
    }
}

#[derive(Debug, Error)]
pub enum DocumentLinearFormErrorV1 {
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Convert one exact selected-atom path immediately in the authoritative session.
pub fn convert_document_linear_form_v1(
    session: &mut DocumentSession,
    request: DocumentLinearFormRequestV1,
) -> Result<DocumentLinearFormResultV1, DocumentLinearFormErrorV1> {
    let observation = session.observe(request.expected_revision)?;
    verify_molecule_observation_v1(
        &observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;
    match session.prepare_convert_linear_form_v1(
        request.expected_revision,
        &request.molecule_id,
        &request.selected_atom_ids,
    )? {
        PreparedLinearFormConvertResultV1::NoChange(result) => {
            Ok(DocumentLinearFormResultV1::NoChange(*result))
        }
        PreparedLinearFormConvertResultV1::Pending(mut pending) => session
            .commit_convert_linear_form_v1(request.expected_revision, &mut pending)
            .map(DocumentLinearFormResultV1::Changed)
            .map_err(Into::into),
    }
}
