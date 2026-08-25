//! Bounded, deterministic materialization of ordinary explicit hydrogen topology.

use std::collections::HashMap;

use ferrum_core::{BondOrder, BondStyle, Position, VertexRef};
use thiserror::Error;
use xot::Node;

use crate::{
    CDML_NAMESPACE, DocumentObjectIdV1, DocumentSnapshot, PersistentId, TypedClass, TypedDocument,
    TypedDocumentError, element_name,
};

const MAX_ATOMS: usize = 256;
const MAX_BONDS: usize = 512;
const HYDROGEN_BOND_LENGTH: f64 = 24.0;

/// One fenced request to make a direct root's hydrogen topology explicit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeHydrogenMaterializationRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    anchor_atom_id: DocumentObjectIdV1,
}

impl DocumentMoleculeHydrogenMaterializationRequestV1 {
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        anchor_atom_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            anchor_atom_id,
        }
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub const fn anchor_atom_id(&self) -> &DocumentObjectIdV1 {
        &self.anchor_atom_id
    }
}

/// Closed source-profile refusals before a materialized candidate exists.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentMoleculeHydrogenMaterializationRefusalV1 {
    #[error("document revision is stale")]
    StaleObservation,
    #[error("document digest is stale")]
    DigestMismatch,
    #[error("selected molecule is not one durable direct root")]
    UnknownDirectMolecule,
    #[error("selected anchor is not a durable authored atom")]
    UnknownAnchorAtom,
    #[error("selected anchor does not belong to the selected direct root")]
    AnchorNotInSelectedRoot,
    #[error("selected root has an unsupported element")]
    ElementOutsideProfile,
    #[error("selected root has a nonzero formal charge")]
    NonzeroFormalCharge,
    #[error("selected root has a nonzero stored explicit-hydrogen count")]
    NonzeroExplicitHydrogens,
    #[error("selected root has radical, aromatic, styled, or unsupported bond facts")]
    UnsupportedBondOrRadical,
    #[error("an existing hydrogen must have exactly one ordinary single bond")]
    ExistingHydrogenTopology,
    #[error("selected root exceeds a neutral ordinary valence")]
    ValenceExceeded,
    #[error("selected root has unsupported atom facts or positions")]
    UnsupportedDocument,
    #[error("completed root exceeds the bounded oxidation profile")]
    ResourceLimit,
    #[error("completed root fails document projection-safety admission")]
    UnrenderableCandidate,
    #[error("completed root was refused by renderer admission")]
    RendererAdmission,
    #[error("completed root was not accepted by oxidation-state V1")]
    OxidationPostcondition,
}

/// Immutable materialization plan, deliberately free of allocated durable IDs.
#[derive(Clone, Debug)]
pub(crate) struct HydrogenMaterializationPlanV1 {
    pub(crate) molecule: PersistentId,
    additions: Vec<HydrogenAdditionV1>,
    needs_normalization: bool,
}

impl HydrogenMaterializationPlanV1 {
    #[must_use]
    pub(crate) fn added_hydrogen_count(&self) -> usize {
        self.additions.len()
    }
    #[must_use]
    pub(crate) fn is_already_materialized(&self) -> bool {
        self.additions.is_empty() && !self.needs_normalization
    }
}

/// Stable outcome facts for one accepted materialization attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeHydrogenMaterializationResultV1 {
    added_hydrogen_count: usize,
    changed: bool,
    anchor_atom_id: DocumentObjectIdV1,
}

impl DocumentMoleculeHydrogenMaterializationResultV1 {
    pub(crate) const fn new(
        added_hydrogen_count: usize,
        changed: bool,
        anchor_atom_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            added_hydrogen_count,
            changed,
            anchor_atom_id,
        }
    }
    #[must_use]
    pub const fn added_hydrogen_count(&self) -> usize {
        self.added_hydrogen_count
    }
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }
    #[must_use]
    pub const fn anchor_atom_id(&self) -> &DocumentObjectIdV1 {
        &self.anchor_atom_id
    }
}

#[derive(Clone, Debug)]
struct HydrogenAdditionV1 {
    parent: PersistentId,
    position: Position,
}

