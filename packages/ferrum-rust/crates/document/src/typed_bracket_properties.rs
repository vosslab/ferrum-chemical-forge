//! Atomic common appearance mutation for one retained bracket pair.

use xot::{Node, Xot};

use ferrum_document_projection::DocumentObjectIdV1;

use super::{
    BracketPropertiesPatchV1, BracketPropertyChangeV1, CDML_NAMESPACE, TypedClass, TypedDocument,
    TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one bracket-pair appearance patch applied.
    pub(crate) fn with_bracket_properties(
        &self,
        patch: &BracketPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let pair = super::bracket_pair_projection_v1::bracket_pairs(self)?
            .into_iter()
            .find(|pair| pair.members() == patch.members())
            .ok_or_else(|| TypedDocumentError::InvalidBracketPair(patch.members().clone()))?;
        validate_pair_geometry(self, &pair)?;

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        for identifier in pair.members() {
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
    for identifier in pair.members() {
        let mut member = None;
        for record in document
            .root()
            .typed_children()
            .iter()
            .map(super::TypedChild::record)
        {
            if record.class() != TypedClass::Polyline {
                continue;
            }
            let object_id =
                crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)
                    .map_err(|_| TypedDocumentError::InvalidBracketPair(pair.members().clone()))?;
            if &object_id == identifier {
                member = Some(record);
                break;
            }
        }
        let record =
            member.ok_or_else(|| TypedDocumentError::InvalidBracketPair(pair.members().clone()))?;
        if !super::bracket_pair_projection_v1::valid_bracket_member(record) {
            return Err(TypedDocumentError::InvalidBracketPair(
                pair.members().clone(),
            ));
        }
    }
    Ok(())
}

fn direct_polyline(
    tree: &mut Xot,
    document: Node,
    identifier: &DocumentObjectIdV1,
) -> Option<Node> {
    let id_name = document_object_id_name(tree);
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "polyline")
            && tree.get_attribute(*node, id_name) == Some(identifier.as_str())
    })
}

fn document_object_id_name(tree: &mut Xot) -> xot::NameId {
    let namespace =
        tree.add_namespace(super::document_object_identity_v1::DOCUMENT_OBJECT_NAMESPACE_V1);
    tree.add_name_ns("id", namespace)
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
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
