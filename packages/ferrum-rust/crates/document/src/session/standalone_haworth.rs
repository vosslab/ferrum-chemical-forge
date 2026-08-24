use super::*;
use crate::{
    CreateHaworthMoleculeV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1,
};
use ferrum_domain::haworth::StandaloneDGlucoseHaworthRecipeV1;

impl DocumentSession {
    /// Resolve one standalone D-glucose recipe into generic transition authority.
    pub fn resolve_standalone_haworth_transition_v1(
        &self,
        expected_revision: u64,
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    ) -> Result<SessionOperationTransitionRequestV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        Ok(SessionOperationTransitionRequestV1::new(
            expected_revision,
            SessionOperation::V1(SessionOperationV1::CreateHaworthMoleculeV1(
                CreateHaworthMoleculeV1::standalone_d_glucose(recipe, anchor),
            )),
            TransitionAuthorizationV1::authoring_capability(self.issue_authoring_capability_v1()),
        ))
    }
}
