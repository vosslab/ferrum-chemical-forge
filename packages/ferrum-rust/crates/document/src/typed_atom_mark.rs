//! Direct atom-mark mutation over one detached retained CDML candidate.

use std::f64::consts::PI;

use xot::{Node, Xot};

use super::{
    AtomMarkActionV1, AtomMarkKindV1, CDML_NAMESPACE, PersistentId, TypedDocument,
    TypedDocumentError, element_name,
};

const CENTIMETRES_PER_POINT: f64 = 2.54 / 72.0;

impl TypedDocument {
    /// Return a detached candidate with one supported direct atom mark added or removed.
    pub(crate) fn with_atom_mark(
        &self,
        molecule_id: &PersistentId,
        atom_id: &PersistentId,
        action: AtomMarkActionV1,
        kind: AtomMarkKindV1,
        matching_mark_index: Option<u32>,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let Some(atom) = direct_atom(
            &mut indexed.xml.tree,
            indexed.xml.document,
            molecule_id.as_str(),
            atom_id.as_str(),
        ) else {
            return Ok(None);
        };
        let matching = matching_marks(&mut indexed.xml.tree, atom, kind);
        let selected = match matching_mark_index {
            Some(index) => Some(*matching.get(index as usize).ok_or_else(|| {
                TypedDocumentError::AtomMarkIndexOutOfRange {
                    atom: atom_id.clone(),
                    kind,
                    index,
                }
            })?),
            None => matching.first().copied(),
        };
        if action == AtomMarkActionV1::Remove && selected.is_none() {
            return Ok(Some(candidate));
        }
        match action {
            AtomMarkActionV1::Add => append_mark(&mut indexed.xml.tree, atom, atom_id, kind)?,
            AtomMarkActionV1::Remove => indexed
                .xml
                .tree
                .remove(selected.expect("a removal without a match returned above"))
                .map_err(TypedDocumentError::Mutation)?,
        }
        apply_scalar_delta(&mut indexed.xml.tree, atom, atom_id, kind, action)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn append_mark(
    tree: &mut Xot,
    atom: Node,
    atom_id: &PersistentId,
    kind: AtomMarkKindV1,
) -> Result<(), TypedDocumentError> {
    let (atom_x, atom_y) = direct_atom_position(tree, atom, atom_id)?;
    let angle = authored_angle(kind);
    let offset = angle.map_or(0.0, |_| 12.0 * CENTIMETRES_PER_POINT);
    let radians = angle.unwrap_or(0.0) * PI / 180.0;
    let x = atom_x + offset * radians.cos();
    let y = atom_y + offset * radians.sin();
    if !x.is_finite() || !y.is_finite() {
        return Err(TypedDocumentError::InvalidAtomMarkPoint(atom_id.clone()));
    }
    let namespace = element_name(tree, atom)
        .expect("a typed atom is an element")
        .1;
    let mark_name = element_name_id(tree, "mark", &namespace);
    let mark = tree.new_element(mark_name);
    set(tree, mark, "type", kind.as_str());
    set(tree, mark, "x", canonical_centimetres(x));
    set(tree, mark, "y", canonical_centimetres(y));
    set(tree, mark, "auto", "0");
    set(tree, mark, "size", authored_size(kind));
    match kind {
        AtomMarkKindV1::Plus | AtomMarkKindV1::Minus => {
            set(tree, mark, "draw_circle", "yes");
        }
        AtomMarkKindV1::Electronpair => set(tree, mark, "line_width", "2"),
        AtomMarkKindV1::Radical
        | AtomMarkKindV1::Biradical
        | AtomMarkKindV1::DottedElectronpair
        | AtomMarkKindV1::PzOrbital => {}
    }
    tree.append(atom, mark)
        .map_err(TypedDocumentError::Mutation)
}

fn direct_atom_position(
    tree: &mut Xot,
    atom: Node,
    atom_id: &PersistentId,
) -> Result<(f64, f64), TypedDocumentError> {
    let points = tree
        .children(atom)
        .filter(|node| is_cdml_element(tree, *node, "point"))
        .collect::<Vec<_>>();
    let [point] = points.as_slice() else {
        return Err(TypedDocumentError::InvalidAtomMarkPoint(atom_id.clone()));
    };
    if tree
        .children(*point)
        .any(|node| element_name(tree, node).is_some())
    {
        return Err(TypedDocumentError::InvalidAtomMarkPoint(atom_id.clone()));
    }
    let x = source_centimetres(tree, *point, "x")
        .ok_or_else(|| TypedDocumentError::InvalidAtomMarkPoint(atom_id.clone()))?;
    let y = source_centimetres(tree, *point, "y")
        .ok_or_else(|| TypedDocumentError::InvalidAtomMarkPoint(atom_id.clone()))?;
    let z_name = tree.add_name("z");
    if tree.get_attribute(*point, z_name).is_some()
        && source_centimetres(tree, *point, "z").is_none()
    {
        return Err(TypedDocumentError::InvalidAtomMarkPoint(atom_id.clone()));
    }
    Ok((x, y))
}

fn source_centimetres(tree: &mut Xot, node: Node, field: &str) -> Option<f64> {
    let name = tree.add_name(field);
    let value = tree.get_attribute(node, name)?;
    let (raw, scale) = value
        .strip_suffix("cm")
        .map_or((value, CENTIMETRES_PER_POINT), |raw| (raw, 1.0));
    let value = raw.parse::<f64>().ok()? * scale;
    value.is_finite().then_some(value)
}

fn apply_scalar_delta(
    tree: &mut Xot,
    atom: Node,
    atom_id: &PersistentId,
    kind: AtomMarkKindV1,
    action: AtomMarkActionV1,
) -> Result<(), TypedDocumentError> {
    let Some((field, delta, default, minimum, maximum)) = scalar_delta(kind) else {
        return Ok(());
    };
    let name = tree.add_name(field);
    let current = match tree.get_attribute(atom, name) {
        Some(source) => source
            .parse::<i32>()
            .ok()
            .filter(|value| value.to_string() == source),
        None => Some(default),
    }
    .ok_or_else(|| TypedDocumentError::InvalidAtomMarkScalar {
        atom: atom_id.clone(),
        field,
    })?;
    if !(minimum..=maximum).contains(&current) {
        return Err(TypedDocumentError::AtomMarkScalarOutOfRange {
            atom: atom_id.clone(),
            field,
            value: current,
        });
    }
    let signed_delta = if action == AtomMarkActionV1::Add {
        delta
    } else {
        -delta
    };
    let result = current + signed_delta;
    if !(minimum..=maximum).contains(&result) {
        return Err(TypedDocumentError::AtomMarkScalarOutOfRange {
            atom: atom_id.clone(),
            field,
            value: result,
        });
    }
    if result == default {
        tree.remove_attribute(atom, name);
    } else {
        tree.set_attribute(atom, name, result.to_string());
    }
    Ok(())
}

fn scalar_delta(kind: AtomMarkKindV1) -> Option<(&'static str, i32, i32, i32, i32)> {
    match kind {
        AtomMarkKindV1::Plus => Some(("charge", 1, 0, -9, 9)),
        AtomMarkKindV1::Minus => Some(("charge", -1, 0, -9, 9)),
        AtomMarkKindV1::Radical => Some(("multiplicity", 1, 1, 1, 3)),
        AtomMarkKindV1::Biradical => Some(("multiplicity", 2, 1, 1, 3)),
        AtomMarkKindV1::Electronpair
        | AtomMarkKindV1::DottedElectronpair
        | AtomMarkKindV1::PzOrbital => None,
    }
}

fn authored_angle(kind: AtomMarkKindV1) -> Option<f64> {
    match kind {
        AtomMarkKindV1::Plus | AtomMarkKindV1::Minus => Some(45.0),
        AtomMarkKindV1::Radical | AtomMarkKindV1::Biradical => Some(90.0),
        AtomMarkKindV1::Electronpair | AtomMarkKindV1::DottedElectronpair => Some(180.0),
        AtomMarkKindV1::PzOrbital => None,
    }
}

fn authored_size(kind: AtomMarkKindV1) -> &'static str {
    match kind {
        AtomMarkKindV1::Radical
        | AtomMarkKindV1::Biradical
        | AtomMarkKindV1::DottedElectronpair => "4",
        AtomMarkKindV1::PzOrbital => "40",
        AtomMarkKindV1::Plus | AtomMarkKindV1::Minus | AtomMarkKindV1::Electronpair => "10",
    }
}

fn canonical_centimetres(value: f64) -> String {
    let rounded = if value.abs() < 0.0005 { 0.0 } else { value };
    format!("{rounded:.3}cm")
}

fn matching_marks(tree: &mut Xot, atom: Node, kind: AtomMarkKindV1) -> Vec<Node> {
    let type_name = tree.add_name("type");
    tree.children(atom)
        .filter(|node| {
            is_cdml_element(tree, *node, "mark")
                && tree.get_attribute(*node, type_name) == Some(kind.as_str())
        })
        .collect()
}

fn direct_atom(tree: &mut Xot, document: Node, molecule_id: &str, atom_id: &str) -> Option<Node> {
    let id_name = tree.add_name("id");
    let root = tree
        .document_element(document)
        .expect("a parsed CDML document has a document element");
    let molecule = tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "molecule")
            && tree.get_attribute(*node, id_name) == Some(molecule_id)
    })?;
    tree.children(molecule).find(|node| {
        is_cdml_element(tree, *node, "atom") && tree.get_attribute(*node, id_name) == Some(atom_id)
    })
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
