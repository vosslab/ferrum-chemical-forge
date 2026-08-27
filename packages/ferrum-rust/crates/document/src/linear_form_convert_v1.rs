//! Typed-document extraction and detached candidate construction for linear forms.
use std::collections::HashMap;

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_domain::linear_form::{
    LinearFormAtomV1, LinearFormBondV1, LinearFormGraphV1, LinearFormRequestV1, plan_linear_form_v1,
};
use ferrum_geometry::Point2;
use thiserror::Error;
use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, DocumentObjectIdV1, PersistentId, TypedClass, TypedDocument,
    TypedDocumentError, element_name,
};

/// Result of a fully authenticated, detached linear-form conversion.
#[derive(Debug)]
pub(crate) enum LinearFormCandidateV1 {
    /// The canonical document was already present.
    NoChange,
    /// A matching generated record was repaired without allocating an identity.
    Repair {
        candidate: Box<TypedDocument>,
        fragment_id: PersistentId,
    },
    /// A caller-owned session must supply this already-reserved fragment identity.
    NeedFragmentId,
}

/// Typed failures at the document/domain boundary.
#[derive(Debug, Error)]
pub(crate) enum LinearFormDocumentErrorV1 {
    /// The pure graph planner declined the durable graph facts.
    #[error("linear-form planning refused: {0}")]
    Plan(#[from] ferrum_domain::linear_form::LinearFormPlanErrorV1),
    /// Typed retained-CDML extraction or mutation failed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
}

impl TypedDocument {
    /// Authenticate, extract, and classify a conversion before a session allocates an ID.
    pub(crate) fn prepare_linear_form_convert_v1(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
        selected_atoms: &[PersistentId],
    ) -> Result<LinearFormCandidateV1, LinearFormDocumentErrorV1> {
        let molecule_id = direct_molecule_id(self, molecule_object_id)?;
        let extracted = extract(self, &molecule_id, selected_atoms)?;
        let plan = plan_linear_form_v1(&extracted.request)?;
        let (atoms, bonds) = source_members(&extracted.ids, &plan)?;
        let molecule = direct_molecule(
            &self.indexed().xml.tree,
            self.indexed().xml.document,
            &molecule_id,
        )
        .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
        let existing = super::typed_linear_form_metadata::matching_generated_linear_form_id(
            &self.indexed().xml.tree,
            molecule,
            &atoms,
            &bonds,
        )?;
        let Some(existing) = existing else {
            return Ok(LinearFormCandidateV1::NeedFragmentId);
        };
        let show_hydrogens = self.indexed().xml.tree.name("show_hydrogens");
        let all_atoms_show_hydrogens = show_hydrogens.is_some_and(|name| {
            atoms.iter().all(|id| {
                let id = PersistentId::new(id.clone())
                    .expect("linear-form source members retain validated persistent IDs");
                direct_atom(&self.indexed().xml.tree, molecule, &id).is_some_and(|atom| {
                    self.indexed().xml.tree.get_attribute(atom, name) == Some("on")
                })
            })
        });
        if all_atoms_show_hydrogens
            && super::typed_linear_form_metadata::matching_generated_linear_form_is_valid(
                &self.indexed().xml.tree,
                molecule,
                &atoms,
                &bonds,
            )?
        {
            return Ok(LinearFormCandidateV1::NoChange);
        }
        let fragment_id = PersistentId::new(existing)
            .map_err(|error| TypedDocumentError::Indexed(error.into()))?;
        let candidate = apply(self, &molecule_id, &extracted.ids, &plan, &fragment_id)?;
        if candidate.to_xml().map_err(TypedDocumentError::Serialize)?
            == self.to_xml().map_err(TypedDocumentError::Serialize)?
        {
            Ok(LinearFormCandidateV1::NoChange)
        } else {
            Ok(LinearFormCandidateV1::Repair {
                candidate: Box::new(candidate),
                fragment_id,
            })
        }
    }

