//! Structured geometric appearance mutation preserving retained XML.

use xot::{Node, Xot};

use super::presentation_polyline_projection_v1::{coordinate, point};
use super::{
    CDML_NAMESPACE, GeometricLineWidthV1, GeometricPropertiesPatchV1, GeometricPropertyChangeV1,
    PersistentId, PresentationLengthV1, Rgb24V1, TransparentOrRgb24V1, TypedClass, TypedDocument,
    TypedDocumentError, TypedRecord, UnrecognizedNode, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one geometric appearance patch applied.
    pub(crate) fn with_geometric_properties(
        &self,
        patch: &GeometricPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let Some(record) = direct_geometric_record(self.root(), patch.presentation_id().as_str())
        else {
            return Ok(None);
        };
        validate_editable_geometry(record, patch.presentation_id(), patch.changes())?;
        let current = resolved_appearance(self, record, patch.presentation_id())?;

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let element = direct_geometric(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.presentation_id().as_str(),
            local_name(record.class()),
        )
        .expect("the detached candidate preserves the validated geometric root");
        apply_changes(&mut indexed.xml.tree, element, patch.changes(), &current);
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedAppearanceV1 {
    width: f64,
    stroke: Rgb24V1,
    fill: Option<Rgb24V1>,
}

fn direct_geometric_record<'a>(root: &'a TypedRecord, identifier: &str) -> Option<&'a TypedRecord> {
    root.typed_children()
        .iter()
        .map(super::TypedChild::record)
        .find(|record| {
            is_geometric_class(record.class()) && record.attribute("id") == Some(identifier)
        })
}

fn is_geometric_class(class: TypedClass) -> bool {
    matches!(
        class,
        TypedClass::Rectangle
            | TypedClass::Square
            | TypedClass::Oval
            | TypedClass::Circle
            | TypedClass::Polygon
            | TypedClass::Polyline
    )
}

fn is_closed_shape(class: TypedClass) -> bool {
    class != TypedClass::Polyline
}

fn validate_editable_geometry(
    record: &TypedRecord,
    identifier: &PersistentId,
    changes: &[GeometricPropertyChangeV1],
) -> Result<(), TypedDocumentError> {
    if record.class() == TypedClass::Polyline && record.attribute("style") == Some("wavy") {
        return Err(TypedDocumentError::SpecializedGeometricTarget(
            identifier.clone(),
        ));
    }
    if record.class() == TypedClass::Polyline
        && changes
            .iter()
            .any(|change| matches!(change, GeometricPropertyChangeV1::FillColor(_)))
    {
        return Err(TypedDocumentError::InapplicableGeometricProperty(
            identifier.clone(),
        ));
    }
    match record.class() {
        TypedClass::Rectangle | TypedClass::Square | TypedClass::Oval | TypedClass::Circle => {
            for field in ["x1", "y1", "x2", "y2"] {
                coordinate(record, field).map_err(|_| {
                    TypedDocumentError::InvalidGeometricStructure(identifier.clone())
                })?;
            }
            if !record.typed_children().is_empty() || has_unsupported_core_content(record) {
                return Err(TypedDocumentError::InvalidGeometricStructure(
                    identifier.clone(),
                ));
            }
        }
        TypedClass::Polygon | TypedClass::Polyline => {
            let minimum = if record.class() == TypedClass::Polygon {
                3
            } else {
                2
            };
            let points = record.children_of(TypedClass::Point).collect::<Vec<_>>();
            if points.len() < minimum
                || record
                    .typed_children()
                    .iter()
                    .any(|child| child.record().class() != TypedClass::Point)
                || has_unsupported_core_content(record)
            {
                return Err(TypedDocumentError::InvalidGeometricStructure(
                    identifier.clone(),
                ));
            }
            for source_point in points {
                if !source_point.typed_children().is_empty()
                    || has_unsupported_core_content(source_point)
                    || point(source_point).is_err()
                {
                    return Err(TypedDocumentError::InvalidGeometricStructure(
                        identifier.clone(),
                    ));
                }
            }
        }
        _ => unreachable!("geometric target lookup accepts only closed classes"),
    }
    Ok(())
}

fn has_unsupported_core_content(record: &TypedRecord) -> bool {
    record
        .unrecognized_children()
        .iter()
        .any(|child| match child.node() {
            UnrecognizedNode::Element { name, .. } => name.namespace() == CDML_NAMESPACE,
            UnrecognizedNode::Text(value) => !value.trim().is_empty(),
            UnrecognizedNode::Comment(_) | UnrecognizedNode::ProcessingInstruction { .. } => false,
        })
}

