//! Atomic ordering of durable direct-root presentation records.

use std::collections::HashSet;

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PresentationStackOrderV1, PresentationStackReorderV1, TypedDocument,
    TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with presentation roots reordered in element slots.
    pub(crate) fn with_reorder_presentation_roots(
        &self,
        reorder: &PresentationStackReorderV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        validate_complete_bracket_selection(self, reorder)?;
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let elements = indexed
            .xml
            .tree
            .children(root)
            .filter(|node| indexed.xml.tree.is_element(*node))
            .collect::<Vec<_>>();
        let mut selected = Vec::with_capacity(reorder.targets().len());
        for target in reorder.targets() {
            let matches = elements
                .iter()
                .copied()
                .filter(|node| {
                    is_cdml_element(&indexed.xml.tree, *node, target.kind().local_name())
                        && indexed.xml.tree.get_attribute(*node, id_name)
                            == Some(target.presentation_id().as_str())
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Ok(None);
            }
            selected.push(matches[0]);
        }
        let selected_set = selected.iter().copied().collect::<HashSet<_>>();
        let selected_in_source_order = elements
            .iter()
            .copied()
            .filter(|node| selected_set.contains(node))
            .collect::<Vec<_>>();
        let ordered =
            ordered_elements(&elements, &selected_set, &selected_in_source_order, reorder);
        if ordered == elements {
            return Ok(Some(candidate));
        }
        let replacements = ordered
            .iter()
            .map(|node| indexed.xml.tree.clone_node(*node))
            .collect::<Vec<_>>();
        for (original, replacement) in elements.into_iter().zip(replacements) {
            indexed
                .xml
                .tree
                .insert_before(original, replacement)
                .map_err(TypedDocumentError::Mutation)?;
            indexed
                .xml
                .tree
                .remove(original)
                .map_err(TypedDocumentError::Mutation)?;
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn validate_complete_bracket_selection(
    document: &TypedDocument,
    reorder: &PresentationStackReorderV1,
) -> Result<(), TypedDocumentError> {
    let selected = reorder
        .targets()
        .iter()
        .map(|target| target.presentation_id().as_str())
        .collect::<HashSet<_>>();
    for pair in super::bracket_pair_projection_v1::bracket_pairs(document) {
        let selected_members = pair
            .member_ids()
            .iter()
            .filter(|identifier| selected.contains(identifier.as_str()))
            .count();
        if selected_members == 1 {
            return Err(TypedDocumentError::PartialBracketStackSelection(
                pair.pair_id().to_owned(),
            ));
        }
    }
    Ok(())
}

fn ordered_elements(
    elements: &[Node],
    selected: &HashSet<Node>,
    selected_in_source_order: &[Node],
    reorder: &PresentationStackReorderV1,
) -> Vec<Node> {
    match reorder.order() {
        PresentationStackOrderV1::BringToFront => elements
            .iter()
            .copied()
            .filter(|node| !selected.contains(node))
            .chain(selected_in_source_order.iter().copied())
            .collect(),
        PresentationStackOrderV1::SendToBack => selected_in_source_order
            .iter()
            .copied()
            .chain(
                elements
                    .iter()
                    .copied()
                    .filter(|node| !selected.contains(node)),
            )
            .collect(),
        PresentationStackOrderV1::ReverseSelectedSlots => {
            let mut reversed = selected_in_source_order.iter().copied().rev();
            elements
                .iter()
                .copied()
                .map(|node| {
                    if selected.contains(&node) {
                        reversed.next().expect("validated selected slot count")
                    } else {
                        node
                    }
                })
                .collect()
        }
    }
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