    /// Apply a session-supplied collision-checked fragment ID after prior classification.
    pub(crate) fn apply_linear_form_convert_v1(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
        selected_atoms: &[PersistentId],
        fragment_id: &PersistentId,
    ) -> Result<LinearFormCandidateV1, LinearFormDocumentErrorV1> {
        if self.indexed().resolve_id(fragment_id).is_some() {
            return Err(
                TypedDocumentError::DuplicateLinearFormFragment(fragment_id.clone()).into(),
            );
        }
        let molecule_id = direct_molecule_id(self, molecule_object_id)?;
        let extracted = extract(self, &molecule_id, selected_atoms)?;
        let plan = plan_linear_form_v1(&extracted.request)?;
        let candidate = apply(self, &molecule_id, &extracted.ids, &plan, fragment_id)?;
        if candidate.to_xml().map_err(TypedDocumentError::Serialize)?
            == self.to_xml().map_err(TypedDocumentError::Serialize)?
        {
            Ok(LinearFormCandidateV1::NoChange)
        } else {
            Ok(LinearFormCandidateV1::Repair {
                candidate: Box::new(candidate),
                fragment_id: fragment_id.clone(),
            })
        }
    }
}

struct Extracted {
    request: LinearFormRequestV1,
    ids: HashMap<RecordId, PersistentId>,
}

fn direct_molecule_id(
    document: &TypedDocument,
    selector: &DocumentObjectIdV1,
) -> Result<PersistentId, TypedDocumentError> {
    let record = document
        .resolve_document_object_id(selector)
        .map_err(|_| TypedDocumentError::InvalidLinearFormMolecule)?
        .filter(|record| {
            record.class() == TypedClass::Molecule && record.path().components().len() == 1
        })
        .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
    persistent_id(
        record
            .attribute("id")
            .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?,
    )
}

fn extract(
    document: &TypedDocument,
    molecule_id: &PersistentId,
    selected_atoms: &[PersistentId],
) -> Result<Extracted, TypedDocumentError> {
    let tree = &document.indexed().xml.tree;
    let molecule = direct_molecule(tree, document.indexed().xml.document, molecule_id)
        .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
    let child_count = tree.children(molecule).count();
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let mut ids = HashMap::new();
    atoms
        .try_reserve_exact(child_count)
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    bonds
        .try_reserve_exact(child_count)
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    ids.try_reserve(child_count)
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for atom in tree
        .children(molecule)
        .filter(|node| is_core(tree, *node, "atom"))
    {
        let source = attr(tree, atom, "id")
            .map(persistent_id)
            .transpose()?
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(molecule_id.clone()))?;
        let point = only_point(tree, atom, &source)?;
        validate_marks(tree, atom, &source)?;
        let record = record_id(RecordKind::Atom, &source)?;
        ids.insert(record.clone(), source);
        atoms.push(LinearFormAtomV1::new(record, point));
    }
    for bond in tree
        .children(molecule)
        .filter(|node| is_core(tree, *node, "bond"))
    {
        let source = attr(tree, bond, "id")
            .map(persistent_id)
            .transpose()?
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(molecule_id.clone()))?;
        let start = attr(tree, bond, "start")
            .map(persistent_id)
            .transpose()?
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(source.clone()))?;
        let end = attr(tree, bond, "end")
            .map(persistent_id)
            .transpose()?
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(source.clone()))?;
        let record = record_id(RecordKind::Bond, &source)?;
        ids.insert(record.clone(), source);
        bonds.push(LinearFormBondV1::new(
            record,
            record_id(RecordKind::Atom, &start)?,
            record_id(RecordKind::Atom, &end)?,
        ));
    }
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(selected_atoms.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for atom in selected_atoms {
        let record = record_id(RecordKind::Atom, atom)?;
        if !ids.contains_key(&record) {
            return Err(TypedDocumentError::InvalidLinearFormAtom(atom.clone()));
        }
        selected.push(record);
    }
    Ok(Extracted {
        request: LinearFormRequestV1::new(selected, LinearFormGraphV1::new(atoms, bonds)),
        ids,
    })
}

