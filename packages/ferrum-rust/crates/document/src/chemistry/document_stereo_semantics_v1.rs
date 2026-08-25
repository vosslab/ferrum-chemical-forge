//! Durable stereo descriptors and molecule-relative semantic admission.

use super::complete_graph_document_preparation::DocumentMoleculePreparationErrorV2;

use crate::{
    DocumentBondOrderV1, DocumentBondPresentationV1, MoleculeInsertionAtomV1, MoleculeInsertionV1,
};

use thiserror::Error;

/// One ordered ligand in a durable tetrahedral descriptor.
///
/// Atom ligands retain their zero-based graph position. `ExplicitHydrogen` is a
/// tagged value, never a synthetic atom position. The preparation convention
/// orders atom ligands by ascending atom position and places this sentinel last.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentStereoLigandV1 {
    /// One atom in the prepared molecule.
    Atom(usize),
    /// The one explicit hydrogen recorded on the tetrahedral center.
    ExplicitHydrogen,
}

/// The source-owned tetrahedral parity after applying the documented ligand order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentTetrahedralParityV1 {
    /// Clockwise source chirality.
    Clockwise,
    /// Counter-clockwise source chirality.
    CounterClockwise,
}

/// Durable tetrahedral molecular semantics, separate from wedge presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentTetrahedralStereoV1 {
    center: usize,
    ligands: [DocumentStereoLigandV1; 4],
    parity: DocumentTetrahedralParityV1,
}

impl DocumentTetrahedralStereoV1 {
    /// Construct a descriptor with four distinct ordered ligands.
    pub fn new(
        center: usize,
        ligands: [DocumentStereoLigandV1; 4],
        parity: DocumentTetrahedralParityV1,
    ) -> Result<Self, DocumentMoleculePreparationErrorV2> {
        let explicit_hydrogen_count = ligands
            .iter()
            .filter(|ligand| matches!(ligand, DocumentStereoLigandV1::ExplicitHydrogen))
            .count();
        let mut atoms = ligands
            .iter()
            .filter_map(|ligand| match ligand {
                DocumentStereoLigandV1::Atom(index) => Some(*index),
                DocumentStereoLigandV1::ExplicitHydrogen => None,
            })
            .collect::<Vec<_>>();
        atoms.sort_unstable();
        if explicit_hydrogen_count > 1 || atoms.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(DocumentMoleculePreparationErrorV2::UnrepresentableTetrahedral { center });
        }
        Ok(Self {
            center,
            ligands,
            parity,
        })
    }

    /// Return the tetrahedral center atom position.
    #[must_use]
    pub const fn center(&self) -> usize {
        self.center
    }

    /// Return ligands in the preparation ordering convention.
    #[must_use]
    pub const fn ligands(&self) -> &[DocumentStereoLigandV1; 4] {
        &self.ligands
    }

    /// Return the source parity after ordering the ligands.
    #[must_use]
    pub const fn parity(&self) -> DocumentTetrahedralParityV1 {
        self.parity
    }
}

/// The admitted E/Z configuration of one double bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentDoubleBondConfigurationV1 {
    /// The selected ligands are opposite across the double bond.
    E,
    /// The selected ligands are together across the double bond.
    Z,
}

/// Durable E/Z semantics for one source double bond.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentDoubleBondStereoV1 {
    bond_index: usize,
    start_ligand: usize,
    end_ligand: usize,
    configuration: DocumentDoubleBondConfigurationV1,
}

impl DocumentDoubleBondStereoV1 {
    /// Construct a descriptor whose graph-relative facts are validated by preparation.
    pub fn new(
        bond_index: usize,
        start_ligand: usize,
        end_ligand: usize,
        configuration: DocumentDoubleBondConfigurationV1,
    ) -> Result<Self, DocumentMoleculePreparationErrorV2> {
        if start_ligand == end_ligand {
            return Err(DocumentMoleculePreparationErrorV2::InvalidStereoReference { bond_index });
        }
        Ok(Self {
            bond_index,
            start_ligand,
            end_ligand,
            configuration,
        })
    }

    /// Return the source-order bond position.
    #[must_use]
    pub const fn bond_index(&self) -> usize {
        self.bond_index
    }

