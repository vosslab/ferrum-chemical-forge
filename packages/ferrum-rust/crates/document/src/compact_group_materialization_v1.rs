//! Closed typed-CDML replacement for one direct compact group.

use std::collections::HashSet;

use ferrum_document_model::{
    CompactGroupCatalogKeyV1, CompactGroupRecipeBondOrderV1, CompactGroupRecipeBondPresentationV1,
    materialization_recipe_v1,
};
use xot::{Node, Xot};

use super::{CDML_NAMESPACE, PersistentId, TypedClass, TypedDocument, element_name};

/// Caller-owned identities reserved for one closed compact-group replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TypedCompactGroupMaterializationRequestV1 {
    molecule_id: PersistentId,
    compact_group_id: PersistentId,
    atom_ids: Vec<PersistentId>,
    bond_ids: Vec<PersistentId>,
}

impl TypedCompactGroupMaterializationRequestV1 {
    pub(crate) fn new(
        molecule_id: PersistentId,
        compact_group_id: PersistentId,
        atom_ids: Vec<PersistentId>,
        bond_ids: Vec<PersistentId>,
    ) -> Self {
        Self {
            molecule_id,
            compact_group_id,
            atom_ids,
            bond_ids,
        }
    }
}

/// Closed reasons a typed compact-group replacement is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompactGroupMaterializationRefusalV1 {
    InvalidTarget,
    UnsupportedRecipe,
    InvalidTopology,
    InvalidSuppliedIds,
    InvalidCandidate,
    StalePlan,
}

/// Exhaustive source topologies accepted by the compact-group replacement plan.
///
/// Attached retains the original exterior-bond rewrite. DirectRoot represents one
/// self-contained free compact group whose recipe becomes the molecule's complete graph.
#[derive(Clone, Debug, Eq, PartialEq)]
enum CompactGroupMaterializationTopologyV1 {
    Attached {
        exterior_bond_id: PersistentId,
        exterior_endpoint_is_start: bool,
    },
    DirectRoot,
}

/// Fully resolved replacement facts, detached from session and protocol concerns.
#[derive(Clone, Debug)]
pub(crate) struct CompactGroupMaterializationPlanV1 {
    request: TypedCompactGroupMaterializationRequestV1,
    catalog_key: CompactGroupCatalogKeyV1,
    attachment_index: u8,
    topology: CompactGroupMaterializationTopologyV1,
    anchor_x: f64,
    anchor_y: f64,
    orientation_radians: f64,
    recipe: ferrum_document_model::CompactGroupMaterializationRecipeV1,
}

impl CompactGroupMaterializationPlanV1 {
    pub(crate) fn atom_count(&self) -> usize {
        self.recipe.atoms.len()
    }

    pub(crate) fn bond_count(&self) -> usize {
        self.recipe.bonds.len()
    }

    fn has_same_provenance_as(&self, current: &Self) -> bool {
        self.catalog_key == current.catalog_key
            && self.attachment_index == current.attachment_index
            && self.topology == current.topology
            && self.anchor_x.to_bits() == current.anchor_x.to_bits()
            && self.anchor_y.to_bits() == current.anchor_y.to_bits()
            && self.orientation_radians.to_bits() == current.orientation_radians.to_bits()
            && self.recipe == current.recipe
    }
}

/// Typed candidate and authoritative attachment focus after replacement.
#[derive(Debug)]
pub(crate) struct CompactGroupMaterializationResultV1 {
    candidate: TypedDocument,
    attachment_focus: PersistentId,
}

impl CompactGroupMaterializationResultV1 {
    #[cfg(test)]
    pub(crate) fn candidate(&self) -> &TypedDocument {
        &self.candidate
    }

    pub(crate) fn attachment_focus(&self) -> &PersistentId {
        &self.attachment_focus
    }

    pub(crate) fn into_candidate(self) -> TypedDocument {
        self.candidate
    }
}

