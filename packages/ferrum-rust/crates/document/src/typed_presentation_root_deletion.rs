//! Structured deletion of one durable direct-root presentation record.

use xot::Xot;

use super::{
    CDML_NAMESPACE, PresentationRecordKindV1, PresentationRootDeletionSetV1,
    PresentationRootDeletionV1, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate without one exact durable presentation root.
    ///
    /// Bracket members require a future pair operation so this single-root seam
    /// cannot leave half of one authoritative bracket relationship behind.
    pub(crate) fn with_delete_presentation_root(
        &self,
        deletion: &PresentationRootDeletionV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        if deletion.kind() == PresentationRecordKindV1::Polyline
            && super::bracket_pair_projection_v1::bracket_pairs(self)
                .iter()
                .flat_map(|pair| pair.member_ids())
                .any(|identifier| identifier == deletion.presentation_id().as_str())
        {
            return Err(TypedDocumentError::PresentationRootIsBracketMember(
                deletion.presentation_id().clone(),
            ));
        }
        let deletions = PresentationRootDeletionSetV1::new(vec![deletion.clone()])
            .expect("one validated selector is a valid deletion set");
        self.with_delete_presentation_roots(&deletions)
    }

    /// Return a detached candidate without one complete exact durable target set.
    pub(crate) fn with_delete_presentation_roots(
        &self,
        deletions: &PresentationRootDeletionSetV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let children = indexed.xml.tree.children(root).collect::<Vec<_>>();
        let mut targets = Vec::with_capacity(deletions.targets().len());
        for deletion in deletions.targets() {
            let matches = children
                .iter()
                .copied()
                .filter(|node| {
                    is_cdml_element(&indexed.xml.tree, *node, deletion.kind().local_name())
                        && indexed.xml.tree.get_attribute(*node, id_name)
                            == Some(deletion.presentation_id().as_str())
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Ok(None);
            }
            targets.push(matches[0]);
        }
        validate_complete_bracket_deletion(self, deletions)?;
        for target in targets {
            indexed
                .xml
                .tree
                .remove(target)
                .map_err(TypedDocumentError::Mutation)?;
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn validate_complete_bracket_deletion(
    document: &TypedDocument,
    deletions: &PresentationRootDeletionSetV1,
) -> Result<(), TypedDocumentError> {
    let selected = deletions
        .targets()
        .iter()
        .map(|target| target.presentation_id().as_str())
        .collect::<std::collections::HashSet<_>>();
    for pair in super::bracket_pair_projection_v1::bracket_pairs(document) {
        let selected_members = pair
            .member_ids()
            .iter()
            .filter(|identifier| selected.contains(identifier.as_str()))
            .count();
        if selected_members == 1 {
            return Err(TypedDocumentError::PartialBracketDeletion(
                pair.pair_id().to_owned(),
            ));
        }
    }
    Ok(())
}

fn is_cdml_element(tree: &Xot, node: xot::Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
