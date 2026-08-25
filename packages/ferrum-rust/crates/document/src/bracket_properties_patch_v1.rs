//! Closed common appearance intent for one durable bracket pair.

use std::collections::HashSet;

use thiserror::Error;

use super::{DocumentObjectIdV1, GeometricLineWidthV1, Rgb24V1};

/// One supported common bracket-pair appearance change.
#[derive(Clone, Debug, PartialEq)]
pub enum BracketPropertyChangeV1 {
    /// Replace both sides' visible line width.
    LineWidth(GeometricLineWidthV1),
    /// Replace both sides' visible line colour.
    LineColor(Rgb24V1),
}

impl BracketPropertyChangeV1 {
    fn kind(&self) -> BracketPropertyKindV1 {
        match self {
            Self::LineWidth(_) => BracketPropertyKindV1::LineWidth,
            Self::LineColor(_) => BracketPropertyKindV1::LineColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BracketPropertyKindV1 {
    LineWidth,
    LineColor,
}

impl BracketPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::LineWidth => "line width",
            Self::LineColor => "line color",
        }
    }
}

/// One validated left/right-member bracket appearance patch.
#[derive(Clone, Debug, PartialEq)]
pub struct BracketPropertiesPatchV1 {
    members: [DocumentObjectIdV1; 2],
    changes: Vec<BracketPropertyChangeV1>,
}

impl BracketPropertiesPatchV1 {
    /// Validate one complete two-field edit intent without reading a document.
    pub fn new(
        members: [DocumentObjectIdV1; 2],
        changes: Vec<BracketPropertyChangeV1>,
    ) -> Result<Self, BracketPropertiesPatchV1Error> {
        if members[0] == members[1] {
            return Err(BracketPropertiesPatchV1Error::DuplicateMembers);
        }
        if changes.len() > 2 {
            return Err(BracketPropertiesPatchV1Error::TooManyChanges);
        }
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(BracketPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
        }
        Ok(Self { members, changes })
    }

    /// Return the two distinct durable bracket members in caller-preserved order.
    #[must_use]
    pub fn members(&self) -> &[DocumentObjectIdV1; 2] {
        &self.members
    }

    /// Return unique common appearance changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[BracketPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid bracket-pair appearance intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BracketPropertiesPatchV1Error {
    /// Both bracket members used the same durable document-object selector.
    #[error("bracket properties require two distinct document-object members")]
    DuplicateMembers,
    /// A request exceeded the two-field closed grammar.
    #[error("bracket properties accept at most two changes")]
    TooManyChanges,
    /// One closed property appeared more than once in one patch.
    #[error("bracket property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object_id(entropy: u8) -> DocumentObjectIdV1 {
        DocumentObjectIdV1::from_entropy_bytes([entropy; 16])
    }

    #[test]
    fn accepts_and_preserves_authoritative_opaque_member_order() {
        let members = [object_id(0x20), object_id(0x10)];
        let patch = BracketPropertiesPatchV1::new(members.clone(), Vec::new())
            .expect("authoritative opaque member order must be accepted");

        assert_eq!(patch.members(), &members);
    }

    #[test]
    fn rejects_duplicate_opaque_members() {
        let first = object_id(0x10);

        assert_eq!(
            BracketPropertiesPatchV1::new([first.clone(), first.clone()], Vec::new()),
            Err(BracketPropertiesPatchV1Error::DuplicateMembers)
        );
    }
}