impl TypedDocument {
    /// Resolve one direct compact group into a closed detached replacement plan.
    pub(crate) fn prepare_compact_group_materialization_v1(
        &self,
        request: TypedCompactGroupMaterializationRequestV1,
    ) -> Result<CompactGroupMaterializationPlanV1, CompactGroupMaterializationRefusalV1> {
        let indexed = self.indexed();
        let tree = &indexed.xml().tree;
        let root = tree
            .document_element(indexed.xml().document)
            .expect("a parsed CDML document has a document element");
        let molecule = direct_element_by_id(tree, root, "molecule", &request.molecule_id)
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        direct_element_by_id(tree, molecule, "compact-group", &request.compact_group_id)
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let typed_molecule = self
            .root()
            .children_of(TypedClass::Molecule)
            .find(|record| record.attribute("id") == Some(request.molecule_id.as_str()))
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let typed_group = typed_molecule
            .typed_children()
            .iter()
            .find(|child| {
                child.record().class() == TypedClass::CompactGroup
                    && child.record().attribute("id") == Some(request.compact_group_id.as_str())
            })
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let projection = super::compact_group_projection_v1::compact_group(typed_group)
            .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let recipe = materialization_recipe_v1(projection.catalog_key())
            .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedRecipe)?;
        let attachment_index = typed_group
            .record()
            .attribute("attachment-index")
            .and_then(|value| value.parse::<u8>().ok())
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let direct_atoms = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "atom"))
            .filter_map(|node| attribute(tree, node, "id").map(|id| (id, node)))
            .collect::<Vec<_>>();
        let direct_groups = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "compact-group"))
            .collect::<Vec<_>>();
        let direct_bonds = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "bond"))
            .collect::<Vec<_>>();
        let exterior = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "bond"))
            .filter_map(|bond| exterior_bond(tree, bond, request.compact_group_id.as_str()))
            .collect::<Vec<_>>();
        let topology = if direct_atoms.is_empty()
            && direct_bonds.is_empty()
            && direct_groups.len() == 1
            && attribute(tree, direct_groups[0], "id") == Some(request.compact_group_id.as_str())
            && typed_molecule
                .typed_children()
                .iter()
                .filter(|child| child.record().class() == TypedClass::CompactGroup)
                .count()
                == 1
        {
            CompactGroupMaterializationTopologyV1::DirectRoot
        } else {
            let [(exterior_bond, other_endpoint, exterior_endpoint_is_start)] = exterior.as_slice()
            else {
                return Err(CompactGroupMaterializationRefusalV1::InvalidTopology);
            };
            if !direct_atoms
                .iter()
                .any(|(id, _)| *id == other_endpoint.as_str())
            {
                return Err(CompactGroupMaterializationRefusalV1::InvalidTopology);
            }
            let exterior_bond_id = PersistentId::new(
                attribute(tree, *exterior_bond, "id")
                    .unwrap_or_default()
                    .to_owned(),
            )
            .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidTopology)?;
            CompactGroupMaterializationTopologyV1::Attached {
                exterior_bond_id,
                exterior_endpoint_is_start: *exterior_endpoint_is_start,
            }
        };
        if (!request.atom_ids.is_empty() || !request.bond_ids.is_empty())
            && (request.atom_ids.len() != recipe.atoms.len()
                || request.bond_ids.len() != recipe.bonds.len()
                || !unique_ids(&request.atom_ids)
                || !unique_ids(&request.bond_ids)
                || request
                    .atom_ids
                    .iter()
                    .chain(&request.bond_ids)
                    .any(|id| indexed.resolve_id(id).is_some()))
        {
            return Err(CompactGroupMaterializationRefusalV1::InvalidSuppliedIds);
        }
        Ok(CompactGroupMaterializationPlanV1 {
            request,
            catalog_key: projection.catalog_key(),
            attachment_index,
            topology,
            anchor_x: projection.anchor().x(),
            anchor_y: projection.anchor().y(),
            orientation_radians: projection.orientation_degrees().to_radians(),
            recipe,
        })
    }

    /// Apply one closed replacement plan and re-admit the exact typed candidate.
    pub(crate) fn materialize_compact_group_v1(
        &self,
        plan: &CompactGroupMaterializationPlanV1,
    ) -> Result<CompactGroupMaterializationResultV1, CompactGroupMaterializationRefusalV1> {
        let current = self
            .prepare_compact_group_materialization_v1(plan.request.clone())
            .map_err(|_| CompactGroupMaterializationRefusalV1::StalePlan)?;
        if !plan.has_same_provenance_as(&current) {
            return Err(CompactGroupMaterializationRefusalV1::StalePlan);
        }
        if plan.request.atom_ids.len() != plan.recipe.atoms.len()
            || plan.request.bond_ids.len() != plan.recipe.bonds.len()
        {
            return Err(CompactGroupMaterializationRefusalV1::InvalidSuppliedIds);
        }
        let mut candidate = self
            .detached_candidate()
            .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let molecule = direct_element_by_id(tree, root, "molecule", &plan.request.molecule_id)
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let group = direct_element_by_id(
            tree,
            molecule,
            "compact-group",
            &plan.request.compact_group_id,
        )
        .ok_or(CompactGroupMaterializationRefusalV1::InvalidTarget)?;
        let namespace = element_name(tree, molecule)
            .map(|(_, namespace)| namespace)
            .unwrap_or_default();
        let attachment_index = plan
            .recipe
            .atoms
            .iter()
            .position(|atom| atom.role == plan.recipe.attachment_atom_role)
            .ok_or(CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        for (atom, id) in plan.recipe.atoms.iter().zip(&plan.request.atom_ids) {
            let (x, y) = rotate(atom.x, atom.y, plan.orientation_radians);
            let node = new_atom(
                tree,
                &namespace,
                id,
                atom.element,
                atom.formal_charge,
                plan.anchor_x + x,
                plan.anchor_y + y,
            );
            tree.insert_before(group, node)
                .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        }
        for (bond, id) in plan.recipe.bonds.iter().zip(&plan.request.bond_ids) {
            let start = recipe_atom_id(plan, bond.start_role)?;
            let end = recipe_atom_id(plan, bond.end_role)?;
            let node = new_bond(
                tree,
                &namespace,
                id,
                start,
                end,
                bond.order,
                bond.presentation,
            );
            tree.insert_before(group, node)
                .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        }
        if let CompactGroupMaterializationTopologyV1::Attached {
            exterior_bond_id,
            exterior_endpoint_is_start,
        } = &plan.topology
        {
            let exterior_bond = direct_element_by_id(tree, molecule, "bond", exterior_bond_id)
                .ok_or(CompactGroupMaterializationRefusalV1::InvalidTopology)?;
            let endpoint_name = tree.add_name(if *exterior_endpoint_is_start {
                "start"
            } else {
                "end"
            });
            tree.set_attribute(
                exterior_bond,
                endpoint_name,
                plan.request.atom_ids[attachment_index].as_str(),
            );
        }
        tree.remove(group)
            .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        let serialized = candidate
            .to_xml()
            .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        let candidate = Self::parse(&serialized)
            .map_err(|_| CompactGroupMaterializationRefusalV1::InvalidCandidate)?;
        Ok(CompactGroupMaterializationResultV1 {
            attachment_focus: plan.request.atom_ids[attachment_index].clone(),
            candidate,
        })
    }
}

