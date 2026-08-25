//! Structured deletion of one durable direct-root presentation record.

use xot::Xot;

use super::{
    CDML_NAMESPACE, PresentationRecordKindV1, PresentationRootDeletionSetV1,
    PresentationRootDeletionV1, TypedDocument, TypedDocumentError, element_name,
    reaction_reference_graph_v1::direct_reaction_reference_graph,
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
                .flat_map(|pair| pair.members())
                .any(|identifier| identifier == deletion.document_object_id())
        {
            return Err(TypedDocumentError::PresentationRootIsBracketMember(
                deletion.document_object_id().clone(),
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
        let id_name = document_object_id_name(&mut indexed.xml.tree);
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
                            == Some(deletion.document_object_id().as_str())
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Ok(None);
            }
            targets.push(matches[0]);
        }
        validate_complete_bracket_deletion(self, deletions)?;
        validate_reaction_references(self, deletions)?;
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

fn validate_reaction_references(
    document: &TypedDocument,
    deletions: &PresentationRootDeletionSetV1,
) -> Result<(), TypedDocumentError> {
    let selected = deletions
        .targets()
        .iter()
        .map(|target| target.document_object_id())
        .collect::<std::collections::HashSet<_>>();
    let references = direct_reaction_reference_graph(document);
    for child in document.root().typed_children() {
        let record = child.record();
        let Some(identifier) = crate::document_object_id_from_record_v1(record) else {
            continue;
        };
        if selected.contains(&identifier)
            && record
                .attribute("id")
                .is_some_and(|source_id| references.contains(source_id))
        {
            return Err(TypedDocumentError::ReactionReferencedPresentationDeletion(
                identifier,
            ));
        }
    }
    Ok(())
}

fn validate_complete_bracket_deletion(
    document: &TypedDocument,
    deletions: &PresentationRootDeletionSetV1,
) -> Result<(), TypedDocumentError> {
    let selected = deletions
        .targets()
        .iter()
        .map(|target| target.document_object_id())
        .collect::<std::collections::HashSet<_>>();
    for pair in super::bracket_pair_projection_v1::bracket_pairs(document) {
        let selected_members = pair
            .members()
            .iter()
            .filter(|identifier| selected.contains(*identifier))
            .count();
        if selected_members == 1 {
            return Err(TypedDocumentError::PartialBracketDeletion(
                pair.members().clone(),
            ));
        }
    }
    Ok(())
}

fn is_cdml_element(tree: &Xot, node: xot::Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}

fn document_object_id_name(tree: &mut Xot) -> xot::NameId {
    let namespace =
        tree.add_namespace(super::document_object_identity_v1::DOCUMENT_OBJECT_NAMESPACE_V1);
    tree.add_name_ns("id", namespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaction_referenced_arrow_text_and_plus_are_atomic_presentation_refusals() {
        let document = TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"a\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><text id=\"t\"><point x=\"0\" y=\"10\"/><ftext>conditions</ftext></text><plus id=\"p\"><point x=\"20\" y=\"0\"/></plus><reaction id=\"r\"><arrow idref=\"a\"/><condition idref=\"t\"/><plus idref=\"p\"/></reaction></cdml>").expect("fixture parses");
        for (id, kind) in [
            ("a", PresentationRecordKindV1::Arrow),
            ("t", PresentationRecordKindV1::Text),
            ("p", PresentationRecordKindV1::Plus),
        ] {
            let object_id = document
                .root()
                .typed_children()
                .iter()
                .find(|child| child.record().attribute("id") == Some(id))
                .and_then(|child| crate::document_object_id_from_record_v1(child.record()))
                .expect("typed ingress persists the durable root identity");
            let deletion = PresentationRootDeletionV1::new(object_id, kind);
            assert!(matches!(
                document.with_delete_presentation_root(&deletion),
                Err(TypedDocumentError::ReactionReferencedPresentationDeletion(
                    _
                ))
            ));
        }
    }
}
