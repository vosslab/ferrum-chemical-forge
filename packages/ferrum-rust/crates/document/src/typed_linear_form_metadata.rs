//! Narrow recognition and validity checks for backend-generated linear forms.

use std::collections::{HashMap, HashSet};

use xot::{Node, Value, Xot};

use super::{
    CDML_NAMESPACE, TypedClass, TypedDocumentError, TypedRecord, UnrecognizedNode, element_name,
};

const BOND_LENGTH_POINTS: f64 = 10.0;
// `canonical_authored_coordinate` stores centimetres to three decimals, which can
// round a ten-point gap by roughly 0.0142 points.
const AUTHORED_COORDINATE_TOLERANCE_POINTS: f64 = 0.02;
type LinearFormMembers = (Vec<String>, Vec<String>);

pub(crate) fn is_exact_generated_linear_form_record(record: &TypedRecord) -> bool {
    if record.class() != TypedClass::Fragment
        || record.attribute("id").is_none_or(str::is_empty)
        || record.attribute("type") != Some("linear_form")
        || record.typed_attributes().len() != 2
        || !record.unknown_attributes().is_empty()
        || !record.diagnostics().is_empty()
        || !only_whitespace_unrecognized(record)
    {
        return false;
    }
    let children = record.typed_children();
    let Some(last_child) = children.last() else {
        return false;
    };
    if children.len() < 3
        || !exact_fragment_name(children[0].record())
        || !exact_fragment_property(last_child.record())
    {
        return false;
    }
    let mut ids = HashSet::new();
    let mut saw_vertex = false;
    let mut vertex_count = 0_usize;
    for child in &children[1..children.len() - 1] {
        let child = child.record();
        if !child.unknown_attributes().is_empty()
            || !child.diagnostics().is_empty()
            || !child.typed_children().is_empty()
            || !only_whitespace_unrecognized(child)
        {
            return false;
        }
        let Some(identifier) = child.attribute("id").filter(|value| !value.is_empty()) else {
            return false;
        };
        if !ids.insert(identifier) {
            return false;
        }
        match child.class() {
            TypedClass::FragmentBond if !saw_vertex => {}
            TypedClass::FragmentVertex => {
                saw_vertex = true;
                vertex_count += 1;
            }
            _ => return false,
        }
    }
    vertex_count > 0
}

fn exact_fragment_name(record: &TypedRecord) -> bool {
    record.class() == TypedClass::FragmentName
        && record.typed_attributes().is_empty()
        && record.unknown_attributes().is_empty()
        && record.typed_children().is_empty()
        && record.unrecognized_children().is_empty()
        && record.diagnostics().is_empty()
        && record.text_content() == "linear_form"
}

fn exact_fragment_property(record: &TypedRecord) -> bool {
    record.class() == TypedClass::FragmentProperty
        && record.attribute("name") == Some("bond_length")
        && record.attribute("value") == Some("10")
        && record.attribute("type") == Some("IntType")
        && record.typed_attributes().len() == 3
        && record.unknown_attributes().is_empty()
        && record.typed_children().is_empty()
        && record.diagnostics().is_empty()
        && only_whitespace_unrecognized(record)
}

fn only_whitespace_unrecognized(record: &TypedRecord) -> bool {
    record
        .unrecognized_children()
        .iter()
        .all(|child| matches!(child.node(), UnrecognizedNode::Text(text) if text.trim().is_empty()))
}

pub(crate) fn retire_invalid_generated_linear_forms(
    tree: &mut Xot,
    molecule: Node,
) -> Result<(), TypedDocumentError> {
    let atoms = unique_direct_records(tree, molecule, "atom")?;
    let bonds = unique_direct_records(tree, molecule, "bond")?;
    let mut forms = Vec::new();
    forms
        .try_reserve_exact(tree.children(molecule).count())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for node in tree.children(molecule) {
        if let Some(members) = owned_members(tree, node)? {
            forms.push((node, members));
        }
    }
    for (form, (atom_ids, bond_ids)) in forms {
        if !form_is_valid(tree, &atoms, &bonds, &atom_ids, &bond_ids)? {
            tree.remove(form).map_err(TypedDocumentError::Mutation)?;
        }
    }
    Ok(())
}

