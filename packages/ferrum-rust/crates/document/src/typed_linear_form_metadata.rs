//! Narrow recognition and validity checks for backend-generated linear forms.

use std::collections::{HashMap, HashSet};

use xot::{Node, Value, Xot};

use super::{CDML_NAMESPACE, TypedDocumentError, element_name};

const BOND_LENGTH_POINTS: f64 = 10.0;

pub(crate) fn retire_invalid_generated_linear_forms(
    tree: &mut Xot,
    molecule: Node,
) -> Result<(), TypedDocumentError> {
    let atoms = unique_direct_records(tree, molecule, "atom");
    let bonds = unique_direct_records(tree, molecule, "bond");
    let forms = tree
        .children(molecule)
        .filter_map(|node| owned_members(tree, node).map(|members| (node, members)))
        .collect::<Vec<_>>();
    for (form, (atom_ids, bond_ids)) in forms {
        if !form_is_valid(tree, &atoms, &bonds, &atom_ids, &bond_ids) {
            tree.remove(form).map_err(TypedDocumentError::Mutation)?;
        }
    }
    Ok(())
}

fn owned_members(tree: &Xot, fragment: Node) -> Option<(Vec<String>, Vec<String>)> {
    if !is_core_element(tree, fragment, "fragment")
        || !exact_attributes(
            tree,
            fragment,
            &[("id", None), ("type", Some("linear_form"))],
        )
        || !whitespace_children(tree, fragment)
        || unqualified_attribute(tree, fragment, "id")?.is_empty()
    {
        return None;
    }
    let children = element_children(tree, fragment);
    if children.len() < 3
        || !is_exact_name(tree, children[0])
        || !is_exact_property(tree, *children.last()?)
    {
        return None;
    }
    let mut bond_ids = Vec::new();
    let mut atom_ids = Vec::new();
    let mut saw_vertex = false;
    for child in &children[1..children.len() - 1] {
        if !exact_attributes(tree, *child, &[("id", None)])
            || !whitespace_children(tree, *child)
            || !element_children(tree, *child).is_empty()
        {
            return None;
        }
        let id = unqualified_attribute(tree, *child, "id")?;
        if id.is_empty() {
            return None;
        }
        if is_core_element(tree, *child, "bond") && !saw_vertex {
            bond_ids.push(id.to_owned());
        } else if is_core_element(tree, *child, "vertex") {
            saw_vertex = true;
            atom_ids.push(id.to_owned());
        } else {
            return None;
        }
    }
    let unique_atoms = atom_ids.iter().collect::<HashSet<_>>().len() == atom_ids.len();
    let unique_bonds = bond_ids.iter().collect::<HashSet<_>>().len() == bond_ids.len();
    (!atom_ids.is_empty() && unique_atoms && unique_bonds).then_some((atom_ids, bond_ids))
}

fn is_exact_name(tree: &Xot, node: Node) -> bool {
    is_core_element(tree, node, "name")
        && exact_attributes(tree, node, &[])
        && element_children(tree, node).is_empty()
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
        && element_children(tree, node).is_empty()
        && whitespace_children(tree, node)
}

fn form_is_valid(
    tree: &Xot,
    atoms: &HashMap<String, Node>,
    bonds: &HashMap<String, Node>,
    atom_ids: &[String],
    bond_ids: &[String],
) -> bool {
    if bond_ids.len() != atom_ids.len().saturating_sub(1)
        || atom_ids.iter().any(|id| !atoms.contains_key(id))
        || bond_ids.iter().any(|id| !bonds.contains_key(id))
    {
        return false;
    }
    let selected = atom_ids.iter().map(String::as_str).collect::<HashSet<_>>();
    let induced = bonds
        .iter()
        .filter_map(|(id, bond)| {
            let start = unqualified_attribute(tree, *bond, "start")?;
            let end = unqualified_attribute(tree, *bond, "end")?;
            (selected.contains(start) && selected.contains(end)).then_some(id.as_str())
        })
        .collect::<HashSet<_>>();
    if induced != bond_ids.iter().map(String::as_str).collect() {
        return false;
    }
    for (index, bond_id) in bond_ids.iter().enumerate() {
        let bond = bonds[bond_id];
        let Some(start) = unqualified_attribute(tree, bond, "start") else {
            return false;
        };
        let Some(end) = unqualified_attribute(tree, bond, "end") else {
            return false;
        };
        if !((start == atom_ids[index] && end == atom_ids[index + 1])
            || (end == atom_ids[index] && start == atom_ids[index + 1]))
        {
            return false;
        }
    }
    let Some(points) = atom_ids
        .iter()
        .map(|id| atom_point(tree, atoms[id]))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let (first_x, first_y) = points[0];
    points.iter().enumerate().all(|(index, (x, y))| {
        (y - first_y).abs() < 1e-9
            && (x - (first_x + index as f64 * BOND_LENGTH_POINTS)).abs() < 1e-9
    })
}

fn atom_point(tree: &Xot, atom: Node) -> Option<(f64, f64)> {
    let points = tree
        .children(atom)
        .filter(|child| is_core_element(tree, *child, "point"))
        .collect::<Vec<_>>();
    if points.len() != 1 {
        return None;
    }
    let x = super::typed_coordinate::parse_coordinate(unqualified_attribute(tree, points[0], "x")?)
        .ok()?;
    let y = super::typed_coordinate::parse_coordinate(unqualified_attribute(tree, points[0], "y")?)
        .ok()?;
    Some((x, y))
}

fn unique_direct_records(tree: &Xot, parent: Node, expected: &str) -> HashMap<String, Node> {
    let mut records = HashMap::new();
    let mut ambiguous = HashSet::new();
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
            ambiguous.insert(id.to_owned());
        } else {
            records.insert(id.to_owned(), child);
        }
    }
    records
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
    let mut text = String::new();
    for child in tree.children(node) {
        let Value::Text(value) = tree.value(child) else {
            return false;
        };
        text.push_str(value.get());
    }
    text == expected
}

fn whitespace_children(tree: &Xot, node: Node) -> bool {
    tree.children(node).all(|child| match tree.value(child) {
        Value::Element(_) => true,
        Value::Text(value) => value.get().trim().is_empty(),
        _ => false,
    })
}

fn element_children(tree: &Xot, node: Node) -> Vec<Node> {
    tree.children(node)
        .filter(|child| tree.is_element(*child))
        .collect()
}

fn is_core_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(name, namespace)| {
        name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
