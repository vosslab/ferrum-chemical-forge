//! Closed direct-Haworth insertion facts and durable committed receipts.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_domain::haworth::{
    AuthoredDirectGlycosidicHaworthDepictionV1, DirectGlycosidicHaworthAuthoringAtomElementV1,
    DirectGlycosidicHaworthAuthoringReceiptV1, DirectGlycosidicHaworthBondStyleV1,
    DirectGlycosidicHaworthPositionV1,
};

use super::{
    DocumentHaworthPositionV1, MoleculeInsertionAtomV1, PersistentId, Point3V1,
    SessionDocumentObservationV1, SessionOperationError, TypedDocument, TypedDocumentError,
    typed_molecule_insertion::{InsertionNames, append_atom, valid_cdml_namespace},
};

/// Closed persisted direct-Haworth bond token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentDirectHaworthBondTokenV1 {
    Q1,
    W1,
    N1,
}

/// Explicit graph role for a direct-Haworth bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentDirectHaworthBondRoleV1 {
    Ring,
    Bridge,
}

/// Immutable durable fact for one committed direct-Haworth bond.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedDirectHaworthBondFactV1 {
    bond_identifier: PersistentId,
    endpoints: [PersistentId; 2],
    token: DocumentDirectHaworthBondTokenV1,
    haworth_position: Option<DocumentHaworthPositionV1>,
    role: DocumentDirectHaworthBondRoleV1,
}

impl CommittedDirectHaworthBondFactV1 {
    #[must_use]
    pub fn bond_identifier(&self) -> &PersistentId {
        &self.bond_identifier
    }
    #[must_use]
    pub fn endpoints(&self) -> &[PersistentId; 2] {
        &self.endpoints
    }
    #[must_use]
    pub const fn token(&self) -> DocumentDirectHaworthBondTokenV1 {
        self.token
    }
    #[must_use]
    pub const fn haworth_position(&self) -> Option<DocumentHaworthPositionV1> {
        self.haworth_position
    }
    #[must_use]
    pub const fn role(&self) -> DocumentDirectHaworthBondRoleV1 {
        self.role
    }
}

pub(crate) struct PreparedDirectHaworthReceiptV1 {
    molecule_identifier: PersistentId,
    atom_identifiers: Vec<PersistentId>,
    bond_identifiers: Vec<PersistentId>,
    bond_facts: Vec<CommittedDirectHaworthBondFactV1>,
    authored_depiction: AuthoredDirectGlycosidicHaworthDepictionV1,
}