    /// Return the ligand adjacent to the bond start atom.
    #[must_use]
    pub const fn start_ligand(&self) -> usize {
        self.start_ligand
    }

    /// Return the ligand adjacent to the bond end atom.
    #[must_use]
    pub const fn end_ligand(&self) -> usize {
        self.end_ligand
    }

    /// Return the admitted E/Z configuration.
    #[must_use]
    pub const fn configuration(&self) -> DocumentDoubleBondConfigurationV1 {
        self.configuration
    }
}

/// Canonical directed presentation selected for an admitted tetrahedral descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentDirectedBondDepictionV1 {
    bond_index: usize,
    start: usize,
    end: usize,
    presentation: DocumentBondPresentationV1,
}

/// One authored carrier mark that draws an admitted E/Z double-bond descriptor.
///
/// This is a drawing fact only. Its associated `DocumentDoubleBondStereoV1`
/// remains the sole source of chemical configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentDoubleBondCarrierMarkV1 {
    /// The native carrier bond was authored with an up direction.
    Up,
    /// The native carrier bond was authored with a down direction.
    Down,
}

/// One native directional single-bond carrier retained as an E/Z drawing fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentDoubleBondCarrierMarkDepictionV1 {
    double_bond_index: usize,
    carrier_bond_index: usize,
    mark: DocumentDoubleBondCarrierMarkV1,
}

impl DocumentDoubleBondCarrierMarkDepictionV1 {
    /// Construct one explicit drawing association; graph facts are checked on admission.
    #[must_use]
    pub const fn new(
        double_bond_index: usize,
        carrier_bond_index: usize,
        mark: DocumentDoubleBondCarrierMarkV1,
    ) -> Self {
        Self {
            double_bond_index,
            carrier_bond_index,
            mark,
        }
    }

    /// Return the source-order E/Z double-bond position.
    #[must_use]
    pub const fn double_bond_index(&self) -> usize {
        self.double_bond_index
    }

    /// Return the source-order directional carrier-bond position.
    #[must_use]
    pub const fn carrier_bond_index(&self) -> usize {
        self.carrier_bond_index
    }

    /// Return the native directional mark retained for drawing.
    #[must_use]
    pub const fn mark(&self) -> DocumentDoubleBondCarrierMarkV1 {
        self.mark
    }
}

impl DocumentDirectedBondDepictionV1 {
    /// Construct one directed single-bond presentation with ordered endpoints.
    pub fn new(
        bond_index: usize,
        start: usize,
        end: usize,
        presentation: DocumentBondPresentationV1,
    ) -> Result<Self, DocumentMoleculePreparationErrorV2> {
        if start == end
            || !matches!(
                presentation,
                DocumentBondPresentationV1::SolidWedge | DocumentBondPresentationV1::HashedWedge
            )
        {
            return Err(
                DocumentMoleculePreparationErrorV2::UnrepresentableTetrahedral { center: start },
            );
        }
        Ok(Self {
            bond_index,
            start,
            end,
            presentation,
        })
    }

    /// Return the source-order bond position.
    #[must_use]
    pub const fn bond_index(&self) -> usize {
        self.bond_index
    }

    /// Return the ordered presentation endpoints.
    #[must_use]
    pub const fn endpoints(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    /// Return the selected wedge/hash presentation.
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.presentation
    }
}

/// Owned semantic report independent of CDML spelling and renderer inference.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentStereoSemanticReportV1 {
    tetrahedral: Vec<DocumentTetrahedralStereoV1>,
    double_bonds: Vec<DocumentDoubleBondStereoV1>,
}

/// Owned stereo drawing facts, distinct from chemical stereo semantics.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentStereoDepictionReportV1 {
    directed_bonds: Vec<DocumentDirectedBondDepictionV1>,
    double_bond_carrier_marks: Vec<DocumentDoubleBondCarrierMarkDepictionV1>,
}

/// A stereo report is not a valid document fact until it is related to a molecule.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("stereo semantics do not match the molecule graph")]
pub enum DocumentStereoSemanticsErrorV1 {
    /// A descriptor names an invalid, disconnected, duplicate, or noncanonical fact.
    Invalid,
}

