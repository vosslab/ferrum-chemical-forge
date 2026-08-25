//! Recursive typed-tree recognition and structural projection.

use std::collections::BTreeMap;

use xot::xmlname::NameStrInfo;
use xot::{Node, Value, Xot};

use super::super::typed_schema::typed_attribute_names;
use super::super::{
    CDML_NAMESPACE, ElementPath, TypedDiagnostic, TypedDiagnosticKind, TypedDocumentError,
};
use super::{
    ExpandedName, NamespaceBinding, TypedChild, TypedClass, TypedRecord, TypedText,
    UnknownAttribute, UnrecognizedChild, UnrecognizedNode,
};

struct ProjectedAttributes {
    typed_attributes: BTreeMap<String, String>,
    document_object_id_metadata_v1: Option<String>,
    unknown_attributes: Vec<UnknownAttribute>,
}

pub(super) fn project_record(
    tree: &Xot,
    node: Node,
    class: TypedClass,
    path: Vec<u32>,
) -> Result<TypedRecord, TypedDocumentError> {
    let attributes = project_attributes(tree, node, class)?;
    let mut typed_children = Vec::new();
    let mut typed_text = Vec::new();
    let mut unrecognized_children = Vec::new();
    let mut diagnostics = Vec::new();
    let mut counts = BTreeMap::<TypedClass, u32>::new();
    let mut element_index = 0_u32;

    for (position, child) in tree.children(node).enumerate() {
        let position = u32::try_from(position).expect("XML child count fits u32");
        if let Some((local_name, namespace)) = super::element_name(tree, child) {
            let mut child_path = path.clone();
            child_path.push(element_index);
            element_index += 1;
            let candidate = if namespace == CDML_NAMESPACE {
                child_class(class, &local_name)
            } else {
                None
            };
            if let Some(child_class) = candidate {
                if class == TypedClass::Molecule && child_class == TypedClass::Group {
                    return Err(TypedDocumentError::UnsupportedLegacyGroup);
                }
                let count = counts.entry(child_class).or_default();
                let (_, maximum) = child_cardinality(class, child_class);
                if maximum.is_some_and(|maximum| *count >= maximum) {
                    diagnostics.push(TypedDiagnostic {
                        kind: TypedDiagnosticKind::ExcessChild,
                        child_class,
                        message: format!(
                            "{} has more than {maximum:?} {} child",
                            class.name(),
                            child_class.name()
                        ),
                    });
                    unrecognized_children.push(unrecognized_child(tree, child, position)?);
                } else {
                    let record = project_record(tree, child, child_class, child_path)?;
                    typed_children.push(TypedChild { position, record });
                    *count += 1;
                }
            } else {
                unrecognized_children.push(unrecognized_child(tree, child, position)?);
            }
            continue;
        }

        if let Some(text) = tree.text_str(child) {
            if class_carries_text(class) {
                typed_text.push(TypedText {
                    position,
                    value: text.to_owned(),
                });
            } else {
                unrecognized_children.push(UnrecognizedChild {
                    position,
                    node: UnrecognizedNode::Text(text.to_owned()),
                });
            }
            continue;
        }
        unrecognized_children.push(unrecognized_child(tree, child, position)?);
    }

    for child_class in permitted_children(class) {
        let (minimum, _) = child_cardinality(class, *child_class);
        let actual = counts.get(child_class).copied().unwrap_or(0);
        if actual < minimum {
            diagnostics.push(TypedDiagnostic {
                kind: TypedDiagnosticKind::MissingChild,
                child_class: *child_class,
                message: format!(
                    "{} requires at least {minimum} {} child",
                    class.name(),
                    child_class.name()
                ),
            });
        }
    }

    validate_compact_group_content(class, &typed_children, &unrecognized_children)?;

    Ok(TypedRecord {
        class,
        path: ElementPath(path),
        typed_attributes: attributes.typed_attributes,
        document_object_id_metadata_v1: attributes.document_object_id_metadata_v1,
        unknown_attributes: attributes.unknown_attributes,
        typed_children,
        typed_text,
        unrecognized_children,
        diagnostics,
    })
}

fn validate_compact_group_content(
    class: TypedClass,
    typed_children: &[TypedChild],
    unrecognized_children: &[UnrecognizedChild],
) -> Result<(), TypedDocumentError> {
    if class != TypedClass::CompactGroup {
        return Ok(());
    }
    let has_exactly_one_anchor = matches!(typed_children, [child]
        if child.record().class() == TypedClass::Point);
    let has_only_whitespace = unrecognized_children.iter().all(
        |child| matches!(child.node(), UnrecognizedNode::Text(text) if text.trim().is_empty()),
    );
    if has_exactly_one_anchor && has_only_whitespace {
        return Ok(());
    }
    Err(TypedDocumentError::UndeclaredCompactGroupContent)
}

