//! Structured direct-root Text mutation preserving unrelated retained XML.

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PersistentId, TextEditRunV1, TextEditStyleV1, TextPropertiesPatchV1,
    TextPropertyChangeV1, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one complete Text-properties patch applied.
    pub(crate) fn with_text_properties(
        &self,
        patch: &TextPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let text = direct_text(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.text_id().as_str(),
        );
        let Some(text) = text else {
            return Ok(None);
        };
        let (point, ftext) = editable_core(&indexed.xml.tree, text, patch.text_id())?;
        let create_font = patch.changes().iter().any(|change| {
            matches!(
                change,
                TextPropertyChangeV1::FontFamily(Some(_))
                    | TextPropertyChangeV1::FontSize(_)
                    | TextPropertyChangeV1::Color(_)
            )
        });
        let font = editable_font(
            &mut indexed.xml.tree,
            text,
            point,
            patch.text_id(),
            create_font,
        )?;
        apply_changes(
            &mut indexed.xml.tree,
            text,
            ftext,
            font,
            patch.changes(),
            patch.text_id(),
        )?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_text(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "text")
            && tree.get_attribute(*node, id_name) == Some(identifier)
    })
}

fn editable_core(
    tree: &Xot,
    text: Node,
    text_id: &PersistentId,
) -> Result<(Node, Node), TypedDocumentError> {
    let points = tree
        .children(text)
        .filter(|node| is_cdml_element(tree, *node, "point"))
        .collect::<Vec<_>>();
    let ftexts = tree
        .children(text)
        .filter(|node| is_cdml_element(tree, *node, "ftext"))
        .collect::<Vec<_>>();
    match (points.as_slice(), ftexts.as_slice()) {
        ([point], [ftext]) => Ok((*point, *ftext)),
        _ => Err(TypedDocumentError::InvalidTextStructure(text_id.clone())),
    }
}

fn editable_font(
    tree: &mut Xot,
    text: Node,
    point: Node,
    text_id: &PersistentId,
    create: bool,
) -> Result<Option<Node>, TypedDocumentError> {
    let fonts = tree
        .children(text)
        .filter(|node| is_cdml_element(tree, *node, "font"))
        .collect::<Vec<_>>();
    match fonts.as_slice() {
        [font] => Ok(Some(*font)),
        [] if !create => Ok(None),
        [] => {
            let namespace = element_name(tree, text)
                .expect("a typed Text is an element")
                .1;
            let name = element_name_id(tree, "font", &namespace);
            let font = tree.new_element(name);
            tree.insert_after(point, font)
                .map_err(TypedDocumentError::Mutation)?;
            Ok(Some(font))
        }
        _ => Err(TypedDocumentError::AmbiguousTextFonts(text_id.clone())),
    }
}

fn apply_changes(
    tree: &mut Xot,
    text: Node,
    ftext: Node,
    font: Option<Node>,
    changes: &[TextPropertyChangeV1],
    text_id: &PersistentId,
) -> Result<(), TypedDocumentError> {
    for change in changes {
        match change {
            TextPropertyChangeV1::Runs(runs) => replace_runs(tree, ftext, runs, text_id)?,
            TextPropertyChangeV1::FontFamily(Some(value)) => set(
                tree,
                font.expect("family edits resolve one direct font"),
                "family",
                value,
            ),
            TextPropertyChangeV1::FontFamily(None) => {
                if let Some(font) = font {
                    remove(tree, font, "family");
                }
            }
            TextPropertyChangeV1::FontSize(value) => set(
                tree,
                font.expect("size edits resolve one direct font"),
                "size",
                value.to_string(),
            ),
            TextPropertyChangeV1::Color(value) => set(
                tree,
                font.expect("color edits resolve one direct font"),
                "color",
                value.as_str(),
            ),
            TextPropertyChangeV1::BackgroundColor(Some(value)) => {
                set(tree, text, "background-color", value.as_str())
            }
            TextPropertyChangeV1::BackgroundColor(None) => set(tree, text, "background-color", ""),
        }
    }
    Ok(())
}

fn replace_runs(
    tree: &mut Xot,
    ftext: Node,
    runs: &[TextEditRunV1],
    text_id: &PersistentId,
) -> Result<(), TypedDocumentError> {
    let children = tree.children(ftext).collect::<Vec<_>>();
    if children.iter().any(|node| tree.text(*node).is_none()) {
        return Err(TypedDocumentError::InvalidTextStructure(text_id.clone()));
    }
    for child in children {
        tree.remove(child).map_err(TypedDocumentError::Mutation)?;
    }
    tree.append_text(ftext, &encode_runs(runs))
        .map_err(TypedDocumentError::Mutation)
}

fn encode_runs(runs: &[TextEditRunV1]) -> String {
    let mut result = String::new();
    for run in runs {
        for style in run.styles() {
            result.push('<');
            result.push_str(style_tag(*style));
            result.push('>');
        }
        push_escaped(&mut result, run.text());
        for style in run.styles().iter().rev() {
            result.push_str("</");
            result.push_str(style_tag(*style));
            result.push('>');
        }
    }
    result
}

fn push_escaped(result: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(character),
        }
    }
}

const fn style_tag(style: TextEditStyleV1) -> &'static str {
    match style {
        TextEditStyleV1::Bold => "b",
        TextEditStyleV1::Italic => "i",
        TextEditStyleV1::Subscript => "sub",
        TextEditStyleV1::Superscript => "sup",
    }
}

fn set(tree: &mut Xot, node: Node, name: &str, value: impl AsRef<str>) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value.as_ref());
}

fn remove(tree: &mut Xot, node: Node, name: &str) {
    let name = tree.add_name(name);
    tree.remove_attribute(node, name);
}

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
