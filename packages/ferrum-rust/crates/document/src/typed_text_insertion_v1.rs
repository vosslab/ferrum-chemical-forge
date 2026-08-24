//! Canonical standalone Text insertion owned by the typed CDML layer.

use super::{
    PersistentId, PresentationGesturePoint2V1, Rgb24V1, TextEditRunV1, TextEditStyleV1,
    TypedDocument, TypedDocumentError, element_name,
    typed_coordinate::canonical_authored_coordinate,
};
use xot::Xot;

impl TypedDocument {
    pub(crate) fn with_insert_authored_text_v1(
        &self,
        identifier: &PersistentId,
        anchor: PresentationGesturePoint2V1,
        runs: &[TextEditRunV1],
        font_size: Option<u16>,
        color: Option<&Rgb24V1>,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(identifier).is_some() {
            return Err(TypedDocumentError::DuplicateBondId(identifier.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("parsed CDML has root");
        let namespace = element_name(&indexed.xml.tree, root)
            .map(|(_, value)| value)
            .unwrap_or_default();
        let text_name = name(&mut indexed.xml.tree, "text", &namespace);
        let text = indexed.xml.tree.new_element(text_name);
        let id_name = indexed.xml.tree.add_name("id");
        indexed
            .xml
            .tree
            .set_attribute(text, id_name, identifier.as_str());
        let point_name = name(&mut indexed.xml.tree, "point", &namespace);
        let point = indexed.xml.tree.new_element(point_name);
        let x_name = indexed.xml.tree.add_name("x");
        let y_name = indexed.xml.tree.add_name("y");
        indexed
            .xml
            .tree
            .set_attribute(point, x_name, canonical_authored_coordinate(anchor.x()));
        indexed
            .xml
            .tree
            .set_attribute(point, y_name, canonical_authored_coordinate(anchor.y()));
        indexed
            .xml
            .tree
            .append(text, point)
            .map_err(TypedDocumentError::Mutation)?;
        if font_size.is_some() || color.is_some() {
            let font_name = name(&mut indexed.xml.tree, "font", &namespace);
            let font = indexed.xml.tree.new_element(font_name);
            if let Some(value) = font_size {
                let size_name = indexed.xml.tree.add_name("size");
                indexed
                    .xml
                    .tree
                    .set_attribute(font, size_name, value.to_string());
            }
            if let Some(value) = color {
                let color_name = indexed.xml.tree.add_name("color");
                indexed
                    .xml
                    .tree
                    .set_attribute(font, color_name, value.as_str());
            }
            indexed
                .xml
                .tree
                .append(text, font)
                .map_err(TypedDocumentError::Mutation)?;
        }
        let ftext_name = name(&mut indexed.xml.tree, "ftext", &namespace);
        let ftext = indexed.xml.tree.new_element(ftext_name);
        indexed
            .xml
            .tree
            .append_text(ftext, &encode_runs(runs))
            .map_err(TypedDocumentError::Mutation)?;
        indexed
            .xml
            .tree
            .append(text, ftext)
            .map_err(TypedDocumentError::Mutation)?;
        indexed
            .xml
            .tree
            .append(root, text)
            .map_err(TypedDocumentError::Mutation)?;
        Self::parse(&candidate.to_xml()?)
    }
}

fn name(tree: &mut Xot, local: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local)
    } else {
        let ns = tree.add_namespace(namespace);
        tree.add_name_ns(local, ns)
    }
}
fn encode_runs(runs: &[TextEditRunV1]) -> String {
    let mut output = String::new();
    for run in runs {
        for style in run.styles() {
            output.push('<');
            output.push_str(tag(*style));
            output.push('>');
        }
        for value in run.text().chars() {
            match value {
                '&' => output.push_str("&amp;"),
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                _ => output.push(value),
            }
        }
        for style in run.styles().iter().rev() {
            output.push_str("</");
            output.push_str(tag(*style));
            output.push('>');
        }
    }
    output
}
const fn tag(style: TextEditStyleV1) -> &'static str {
    match style {
        TextEditStyleV1::Bold => "b",
        TextEditStyleV1::Italic => "i",
        TextEditStyleV1::Subscript => "sub",
        TextEditStyleV1::Superscript => "sup",
    }
}
