//! Semantic direct-root reaction-reference indexing for destructive operations.
//!
//! CDML compatibility payloads remain opaque.  Only a direct CDML `<reaction>`
//! root and one of its direct recognized role children contribute an ownership
//! edge, so deletion code never mistakes nested or foreign lookalikes for a
//! reaction reference.

use std::collections::HashSet;

use super::{DirectCdmlRootKindV1, DirectCdmlSemanticIndexV1, TypedDocument};

/// Complete set of durable IDs referenced by recognized direct reaction roles.
pub(crate) struct DirectReactionReferenceGraphV1 {
    referenced_ids: HashSet<String>,
}

impl DirectReactionReferenceGraphV1 {
    /// Return whether a recognized reaction role directly references this ID.
    pub(crate) fn contains(&self, identifier: &str) -> bool {
        self.referenced_ids.contains(identifier)
    }
}

/// Index direct semantic reaction references from one retained CDML document.
pub(crate) fn direct_reaction_reference_graph(
    document: &TypedDocument,
) -> DirectReactionReferenceGraphV1 {
    let semantic = DirectCdmlSemanticIndexV1::from_document(document);
    let referenced_ids = semantic
        .roots()
        .iter()
        .filter(|root| root.kind() == DirectCdmlRootKindV1::Reaction)
        .flat_map(|reaction| reaction.reaction_members().iter().cloned())
        .collect();
    DirectReactionReferenceGraphV1 { referenced_ids }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_only_direct_core_reaction_roles() {
        let document = TypedDocument::parse(concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:c=\"urn:ferrum:cdml\" ",
            "xmlns:v=\"urn:vendor\"><c:reaction id=\"r\">",
            "<c:arrow idref=\"arrow\"/><c:condition idref=\"text\"/>",
            "<c:plus idref=\"plus\"/><c:note idref=\"note\"/>",
            "<c:container><c:arrow idref=\"nested\"/></c:container>",
            "<c:arrow v:idref=\"foreign-attribute\"/></c:reaction>",
            "<v:reaction><v:arrow idref=\"foreign-role\"/></v:reaction></cdml>"
        ))
        .expect("fixture parses");
        let graph = direct_reaction_reference_graph(&document);
        assert!(graph.contains("arrow"));
        assert!(graph.contains("text"));
        assert!(graph.contains("plus"));
        assert!(!graph.contains("note"));
        assert!(!graph.contains("nested"));
        assert!(!graph.contains("foreign-attribute"));
        assert!(!graph.contains("foreign-role"));
    }
}