fn resolved_appearance(
    document: &TypedDocument,
    record: &TypedRecord,
    identifier: &PersistentId,
) -> Result<ResolvedAppearanceV1, TypedDocumentError> {
    let standard = document
        .root()
        .typed_children()
        .iter()
        .find(|child| child.record().class() == TypedClass::Standard)
        .map(super::TypedChild::record);
    let width = first_length(
        [(Some(record), "width"), (standard, "line_width")],
        1.0,
        identifier,
    )?;
    let stroke = first_color(
        [
            (Some(record), "line_color"),
            (Some(record), "color"),
            (standard, "line_color"),
        ],
        "#000000",
        identifier,
    )?;
    let fill = if is_closed_shape(record.class()) {
        first_fill(
            [
                (Some(record), "area_color"),
                (Some(record), "background-color"),
                (standard, "area_color"),
            ],
            identifier,
        )?
    } else {
        None
    };
    Ok(ResolvedAppearanceV1 {
        width,
        stroke,
        fill,
    })
}

fn first_length<const N: usize>(
    sources: [(Option<&TypedRecord>, &'static str); N],
    fallback: f64,
    identifier: &PersistentId,
) -> Result<f64, TypedDocumentError> {
    for (record, field) in sources {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        let parsed = PresentationLengthV1::parse(value)
            .map(|length| length.value().value())
            .and_then(GeometricLineWidthV1::new)
            .map(GeometricLineWidthV1::value)
            .ok_or_else(|| TypedDocumentError::InvalidGeometricProperty(identifier.clone()))?;
        return Ok(parsed);
    }
    Ok(fallback)
}

fn first_color<const N: usize>(
    sources: [(Option<&TypedRecord>, &'static str); N],
    fallback: &str,
    identifier: &PersistentId,
) -> Result<Rgb24V1, TypedDocumentError> {
    for (record, field) in sources {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        return Rgb24V1::new(value)
            .ok_or_else(|| TypedDocumentError::InvalidGeometricProperty(identifier.clone()));
    }
    Ok(Rgb24V1::new(fallback).expect("closed fallback colour is valid"))
}

fn first_fill<const N: usize>(
    sources: [(Option<&TypedRecord>, &'static str); N],
    identifier: &PersistentId,
) -> Result<Option<Rgb24V1>, TypedDocumentError> {
    for (record, field) in sources {
        let Some(value) = record.and_then(|record| record.attribute(field)) else {
            continue;
        };
        return TransparentOrRgb24V1::new(value)
            .map(|color| match color {
                TransparentOrRgb24V1::Transparent => None,
                TransparentOrRgb24V1::Rgb24(color) => Some(color),
            })
            .ok_or_else(|| TypedDocumentError::InvalidGeometricProperty(identifier.clone()));
    }
    Ok(None)
}

fn direct_geometric(
    tree: &mut Xot,
    document: Node,
    identifier: &str,
    expected: &str,
) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, expected)
            && tree.get_attribute(*node, id_name) == Some(identifier)
    })
}

fn apply_changes(
    tree: &mut Xot,
    element: Node,
    changes: &[GeometricPropertyChangeV1],
    current: &ResolvedAppearanceV1,
) {
    for change in changes {
        match change {
            GeometricPropertyChangeV1::LineWidth(value) if value.value() != current.width => {
                set(tree, element, "width", value.value().to_string());
            }
            GeometricPropertyChangeV1::StrokeColor(value) if value != &current.stroke => {
                set(tree, element, "line_color", value.as_str());
            }
            GeometricPropertyChangeV1::FillColor(value) if value != &current.fill => {
                set(
                    tree,
                    element,
                    "area_color",
                    value.as_ref().map_or("none", Rgb24V1::as_str),
                );
            }
            _ => {}
        }
    }
}

fn set(tree: &mut Xot, node: Node, name: &str, value: impl AsRef<str>) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value.as_ref());
}

fn local_name(class: TypedClass) -> &'static str {
    match class {
        TypedClass::Rectangle => "rect",
        TypedClass::Square => "square",
        TypedClass::Oval => "oval",
        TypedClass::Circle => "circle",
        TypedClass::Polygon => "polygon",
        TypedClass::Polyline => "polyline",
        _ => unreachable!("geometric target lookup accepts only closed classes"),
    }
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