impl DocumentStereoSemanticReportV1 {
    /// Construct one closed report from already-admitted descriptors.
    #[must_use]
    pub fn new(
        tetrahedral: Vec<DocumentTetrahedralStereoV1>,
        double_bonds: Vec<DocumentDoubleBondStereoV1>,
    ) -> Self {
        Self {
            tetrahedral,
            double_bonds,
        }
    }

    /// Return whether this report has no chemical stereo facts.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.tetrahedral.is_empty() && self.double_bonds.is_empty()
    }

    /// Return admitted tetrahedral descriptors in source order.
    #[must_use]
    pub fn tetrahedral(&self) -> &[DocumentTetrahedralStereoV1] {
        &self.tetrahedral
    }

    /// Return admitted E/Z descriptors in source order.
    #[must_use]
    pub fn double_bonds(&self) -> &[DocumentDoubleBondStereoV1] {
        &self.double_bonds
    }

    /// Validate graph-relative facts and return their canonical source ordering.
    ///
    /// This is the one admission rule used by prepared chemistry, generic
    /// insertion serialization, and CDML reopening.  Constructors can check
    /// local shape, but only this method can prove descriptor references
    /// describe this molecule.
    pub(crate) fn canonicalize_for_molecule(
        self,
        molecule: &MoleculeInsertionV1,
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        let hydrogens = molecule
            .atoms()
            .iter()
            .map(MoleculeInsertionAtomV1::explicit_hydrogens)
            .collect::<Vec<_>>();
        let bonds = molecule
            .bonds()
            .iter()
            .map(|bond| (bond.start(), bond.end(), bond.order()))
            .collect::<Vec<_>>();
        self.canonicalize_for_graph(&hydrogens, &bonds)
    }

    pub(crate) fn canonicalize_for_graph(
        mut self,
        explicit_hydrogens: &[Option<u16>],
        bonds: &[(usize, usize, DocumentBondOrderV1)],
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        for tetrahedral in &self.tetrahedral {
            let center = tetrahedral.center();
            let Some(center_hydrogens) = explicit_hydrogens.get(center) else {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            };
            let mut previous_atom = None;
            let mut explicit_hydrogen_count = 0;
            for (ligand_position, ligand) in tetrahedral.ligands().iter().enumerate() {
                match ligand {
                    DocumentStereoLigandV1::Atom(index) => {
                        if *index == center
                            || *index >= explicit_hydrogens.len()
                            || previous_atom.is_some_and(|previous| previous >= *index)
                            || !bonds.iter().any(|(start, end, _)| {
                                (*start == center && *end == *index)
                                    || (*end == center && *start == *index)
                            })
                        {
                            return Err(DocumentStereoSemanticsErrorV1::Invalid);
                        }
                        previous_atom = Some(*index);
                    }
                    DocumentStereoLigandV1::ExplicitHydrogen => {
                        if ligand_position != 3 {
                            return Err(DocumentStereoSemanticsErrorV1::Invalid);
                        }
                        explicit_hydrogen_count += 1;
                    }
                }
            }
            if explicit_hydrogen_count > 1
                || (explicit_hydrogen_count == 1 && previous_atom.is_none())
                || match center_hydrogens {
                    Some(1) => explicit_hydrogen_count != 1,
                    None => explicit_hydrogen_count != 0,
                    Some(_) => true,
                }
            {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            }
        }
        for double_bond in &self.double_bonds {
            let Some((start, end, order)) = bonds.get(double_bond.bond_index()) else {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            };
            let is_neighbor = |endpoint: usize, ligand: usize| {
                ligand != *start
                    && ligand != *end
                    && ligand < explicit_hydrogens.len()
                    && bonds.iter().any(|(candidate_start, candidate_end, _)| {
                        (*candidate_start == endpoint && *candidate_end == ligand)
                            || (*candidate_end == endpoint && *candidate_start == ligand)
                    })
            };
            if *order != DocumentBondOrderV1::Double
                || double_bond.start_ligand() == double_bond.end_ligand()
                || !is_neighbor(*start, double_bond.start_ligand())
                || !is_neighbor(*end, double_bond.end_ligand())
            {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            }
        }
        self.tetrahedral
            .sort_by_key(DocumentTetrahedralStereoV1::center);
        self.double_bonds
            .sort_by_key(DocumentDoubleBondStereoV1::bond_index);
        if self
            .tetrahedral
            .windows(2)
            .any(|pair| pair[0].center() == pair[1].center())
            || self
                .double_bonds
                .windows(2)
                .any(|pair| pair[0].bond_index() == pair[1].bond_index())
        {
            return Err(DocumentStereoSemanticsErrorV1::Invalid);
        }
        Ok(self)
    }
}

