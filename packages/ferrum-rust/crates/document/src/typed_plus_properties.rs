//! Structured direct-root Plus property mutation preserving retained XML.

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PersistentId, PlusPropertiesPatchV1, PlusPropertyChangeV1, TypedDocument,
    TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return a detached candidate with one complete Plus-properties patch applied.
    pub(crate) fn with_plus_properties(
        &self,
        patch: &PlusPropertiesPatchV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let plus = direct_plus(
            &mut indexed.xml.tree,
            indexed.xml.document,
            patch.plus_id().as_str(),
        );
        let Some(plus) = plus else {
            return Ok(None);
        };
        let point = editable_point(&indexed.xml.tree, plus, patch.plus_id())?;
        let font_change = patch
            .changes()
            .iter()
            .any(|change| matches!(change, PlusPropertyChangeV1::FontFace(_)));
        let font = editable_font(
            &mut indexed.xml.tree,
            plus,
            point,
            patch.plus_id(),
            font_change,
        )?;
        apply_changes(&mut indexed.xml.tree, plus, font, patch.changes());
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_plus(tree: &mut Xot, document: Node, identifier: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "plus")
            && tree.get_attribute(*node, id_name) == Some(identifier)
    })
}

fn editable_point(
    tree: &Xot,
    plus: Node,
    plus_id: &PersistentId,
) -> Result<Node, TypedDocumentError> {
    let points = tree
        .children(plus)
        .filter(|node| is_cdml_element(tree, *node, "point"))
        .collect::<Vec<_>>();
    match points.as_slice() {
        [point] => Ok(*point),
        _ => Err(TypedDocumentError::InvalidPlusStructure(plus_id.clone())),
    }
}

fn editable_font(
    tree: &mut Xot,
    plus: Node,
    point: Node,
    plus_id: &PersistentId,
    create: bool,
) -> Result<Option<Node>, TypedDocumentError> {
    let fonts = tree
        .children(plus)
        .filter(|node| is_cdml_element(tree, *node, "font"))
        .collect::<Vec<_>>();
    match fonts.as_slice() {
        [font] => Ok(Some(*font)),
        [] if !create => Ok(None),
        [] => {
            let namespace = element_name(tree, plus)
                .expect("a typed Plus is an element")
                .1;
            let name = element_name_id(tree, "font", &namespace);
            let font = tree.new_element(name);
            tree.insert_after(point, font)
                .map_err(TypedDocumentError::Mutation)?;
            Ok(Some(font))
        }
        _ => Err(TypedDocumentError::AmbiguousPlusFonts(plus_id.clone())),
    }
}

fn apply_changes(tree: &mut Xot, plus: Node, font: Option<Node>, changes: &[PlusPropertyChangeV1]) {
    for change in changes {
        match change {
            PlusPropertyChangeV1::FontFace(value) => set(
                tree,
                font.expect("family edits resolve one direct font"),
                "family",
                value.cdml_family(),
            ),
            PlusPropertyChangeV1::FontSize(value) => {
                set(tree, plus, "font_size", value.to_string())
            }
            PlusPropertyChangeV1::Color(value) => set(tree, plus, "color", value.as_str()),
            PlusPropertyChangeV1::BackgroundColor(Some(value)) => {
                set(tree, plus, "background-color", value.as_str())
            }
            PlusPropertyChangeV1::BackgroundColor(None) => set(tree, plus, "background-color", ""),
        }
    }
}

fn set(tree: &mut Xot, node: Node, name: &str, value: impl AsRef<str>) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value.as_ref());
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
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