fn apply(
    source: &TypedDocument,
    molecule_id: &PersistentId,
    ids: &HashMap<RecordId, PersistentId>,
    plan: &ferrum_domain::linear_form::LinearFormPlanV1,
    fragment_id: &PersistentId,
) -> Result<TypedDocument, TypedDocumentError> {
    let mut candidate = source.detached_candidate()?;
    let indexed = candidate.detached_indexed_mut();
    let molecule = direct_molecule(&indexed.xml.tree, indexed.xml.document, molecule_id)
        .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
    for replacement in plan
        .selected_replacements()
        .iter()
        .chain(plan.exterior_replacements())
    {
        let id = ids
            .get(replacement.atom_id())
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(molecule_id.clone()))?;
        let atom = direct_atom(&indexed.xml.tree, molecule, id)
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormAtom(id.clone()))?;
        move_atom_and_marks(&mut indexed.xml.tree, atom, id, replacement.point())?;
    }
    for record in plan.hydrogen_visible_atoms() {
        let id = ids
            .get(record)
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(molecule_id.clone()))?;
        let atom = direct_atom(&indexed.xml.tree, molecule, id)
            .ok_or_else(|| TypedDocumentError::InvalidLinearFormAtom(id.clone()))?;
        let name = indexed.xml.tree.add_name("show_hydrogens");
        if indexed.xml.tree.get_attribute(atom, name) != Some("on") {
            indexed.xml.tree.set_attribute(atom, name, "on");
        }
    }
    super::typed_linear_form_metadata::remove_invalid_generated_linear_forms(
        &mut indexed.xml.tree,
        molecule,
    )?;
    let (atoms, bonds) = source_members(ids, plan)?;
    super::typed_linear_form_metadata::write_generated_linear_form(
        &mut indexed.xml.tree,
        molecule,
        fragment_id.as_str(),
        &atoms,
        &bonds,
    )?;
    TypedDocument::parse(&candidate.to_xml()?)
}

fn source_members(
    ids: &HashMap<RecordId, PersistentId>,
    plan: &ferrum_domain::linear_form::LinearFormPlanV1,
) -> Result<(Vec<String>, Vec<String>), TypedDocumentError> {
    let mut atoms = Vec::new();
    atoms
        .try_reserve_exact(plan.metadata().atom_members().len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for record in plan.metadata().atom_members() {
        let id = ids
            .get(record)
            .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
        atoms.push(copy_string(id.as_str())?);
    }
    let mut bonds = Vec::new();
    bonds
        .try_reserve_exact(plan.metadata().bond_members().len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    for record in plan.metadata().bond_members() {
        let id = ids
            .get(record)
            .ok_or(TypedDocumentError::InvalidLinearFormMolecule)?;
        bonds.push(copy_string(id.as_str())?);
    }
    Ok((atoms, bonds))
}

fn move_atom_and_marks(
    tree: &mut Xot,
    atom: Node,
    id: &PersistentId,
    replacement: Point2,
) -> Result<(), TypedDocumentError> {
    let point = only_point_node(tree, atom, id)?;
    let old_x = coordinate(tree, point, "x", id)?;
    let old_y = coordinate(tree, point, "y", id)?;
    set_coordinate(tree, point, "x", old_x, replacement.x());
    set_coordinate(tree, point, "y", old_y, replacement.y());
    let mark_count = tree
        .children(atom)
        .filter(|node| is_core(tree, *node, "mark"))
        .count();
    let mut marks = Vec::new();
    marks
        .try_reserve_exact(mark_count)
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    marks.extend(
        tree.children(atom)
            .filter(|node| is_core(tree, *node, "mark")),
    );
    for mark in marks {
        let old_mark_x = coordinate(tree, mark, "x", id)?;
        let old_mark_y = coordinate(tree, mark, "y", id)?;
        let x = old_mark_x + replacement.x() - old_x;
        let y = old_mark_y + replacement.y() - old_y;
        set_coordinate(tree, mark, "x", old_mark_x, x);
        set_coordinate(tree, mark, "y", old_mark_y, y);
    }
    Ok(())
}

fn validate_marks(tree: &Xot, atom: Node, id: &PersistentId) -> Result<(), TypedDocumentError> {
    for mark in tree
        .children(atom)
        .filter(|node| is_core(tree, *node, "mark"))
    {
        coordinate(tree, mark, "x", id)?;
        coordinate(tree, mark, "y", id)?;
    }
    Ok(())
}

fn only_point(tree: &Xot, atom: Node, id: &PersistentId) -> Result<Point2, TypedDocumentError> {
    let point = only_point_node(tree, atom, id)?;
    if attr(tree, point, "z").is_some_and(|z| super::typed_coordinate::parse_coordinate(z).is_err())
    {
        return Err(TypedDocumentError::InvalidLinearFormSource(id.clone()));
    }
    Point2::new(
        coordinate(tree, point, "x", id)?,
        coordinate(tree, point, "y", id)?,
    )
    .map_err(|_| TypedDocumentError::InvalidLinearFormSource(id.clone()))
}

fn only_point_node(tree: &Xot, atom: Node, id: &PersistentId) -> Result<Node, TypedDocumentError> {
    let mut points = tree
        .children(atom)
        .filter(|node| is_core(tree, *node, "point"));
    let point = points
        .next()
        .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(id.clone()))?;
    points
        .next()
        .is_none()
        .then_some(point)
        .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(id.clone()))
}