impl DocumentStereoDepictionReportV1 {
    /// Construct one closed report from already-admitted drawing facts.
    #[must_use]
    pub fn new(
        directed_bonds: Vec<DocumentDirectedBondDepictionV1>,
        double_bond_carrier_marks: Vec<DocumentDoubleBondCarrierMarkDepictionV1>,
    ) -> Self {
        Self {
            directed_bonds,
            double_bond_carrier_marks,
        }
    }

    /// Return whether this report has no authored stereo depictions.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.directed_bonds.is_empty() && self.double_bond_carrier_marks.is_empty()
    }

    /// Return canonical tetrahedral wedge/hash depictions.
    #[must_use]
    pub fn directed_bonds(&self) -> &[DocumentDirectedBondDepictionV1] {
        &self.directed_bonds
    }

    /// Return canonical E/Z carrier-mark depictions.
    #[must_use]
    pub fn double_bond_carrier_marks(&self) -> &[DocumentDoubleBondCarrierMarkDepictionV1] {
        &self.double_bond_carrier_marks
    }

    fn canonicalize_for_graph(
        mut self,
        semantics: Option<&DocumentStereoSemanticReportV1>,
        atom_count: usize,
        bonds: &[(usize, usize, DocumentBondOrderV1)],
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        for directed_bond in &self.directed_bonds {
            let Some((start, end, order)) = bonds.get(directed_bond.bond_index()) else {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            };
            if *order != DocumentBondOrderV1::Single || directed_bond.endpoints() != (*start, *end)
            {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            }
        }
        self.directed_bonds
            .sort_by_key(DocumentDirectedBondDepictionV1::bond_index);
        if self
            .directed_bonds
            .windows(2)
            .any(|pair| pair[0].bond_index() == pair[1].bond_index())
        {
            return Err(DocumentStereoSemanticsErrorV1::Invalid);
        }

        let Some(semantics) = semantics else {
            if self.double_bond_carrier_marks.is_empty() {
                return Ok(self);
            }
            return Err(DocumentStereoSemanticsErrorV1::Invalid);
        };
        for mark in &self.double_bond_carrier_marks {
            let Some(stereo) = semantics
                .double_bonds()
                .iter()
                .find(|stereo| stereo.bond_index() == mark.double_bond_index())
            else {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            };
            let Some((double_start, double_end, double_order)) =
                bonds.get(mark.double_bond_index())
            else {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            };
            let Some((carrier_start, carrier_end, carrier_order)) =
                bonds.get(mark.carrier_bond_index())
            else {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            };
            if mark.double_bond_index() == mark.carrier_bond_index()
                || *double_order != DocumentBondOrderV1::Double
                || *carrier_order != DocumentBondOrderV1::Single
                || *carrier_start >= atom_count
                || *carrier_end >= atom_count
                || !carrier_matches_double_bond_ligand(
                    *double_start,
                    *double_end,
                    stereo.start_ligand(),
                    stereo.end_ligand(),
                    *carrier_start,
                    *carrier_end,
                )
            {
                return Err(DocumentStereoSemanticsErrorV1::Invalid);
            }
        }
        self.double_bond_carrier_marks
            .sort_by_key(|mark| (mark.double_bond_index(), mark.carrier_bond_index()));
        if self.double_bond_carrier_marks.windows(2).any(|pair| {
            pair[0].double_bond_index() == pair[1].double_bond_index()
                && pair[0].carrier_bond_index() == pair[1].carrier_bond_index()
        }) {
            return Err(DocumentStereoSemanticsErrorV1::Invalid);
        }
        Ok(self)
    }
}

fn carrier_matches_double_bond_ligand(
    double_start: usize,
    double_end: usize,
    start_ligand: usize,
    end_ligand: usize,
    carrier_start: usize,
    carrier_end: usize,
) -> bool {
    (carrier_start == double_start && carrier_end == start_ligand)
        || (carrier_end == double_start && carrier_start == start_ligand)
        || (carrier_start == double_end && carrier_end == end_ligand)
        || (carrier_end == double_end && carrier_start == end_ligand)
}

