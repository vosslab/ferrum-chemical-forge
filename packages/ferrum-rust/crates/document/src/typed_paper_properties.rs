//! Detached mutation of the first direct core paper record.

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PaperOrientationV1, PaperPropertiesPatchV1, PaperPropertyChangeV1, TypedClass,
    TypedDocument, TypedDocumentError, TypedRecord, element_name, paper_size_v1,
};

impl TypedDocument {
    pub(crate) fn paper_defaults_v1(&self) -> (String, PaperOrientationV1) {
        let standard = first_direct_record(self, TypedClass::Standard);
        let type_name = standard
            .and_then(|record| record.attribute("paper_type"))
            .filter(|value| paper_size_v1(value).is_some_and(|paper| paper.name() != "custom"))
            .unwrap_or("A4")
            .to_owned();
        let orientation = standard
            .and_then(|record| record.attribute("paper_orientation"))
            .and_then(PaperOrientationV1::parse)
            .unwrap_or(PaperOrientationV1::Portrait);
        (type_name, orientation)
    }

    pub(crate) fn paper_type_or_default_v1(&self) -> String {
        first_direct_record(self, TypedClass::Paper)
            .and_then(|record| record.attribute("type"))
            .map(str::to_owned)
            .unwrap_or_else(|| self.paper_defaults_v1().0)
    }

    pub(crate) fn with_paper_properties(
        &self,
        patch: &PaperPropertiesPatchV1,
    ) -> Result<Self, TypedDocumentError> {
        let (default_type, default_orientation) = self.paper_defaults_v1();
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let paper = if let Some(paper) = direct_child(&indexed.xml.tree, root, "paper") {
            paper
        } else {
            let namespace = element_name(&indexed.xml.tree, root)
                .expect("a typed CDML root is an element")
                .1;
            let name = element_name_id(&mut indexed.xml.tree, "paper", &namespace);
            let paper = indexed.xml.tree.new_element(name);
            if let Some(viewport) = direct_child(&indexed.xml.tree, root, "viewport") {
                indexed
                    .xml
                    .tree
                    .insert_before(viewport, paper)
                    .map_err(TypedDocumentError::Mutation)?;
            } else {
                indexed
                    .xml
                    .tree
                    .append(root, paper)
                    .map_err(TypedDocumentError::Mutation)?;
            }
            set(&mut indexed.xml.tree, paper, "type", &default_type);
            set(
                &mut indexed.xml.tree,
                paper,
                "orientation",
                default_orientation.as_str(),
            );
            paper
        };
        apply_changes(&mut indexed.xml.tree, paper, patch.changes());
        Self::parse(&candidate.to_xml()?)
    }
}

fn apply_changes(tree: &mut Xot, paper: Node, changes: &[PaperPropertyChangeV1]) {
    for change in changes {
        match change {
            PaperPropertyChangeV1::Type(value) => {
                set(tree, paper, "type", value);
                if value != "custom" {
                    remove(tree, paper, "size_x");
                    remove(tree, paper, "size_y");
                }
            }
            PaperPropertyChangeV1::Orientation(value) => {
                set(tree, paper, "orientation", value.as_str())
            }
            PaperPropertyChangeV1::CropSvg(value) => set_bool(tree, paper, "crop_svg", *value),
            PaperPropertyChangeV1::CropMargin(value) => {
                set(tree, paper, "crop_margin", &value.to_string())
            }
            PaperPropertyChangeV1::UseRealMinus(value) => {
                set_bool(tree, paper, "use_real_minus", *value)
            }
            PaperPropertyChangeV1::ReplaceMinus(value) => {
                set_bool(tree, paper, "replace_minus", *value)
            }
            PaperPropertyChangeV1::Dimensions(value) => {
                set(tree, paper, "size_x", &value.width().to_string());
                set(tree, paper, "size_y", &value.height().to_string());
            }
        }
    }
}

fn first_direct_record(document: &TypedDocument, class: TypedClass) -> Option<&TypedRecord> {
    document
        .root()
        .typed_children()
        .iter()
        .map(|child| child.record())
        .find(|record| record.class() == class)
}

fn direct_child(tree: &Xot, root: Node, expected: &str) -> Option<Node> {
    tree.children(root)
        .find(|node| is_cdml_element(tree, *node, expected))
}

fn set_bool(tree: &mut Xot, node: Node, name: &str, value: bool) {
    set(tree, node, name, if value { "1" } else { "0" });
}

fn set(tree: &mut Xot, node: Node, name: &str, value: &str) {
    let name = tree.add_name(name);
    tree.set_attribute(node, name, value);
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
        local_name == expected && (namespace == CDML_NAMESPACE)
    })
}