fn coordinate(
    tree: &Xot,
    node: Node,
    field: &str,
    id: &PersistentId,
) -> Result<f64, TypedDocumentError> {
    attr(tree, node, field)
        .and_then(|value| super::typed_coordinate::parse_coordinate(value).ok())
        .ok_or_else(|| TypedDocumentError::InvalidLinearFormSource(id.clone()))
}
fn set_coordinate(tree: &mut Xot, node: Node, field: &str, old: f64, value: f64) {
    if !super::typed_coordinate::coordinate_changes(old, value) {
        return;
    }
    let name = tree.add_name(field);
    tree.set_attribute(
        node,
        name,
        super::typed_coordinate::canonical_authored_coordinate(value),
    );
}
fn record_id(kind: RecordKind, id: &PersistentId) -> Result<RecordId, TypedDocumentError> {
    RecordId::new(
        kind,
        Identifier::new(copy_string(id.as_str())?)
            .map_err(|_| TypedDocumentError::InvalidLinearFormSource(id.clone()))?,
    )
    .map_err(|_| TypedDocumentError::InvalidLinearFormSource(id.clone()))
}
fn direct_molecule(tree: &Xot, document: Node, id: &PersistentId) -> Option<Node> {
    let root = tree.document_element(document).ok()?;
    exactly_one(tree.children(root).filter(|node| {
        is_core(tree, *node, "molecule") && attr(tree, *node, "id") == Some(id.as_str())
    }))
}
fn direct_atom(tree: &Xot, molecule: Node, id: &PersistentId) -> Option<Node> {
    exactly_one(tree.children(molecule).filter(|node| {
        is_core(tree, *node, "atom") && attr(tree, *node, "id") == Some(id.as_str())
    }))
}
fn exactly_one(mut nodes: impl Iterator<Item = Node>) -> Option<Node> {
    let node = nodes.next()?;
    nodes.next().is_none().then_some(node)
}
fn persistent_id(value: &str) -> Result<PersistentId, TypedDocumentError> {
    PersistentId::new(copy_string(value)?)
        .map_err(|error| TypedDocumentError::Indexed(error.into()))
}
fn copy_string(value: &str) -> Result<String, TypedDocumentError> {
    #[cfg(test)]
    if TEST_FAIL_NEXT_STRING_RESERVATION.with(|flag| flag.replace(false)) {
        return Err(TypedDocumentError::LinearFormResourceExhausted);
    }
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| TypedDocumentError::LinearFormResourceExhausted)?;
    result.push_str(value);
    Ok(result)
}

