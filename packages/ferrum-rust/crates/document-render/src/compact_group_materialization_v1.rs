//! Renderer-admitted compact-group materialization transaction.

use ferrum_document::{
    CompactGroupMaterializationRefusalV1, CompactGroupMaterializationRequestV1,
    CompactGroupMaterializationResultV1, DocumentSession, PendingCompactGroupMaterializationV1,
    SessionOperationResultV1,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompactGroupMaterializationErrorV1 {
    #[error("compact-group materialization was refused: {0}")]
    Refusal(#[from] CompactGroupMaterializationRefusalV1),
    #[error("compact-group replacement could not complete the normal document render plan")]
    RenderPreparation,
    #[error("compact-group materialization receipt was already consumed")]
    Replayed,
}

#[derive(Debug)]
pub struct PreparedCompactGroupMaterializationV1 {
    pending: PendingCompactGroupMaterializationV1,
}

impl PreparedCompactGroupMaterializationV1 {
    /// Return the prepared replacement outcome before renderer-gated commit.
    #[must_use]
    pub fn materialization(&self) -> &CompactGroupMaterializationResultV1 {
        self.pending.result()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommittedCompactGroupMaterializationV1 {
    materialization: CompactGroupMaterializationResultV1,
    operation_result: SessionOperationResultV1,
}

impl CommittedCompactGroupMaterializationV1 {
    #[must_use]
    pub const fn materialization(&self) -> &CompactGroupMaterializationResultV1 {
        &self.materialization
    }
    #[must_use]
    pub const fn operation_result(&self) -> &SessionOperationResultV1 {
        &self.operation_result
    }
}

pub fn prepare_compact_group_materialization_v1(
    session: &mut DocumentSession,
    request: &CompactGroupMaterializationRequestV1,
) -> Result<PreparedCompactGroupMaterializationV1, CompactGroupMaterializationErrorV1> {
    let pending = session
        .prepare_compact_group_materialization_v1(request)
        .map_err(|error| match error {
            CompactGroupMaterializationRefusalV1::RendererAdmission => {
                CompactGroupMaterializationErrorV1::RenderPreparation
            }
            other => CompactGroupMaterializationErrorV1::Refusal(other),
        })?;
    Ok(PreparedCompactGroupMaterializationV1 { pending })
}

pub fn commit_compact_group_materialization_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCompactGroupMaterializationV1,
) -> Result<CommittedCompactGroupMaterializationV1, CompactGroupMaterializationErrorV1> {
    if prepared.pending.is_consumed_v1() {
        return Err(CompactGroupMaterializationErrorV1::Replayed);
    }
    match session.commit_compact_group_materialization_v1(&mut prepared.pending) {
        Ok((materialization, operation_result)) => Ok(CommittedCompactGroupMaterializationV1 {
            materialization,
            operation_result,
        }),
        Err(CompactGroupMaterializationRefusalV1::RendererAdmission) => {
            Err(CompactGroupMaterializationErrorV1::RenderPreparation)
        }
        Err(error) => Err(error.into()),
    }
}