fn project_attributes(
    tree: &Xot,
    node: Node,
    class: TypedClass,
) -> Result<ProjectedAttributes, TypedDocumentError> {
    let mut typed = BTreeMap::new();
    let mut document_object_id_metadata_v1 = None;
    let mut unknown = Vec::new();
    let context = namespace_context(tree, node);
    for (name, value) in tree.attributes(node).iter() {
        let (local_name, namespace) = tree.name_ns_str(name);
        if namespace.is_empty() && typed_attribute_names(class).contains(&local_name) {
            typed.insert(local_name.to_owned(), value.clone());
            continue;
        }
        if super::super::document_object_identity_v1::is_document_object_attribute_v1(
            namespace, local_name,
        ) {
            document_object_id_metadata_v1 = Some(value.clone());
            continue;
        }
        let reference = tree
            .name_ref(name, node)
            .map_err(TypedDocumentError::AttributeName)?;
        if class == TypedClass::CompactGroup {
            return Err(TypedDocumentError::UndeclaredCompactGroupAttribute {
                attribute: reference.full_name().into_owned(),
            });
        }
        unknown.push(UnknownAttribute {
            qualified_name: reference.full_name().into_owned(),
            expanded_name: ExpandedName {
                namespace: namespace.to_owned(),
                local_name: local_name.to_owned(),
            },
            value: value.clone(),
            namespace_context: context.clone(),
        });
    }
    unknown.sort_by(|left, right| {
        (
            &left.expanded_name.namespace,
            &left.expanded_name.local_name,
            &left.qualified_name,
        )
            .cmp(&(
                &right.expanded_name.namespace,
                &right.expanded_name.local_name,
                &right.qualified_name,
            ))
    });
    Ok(ProjectedAttributes {
        typed_attributes: typed,
        document_object_id_metadata_v1,
        unknown_attributes: unknown,
    })
}

fn namespace_context(tree: &Xot, node: Node) -> Vec<NamespaceBinding> {
    let mut context = tree
        .namespaces_in_scope(node)
        .map(|(prefix, namespace)| NamespaceBinding {
            prefix: tree.prefix_str(prefix).to_owned(),
            namespace: tree.namespace_str(namespace).to_owned(),
        })
        .collect::<Vec<_>>();
    context.sort();
    context
}

fn unrecognized_child(
    tree: &Xot,
    node: Node,
    position: u32,
) -> Result<UnrecognizedChild, TypedDocumentError> {
    let retained = match tree.value(node) {
        Value::Element(element) => {
            let (local_name, namespace) = tree.name_ns_str(element.name());
            UnrecognizedNode::Element {
                name: ExpandedName {
                    namespace: namespace.to_owned(),
                    local_name: local_name.to_owned(),
                },
                xml: tree
                    .to_string(node)
                    .map_err(TypedDocumentError::OpaqueSnapshot)?,
            }
        }
        Value::Text(text) => UnrecognizedNode::Text(text.get().to_owned()),
        Value::Comment(comment) => UnrecognizedNode::Comment(comment.get().to_owned()),
        Value::ProcessingInstruction(instruction) => {
            let (local_name, namespace) = tree.name_ns_str(instruction.target());
            UnrecognizedNode::ProcessingInstruction {
                target: ExpandedName {
                    namespace: namespace.to_owned(),
                    local_name: local_name.to_owned(),
                },
                data: instruction.data().map(str::to_owned),
            }
        }
        Value::Document | Value::Attribute(_) | Value::Namespace(_) => {
            unreachable!("only normal XML children are projected")
        }
    };
    Ok(UnrecognizedChild {
        position,
        node: retained,
    })
}

fn class_carries_text(class: TypedClass) -> bool {
    matches!(
        class,
        TypedClass::AuthorProgram
            | TypedClass::Author
            | TypedClass::Note
            | TypedClass::FragmentName
            | TypedClass::FormattedText
    )
}

fn child_class(parent: TypedClass, local_name: &str) -> Option<TypedClass> {
    permitted_children(parent)
        .iter()
        .copied()
        .find(|class| child_local_name(*class) == local_name)
}