#[cfg(test)]
thread_local! {
    static TEST_FAIL_NEXT_STRING_RESERVATION: std::cell::Cell<bool> = const {
        std::cell::Cell::new(false)
    };
}
fn attr<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (local == expected && namespace.is_empty()).then_some(value.as_str())
    })
}
fn is_core(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(name, namespace)| name == expected && (namespace == CDML_NAMESPACE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selector(document: &TypedDocument) -> DocumentObjectIdV1 {
        let molecule = document
            .root()
            .children_of(TypedClass::Molecule)
            .next()
            .expect("molecule");
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(molecule)
            .expect("typed ingress persists the durable molecule identity")
    }

    fn ids(names: &[&str]) -> Vec<PersistentId> {
        names
            .iter()
            .map(|name| PersistentId::new((*name).to_owned()).expect("persistent id"))
            .collect()
    }

    fn fragment(identifier: &str, atoms: &[&str], bonds: &[&str]) -> String {
        let mut result =
            format!("<fragment id=\"{identifier}\" type=\"linear_form\"><name>linear_form</name>");
        for bond in bonds {
            result.push_str(&format!("<bond id=\"{bond}\"/>"));
        }
        for atom in atoms {
            result.push_str(&format!("<vertex id=\"{atom}\"/>"));
        }
        result
            .push_str("<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>");
        result
    }

    #[test]
    fn source_order_controls_members_and_apply_preserves_z_and_marks() {
        let source = concat!(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="b"><point x="20" y="0" z="7"/>"#,
            r#"<mark x="21" y="1"/></atom><atom id="a"><point x="0" y="0"/>"#,
            r#"<mark x="1" y="2"/></atom>"#,
            r#"<bond id="ab" start="a" end="b"/><vendor opaque="yes"/></molecule></cdml>"#,
        );
        let document = TypedDocument::parse(source).expect("typed document");
        let selector = selector(&document);
        let selected = ids(&["a", "b"]);
        assert!(matches!(
            document
                .prepare_linear_form_convert_v1(&selector, &selected)
                .expect("prepare"),
            LinearFormCandidateV1::NeedFragmentId
        ));
        let fragment = PersistentId::new("ferrum-fragment-v1-0").expect("fragment");
        let LinearFormCandidateV1::Repair { candidate, .. } = document
            .apply_linear_form_convert_v1(&selector, &selected, &fragment)
            .expect("apply")
        else {
            panic!("candidate must change")
        };
        let xml = candidate.to_xml().expect("serialize");
        assert!(xml.find("id=\"b\"").expect("b") < xml.find("id=\"a\"").expect("a"));
        assert!(xml.contains("z=\"7\""));
        assert!(xml.contains("show_hydrogens=\"on\""));
        assert!(xml.contains("opaque=\"yes\""));
        assert!(xml.contains("x=\"20\" y=\"0\" z=\"7\""));
        assert!(xml.contains("<point x=\"30\" y=\"0\"/>"));
        assert!(xml.contains("<mark x=\"31\" y=\"2\"/>"));
    }

    #[test]
    fn source_located_fragments_repair_with_their_existing_identity() {
        let owned = fragment("owned", &["a", "b"], &["ab"]);
        let source = format!(
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"b\"><point x=\"20\" y=\"0\"/></atom>",
                "<atom id=\"a\"><point x=\"0\" y=\"0\"/></atom>",
                "<bond id=\"ab\" start=\"a\" end=\"b\"/>{owned}</molecule></cdml>",
            ),
            owned = owned
        );
        let document = TypedDocument::parse(&source).expect("typed document");
        assert!(matches!(
            document.prepare_linear_form_convert_v1(&selector(&document), &ids(&["a", "b"])),
            Ok(LinearFormCandidateV1::Repair { fragment_id, .. }) if fragment_id.as_str() == "owned"
        ));
    }

    #[test]
    fn richer_fragment_and_opaque_collision_are_preserved_or_refused() {
        let source = concat!(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a"><point x="0" y="0"/></atom>"#,
            r#"<atom id="b"><point x="20" y="0"/></atom><bond id="ab" start="a" end="b"/>"#,
            r#"<fragment id="rich" type="linear_form" vendor="keep"><name>linear_form</name>"#,
            r#"<vertex id="a"/><vertex id="b"/>"#,
            r#"<property name="bond_length" value="10" type="IntType"/>"#,
            r#"</fragment><vendor id="ferrum-fragment-v1-0"/></molecule></cdml>"#,
        );
        let document = TypedDocument::parse(source).expect("typed document");
        assert!(matches!(
            document.prepare_linear_form_convert_v1(&selector(&document), &ids(&["a", "b"])),
            Ok(LinearFormCandidateV1::NeedFragmentId)
        ));
        let collision = PersistentId::new("ferrum-fragment-v1-0").expect("id");
        assert!(matches!(
            document.apply_linear_form_convert_v1(
                &selector(&document),
                &ids(&["a", "b"]),
                &collision
            ),
            Err(LinearFormDocumentErrorV1::Document(
                TypedDocumentError::DuplicateLinearFormFragment(_)
            ))
        ));
        let accepted = PersistentId::new("ferrum-fragment-v1-1").expect("id");
        let LinearFormCandidateV1::Repair { candidate, .. } = document
            .apply_linear_form_convert_v1(&selector(&document), &ids(&["a", "b"]), &accepted)
            .expect("noncolliding id")
        else {
            panic!("must mutate")
        };
        assert!(candidate.to_xml().expect("xml").contains("vendor=\"keep\""));
    }

    #[test]
    fn malformed_geometry_and_domain_refusals_are_typed() {
        for source in [
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"><point x=\"0\"/></atom></molecule></cdml>",
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"><point x=\"0\" y=\"0\"/>",
                "<mark x=\"0\"/></atom></molecule></cdml>",
            ),
        ] {
            let document = TypedDocument::parse(source).expect("typed document");
            assert!(matches!(
                document.prepare_linear_form_convert_v1(&selector(&document), &ids(&["a"])),
                Err(LinearFormDocumentErrorV1::Document(
                    TypedDocumentError::InvalidLinearFormSource(_)
                ))
            ));
        }
        let source = concat!(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a"><point x="0" y="0"/></atom>"#,
            r#"<atom id="b"><point x="1" y="0"/></atom><atom id="c"><point x="2" y="0"/></atom>"#,
            r#"<atom id="d"><point x="3" y="0"/></atom><bond id="ab" start="a" end="b"/>"#,
            r#"<bond id="ac" start="a" end="c"/>"#,
            r#"<bond id="ad" start="a" end="d"/></molecule></cdml>"#,
        );
        let document = TypedDocument::parse(source).expect("typed document");
        assert!(matches!(
            document
                .prepare_linear_form_convert_v1(&selector(&document), &ids(&["a", "b", "c", "d"])),
            Err(LinearFormDocumentErrorV1::Plan(
                ferrum_domain::linear_form::LinearFormPlanErrorV1::NotSinglePath
            ))
        ));
    }

    #[test]
    fn direct_root_and_foreign_selection_are_refused_without_a_candidate() {
        let source = concat!(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a"><point x="0" y="0"/></atom>"#,
            r#"<atom id="b"><point x="20" y="0"/></atom><bond id="ab" start="a" end="b"/>"#,
            r#"</molecule></cdml>"#,
        );
        let document = TypedDocument::parse(source).expect("typed document");
        let atom = document
            .root()
            .children_of(TypedClass::Molecule)
            .next()
            .expect("molecule")
            .children_of(TypedClass::Atom)
            .next()
            .expect("atom");
        let atom_selector =
            crate::projection_identity_v1::projection_document_object_id_from_record_v1(atom)
                .expect("typed ingress persists the durable atom identity");
        assert!(matches!(
            document.prepare_linear_form_convert_v1(&atom_selector, &ids(&["a"])),
            Err(LinearFormDocumentErrorV1::Document(
                TypedDocumentError::InvalidLinearFormMolecule
            ))
        ));
        assert!(matches!(
            document.prepare_linear_form_convert_v1(&selector(&document), &ids(&["outside"])),
            Err(LinearFormDocumentErrorV1::Document(
                TypedDocumentError::InvalidLinearFormAtom(_)
            ))
        ));
    }

    #[test]
    fn source_located_canonical_fragment_is_history_free_and_exterior_moves_with_its_anchor() {
        let owned = fragment("owned", &["b", "a"], &["ab"]);
        let source = format!(
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"b\" show_hydrogens=\"on\">",
                "<point x=\"0cm\" y=\"0cm\"/></atom><atom id=\"a\" show_hydrogens=\"on\">",
                "<point x=\"10\" y=\"0cm\"/></atom><atom id=\"c\">",
                "<point x=\"1\" y=\"0cm\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\"/>",
                "<bond id=\"bc\" start=\"b\" end=\"c\"/>{owned}</molecule></cdml>",
            ),
            owned = owned
        );
        let document = TypedDocument::parse(&source).expect("typed document");
        let repeat =
            document.prepare_linear_form_convert_v1(&selector(&document), &ids(&["a", "b"]));
        assert!(
            matches!(repeat, Ok(LinearFormCandidateV1::NoChange)),
            "{repeat:?}"
        );

        let moved_source = concat!(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="b"><point x="20" y="0"/></atom>"#,
            r#"<atom id="a"><point x="0" y="0"/></atom><atom id="c"><point x="1" y="0"/></atom>"#,
            r#"<bond id="ab" start="a" end="b"/>"#,
            r#"<bond id="ac" start="a" end="c"/></molecule></cdml>"#,
        );
        let moved = TypedDocument::parse(moved_source).expect("typed document");
        let id = PersistentId::new("new").expect("id");
        let LinearFormCandidateV1::Repair { candidate, .. } = moved
            .apply_linear_form_convert_v1(&selector(&moved), &ids(&["a", "b"]), &id)
            .expect("candidate")
        else {
            panic!("must mutate")
        };
        assert!(candidate.to_xml().expect("xml").contains("x=\"31\""));
    }

    #[test]
    fn conversion_resource_failure_is_typed_and_leaves_source_unchanged() {
        let source = concat!(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a"><point x="0" y="0"/></atom>"#,
            r#"</molecule></cdml>"#,
        );
        let document = TypedDocument::parse(source).expect("typed document");
        let before = document.to_xml().expect("source xml");
        let selector = selector(&document);
        TEST_FAIL_NEXT_STRING_RESERVATION.with(|flag| flag.set(true));
        assert!(matches!(
            document.prepare_linear_form_convert_v1(&selector, &ids(&["a"])),
            Err(LinearFormDocumentErrorV1::Document(
                TypedDocumentError::LinearFormResourceExhausted
            ))
        ));
        assert_eq!(document.to_xml().expect("source xml"), before);
    }
}
