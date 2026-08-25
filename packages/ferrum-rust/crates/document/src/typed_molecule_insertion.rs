//! Structured XML mutation for complete, validated molecule insertions.

use std::collections::BTreeSet;

mod stereo;
mod xml;

pub(crate) use stereo::validate_document_stereo_semantics;
pub(crate) use xml::{InsertionNames, append_atom, valid_cdml_namespace};

use stereo::{
    MoleculeStereoReports, append_stereo_depictions, append_stereo_semantics, attribute,
    decode_molecule_stereo_reports, is_cdml_element, require_ez_depictions,
};
use xml::{append_interchange_metadata, xml_attribute_safe};

use super::{
    DocumentObjectIdV1, DocumentStereoDepictionReportV1, DocumentStereoSemanticReportV1,
    InterchangeRecordInsertionV1, MoleculeInsertionRequestV1, MoleculeInsertionV1, PersistentId,
    TypedDocument, TypedDocumentError, element_name,
};

struct MoleculeRecordInsertion<'a> {
    molecule_id: &'a PersistentId,
    atom_ids: &'a [PersistentId],
    bond_ids: &'a [PersistentId],
    molecule: &'a MoleculeInsertionV1,
    interchange_record: Option<&'a InterchangeRecordInsertionV1>,
    stereo_semantics: Option<&'a DocumentStereoSemanticReportV1>,
    stereo_depictions: Option<&'a DocumentStereoDepictionReportV1>,
}

impl TypedDocument {
    /// Return the admitted durable stereo semantics for one direct molecule.
    ///
    /// The descriptor is decoded from the retained canonical CDML rather than
    /// inferred from drawing wedges, so reopening a document retains source facts.
    pub fn molecule_stereo_semantics_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<Option<DocumentStereoSemanticReportV1>, TypedDocumentError> {
        self.molecule_stereo_reports_v1(molecule_id)
            .map(|reports| reports.and_then(|reports| reports.semantics))
    }

    /// Return the admitted durable stereo drawing facts for one direct molecule.
    ///
    /// This typed observation keeps projection and rendering consumers out of
    /// the raw CDML tree. A carrier mark remains a drawing fact and does not
    /// derive E/Z chemistry from geometry.
    pub fn molecule_stereo_depictions_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<Option<DocumentStereoDepictionReportV1>, TypedDocumentError> {
        self.molecule_stereo_reports_v1(molecule_id)
            .map(|reports| reports.and_then(|reports| reports.depictions))
    }

    fn molecule_stereo_reports_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<Option<MoleculeStereoReports>, TypedDocumentError> {
        let Some(record) = self.resolve_document_object_id(molecule_id) else {
            return Ok(None);
        };
        if record.class() != super::TypedClass::Molecule || record.path().components().len() != 1 {
            return Ok(None);
        }
        let Some(source_id) = record.attribute("id") else {
            return Ok(None);
        };
        let tree = &self.indexed().xml.tree;
        let root = tree
            .document_element(self.indexed().xml.document)
            .expect("a parsed CDML document has a document element");
        let molecule = tree.children(root).find(|node| {
            is_cdml_element(tree, *node, "molecule")
                && attribute(tree, *node, "id") == Some(source_id)
        });
        let Some(molecule) = molecule else {
            return Ok(None);
        };
        decode_molecule_stereo_reports(tree, molecule).map(Some)
    }

    pub(crate) fn with_insert_molecule(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        molecule: &MoleculeInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(MoleculeRecordInsertion {
            molecule_id,
            atom_ids,
            bond_ids,
            molecule,
            interchange_record: None,
            stereo_semantics: None,
            stereo_depictions: None,
        })
    }

    pub(crate) fn with_insert_molecule_request(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        request: &MoleculeInsertionRequestV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(MoleculeRecordInsertion {
            molecule_id,
            atom_ids,
            bond_ids,
            molecule: request.molecule(),
            interchange_record: None,
            stereo_semantics: request.stereo_semantics(),
            stereo_depictions: request.stereo_depictions(),
        })
    }