fn child_local_name(class: TypedClass) -> &'static str {
    match class {
        TypedClass::Cdml => "cdml",
        TypedClass::Info => "info",
        TypedClass::AuthorProgram => "author_program",
        TypedClass::Author => "author",
        TypedClass::Note => "note",
        TypedClass::Metadata => "metadata",
        TypedClass::MetadataDocument => "doc",
        TypedClass::Standard => "standard",
        TypedClass::StandardBond | TypedClass::Bond | TypedClass::FragmentBond => "bond",
        TypedClass::StandardArrow | TypedClass::CanvasArrow | TypedClass::ReactionArrow => "arrow",
        TypedClass::StandardAtom | TypedClass::Atom => "atom",
        TypedClass::CompactGroup => "compact-group",
        TypedClass::Paper => "paper",
        TypedClass::Viewport => "viewport",
        TypedClass::Molecule => "molecule",
        TypedClass::CanvasPlus | TypedClass::ReactionPlus => "plus",
        TypedClass::CanvasText | TypedClass::MoleculeText => "text",
        TypedClass::Rectangle => "rect",
        TypedClass::Square => "square",
        TypedClass::Oval => "oval",
        TypedClass::Circle => "circle",
        TypedClass::Polygon => "polygon",
        TypedClass::Polyline => "polyline",
        TypedClass::Reaction => "reaction",
        TypedClass::ReactionReactant => "reactant",
        TypedClass::ReactionProduct => "product",
        TypedClass::ReactionCondition => "condition",
        TypedClass::ExternalData => "external-data",
        TypedClass::Group => "group",
        TypedClass::Query => "query",
        TypedClass::Template => "template",
        TypedClass::Fragment => "fragment",
        TypedClass::DisplayForm => "display-form",
        TypedClass::UserData => "user-data",
        TypedClass::FragmentName => "name",
        TypedClass::FragmentVertex => "vertex",
        TypedClass::FragmentProperty => "property",
        TypedClass::Point => "point",
        TypedClass::Font => "font",
        TypedClass::FormattedText => "ftext",
        TypedClass::Mark => "mark",
    }
}

fn permitted_children(class: TypedClass) -> &'static [TypedClass] {
    use TypedClass as C;
    match class {
        C::Cdml => &[
            C::Info,
            C::Metadata,
            C::Standard,
            C::Paper,
            C::Viewport,
            C::Molecule,
            C::CanvasArrow,
            C::CanvasPlus,
            C::CanvasText,
            C::Rectangle,
            C::Square,
            C::Oval,
            C::Circle,
            C::Polygon,
            C::Polyline,
            C::Reaction,
            C::ExternalData,
        ],
        C::Info => &[C::AuthorProgram, C::Author, C::Note],
        C::Metadata => &[C::MetadataDocument],
        C::Standard => &[C::StandardBond, C::StandardArrow, C::StandardAtom],
        C::Molecule => &[
            C::Template,
            C::Atom,
            C::CompactGroup,
            C::Group,
            C::MoleculeText,
            C::Query,
            C::Bond,
            C::Fragment,
            C::DisplayForm,
            C::UserData,
        ],
        C::CanvasArrow | C::Polygon | C::Polyline => &[C::Point],
        C::CanvasPlus => &[C::Point, C::Font],
        C::CanvasText | C::MoleculeText => &[C::Font, C::Point, C::FormattedText],
        C::Reaction => &[
            C::ReactionReactant,
            C::ReactionProduct,
            C::ReactionArrow,
            C::ReactionCondition,
            C::ReactionPlus,
        ],
        C::Atom => &[C::Point, C::Font, C::FormattedText, C::Mark],
        C::CompactGroup | C::Group | C::Query => &[C::Point],
        C::Fragment => &[
            C::FragmentName,
            C::FragmentBond,
            C::FragmentVertex,
            C::FragmentProperty,
        ],
        _ => &[],
    }
}

fn child_cardinality(parent: TypedClass, child: TypedClass) -> (u32, Option<u32>) {
    use TypedClass as C;
    match (parent, child) {
        (C::CanvasArrow, C::Point) | (C::Polyline, C::Point) => (2, None),
        (C::Polygon, C::Point) => (3, None),
        (C::Atom | C::CompactGroup | C::Group | C::MoleculeText | C::Query, C::Point)
        | (C::CanvasPlus | C::CanvasText, C::Point) => (1, Some(1)),
        (C::Atom | C::MoleculeText | C::CanvasPlus | C::CanvasText, C::Font)
        | (C::Atom | C::MoleculeText | C::CanvasText, C::FormattedText)
        | (C::Fragment, C::FragmentName)
        | (C::Metadata, C::MetadataDocument)
        | (C::Standard, C::StandardBond | C::StandardArrow | C::StandardAtom)
        | (C::Info, C::AuthorProgram | C::Author | C::Note) => (0, Some(1)),
        _ => (0, None),
    }
}