fn direct_element_by_id(tree: &Xot, parent: Node, local: &str, id: &PersistentId) -> Option<Node> {
    tree.children(parent).find(|node| {
        is_cdml_element(tree, *node, local) && attribute(tree, *node, "id") == Some(id.as_str())
    })
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(local, namespace)| local == expected && namespace == CDML_NAMESPACE)
}

fn attribute<'a>(tree: &'a Xot, node: Node, name: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(candidate, value)| {
        let (local, namespace) = tree.name_ns_str(candidate);
        (namespace.is_empty() && local == name).then_some(value.as_str())
    })
}

fn exterior_bond(tree: &Xot, bond: Node, group_id: &str) -> Option<(Node, String, bool)> {
    let start = attribute(tree, bond, "start")?;
    let end = attribute(tree, bond, "end")?;
    if start == group_id && end != group_id {
        Some((bond, end.to_owned(), true))
    } else if end == group_id && start != group_id {
        Some((bond, start.to_owned(), false))
    } else {
        None
    }
}

fn unique_ids(ids: &[PersistentId]) -> bool {
    ids.iter().collect::<HashSet<_>>().len() == ids.len()
}

fn recipe_atom_id<'a>(
    plan: &'a CompactGroupMaterializationPlanV1,
    role: &str,
) -> Result<&'a PersistentId, CompactGroupMaterializationRefusalV1> {
    plan.recipe
        .atoms
        .iter()
        .position(|atom| atom.role == role)
        .and_then(|index| plan.request.atom_ids.get(index))
        .ok_or(CompactGroupMaterializationRefusalV1::InvalidCandidate)
}