pub(crate) fn plan_hydrogen_materialization_v1(
    document: &TypedDocument,
    snapshot: &DocumentSnapshot,
    request: &DocumentMoleculeHydrogenMaterializationRequestV1,
) -> Result<HydrogenMaterializationPlanV1, DocumentMoleculeHydrogenMaterializationRefusalV1> {
    if snapshot.revision() != request.expected_revision {
        return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation);
    }
    if snapshot.digest() != request.expected_digest() {
        return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::DigestMismatch);
    }
    let root = document
        .resolve_document_object_id(request.molecule_id())
        .filter(|record| {
            record.class() == TypedClass::Molecule && record.path().components().len() == 1
        })
        .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnknownDirectMolecule)?;
    let anchor = document
        .resolve_document_object_id(request.anchor_atom_id())
        .filter(|record| record.class() == TypedClass::Atom)
        .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnknownAnchorAtom)?;
    if anchor.path().components().len() != 2
        || anchor.path().components().first() != root.path().components().first()
    {
        return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::AnchorNotInSelectedRoot);
    }
    let molecule = PersistentId::new(
        root.attribute("id")
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?
            .to_owned(),
    )
    .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
    let core = document
        .core_molecule(request.molecule_id())
        .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?
        .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnknownDirectMolecule)?;
    let mut atoms = Vec::new();
    let mut needs_normalization = false;
    let mut indices = HashMap::new();
    for (index, atom) in core.atoms().iter().enumerate() {
        let id = PersistentId::new(atom.source_id().as_str().to_owned())
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let element = atom
            .element()
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::ElementOutsideProfile)?;
        if !matches!(element, "H" | "C" | "N" | "O") {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::ElementOutsideProfile);
        }
        if atom.formal_charge().unwrap_or(0) != 0 {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::NonzeroFormalCharge);
        }
        if atom.explicit_hydrogens().unwrap_or(0) != 0 {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::NonzeroExplicitHydrogens);
        }
        needs_normalization |=
            atom.formal_charge().is_none() || atom.explicit_hydrogens().is_none();
        if atom.isotope().is_some()
            || atom.valence().is_some()
            || atom.free_sites().is_some()
            || atom.multiplicity().is_some_and(|value| value != 1)
        {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedBondOrRadical);
        }
        indices.insert(atom.identity().clone(), index);
        atoms.push((id, element, atom.position(), 0_u16));
    }
    let mut neighbors = vec![Vec::<(usize, u16)>::new(); atoms.len()];
    for bond in core.bonds() {
        if bond
            .style()
            .is_some_and(|style| style != &BondStyle::Normal)
            || bond.aromatic() == Some(true)
        {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedBondOrRadical);
        }
        let order = match bond.order() {
            Some(BondOrder::Single) => 1,
            Some(BondOrder::Double) => 2,
            Some(BondOrder::Triple) => 3,
            _ => {
                return Err(
                    DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedBondOrRadical,
                );
            }
        };
        let VertexRef::Atom(start) = bond.start() else {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument);
        };
        let VertexRef::Atom(end) = bond.end() else {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument);
        };
        let start = *indices
            .get(start)
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let end = *indices
            .get(end)
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        neighbors[start].push((end, order));
        neighbors[end].push((start, order));
    }
    let mut additions = Vec::new();
    for index in 0..atoms.len() {
        let (_, element, position, _) = &atoms[index];
        let total: u16 = neighbors[index].iter().map(|(_, order)| *order).sum();
        if *element == "H" {
            if neighbors[index].len() != 1 || total != 1 {
                return Err(
                    DocumentMoleculeHydrogenMaterializationRefusalV1::ExistingHydrogenTopology,
                );
            }
            continue;
        }
        let target = match *element {
            "C" => 4,
            "N" => 3,
            "O" => 2,
            _ => unreachable!(),
        };
        if total > target {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::ValenceExceeded);
        }
        let mut planned_positions = Vec::new();
        for _ in 0..usize::from(target - total) {
            let position = free_bearing(*position, &neighbors[index], &atoms, &planned_positions);
            planned_positions.push(position);
            additions.push(HydrogenAdditionV1 {
                parent: atoms[index].0.clone(),
                position,
            });
        }
    }
    if atoms.len().saturating_add(additions.len()) > MAX_ATOMS
        || core.bonds().len().saturating_add(additions.len()) > MAX_BONDS
    {
        return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::ResourceLimit);
    }
    Ok(HydrogenMaterializationPlanV1 {
        molecule,
        additions,
        needs_normalization,
    })
}

