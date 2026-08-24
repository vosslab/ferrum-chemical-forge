use super::model::validate_reaction_members;
use super::*;

/// Closed refusal vocabulary for semantic direct-reaction operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ReactionOperationRefusalV1 {
    #[error("reaction requires nonempty reactant and product members")]
    MissingRequiredMembers,
    #[error("reaction member identifiers must be nonempty")]
    EmptyMemberIdentifier,
    #[error("reaction members must be unique")]
    DuplicateMember,
    #[error("reaction member does not exist")]
    MissingMember,
    #[error("reaction member has the wrong direct-root kind")]
    WrongMemberKind,
    #[error("reaction member already belongs to another reaction")]
    CrossReactionReuse,
    #[error("reaction definition is missing or not strict")]
    InvalidDefinition,
}

/// Semantic request to create one direct reaction with document-owned ID allocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateReactionV1 {
    members: Vec<(DirectReactionRoleV1, String)>,
}

impl CreateReactionV1 {
    pub fn new(
        members: Vec<(DirectReactionRoleV1, String)>,
    ) -> Result<Self, ReactionOperationRefusalV1> {
        validate_reaction_members(&members)?;
        Ok(Self { members })
    }

    #[must_use]
    pub fn members(&self) -> &[(DirectReactionRoleV1, String)] {
        &self.members
    }
}

/// Semantic request to replace all typed members of one strict direct reaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplaceReactionMembersV1 {
    reaction_id: String,
    members: Vec<(DirectReactionRoleV1, String)>,
}

impl ReplaceReactionMembersV1 {
    pub fn new(
        reaction_id: String,
        members: Vec<(DirectReactionRoleV1, String)>,
    ) -> Result<Self, ReactionOperationRefusalV1> {
        if reaction_id.trim().is_empty() {
            return Err(ReactionOperationRefusalV1::InvalidDefinition);
        }
        validate_reaction_members(&members)?;
        Ok(Self {
            reaction_id,
            members,
        })
    }

    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }

    #[must_use]
    pub fn members(&self) -> &[(DirectReactionRoleV1, String)] {
        &self.members
    }
}

/// Semantic request to delete one strict direct reaction definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeleteReactionV1 {
    reaction_id: String,
}

impl DeleteReactionV1 {
    pub fn new(reaction_id: String) -> Result<Self, ReactionOperationRefusalV1> {
        if reaction_id.trim().is_empty() {
            return Err(ReactionOperationRefusalV1::InvalidDefinition);
        }
        Ok(Self { reaction_id })
    }

    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
}
