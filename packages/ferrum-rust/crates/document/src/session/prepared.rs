//! Shared authentication mapping for one-use prepared session receipts.

use super::{
    DocumentSessionError, ProvisionalToken, SessionOperationError, TypedDocument,
    TypedDocumentError,
};

pub(super) fn issue_prepared_token(
    document: &mut TypedDocument,
) -> Result<ProvisionalToken, DocumentSessionError> {
    document
        .try_issue_provisional_token()
        .map_err(SessionOperationError::Candidate)
        .map_err(DocumentSessionError::Operation)
}

pub(super) fn map_prepared_token_error(error: TypedDocumentError) -> DocumentSessionError {
    match error {
        TypedDocumentError::Indexed(super::super::IndexedDocumentError::Identity(
            super::super::DocumentIdentityError::UnknownProvisionalToken { .. },
        )) => DocumentSessionError::PreparedOperationForeignSession,
        other => DocumentSessionError::Operation(SessionOperationError::Candidate(other)),
    }
}
