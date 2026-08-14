//! Structured direct-root Arrow property mutation preserving retained XML.

use xot::{Node, Xot};

use super::presentation_polyline_projection_v1::point;
use super::{
    ArrowPropertiesPatchV1, ArrowPropertyChangeV1, CDML_NAMESPACE, PersistentId,
    PresentationLengthV1, Rgb24V1, TypedClass, TypedDocument, TypedDocumentError, TypedRecord,
    UnrecognizedNode, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one complete Arrow-properties patch applied.
    pub(crate) fn with_arrow_properties(
        &self,
        patch: &ArrowPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let Some(record) = direct_arrow_record(self.root(), patch.arrow_id().as_str()) else {
            return Ok(None);
        };
        validate_editable_arrow(record, patch.arrow_id())?;

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let arrow = direct_arrow(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.arrow_id().as_str(),
        )
        .expect("the detached candidate preserves the validated direct Arrow");
        apply_changes(
            &mut indexed.xml.tree,
            arrow,
            patch.arrow_id(),
            patch.changes(),
        )?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_arrow_record<'a>(root: &'a TypedRecord, identifier: &str) -> Option<&'a TypedRecord> {
    root.typed_children()
        .iter()
        .map(super::TypedChild::record)
        .find(|record| {
            record.class() == TypedClass::CanvasArrow && record.attribute("id") == Some(identifier)
        })
}

fn validate_editable_arrow(
    arrow: &TypedRecord,
    arrow_id: &PersistentId,
) -> Result<(), TypedDocumentError> {
    let points = arrow.children_of(TypedClass::Point).collect::<Vec<_>>();
    if points.len() < 2
        || arrow
            .typed_children()
            .iter()
            .any(|child| child.record().class() != TypedClass::Point)
        || has_unsupported_core_content(arrow)
    {
        return Err(TypedDocumentError::InvalidArrowStructure(arrow_id.clone()));
    }
    for record in points {
        if !record.typed_children().is_empty()
            || has_unsupported_core_content(record)
            || point(record).is_err()
        {
            return Err(TypedDocumentError::InvalidArrowStructure(arrow_id.clone()));
        }
    }
    Ok(())
}

fn has_unsupported_core_content(record: &TypedRecord) -> bool {
    record
        .unrecognized_children()
        .iter()
        .any(|child| match child.node() {
            UnrecognizedNode::Element { name, .. } => {
                name.namespace().is_empty() || name.namespace() == CDML_NAMESPACE
            }
            UnrecognizedNode::Text(value) => !value.trim().is_empty(),
            UnrecognizedNode::Comment(_) | UnrecognizedNode::ProcessingInstruction { .. } => false,
        })
}

fn direct_arrow(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "arrow")
            && tree.get_attribute(*node, id_name) == Some(identifier)
    })
}

fn apply_changes(
    tree: &mut Xot,
    arrow: Node,
    arrow_id: &PersistentId,
    changes: &[ArrowPropertyChangeV1],
) -> Result<(), TypedDocumentError> {
    for change in changes {
        match change {
            ArrowPropertyChangeV1::StartHead(value) => {
                set_changed_bool(tree, arrow, arrow_id, "start", *value, false)?
            }
            ArrowPropertyChangeV1::EndHead(value) => {
                set_changed_bool(tree, arrow, arrow_id, "end", *value, true)?
            }
            ArrowPropertyChangeV1::Spline(value) => {
                set_changed_bool(tree, arrow, arrow_id, "spline", *value, false)?
            }
            ArrowPropertyChangeV1::LineWidth(value) => {
                let name = tree.add_name("width");
                let current = tree.get_attribute(arrow, name).map_or(Ok(1.0), |source| {
                    PresentationLengthV1::parse(source)
                        .map(|width| width.value().value())
                        .ok_or_else(|| TypedDocumentError::InvalidArrowProperty(arrow_id.clone()))
                })?;
                if current != value.value() {
                    tree.set_attribute(arrow, name, value.value().to_string());
                }
            }
            ArrowPropertyChangeV1::Color(value) => {
                let name = tree.add_name("color");
                let current = match tree.get_attribute(arrow, name) {
                    None => Rgb24V1::new("#000000").expect("closed default is valid"),
                    Some(source) => Rgb24V1::new(source).ok_or_else(|| {
                        TypedDocumentError::InvalidArrowProperty(arrow_id.clone())
                    })?,
                };
                if &current != value {
                    tree.set_attribute(arrow, name, value.as_str());
                }
            }
        }
    }
    Ok(())
}

fn set_changed_bool(
    tree: &mut Xot,
    arrow: Node,
    arrow_id: &PersistentId,
    field: &'static str,
    requested: bool,
    default: bool,
) -> Result<(), TypedDocumentError> {
    let name = tree.add_name(field);
    let current = match tree.get_attribute(arrow, name) {
        None => default,
        Some(value) => parse_bool(value)
            .ok_or_else(|| TypedDocumentError::InvalidArrowProperty(arrow_id.clone()))?,
    };
    if current != requested {
        tree.set_attribute(arrow, name, if requested { "yes" } else { "no" });
    }
    Ok(())
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "both" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