pub(crate) fn canonicalize_stereo_reports_for_molecule(
    molecule: &MoleculeInsertionV1,
    semantics: Option<DocumentStereoSemanticReportV1>,
    depictions: Option<DocumentStereoDepictionReportV1>,
) -> Result<
    (
        Option<DocumentStereoSemanticReportV1>,
        Option<DocumentStereoDepictionReportV1>,
    ),
    DocumentStereoSemanticsErrorV1,
> {
    if semantics
        .as_ref()
        .is_some_and(DocumentStereoSemanticReportV1::is_empty)
        || depictions
            .as_ref()
            .is_some_and(DocumentStereoDepictionReportV1::is_empty)
    {
        return Err(DocumentStereoSemanticsErrorV1::Invalid);
    }
    let hydrogens = molecule
        .atoms()
        .iter()
        .map(MoleculeInsertionAtomV1::explicit_hydrogens)
        .collect::<Vec<_>>();
    let bonds = molecule
        .bonds()
        .iter()
        .map(|bond| (bond.start(), bond.end(), bond.order()))
        .collect::<Vec<_>>();
    let semantics = semantics
        .map(|semantics| semantics.canonicalize_for_graph(&hydrogens, &bonds))
        .transpose()?;
    let depictions = depictions
        .map(|depictions| {
            depictions.canonicalize_for_graph(semantics.as_ref(), hydrogens.len(), &bonds)
        })
        .transpose()?;
    Ok((semantics, depictions))
}

/// A fully checked, detached molecule payload ready for generic document insertion.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedDocumentMoleculeV2 {
    molecule_insertion: MoleculeInsertionV1,
    stereo_semantics: Option<DocumentStereoSemanticReportV1>,
    stereo_depictions: Option<DocumentStereoDepictionReportV1>,
}

impl PreparedDocumentMoleculeV2 {
    /// Combine an ordinary insertion payload with no stereo facts of either kind.
    #[must_use]
    pub fn new(molecule_insertion: MoleculeInsertionV1) -> Self {
        Self {
            molecule_insertion,
            stereo_semantics: None,
            stereo_depictions: None,
        }
    }

    /// Combine an insertion payload with chemical stereo facts and no depictions.
    pub fn with_stereo_semantics(
        molecule_insertion: MoleculeInsertionV1,
        stereo_semantics: DocumentStereoSemanticReportV1,
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        Self::with_stereo_reports(molecule_insertion, Some(stereo_semantics), None)
    }

    /// Combine an insertion payload with independently owned semantic and depiction reports.
    pub fn with_stereo_reports(
        molecule_insertion: MoleculeInsertionV1,
        stereo_semantics: Option<DocumentStereoSemanticReportV1>,
        stereo_depictions: Option<DocumentStereoDepictionReportV1>,
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        let (stereo_semantics, stereo_depictions) = canonicalize_stereo_reports_for_molecule(
            &molecule_insertion,
            stereo_semantics,
            stereo_depictions,
        )?;
        Ok(Self {
            molecule_insertion,
            stereo_semantics,
            stereo_depictions,
        })
    }

    /// Return the existing generic insertion payload without allocating document identity.
    #[must_use]
    pub const fn molecule_insertion(&self) -> &MoleculeInsertionV1 {
        &self.molecule_insertion
    }

    /// Return document-owned chemical facts without inspecting presentation geometry.
    #[must_use]
    pub const fn stereo_semantics(&self) -> Option<&DocumentStereoSemanticReportV1> {
        self.stereo_semantics.as_ref()
    }

    /// Return document-owned drawing facts without deriving chemical configuration.
    #[must_use]
    pub const fn stereo_depictions(&self) -> Option<&DocumentStereoDepictionReportV1> {
        self.stereo_depictions.as_ref()
    }

    /// Consume this detached request into the generic insertion request.
    pub fn into_molecule_insertion_request_v1(
        self,
    ) -> Result<crate::MoleculeInsertionRequestV1, DocumentStereoSemanticsErrorV1> {
        crate::MoleculeInsertionRequestV1::with_stereo_reports(
            self.molecule_insertion,
            self.stereo_semantics,
            self.stereo_depictions,
        )
    }
}
