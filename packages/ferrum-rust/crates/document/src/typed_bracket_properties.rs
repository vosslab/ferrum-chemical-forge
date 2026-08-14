//! Atomic common appearance mutation for one retained bracket pair.

use xot::{Node, Xot};

use super::{
    BracketPropertiesPatchV1, BracketPropertyChangeV1, CDML_NAMESPACE, PersistentId, TypedClass,
    TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one bracket-pair appearance patch applied.
    pub(crate) fn with_bracket_properties(
        &self,
        patch: &BracketPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let has_marker = self.root().typed_children().iter().any(|child| {
            child.record().class() == TypedClass::Polyline
                && child.record().attribute("bracket_pair") == Some(patch.pair_id().as_str())
        });
        if !has_marker {
            return Ok(None);
        }
        let pair = super::bracket_pair_projection_v1::bracket_pairs(self)
            .into_iter()
            .find(|pair| pair.pair_id() == patch.pair_id().as_str())
            .ok_or_else(|| TypedDocumentError::InvalidBracketPair(patch.pair_id().clone()))?;
        validate_pair_geometry(self, &pair)?;

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        for identifier in pair.member_ids() {
            let element = direct_polyline(&mut indexed.xml.tree, indexed.xml.document, identifier)
                .expect("the detached candidate preserves the validated bracket member");
            apply_changes(&mut indexed.xml.tree, element, patch.changes(), &pair);
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn validate_pair_geometry(
    document: &TypedDocument,
    pair: &super::BracketPairProjectionV1,
) -> Result<(), TypedDocumentError> {
    for identifier in pair.member_ids() {
        let record = document
            .root()
            .typed_children()
            .iter()
            .map(super::TypedChild::record)
            .find(|record| {
                record.class() == TypedClass::Polyline
                    && record.attribute("id") == Some(identifier.as_str())
            })
            .ok_or_else(|| TypedDocumentError::InvalidBracketPair(pair_id(pair)))?;
        if !super::bracket_pair_projection_v1::valid_bracket_member(record) {
            return Err(TypedDocumentError::InvalidBracketPair(pair_id(pair)));
        }
    }
    Ok(())
}

fn pair_id(pair: &super::BracketPairProjectionV1) -> PersistentId {
    PersistentId::new(pair.pair_id().to_owned()).expect("a projected pair ID is valid")
}

fn direct_polyline(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "polyline")
            && tree.get_attribute(*node, id_name) == Some(identifier)
    })
}

fn apply_changes(
    tree: &mut Xot,
    element: Node,
    changes: &[BracketPropertyChangeV1],
    current: &super::BracketPairProjectionV1,
) {
    for change in changes {
        match change {
            BracketPropertyChangeV1::LineWidth(value)
                if current.line_width().map(|width| width.value()) != Some(value.value()) =>
            {
                set(tree, element, "width", value.value().to_string());
            }
            BracketPropertyChangeV1::LineColor(value) if current.line_color() != Some(value) => {
                set(tree, element, "line_color", value.as_str());
            }
            _ => {}
        }
    }
}

fn set(tree: &mut Xot, node: Node, name: &str, value: impl AsRef<str>) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value.as_ref());
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
