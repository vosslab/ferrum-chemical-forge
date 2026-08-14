//! Structured Wavy appearance mutation preserving retained XML.

use xot::{Node, Xot};

use super::presentation_polyline_projection_v1::point;
use super::{
    CDML_NAMESPACE, GeometricLineWidthV1, PersistentId, PresentationLengthV1, Rgb24V1, TypedClass,
    TypedDocument, TypedDocumentError, TypedRecord, UnrecognizedNode, WavyPropertiesPatchV1,
    WavyPropertyChangeV1, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one Wavy appearance patch applied.
    pub(crate) fn with_wavy_properties(
        &self,
        patch: &WavyPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let Some(record) = direct_wavy_record(self.root(), patch.wavy_id().as_str()) else {
            return Ok(None);
        };
        validate_structure(record, patch.wavy_id())?;
        let current = resolved_appearance(self, record, patch.wavy_id())?;

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let element = direct_wavy(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.wavy_id().as_str(),
        )
        .expect("the detached candidate preserves the validated Wavy root");
        apply_changes(&mut indexed.xml.tree, element, patch.changes(), &current);
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedWavyAppearanceV1 {
    width: f64,
    color: Rgb24V1,
}

fn direct_wavy_record<'a>(root: &'a TypedRecord, identifier: &str) -> Option<&'a TypedRecord> {
    root.typed_children()
        .iter()
        .map(super::TypedChild::record)
        .find(|record| {
            record.class() == TypedClass::Polyline
                && record.attribute("id") == Some(identifier)
                && record.attribute("style") == Some("wavy")
        })
}

fn validate_structure(
    record: &TypedRecord,
    identifier: &PersistentId,
) -> Result<(), TypedDocumentError> {
    let points = record.children_of(TypedClass::Point).collect::<Vec<_>>();
    if points.len() < 2
        || record
            .typed_children()
            .iter()
            .any(|child| child.record().class() != TypedClass::Point)
        || has_unsupported_core_content(record)
    {
        return Err(TypedDocumentError::InvalidWavyStructure(identifier.clone()));
    }
    for source_point in points {
        if !source_point.typed_children().is_empty()
            || has_unsupported_core_content(source_point)
            || point(source_point).is_err()
        {
            return Err(TypedDocumentError::InvalidWavyStructure(identifier.clone()));
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

fn resolved_appearance(
    document: &TypedDocument,
    record: &TypedRecord,
    identifier: &PersistentId,
) -> Result<ResolvedWavyAppearanceV1, TypedDocumentError> {
    let standard = document
        .root()
        .typed_children()
        .iter()
        .find(|child| child.record().class() == TypedClass::Standard)
        .map(super::TypedChild::record);
    let width = first_width(
        [(Some(record), "width"), (standard, "line_width")],
        identifier,
    )?;
    let color = first_color(
        [
            (Some(record), "line_color"),
            (Some(record), "color"),
            (standard, "line_color"),
        ],
        identifier,
    )?;
    Ok(ResolvedWavyAppearanceV1 { width, color })
}

fn first_width<const N: usize>(
    sources: [(Option<&TypedRecord>, &'static str); N],
    identifier: &PersistentId,
) -> Result<f64, TypedDocumentError> {
    for (record, field) in sources {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        return PresentationLengthV1::parse(value)
            .map(|length| length.value().value())
            .and_then(GeometricLineWidthV1::new)
            .map(GeometricLineWidthV1::value)
            .ok_or_else(|| TypedDocumentError::InvalidWavyProperty(identifier.clone()));
    }
    Ok(1.0)
}

fn first_color<const N: usize>(
    sources: [(Option<&TypedRecord>, &'static str); N],
    identifier: &PersistentId,
) -> Result<Rgb24V1, TypedDocumentError> {
    for (record, field) in sources {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        return Rgb24V1::new(value)
            .ok_or_else(|| TypedDocumentError::InvalidWavyProperty(identifier.clone()));
    }
    Ok(Rgb24V1::new("#000000").expect("closed fallback colour is valid"))
}

fn direct_wavy(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let style_name = tree.add_name("style");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "polyline")
            && tree.get_attribute(*node, id_name) == Some(identifier)
            && tree.get_attribute(*node, style_name) == Some("wavy")
    })
}

fn apply_changes(
    tree: &mut Xot,
    element: Node,
    changes: &[WavyPropertyChangeV1],
    current: &ResolvedWavyAppearanceV1,
) {
    for change in changes {
        match change {
            WavyPropertyChangeV1::LineWidth(value) if value.value() != current.width => {
                set(tree, element, "width", value.value().to_string());
            }
            WavyPropertyChangeV1::LineColor(value) if value != &current.color => {
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