fn rotate(x: f64, y: f64, radians: f64) -> (f64, f64) {
    (
        x * radians.cos() - y * radians.sin(),
        x * radians.sin() + y * radians.cos(),
    )
}

fn element_name_id(tree: &mut Xot, local: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local, namespace)
    }
}

fn new_atom(
    tree: &mut Xot,
    namespace: &str,
    id: &PersistentId,
    element: &str,
    formal_charge: Option<i32>,
    x: f64,
    y: f64,
) -> Node {
    let atom_name = element_name_id(tree, "atom", namespace);
    let point_name = element_name_id(tree, "point", namespace);
    let id_name = tree.add_name("id");
    let atom_element_name = tree.add_name("name");
    let charge_name = tree.add_name("charge");
    let x_name = tree.add_name("x");
    let y_name = tree.add_name("y");
    let atom = tree.new_element(atom_name);
    tree.set_attribute(atom, id_name, id.as_str());
    tree.set_attribute(atom, atom_element_name, element);
    if let Some(formal_charge) = formal_charge {
        tree.set_attribute(atom, charge_name, formal_charge.to_string());
    }
    let point = tree.new_element(point_name);
    tree.set_attribute(point, x_name, x.to_string());
    tree.set_attribute(point, y_name, y.to_string());
    tree.append(atom, point)
        .expect("new atom accepts its point");
    atom
}

fn new_bond(
    tree: &mut Xot,
    namespace: &str,
    id: &PersistentId,
    start: &PersistentId,
    end: &PersistentId,
    order: CompactGroupRecipeBondOrderV1,
    presentation: CompactGroupRecipeBondPresentationV1,
) -> Node {
    let bond_name = element_name_id(tree, "bond", namespace);
    let id_name = tree.add_name("id");
    let start_name = tree.add_name("start");
    let end_name = tree.add_name("end");
    let type_name = tree.add_name("type");
    let bond = tree.new_element(bond_name);
    tree.set_attribute(bond, id_name, id.as_str());
    tree.set_attribute(bond, start_name, start.as_str());
    tree.set_attribute(bond, end_name, end.as_str());
    tree.set_attribute(
        bond,
        type_name,
        match (order, presentation) {
            (
                CompactGroupRecipeBondOrderV1::Single,
                CompactGroupRecipeBondPresentationV1::Normal,
            ) => "n1",
            (
                CompactGroupRecipeBondOrderV1::Double,
                CompactGroupRecipeBondPresentationV1::Normal,
            ) => "n2",
            (
                CompactGroupRecipeBondOrderV1::Triple,
                CompactGroupRecipeBondPresentationV1::Normal,
            ) => "n3",
        },
    );
    bond
}

#[cfg(test)]
mod tests {
    use super::*;

    // A half-turn uses trigonometric rotation, which leaves a sub-femtometre
    // scene-coordinate residue instead of an exact zero in f64 arithmetic.
    const SCENE_COORDINATE_TOLERANCE: f64 = 1.0e-10;

    fn id(value: &str) -> PersistentId {
        PersistentId::new(value.to_owned()).expect("test ID")
    }

    fn assert_scene_position_close(actual: (f64, f64), expected: (f64, f64)) {
        assert!(
            (actual.0 - expected.0).abs() <= SCENE_COORDINATE_TOLERANCE,
            "scene x coordinate differs: actual={}, expected={}",
            actual.0,
            expected.0,
        );
        assert!(
            (actual.1 - expected.1).abs() <= SCENE_COORDINATE_TOLERANCE,
            "scene y coordinate differs: actual={}, expected={}",
            actual.1,
            expected.1,
        );
    }

