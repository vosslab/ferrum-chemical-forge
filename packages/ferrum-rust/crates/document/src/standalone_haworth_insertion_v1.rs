//! Private authoring boundary for one complete source-owned D-glucose Haworth drawing.

use ferrum_domain::haworth::{
    StandaloneDGlucoseHaworthReceiptV1, StandaloneHaworthBondTokenV1, StandaloneHaworthPositionV1,
};

use super::{
    DocumentHaworthPositionV1, MoleculeInsertionAtomV1, PersistentId, Point3V1,
    SessionOperationError, TypedDocument, TypedDocumentError,
    typed_molecule_insertion::{InsertionNames, append_atom, valid_cdml_namespace},
};

/// Persisted closed token for the standalone recipe family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentStandaloneHaworthBondTokenV1 {
    N1,
    Q1,
    W1,
}

#[derive(Clone, Debug)]
pub(crate) struct StandaloneHaworthInsertionV1 {
    atoms: Vec<MoleculeInsertionAtomV1>,
    bonds: Vec<Bond>,
}
#[derive(Clone, Debug)]
struct Bond {
    endpoints: [usize; 2],
    token: DocumentStandaloneHaworthBondTokenV1,
    position: Option<DocumentHaworthPositionV1>,
}

impl StandaloneHaworthInsertionV1 {
    pub(crate) fn from_receipt(
        receipt: &StandaloneDGlucoseHaworthReceiptV1,
        anchor: Point3V1,
    ) -> Result<Self, SessionOperationError> {
        if receipt.atoms().len() != 12 || receipt.bonds().len() != 12 {
            return Err(invalid(
                "recipe must contain twelve heavy atoms and twelve bonds",
            ));
        }
        let atoms = receipt
            .atoms()
            .iter()
            .map(|fact| {
                let local = fact.local();
                let point = Point3V1::new(local.x + anchor.x(), local.y + anchor.y(), anchor.z())
                    .map_err(|_| invalid("translated coordinate is not finite"))?;
                MoleculeInsertionAtomV1::new(fact.element(), point, None, None, None)
                    .map_err(|_| invalid("recipe atom is invalid"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bonds = receipt
            .bonds()
            .iter()
            .map(|fact| {
                if fact.start() >= atoms.len()
                    || fact.end() >= atoms.len()
                    || fact.start() == fact.end()
                {
                    return Err(invalid("recipe bond endpoint is invalid"));
                }
                Ok(Bond {
                    endpoints: [fact.start(), fact.end()],
                    token: token(fact.token()),
                    position: position(fact.position()),
                })
            })
            .collect::<Result<Vec<_>, SessionOperationError>>()?;
        Ok(Self { atoms, bonds })
    }
    pub(crate) fn atom_count(&self) -> usize {
        self.atoms.len()
    }
    pub(crate) fn bond_count(&self) -> usize {
        self.bonds.len()
    }
}

impl TypedDocument {
    pub(crate) fn with_insert_standalone_haworth(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        insertion: &StandaloneHaworthInsertionV1,
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
            append_atom(&mut indexed.xml.tree, molecule, &names, id, atom)?;
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
            if let Some(value) = bond.position {
                indexed
                    .xml
                    .tree
                    .set_attribute(node, names.haworth_position, position_text(value));
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

fn token(value: StandaloneHaworthBondTokenV1) -> DocumentStandaloneHaworthBondTokenV1 {
    match value {
        StandaloneHaworthBondTokenV1::N1 => DocumentStandaloneHaworthBondTokenV1::N1,
        StandaloneHaworthBondTokenV1::Q1 => DocumentStandaloneHaworthBondTokenV1::Q1,
        StandaloneHaworthBondTokenV1::W1 => DocumentStandaloneHaworthBondTokenV1::W1,
    }
}
fn position(value: Option<StandaloneHaworthPositionV1>) -> Option<DocumentHaworthPositionV1> {
    value.map(|item| match item {
        StandaloneHaworthPositionV1::Front => DocumentHaworthPositionV1::Front,
        StandaloneHaworthPositionV1::Back => DocumentHaworthPositionV1::Back,
    })
}
fn token_text(value: DocumentStandaloneHaworthBondTokenV1) -> &'static str {
    match value {
        DocumentStandaloneHaworthBondTokenV1::N1 => "n1",
        DocumentStandaloneHaworthBondTokenV1::Q1 => "q1",
        DocumentStandaloneHaworthBondTokenV1::W1 => "w1",
    }
}
fn position_text(value: DocumentHaworthPositionV1) -> &'static str {
    match value {
        DocumentHaworthPositionV1::Front => "front",
        DocumentHaworthPositionV1::Back => "back",
    }
}
fn invalid(detail: impl Into<String>) -> SessionOperationError {
    SessionOperationError::InvalidStandaloneHaworthInsertion(detail.into())
}
