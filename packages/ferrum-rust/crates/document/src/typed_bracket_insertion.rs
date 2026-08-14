//! Structured insertion of one complete paired bracket presentation.

use xot::{NameId, Xot};

use super::presentation_polyline_projection_v1::parse_width;
use super::{
    BracketInsertionV1, BracketStyleV1, CDML_NAMESPACE, PersistentId, Rgb24V1, TypedClass,
    TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Append two validated bracket sides while preserving retained root content.
    pub(crate) fn with_insert_bracket(
        &self,
        left_id: &PersistentId,
        right_id: &PersistentId,
        insertion: &BracketInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        if left_id == right_id
            || self.indexed().resolve_id(left_id).is_some()
            || self.indexed().resolve_id(right_id).is_some()
        {
            return Err(TypedDocumentError::DuplicateInsertionId(left_id.clone()));
        }
        let (line_width, line_color) = insertion_stroke(self)?;
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let names = BracketNames::new(&mut indexed.xml.tree, namespace);
        for (identifier, side, points) in [
            (left_id, "left", insertion.left()),
            (right_id, "right", insertion.right()),
        ] {
            let polyline = indexed.xml.tree.new_element(names.polyline);
            set(
                &mut indexed.xml.tree,
                polyline,
                names.id,
                identifier.as_str(),
            );
            set(
                &mut indexed.xml.tree,
                polyline,
                names.bracket_pair,
                left_id.as_str(),
            );
            set(&mut indexed.xml.tree, polyline, names.bracket_side, side);
            set(
                &mut indexed.xml.tree,
                polyline,
                names.line_color,
                line_color.as_str(),
            );
            set(
                &mut indexed.xml.tree,
                polyline,
                names.width,
                line_width.to_string(),
            );
            set(
                &mut indexed.xml.tree,
                polyline,
                names.spline,
                match insertion.style() {
                    BracketStyleV1::Rectangular => "no",
                    BracketStyleV1::Round => "yes",
                },
            );
            for source in points {
                let point = indexed.xml.tree.new_element(names.point);
                set(
                    &mut indexed.xml.tree,
                    point,
                    names.x,
                    source.x().to_string(),
                );
                set(
                    &mut indexed.xml.tree,
                    point,
                    names.y,
                    source.y().to_string(),
                );
                indexed
                    .xml
                    .tree
                    .append(polyline, point)
                    .map_err(TypedDocumentError::Mutation)?;
            }
            indexed
                .xml
                .tree
                .append(root, polyline)
                .map_err(TypedDocumentError::Mutation)?;
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}

fn insertion_stroke(document: &TypedDocument) -> Result<(f64, Rgb24V1), TypedDocumentError> {
    let standard = document
        .root()
        .typed_children()
        .iter()
        .find(|child| child.record().class() == TypedClass::Standard)
        .map(super::TypedChild::record);
    let width = match standard.and_then(|record| record.attribute("line_width")) {
        Some(value) => parse_width(value)
            .map(super::PositiveFiniteV1::value)
            .ok_or(TypedDocumentError::InvalidBracketStandard)?,
        None => 1.0,
    };
    let color = match standard.and_then(|record| record.attribute("line_color")) {
        Some(value) => Rgb24V1::new(value).ok_or(TypedDocumentError::InvalidBracketStandard)?,
        None => Rgb24V1::new("#000000").expect("closed built-in colour is valid"),
    };
    Ok((width, color))
}

fn set(tree: &mut Xot, node: xot::Node, name: NameId, value: impl AsRef<str>) {
    tree.set_attribute(node, name, value.as_ref());
}

#[derive(Clone, Copy)]
struct BracketNames {
    polyline: NameId,
    point: NameId,
    id: NameId,
    bracket_pair: NameId,
    bracket_side: NameId,
    line_color: NameId,
    width: NameId,
    spline: NameId,
    x: NameId,
    y: NameId,
}

impl BracketNames {
    fn new(tree: &mut Xot, namespace: String) -> Self {
        let namespace = if namespace.is_empty() || namespace == CDML_NAMESPACE {
            namespace
        } else {
            unreachable!("TypedDocument accepts only no-namespace or CDML roots")
        };
        Self {
            polyline: element_name_id(tree, "polyline", &namespace),
            point: element_name_id(tree, "point", &namespace),
            id: tree.add_name("id"),
            bracket_pair: tree.add_name("bracket_pair"),
            bracket_side: tree.add_name("bracket_side"),
            line_color: tree.add_name("line_color"),
            width: tree.add_name("width"),
            spline: tree.add_name("spline"),
            x: tree.add_name("x"),
            y: tree.add_name("y"),
        }
    }
}

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}