/// Replace or append the one exact generated record for these ordered members.
///
/// This is the sole writer for Ferrum's deliberately narrow linear-form grammar.
pub(crate) fn write_generated_linear_form(
    tree: &mut Xot,
    molecule: Node,
    fragment_id: &str,
    atom_ids: &[String],
    bond_ids: &[String],
) -> Result<(), TypedDocumentError> {
    let existing = matching_generated_linear_form_node(tree, molecule, atom_ids, bond_ids)?;
    if let Some(existing) = existing {
        tree.remove(existing)
            .map_err(TypedDocumentError::Mutation)?;
    }
    let namespace = element_name(tree, molecule)
        .map(|(_, namespace)| namespace)
        .unwrap_or_default();
    let fragment = new_element(tree, "fragment", &namespace);
    let id = tree.add_name("id");
    let kind = tree.add_name("type");
    tree.set_attribute(fragment, id, fragment_id);
    tree.set_attribute(fragment, kind, "linear_form");
    let name = new_element(tree, "name", &namespace);
    tree.append(fragment, name)
        .map_err(TypedDocumentError::Mutation)?;
    let text = tree.new_text("linear_form");
    tree.append(name, text)
        .map_err(TypedDocumentError::Mutation)?;
    for bond_id in bond_ids {
        append_member(tree, fragment, "bond", bond_id, &namespace)?;
    }
    for atom_id in atom_ids {
        append_member(tree, fragment, "vertex", atom_id, &namespace)?;
    }
    let property = new_element(tree, "property", &namespace);
    let property_name = tree.add_name("name");
    let value = tree.add_name("value");
    tree.set_attribute(property, property_name, "bond_length");
    tree.set_attribute(property, value, "10");
    tree.set_attribute(property, kind, "IntType");
    tree.append(fragment, property)
        .map_err(TypedDocumentError::Mutation)?;
    tree.append(molecule, fragment)
        .map_err(TypedDocumentError::Mutation)
}

/// Return the sole exact generated owner for ordered members, if present.
pub(crate) fn matching_generated_linear_form_id(
    tree: &Xot,
    molecule: Node,
    atom_ids: &[String],
    bond_ids: &[String],
) -> Result<Option<String>, TypedDocumentError> {
    let Some(node) = matching_generated_linear_form_node(tree, molecule, atom_ids, bond_ids)?
    else {
        return Ok(None);
    };
    let identifier = unqualified_attribute(tree, node, "id")
        .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
    let mut result = String::new();
    result
        .try_reserve_exact(identifier.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    result.push_str(identifier);
    Ok(Some(result))
}

fn matching_generated_linear_form_node(
    tree: &Xot,
    molecule: Node,
    atom_ids: &[String],
    bond_ids: &[String],
) -> Result<Option<Node>, TypedDocumentError> {
    let mut matching = None;
    for node in tree.children(molecule) {
        let Some((owned_atoms, owned_bonds)) = owned_members(tree, node)? else {
            continue;
        };
        if members_match(&owned_atoms, atom_ids)
            && members_match(&owned_bonds, bond_ids)
            && matching.replace(node).is_some()
        {
            return Err(TypedDocumentError::AmbiguousLinearFormOwnership);
        }
    }
    Ok(matching)
}

fn members_match(owned: &[String], requested: &[String]) -> bool {
    owned == requested || owned.iter().rev().eq(requested)
}

fn append_member(
    tree: &mut Xot,
    fragment: Node,
    kind: &str,
    identifier: &str,
    namespace: &str,
) -> Result<(), TypedDocumentError> {
    let member = new_element(tree, kind, namespace);
    let id = tree.add_name("id");
    tree.set_attribute(member, id, identifier);
    tree.append(fragment, member)
        .map_err(TypedDocumentError::Mutation)
}

fn new_element(tree: &mut Xot, local_name: &str, namespace: &str) -> Node {
    let name = if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    };
    tree.new_element(name)
}