    #[test]
    fn attached_methyl_rewrites_the_start_exterior_endpoint_only() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"group\" end=\"anchor\" type=\"n1\" color=\"blue\"/></molecule></cdml>";
        let document = TypedDocument::parse(source).expect("typed source");
        let request = TypedCompactGroupMaterializationRequestV1::new(
            id("m"),
            id("group"),
            vec![id("methyl-carbon")],
            vec![],
        );
        let plan = document
            .prepare_compact_group_materialization_v1(request)
            .expect("plan");
        let result = document
            .materialize_compact_group_v1(&plan)
            .expect("candidate");
        let xml = result.candidate().to_xml().expect("candidate XML");
        assert_eq!(result.attachment_focus().as_str(), "methyl-carbon");
        assert!(xml.contains(
            "id=\"outside\" start=\"methyl-carbon\" end=\"anchor\" type=\"n1\" color=\"blue\""
        ));
    }

    #[test]
    fn attached_nitro_produces_a_typed_candidate_and_attachment_focus() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"nitro\" attachment-index=\"0\" orientation-degrees=\"90\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>";
        let document = TypedDocument::parse(source).expect("typed source");
        let request = TypedCompactGroupMaterializationRequestV1::new(
            id("m"),
            id("group"),
            vec![id("nitrogen"), id("oxygen-double"), id("oxygen-single")],
            vec![id("nitro-double"), id("nitro-single")],
        );
        let plan = document
            .prepare_compact_group_materialization_v1(request)
            .expect("plan");
        let result = document
            .materialize_compact_group_v1(&plan)
            .expect("candidate");
        assert_eq!(result.attachment_focus().as_str(), "nitrogen");
        assert!(result.candidate().core_projection().is_ok());
    }

    #[test]
    fn attached_ethyl_preserves_exterior_identity_at_two_orientations() {
        for (orientation, terminal_x) in [("0", 44.0), ("180", -4.0)] {
            let source = format!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"ethyl\" attachment-index=\"0\" orientation-degrees=\"{orientation}\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\" color=\"blue\"/></molecule></cdml>"
            );
            let document = TypedDocument::parse(&source).expect("typed source");
            let result = document
                .prepare_compact_group_materialization_v1(
                    TypedCompactGroupMaterializationRequestV1::new(
                        id("m"),
                        id("group"),
                        vec![id("ethyl-attachment"), id("ethyl-terminal")],
                        vec![id("ethyl-internal")],
                    ),
                )
                .and_then(|plan| document.materialize_compact_group_v1(&plan))
                .expect("ethyl materialization");
            let projected = result
                .candidate()
                .core_projection()
                .expect("materialized candidate projects");
            let molecule = &projected.molecules()[0];
            let attachment = molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id().as_str() == "ethyl-attachment")
                .expect("ethyl attachment carbon");
            let terminal = molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id().as_str() == "ethyl-terminal")
                .expect("ethyl terminal carbon");
            let outside = molecule
                .bonds()
                .iter()
                .find(|bond| bond.source_id().as_str() == "outside")
                .expect("retained exterior bond");
            let internal = molecule
                .bonds()
                .iter()
                .find(|bond| bond.source_id().as_str() == "ethyl-internal")
                .expect("ethyl internal bond");
            assert_eq!(result.attachment_focus().as_str(), "ethyl-attachment");
            assert!(molecule.groups().is_empty());
            assert_scene_position_close(
                (attachment.position().x(), attachment.position().y()),
                (20.0, 0.0),
            );
            assert_scene_position_close(
                (terminal.position().x(), terminal.position().y()),
                (terminal_x, 0.0),
            );
            assert_eq!(
                (outside.order(), outside.style()),
                (
                    Some(ferrum_core::BondOrder::Single),
                    Some(&ferrum_core::BondStyle::Normal),
                )
            );
            assert!(matches!(
                outside.start(),
                ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "anchor"
            ));
            assert!(matches!(
                outside.end(),
                ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "ethyl-attachment"
            ));
            assert_eq!(
                (internal.order(), internal.style()),
                (
                    Some(ferrum_core::BondOrder::Single),
                    Some(&ferrum_core::BondStyle::Normal),
                )
            );
            assert!(matches!(
                internal.start(),
                ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "ethyl-attachment"
            ));
            assert!(matches!(
                internal.end(),
                ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "ethyl-terminal"
            ));
        }
    }

    #[test]
    fn direct_root_ethyl_becomes_two_explicit_carbons_with_one_normal_single_bond() {
        let document = TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><compact-group id=\"group\" version=\"1\" catalog-key=\"ethyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group></molecule></cdml>").expect("typed direct root");
        let result = document
            .prepare_compact_group_materialization_v1(
                TypedCompactGroupMaterializationRequestV1::new(
                    id("m"),
                    id("group"),
                    vec![id("ethyl-attachment"), id("ethyl-terminal")],
                    vec![id("ethyl-internal")],
                ),
            )
            .and_then(|plan| document.materialize_compact_group_v1(&plan))
            .expect("direct-root ethyl materialization");
        let xml = result.candidate().to_xml().expect("candidate XML");
        let projected = TypedDocument::parse(&xml)
            .expect("materialized XML reparses")
            .core_projection()
            .expect("materialized XML projects");
        let molecule = &projected.molecules()[0];
        let attachment = molecule
            .atoms()
            .iter()
            .find(|atom| atom.source_id().as_str() == "ethyl-attachment")
            .expect("ethyl attachment carbon");
        let terminal = molecule
            .atoms()
            .iter()
            .find(|atom| atom.source_id().as_str() == "ethyl-terminal")
            .expect("ethyl terminal carbon");
        let internal = molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_id().as_str() == "ethyl-internal")
            .expect("ethyl internal bond");
        assert_eq!(result.attachment_focus().as_str(), "ethyl-attachment");
        assert_eq!(
            (attachment.element(), attachment.formal_charge()),
            (Some("C"), None)
        );
        assert_eq!(
            (terminal.element(), terminal.formal_charge()),
            (Some("C"), None)
        );
        assert_eq!(
            (internal.order(), internal.style()),
            (
                Some(ferrum_core::BondOrder::Single),
                Some(&ferrum_core::BondStyle::Normal),
            )
        );
        assert!(matches!(
            internal.start(),
            ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "ethyl-attachment"
        ));
        assert!(matches!(
            internal.end(),
            ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "ethyl-terminal"
        ));
        assert!(!xml.contains("<compact-group"));
        assert!(!xml.contains("charge="));
    }

    #[test]
    fn methoxy_materialization_preserves_oxygen_focus_and_exterior_identity() {
        let attached = TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methoxy\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>").expect("typed attached root");
        let attached_result = attached
            .prepare_compact_group_materialization_v1(
                TypedCompactGroupMaterializationRequestV1::new(
                    id("m"),
                    id("group"),
                    vec![id("methoxy-oxygen"), id("methoxy-carbon")],
                    vec![id("methoxy-internal")],
                ),
            )
            .and_then(|plan| attached.materialize_compact_group_v1(&plan))
            .expect("attached methoxy materialization");
        let attached_projection = attached_result
            .candidate()
            .core_projection()
            .expect("materialized attached methoxy projects");
        let attached_molecule = &attached_projection.molecules()[0];
        let exterior = attached_molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_id().as_str() == "outside")
            .expect("retained exterior bond");
        assert_eq!(
            attached_result.attachment_focus().as_str(),
            "methoxy-oxygen"
        );
        assert!(attached_molecule.groups().is_empty());
        assert!(matches!(
            exterior.end(),
            ferrum_core::VertexRef::Atom(id) if id.source_id().as_str() == "methoxy-oxygen"
        ));
        assert_eq!(
            (exterior.order(), exterior.style()),
            (
                Some(ferrum_core::BondOrder::Single),
                Some(&ferrum_core::BondStyle::Normal),
            )
        );

        let direct_root = TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><compact-group id=\"group\" version=\"1\" catalog-key=\"methoxy\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group></molecule></cdml>").expect("typed direct root");
        let direct_result = direct_root
            .prepare_compact_group_materialization_v1(
                TypedCompactGroupMaterializationRequestV1::new(
                    id("m"),
                    id("group"),
                    vec![id("methoxy-oxygen"), id("methoxy-carbon")],
                    vec![id("methoxy-internal")],
                ),
            )
            .and_then(|plan| direct_root.materialize_compact_group_v1(&plan))
            .expect("direct-root methoxy materialization");
        let direct_projection = direct_result
            .candidate()
            .core_projection()
            .expect("materialized direct-root methoxy projects");
        let direct_molecule = &direct_projection.molecules()[0];
        assert_eq!(direct_result.attachment_focus().as_str(), "methoxy-oxygen");
        assert!(direct_molecule.groups().is_empty());
        assert_eq!(direct_molecule.atoms().len(), 2);
        assert_eq!(direct_molecule.bonds().len(), 1);
        assert!(direct_molecule.atoms().iter().any(|atom| {
            atom.source_id().as_str() == "methoxy-oxygen"
                && atom.element() == Some("O")
                && atom.formal_charge().is_none()
        }));
        assert!(direct_molecule.atoms().iter().any(|atom| {
            atom.source_id().as_str() == "methoxy-carbon"
                && atom.element() == Some("C")
                && atom.formal_charge().is_none()
        }));
        let internal = &direct_molecule.bonds()[0];
        assert_eq!(internal.source_id().as_str(), "methoxy-internal");
        assert_eq!(
            (internal.order(), internal.style()),
            (
                Some(ferrum_core::BondOrder::Single),
                Some(&ferrum_core::BondStyle::Normal),
            )
        );
    }

    #[test]
    fn direct_root_methyl_becomes_one_explicit_atom_without_a_compact_group() {
        let document = TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group></molecule></cdml>").expect("typed direct root");
        let result = document
            .prepare_compact_group_materialization_v1(
                TypedCompactGroupMaterializationRequestV1::new(
                    id("m"),
                    id("group"),
                    vec![id("methyl-carbon")],
                    vec![],
                ),
            )
            .and_then(|plan| document.materialize_compact_group_v1(&plan))
            .expect("direct-root materialization");
        let molecule = result
            .candidate()
            .root()
            .children_of(TypedClass::Molecule)
            .next()
            .expect("materialized molecule");
        assert_eq!(result.attachment_focus().as_str(), "methyl-carbon");
        assert_eq!(
            molecule
                .typed_children()
                .iter()
                .filter(|child| child.record().class() == TypedClass::Atom)
                .count(),
            1
        );
        assert!(
            molecule
                .typed_children()
                .iter()
                .all(|child| child.record().class() != TypedClass::CompactGroup)
        );
    }

    #[test]
    fn mixed_detached_molecule_refuses_materialization() {
        let document = TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"other\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group></molecule></cdml>").expect("typed mixed direct root");
        assert!(matches!(
            document.prepare_compact_group_materialization_v1(
                TypedCompactGroupMaterializationRequestV1::new(
                    id("m"),
                    id("group"),
                    vec![id("ethyl-attachment"), id("ethyl-terminal")],
                    vec![id("ethyl-internal")],
                ),
            ),
            Err(CompactGroupMaterializationRefusalV1::InvalidTopology)
        ));
    }

    #[test]
    fn malformed_exterior_topology_refuses_without_changing_source_document() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"first\" start=\"anchor\" end=\"group\" type=\"n1\"/><bond id=\"second\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>";
        let document = TypedDocument::parse(source).expect("typed source");
        let before = document.to_xml().expect("source XML");
        let request = TypedCompactGroupMaterializationRequestV1::new(
            id("m"),
            id("group"),
            vec![id("replacement")],
            vec![],
        );
        assert!(matches!(
            document.prepare_compact_group_materialization_v1(request),
            Err(CompactGroupMaterializationRefusalV1::InvalidTopology)
        ));
        assert_eq!(document.to_xml().expect("source XML"), before);
    }

    #[test]
    fn changed_same_id_compact_group_refuses_the_detached_plan() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>";
        let document = TypedDocument::parse(source).expect("typed source");
        let plan = document
            .prepare_compact_group_materialization_v1(
                TypedCompactGroupMaterializationRequestV1::new(
                    id("m"),
                    id("group"),
                    vec![id("replacement")],
                    vec![],
                ),
            )
            .expect("plan");
        let changed = TypedDocument::parse(
            &source.replace("orientation-degrees=\"0\"", "orientation-degrees=\"90\""),
        )
        .expect("changed typed source");
        let before = changed.to_xml().expect("changed source XML");
        assert!(matches!(
            changed.materialize_compact_group_v1(&plan),
            Err(CompactGroupMaterializationRefusalV1::StalePlan)
        ));
        assert_eq!(changed.to_xml().expect("changed source XML"), before);
    }
}