    pub(crate) fn with_insert_interchange_record(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        record: &InterchangeRecordInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(MoleculeRecordInsertion {
            molecule_id,
            atom_ids,
            bond_ids,
            molecule: record.request().molecule(),
            interchange_record: Some(record),
            stereo_semantics: record.request().stereo_semantics(),
            stereo_depictions: record.request().stereo_depictions(),
        })
    }

    fn with_insert_molecule_record(
        &self,
        request: MoleculeRecordInsertion<'_>,
    ) -> Result<Self, TypedDocumentError> {
        let MoleculeRecordInsertion {
            molecule_id,
            atom_ids,
            bond_ids,
            molecule,
            interchange_record,
            stereo_semantics,
            stereo_depictions,
        } = request;
        if atom_ids.len() != molecule.atoms().len() || bond_ids.len() != molecule.bonds().len() {
            return Err(TypedDocumentError::InsertionIdentityCountMismatch);
        }
        let mut supplied_ids = BTreeSet::new();
        for identifier in std::iter::once(molecule_id)
            .chain(atom_ids.iter())
            .chain(bond_ids.iter())
        {
            if self.indexed().resolve_id(identifier).is_some() || !supplied_ids.insert(identifier) {
                return Err(TypedDocumentError::DuplicateInsertionId(identifier.clone()));
            }
        }

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let namespace = valid_cdml_namespace(namespace);
        let names = InsertionNames::new(&mut indexed.xml.tree, namespace);
        let molecule_node = indexed.xml.tree.new_element(names.molecule);
        indexed
            .xml
            .tree
            .set_attribute(molecule_node, names.id, molecule_id.as_str());
        if let Some(name) = molecule.name() {
            indexed
                .xml
                .tree
                .set_attribute(molecule_node, names.name, name);
        } else if let Some(record) = interchange_record
            && !record.title().is_empty()
            && xml_attribute_safe(record.title())
        {
            indexed
                .xml
                .tree
                .set_attribute(molecule_node, names.name, record.title());
        }

        for (atom_id, atom) in atom_ids.iter().zip(molecule.atoms()) {
            append_atom(&mut indexed.xml.tree, molecule_node, &names, atom_id, atom)?;
        }
        for (bond_id, bond) in bond_ids.iter().zip(molecule.bonds()) {
            let bond_node = indexed.xml.tree.new_element(names.bond);
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.id, bond_id.as_str());
            indexed.xml.tree.set_attribute(
                bond_node,
                names.bond_type,
                bond.presentation().cdml_token(),
            );
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.start, atom_ids[bond.start()].as_str());
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.end, atom_ids[bond.end()].as_str());
            indexed
                .xml
                .tree
                .append(molecule_node, bond_node)
                .map_err(TypedDocumentError::Mutation)?;
        }
        if let Some(record) = interchange_record {
            append_interchange_metadata(&mut indexed.xml.tree, molecule_node, record)?;
        }
        let request = MoleculeInsertionRequestV1::with_stereo_reports(
            molecule.clone(),
            stereo_semantics.cloned(),
            stereo_depictions.cloned(),
        )
        .map_err(|_| TypedDocumentError::InvalidStereoSemantics)?;
        require_ez_depictions(request.stereo_semantics(), request.stereo_depictions())?;
        if let Some(stereo_semantics) = request.stereo_semantics() {
            append_stereo_semantics(
                &mut indexed.xml.tree,
                molecule_node,
                &names,
                molecule,
                stereo_semantics,
            )?;
        }
        if let Some(stereo_depictions) = request.stereo_depictions() {
            append_stereo_depictions(
                &mut indexed.xml.tree,
                molecule_node,
                &names,
                stereo_depictions,
            )?;
        }
        indexed
            .xml
            .tree
            .append(root, molecule_node)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}