fn owned_members(
    tree: &Xot,
    fragment: Node,
) -> Result<Option<LinearFormMembers>, TypedDocumentError> {
    if !is_core_element(tree, fragment, "fragment")
        || !exact_attributes(
            tree,
            fragment,
            &[("id", None), ("type", Some("linear_form"))],
        )
        || !whitespace_children(tree, fragment)
        || unqualified_attribute(tree, fragment, "id").is_none_or(str::is_empty)
    {
        return Ok(None);
    }
    let mut children = Vec::new();
    children
        .try_reserve_exact(tree.children(fragment).count())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    children.extend(
        tree.children(fragment)
            .filter(|node| tree.is_element(*node)),
    );
    if children.len() < 3
        || !is_exact_name(tree, children[0])
        || !children
            .last()
            .is_some_and(|child| is_exact_property(tree, *child))
    {
        return Ok(None);
    }
    let mut bond_ids = Vec::new();
    let mut atom_ids = Vec::new();
    bond_ids
        .try_reserve_exact(children.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    atom_ids
        .try_reserve_exact(children.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    let mut saw_vertex = false;
    for child in &children[1..children.len() - 1] {
        if !exact_attributes(tree, *child, &[("id", None)])
            || !whitespace_children(tree, *child)
            || tree.children(*child).any(|node| tree.is_element(node))
        {
            return Ok(None);
        }
        let Some(id) = unqualified_attribute(tree, *child, "id") else {
            return Ok(None);
        };
        if id.is_empty() {
            return Ok(None);
        }
        if is_core_element(tree, *child, "bond") && !saw_vertex {
            bond_ids.push(copy_string(id)?);
        } else if is_core_element(tree, *child, "vertex") {
            saw_vertex = true;
            atom_ids.push(copy_string(id)?);
        } else {
            return Ok(None);
        }
    }
    let unique_atoms = atom_ids
        .iter()
        .enumerate()
        .all(|(index, id)| !atom_ids[..index].contains(id));
    let unique_bonds = bond_ids
        .iter()
        .enumerate()
        .all(|(index, id)| !bond_ids[..index].contains(id));
    Ok((!atom_ids.is_empty() && unique_atoms && unique_bonds).then_some((atom_ids, bond_ids)))
}

fn copy_string(value: &str) -> Result<String, TypedDocumentError> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    result.push_str(value);
    Ok(result)
}

fn is_exact_name(tree: &Xot, node: Node) -> bool {
    is_core_element(tree, node, "name")
        && exact_attributes(tree, node, &[])
        && !tree.children(node).any(|child| tree.is_element(child))
        && exact_text(tree, node, "linear_form")
}

fn is_exact_property(tree: &Xot, node: Node) -> bool {
    is_core_element(tree, node, "property")
        && exact_attributes(
            tree,
            node,
            &[
                ("name", Some("bond_length")),
                ("value", Some("10")),
                ("type", Some("IntType")),
            ],
        )
        && !tree.children(node).any(|child| tree.is_element(child))
        && whitespace_children(tree, node)
}

fn form_is_valid(
    tree: &Xot,
    atoms: &HashMap<String, Node>,
    bonds: &HashMap<String, Node>,
    atom_ids: &[String],
    bond_ids: &[String],
) -> Result<bool, TypedDocumentError> {
    if bond_ids.len() != atom_ids.len().saturating_sub(1)
        || atom_ids.iter().any(|id| !atoms.contains_key(id))
        || bond_ids.iter().any(|id| !bonds.contains_key(id))
    {
        return Ok(false);
    }
    let mut selected = HashSet::new();
    selected
        .try_reserve(atom_ids.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    selected.extend(atom_ids.iter().map(String::as_str));
    let mut induced = HashSet::new();
    induced
        .try_reserve(bonds.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for (id, bond) in bonds {
        let Some(start) = unqualified_attribute(tree, *bond, "start") else {
            continue;
        };
        let Some(end) = unqualified_attribute(tree, *bond, "end") else {
            continue;
        };
        if selected.contains(start) && selected.contains(end) {
            induced.insert(id.as_str());
        }
    }
    if induced.len() != bond_ids.len() || bond_ids.iter().any(|id| !induced.contains(id.as_str())) {
        return Ok(false);
    }
    for (index, bond_id) in bond_ids.iter().enumerate() {
        let bond = bonds[bond_id];
        let Some(start) = unqualified_attribute(tree, bond, "start") else {
            return Ok(false);
        };
        let Some(end) = unqualified_attribute(tree, bond, "end") else {
            return Ok(false);
        };
        if !((start == atom_ids[index] && end == atom_ids[index + 1])
            || (end == atom_ids[index] && start == atom_ids[index + 1]))
        {
            return Ok(false);
        }
    }
    let Some((first_x, first_y)) = atom_ids.first().and_then(|id| atom_point(tree, atoms[id]))
    else {
        return Ok(false);
    };
    Ok(atom_ids.iter().enumerate().all(|(index, id)| {
        atom_point(tree, atoms[id]).is_some_and(|(x, y)| {
            (y - first_y).abs() <= AUTHORED_COORDINATE_TOLERANCE_POINTS
                && (x - (first_x + index as f64 * BOND_LENGTH_POINTS)).abs()
                    <= AUTHORED_COORDINATE_TOLERANCE_POINTS
        })
    }))
}

fn atom_point(tree: &Xot, atom: Node) -> Option<(f64, f64)> {
    let mut points = tree
        .children(atom)
        .filter(|child| is_core_element(tree, *child, "point"));
    let point = points.next()?;
    points.next().is_none().then_some(())?;
    let x =
        super::typed_coordinate::parse_coordinate(unqualified_attribute(tree, point, "x")?).ok()?;
    let y =
        super::typed_coordinate::parse_coordinate(unqualified_attribute(tree, point, "y")?).ok()?;
    Some((x, y))
}

fn unique_direct_records(
    tree: &Xot,
    parent: Node,
    expected: &str,
) -> Result<HashMap<String, Node>, TypedDocumentError> {
    let mut records = HashMap::new();
    let mut ambiguous = HashSet::new();
    let child_count = tree.children(parent).count();
    records
        .try_reserve(child_count)
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    ambiguous
        .try_reserve(child_count)
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for child in tree
        .children(parent)
        .filter(|child| is_core_element(tree, *child, expected))
    {
        let Some(id) = unqualified_attribute(tree, child, "id") else {
            continue;
        };
        if id.is_empty() || ambiguous.contains(id) {
            continue;
        }
        if records.remove(id).is_some() {
            ambiguous.insert(copy_string(id)?);
        } else {
            records.insert(copy_string(id)?, child);
        }
    }
    Ok(records)
}

fn exact_attributes(tree: &Xot, node: Node, expected: &[(&str, Option<&str>)]) -> bool {
    if tree.namespaces(node).iter().next().is_some()
        || tree.attributes(node).len() != expected.len()
    {
        return false;
    }
    expected.iter().all(|(name, value)| {
        unqualified_attribute(tree, node, name)
            .is_some_and(|actual| value.is_none_or(|expected| actual == expected))
    })
}

fn unqualified_attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local_name, namespace) = tree.name_ns_str(name);
        (local_name == expected && namespace.is_empty()).then_some(value.as_str())
    })
}

fn exact_text(tree: &Xot, node: Node, expected: &str) -> bool {
    let mut remaining = expected;
    for child in tree.children(node) {
        let Value::Text(value) = tree.value(child) else {
            return false;
        };
        let value = value.get();
        let Some(next) = remaining.strip_prefix(value) else {
            return false;
        };
        remaining = next;
    }
    remaining.is_empty()
}

fn whitespace_children(tree: &Xot, node: Node) -> bool {
    tree.children(node).all(|child| match tree.value(child) {
        Value::Element(_) => true,
        Value::Text(value) => value.get().trim().is_empty(),
        _ => false,
    })
}

fn is_core_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(name, namespace)| name == expected && (namespace == CDML_NAMESPACE))
}