/// Choose the next bearing from a fixed, document-owned angular lattice.
///
/// Existing neighbor bearings and every earlier new-hydrogen bearing on this
/// parent are occupied. The next position maximizes its minimum angular
/// separation from that complete occupied set. When computed clearances compare
/// equal under `f64::total_cmp`, the lowest lattice step wins.
fn free_bearing(
    parent: Position,
    neighbors: &[(usize, u16)],
    atoms: &[(PersistentId, &str, Position, u16)],
    planned_positions: &[Position],
) -> Position {
    let mut occupied = neighbors
        .iter()
        .map(|(index, _)| {
            let point = atoms[*index].2;
            (point.y() - parent.y()).atan2(point.x() - parent.x())
        })
        .collect::<Vec<_>>();
    occupied.extend(
        planned_positions
            .iter()
            .map(|point| (point.y() - parent.y()).atan2(point.x() - parent.x())),
    );
    let sectors = 24_usize;
    let step = select_free_bearing_step(&occupied, sectors);
    let angle = std::f64::consts::TAU * step as f64 / sectors as f64;
    Position::new(
        parent.x() + HYDROGEN_BOND_LENGTH * angle.cos(),
        parent.y() + HYDROGEN_BOND_LENGTH * angle.sin(),
        parent.z(),
    )
    .expect("finite parent and bounded trigonometry produce finite coordinates")
}

fn select_free_bearing_step(occupied: &[f64], sectors: usize) -> usize {
    (0..sectors)
        .map(|step| (step, std::f64::consts::TAU * step as f64 / sectors as f64))
        .max_by(|(left_step, left), (right_step, right)| {
            angular_clearance(*left, occupied)
                .total_cmp(&angular_clearance(*right, occupied))
                .then_with(|| right_step.cmp(left_step))
        })
        .map_or(0, |(step, _)| step)
}

fn angular_clearance(angle: f64, occupied: &[f64]) -> f64 {
    occupied
        .iter()
        .map(|other| (angle - other).cos())
        .fold(-1.0_f64, f64::max)
        .mul_add(-1.0, 1.0)
}

impl TypedDocument {
    pub(crate) fn with_materialized_hydrogens_v1(
        &self,
        plan: &HydrogenMaterializationPlanV1,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
    ) -> Result<Self, TypedDocumentError> {
        if atom_ids.len() != plan.additions.len() || bond_ids.len() != plan.additions.len() {
            return Err(TypedDocumentError::InsertionIdentityCountMismatch);
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("parsed document has root");
        let id = tree.add_name("id");
        let name = tree.add_name("name");
        let charge = tree.add_name("charge");
        let explicit_hydrogens = tree.add_name("explicit_hydrogens");
        let molecule = tree
            .children(root)
            .find(|node| {
                is_named(tree, *node, "molecule")
                    && tree.get_attribute(*node, id) == Some(plan.molecule.as_str())
            })
            .ok_or_else(|| TypedDocumentError::UnknownMolecule(plan.molecule.clone()))?;
        for atom in tree
            .children(molecule)
            .filter(|node| is_named(tree, *node, "atom"))
            .collect::<Vec<_>>()
        {
            tree.set_attribute(atom, charge, "0");
            tree.set_attribute(atom, explicit_hydrogens, "0");
        }
        let namespace = element_name(tree, molecule)
            .expect("typed molecule is element")
            .1;
        let atom_name = element_name_id(tree, "atom", &namespace);
        let point_name = element_name_id(tree, "point", &namespace);
        let bond_name = element_name_id(tree, "bond", &namespace);
        let x = tree.add_name("x");
        let y = tree.add_name("y");
        let z = tree.add_name("z");
        let start = tree.add_name("start");
        let end = tree.add_name("end");
        let bond_type = tree.add_name("type");
        for ((addition, atom_id), bond_id) in plan.additions.iter().zip(atom_ids).zip(bond_ids) {
            let atom = tree.new_element(atom_name);
            tree.set_attribute(atom, id, atom_id.as_str());
            tree.set_attribute(atom, name, "H");
            tree.set_attribute(atom, charge, "0");
            tree.set_attribute(atom, explicit_hydrogens, "0");
            let point = tree.new_element(point_name);
            tree.set_attribute(point, x, addition.position.x().to_string());
            tree.set_attribute(point, y, addition.position.y().to_string());
            tree.set_attribute(point, z, addition.position.z().to_string());
            tree.append(atom, point)
                .map_err(TypedDocumentError::Mutation)?;
            tree.append(molecule, atom)
                .map_err(TypedDocumentError::Mutation)?;
            let bond = tree.new_element(bond_name);
            tree.set_attribute(bond, id, bond_id.as_str());
            tree.set_attribute(bond, bond_type, "n1");
            tree.set_attribute(bond, start, addition.parent.as_str());
            tree.set_attribute(bond, end, atom_id.as_str());
            tree.append(molecule, bond)
                .map_err(TypedDocumentError::Mutation)?;
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}

fn is_named(tree: &xot::Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(local, namespace)| local == expected && namespace == CDML_NAMESPACE)
}

fn element_name_id(tree: &mut xot::Xot, local: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local, namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::select_free_bearing_step;

    #[test]
    fn equal_clearance_selects_the_lowest_lattice_step() {
        assert_eq!(select_free_bearing_step(&[], 24), 0);
    }
}