impl PreparedDirectHaworthReceiptV1 {
    /// Consume the fully checked, pre-commit receipt payload exactly once.
    pub(crate) fn into_parts(
        self,
    ) -> (
        PersistentId,
        Vec<PersistentId>,
        Vec<PersistentId>,
        Vec<CommittedDirectHaworthBondFactV1>,
        AuthoredDirectGlycosidicHaworthDepictionV1,
    ) {
        (
            self.molecule_identifier,
            self.atom_identifiers,
            self.bond_identifiers,
            self.bond_facts,
            self.authored_depiction,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DirectHaworthInsertionV1 {
    atoms: Vec<DirectAtom>,
    bonds: Vec<DirectBond>,
}
#[derive(Clone, Debug)]
struct DirectAtom {
    atom: MoleculeInsertionAtomV1,
}
#[derive(Clone, Debug)]
struct DirectBond {
    endpoints: [usize; 2],
    token: DocumentDirectHaworthBondTokenV1,
    position: Option<DocumentHaworthPositionV1>,
    role: DocumentDirectHaworthBondRoleV1,
}

impl DirectHaworthInsertionV1 {
    pub(crate) fn from_receipt(
        receipt: &DirectGlycosidicHaworthAuthoringReceiptV1,
        anchor: Point3V1,
    ) -> Result<Self, SessionOperationError> {
        let mut indexes = BTreeMap::new();
        let mut atoms = Vec::with_capacity(receipt.atoms_in_canonical_order().len());
        for fact in receipt.atoms_in_canonical_order() {
            let source = fact.source_atom_identity().clone();
            if indexes.insert(source, atoms.len()).is_some() {
                return invalid("receipt duplicates a selected atom identity");
            }
            let local = fact.local();
            let position = Point3V1::new(local.x + anchor.x(), local.y + anchor.y(), anchor.z())
                .map_err(|_| invalid_error("translated atom coordinate is not finite"))?;
            let element = match fact.element() {
                DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon => "C",
                DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen => "O",
            };
            atoms.push(DirectAtom {
                atom: MoleculeInsertionAtomV1::new(element, position, None, None, None)
                    .map_err(|error| invalid_error(error.to_string()))?,
            });
        }
        if atoms.is_empty() {
            return invalid("receipt has no selected atoms");
        }
        let mut bonds = Vec::with_capacity(receipt.bonds_in_canonical_order().len());
        let mut edges = BTreeSet::new();
        let ring_bond_count = atoms
            .len()
            .checked_sub(1)
            .ok_or_else(|| invalid_error("receipt has no ring atoms"))?;
        for (bond_index, fact) in receipt.bonds_in_canonical_order().iter().enumerate() {
            let [start, end] = fact.endpoints();
            let start = *indexes
                .get(start)
                .ok_or_else(|| invalid_error("receipt bond endpoint is missing"))?;
            let end = *indexes
                .get(end)
                .ok_or_else(|| invalid_error("receipt bond endpoint is missing"))?;
            if start == end || !edges.insert((start.min(end), start.max(end))) {
                return invalid("receipt contains an invalid duplicate or self bond");
            }
            let token = map_token(fact.token());
            let position = map_position(fact.haworth_position());
            let role = if bond_index < ring_bond_count {
                DocumentDirectHaworthBondRoleV1::Ring
            } else {
                DocumentDirectHaworthBondRoleV1::Bridge
            };
            if !valid_profile(role, token, position) {
                return invalid("receipt bond does not satisfy the closed direct Haworth profile");
            }
            bonds.push(DirectBond {
                endpoints: [start, end],
                token,
                position,
                role,
            });
        }
        if bonds.len() != ring_bond_count + 2 {
            return invalid("receipt does not contain exactly two bridge bonds");
        }
        Ok(Self { atoms, bonds })
    }

    pub(crate) fn atom_count(&self) -> usize {
        self.atoms.len()
    }
    pub(crate) fn bond_count(&self) -> usize {
        self.bonds.len()
    }

    pub(crate) fn prepared_receipt(
        &self,
        receipt: &DirectGlycosidicHaworthAuthoringReceiptV1,
        molecule: &PersistentId,
        atoms: &[PersistentId],
        bonds: &[PersistentId],
        anchor: Point3V1,
    ) -> Result<PreparedDirectHaworthReceiptV1, SessionOperationError> {
        let bond_facts = self
            .bonds
            .iter()
            .zip(bonds)
            .map(|(bond, id)| CommittedDirectHaworthBondFactV1 {
                bond_identifier: id.clone(),
                endpoints: [
                    atoms[bond.endpoints[0]].clone(),
                    atoms[bond.endpoints[1]].clone(),
                ],
                token: bond.token,
                haworth_position: bond.position,
                role: bond.role,
            })
            .collect();
        let atom_records = atoms
            .iter()
            .map(|id| typed_record(id, RecordKind::Atom))
            .collect::<Result<Vec<_>, _>>()?;
        let bond_records = bonds
            .iter()
            .map(|id| typed_record(id, RecordKind::Bond))
            .collect::<Result<Vec<_>, _>>()?;
        let authored_depiction = receipt
            .authored_depiction_for_durable_commit_v1(
                &atom_records,
                &bond_records,
                ferrum_domain::haworth::HaworthPoint {
                    x: anchor.x(),
                    y: anchor.y(),
                },
            )
            .map_err(|error| invalid_error(error.to_string()))?;
        Ok(PreparedDirectHaworthReceiptV1 {
            molecule_identifier: molecule.clone(),
            atom_identifiers: atoms.to_vec(),
            bond_identifiers: bonds.to_vec(),
            bond_facts,
            authored_depiction,
        })
    }
}

fn typed_record(
    identifier: &PersistentId,
    kind: RecordKind,
) -> Result<RecordId, SessionOperationError> {
    let source = Identifier::new(identifier.as_str().to_owned())
        .map_err(|_| invalid_error("allocated persistent identity is invalid"))?;
    Ok(RecordId::from_source(kind, &source))
}

impl TypedDocument {
    pub(crate) fn with_insert_direct_haworth(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        insertion: &DirectHaworthInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        if atom_ids.len() != insertion.atoms.len() || bond_ids.len() != insertion.bonds.len() {
            return Err(TypedDocumentError::InsertionIdentityCountMismatch);
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("parsed CDML has a root");
        let (_, namespace) =
            super::element_name(&indexed.xml.tree, root).expect("parsed CDML has a root");
        let names = InsertionNames::new(&mut indexed.xml.tree, valid_cdml_namespace(namespace));
        let molecule = indexed.xml.tree.new_element(names.molecule);
        indexed
            .xml
            .tree
            .set_attribute(molecule, names.id, molecule_id.as_str());
        for (id, atom) in atom_ids.iter().zip(&insertion.atoms) {
            append_atom(&mut indexed.xml.tree, molecule, &names, id, &atom.atom)?;
        }
        for (id, bond) in bond_ids.iter().zip(&insertion.bonds) {
            let node = indexed.xml.tree.new_element(names.bond);
            indexed.xml.tree.set_attribute(node, names.id, id.as_str());
            indexed
                .xml
                .tree
                .set_attribute(node, names.bond_type, token_text(bond.token));
            indexed
                .xml
                .tree
                .set_attribute(node, names.start, atom_ids[bond.endpoints[0]].as_str());
            indexed
                .xml
                .tree
                .set_attribute(node, names.end, atom_ids[bond.endpoints[1]].as_str());
            if let Some(position) = bond.position {
                indexed.xml.tree.set_attribute(
                    node,
                    names.haworth_position,
                    position_text(position),
                );
            }
            indexed
                .xml
                .tree
                .append(molecule, node)
                .map_err(TypedDocumentError::Mutation)?;
        }
        indexed
            .xml
            .tree
            .append(root, molecule)
            .map_err(TypedDocumentError::Mutation)?;
        Self::parse(&candidate.to_xml()?)
    }
}

pub(crate) fn validate_candidate(
    document: &TypedDocument,
    molecule: &PersistentId,
    atoms: &[PersistentId],
    bonds: &[PersistentId],
    insertion: &DirectHaworthInsertionV1,
) -> Result<(), SessionOperationError> {
    let snapshot = super::DocumentSnapshot::new(
        0,
        document
            .to_xml()
            .map_err(SessionOperationError::Serialize)?,
        [0; 32],
        true,
    );
    let observation = SessionDocumentObservationV1::from_snapshot(snapshot)
        .map_err(|error| invalid_error(error.to_string()))?;
    let projected = observation
        .projection()
        .molecules()
        .iter()
        .find(|item| item.source_id() == Some(molecule.as_str()))
        .ok_or_else(|| invalid_error("candidate omits inserted molecule"))?;
    let molecule_path = document
        .indexed()
        .resolve_id(molecule)
        .ok_or_else(|| invalid_error("candidate omits inserted molecule identity"))?
        .path()
        .to_string();
    if observation.projection().issues().iter().any(|issue| {
        issue.path() == molecule_path
            || issue
                .path()
                .strip_prefix(&molecule_path)
                .is_some_and(|suffix| suffix.starts_with('/'))
    }) {
        return invalid("candidate direct Haworth molecule has a projection issue");
    }
    if projected.atoms().len() != atoms.len() || projected.bonds().len() != bonds.len() {
        return invalid("candidate projection does not preserve closed direct Haworth facts");
    }
    for ((projected_atom, expected), identifier) in
        projected.atoms().iter().zip(&insertion.atoms).zip(atoms)
    {
        if projected_atom.source_id() != Some(identifier.as_str())
            || projected_atom.element() != Some(expected.atom.element())
            || projected_atom.position() != expected.atom.position()
            || projected_atom.formal_charge().is_some()
            || projected_atom.isotope().is_some()
            || projected_atom.explicit_hydrogens().is_some()
        {
            return invalid("candidate atom facts differ from translated receipt");
        }
    }
    for ((projected_bond, expected), identifier) in
        projected.bonds().iter().zip(&insertion.bonds).zip(bonds)
    {
        if projected_bond.source_id() != Some(identifier.as_str())
            || projected_bond.source_type() != Some(token_text(expected.token))
            || projected_bond.haworth_position() != expected.position
            || projected_bond.start().source_id() != Some(atoms[expected.endpoints[0]].as_str())
            || projected_bond.end().source_id() != Some(atoms[expected.endpoints[1]].as_str())
            || !valid_profile(expected.role, expected.token, expected.position)
        {
            return invalid("candidate bond facts differ from closed direct Haworth profile");
        }
    }
    Ok(())
}

fn map_token(token: DirectGlycosidicHaworthBondStyleV1) -> DocumentDirectHaworthBondTokenV1 {
    match token {
        DirectGlycosidicHaworthBondStyleV1::Q1 => DocumentDirectHaworthBondTokenV1::Q1,
        DirectGlycosidicHaworthBondStyleV1::W1 => DocumentDirectHaworthBondTokenV1::W1,
        DirectGlycosidicHaworthBondStyleV1::N1 => DocumentDirectHaworthBondTokenV1::N1,
    }
}
fn map_position(
    value: Option<DirectGlycosidicHaworthPositionV1>,
) -> Option<DocumentHaworthPositionV1> {
    value.map(|position| match position {
        DirectGlycosidicHaworthPositionV1::Front => DocumentHaworthPositionV1::Front,
        DirectGlycosidicHaworthPositionV1::Back => DocumentHaworthPositionV1::Back,
    })
}
fn valid_profile(
    role: DocumentDirectHaworthBondRoleV1,
    token: DocumentDirectHaworthBondTokenV1,
    position: Option<DocumentHaworthPositionV1>,
) -> bool {
    matches!(
        (role, token, position),
        (
            DocumentDirectHaworthBondRoleV1::Ring,
            DocumentDirectHaworthBondTokenV1::Q1,
            Some(DocumentHaworthPositionV1::Front)
        ) | (
            DocumentDirectHaworthBondRoleV1::Ring,
            DocumentDirectHaworthBondTokenV1::W1,
            Some(DocumentHaworthPositionV1::Front)
        ) | (
            DocumentDirectHaworthBondRoleV1::Ring,
            DocumentDirectHaworthBondTokenV1::N1,
            Some(DocumentHaworthPositionV1::Back)
        ) | (
            DocumentDirectHaworthBondRoleV1::Bridge,
            DocumentDirectHaworthBondTokenV1::N1,
            None
        )
    )
}
fn token_text(token: DocumentDirectHaworthBondTokenV1) -> &'static str {
    match token {
        DocumentDirectHaworthBondTokenV1::Q1 => "q1",
        DocumentDirectHaworthBondTokenV1::W1 => "w1",
        DocumentDirectHaworthBondTokenV1::N1 => "n1",
    }
}
fn position_text(position: DocumentHaworthPositionV1) -> &'static str {
    match position {
        DocumentHaworthPositionV1::Front => "front",
        DocumentHaworthPositionV1::Back => "back",
    }
}
fn invalid<T>(detail: impl Into<String>) -> Result<T, SessionOperationError> {
    Err(invalid_error(detail))
}
fn invalid_error(detail: impl Into<String>) -> SessionOperationError {
    SessionOperationError::InvalidDirectHaworthInsertion(detail.into())
}
