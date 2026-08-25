//! Native molecular identity and connectivity planning for bounded peptides.
//!
//! The plan deliberately owns chemistry facts only.  A document adapter later
//! assigns persistent identifiers and layout, preserving this module as the
//! sole owner of residue-derived connectivity, charge, and stereochemistry.

use thiserror::Error;

use super::{PeptideSequence, ResidueCode};

/// The only native peptide graph profile admitted by this foundation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FerrumPeptideProfileV1 {
    /// The existing native-17 residue scope with zwitterionic free termini.
    Native17ZwitterionicTermini,
}

impl FerrumPeptideProfileV1 {
    /// Return this closed profile's stable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Native17ZwitterionicTermini => "ferrum-native-peptide-structure-v1",
        }
    }
}

/// A one-based N-to-C position in the source peptide sequence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PeptideResidueIndexV1(usize);

impl PeptideResidueIndexV1 {
    /// Return the one-based source-sequence position.
    #[must_use]
    pub const fn one_based(self) -> usize {
        self.0
    }
}

/// An atom role within one ordered peptide residue.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PeptideAtomSiteV1 {
    /// The backbone amino nitrogen.
    AminoNitrogen,
    /// The backbone alpha carbon.
    AlphaCarbon,
    /// The backbone carbonyl carbon.
    CarbonylCarbon,
    /// The backbone carbonyl oxygen.
    CarbonylOxygen,
    /// The free C-terminal singly bonded oxide oxygen.
    CarboxylateOxygen,
    /// A deterministic side-chain atom position, ordered from its attachment.
    SideChain(u8),
}

/// A stable semantic atom identity before document identifiers exist.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PeptideAtomIdV1 {
    residue: PeptideResidueIndexV1,
    site: PeptideAtomSiteV1,
}

impl PeptideAtomIdV1 {
    /// Return the owning source-residue position.
    #[must_use]
    pub const fn residue(self) -> PeptideResidueIndexV1 {
        self.residue
    }

    /// Return this atom's residue-local role.
    #[must_use]
    pub const fn site(self) -> PeptideAtomSiteV1 {
        self.site
    }
}

/// Closed atom elements used by the native-17 peptide profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeptideAtomElementV1 {
    Carbon,
    Nitrogen,
    Oxygen,
    Sulfur,
}

/// Formal-charge facts, independent from terminal presentation or layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeptideFormalChargeV1 {
    Neutral,
    PositiveOne,
    NegativeOne,
}

/// Tetrahedral facts owned by an atom's residue recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeptideAtomStereochemistryV1 {
    Unspecified,
    TetrahedralS,
    TetrahedralR,
}

/// One immutable native peptide atom fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeptideStructureAtomV1 {
    id: PeptideAtomIdV1,
    element: PeptideAtomElementV1,
    formal_charge: PeptideFormalChargeV1,
    stereochemistry: PeptideAtomStereochemistryV1,
}

impl PeptideStructureAtomV1 {
    #[must_use]
    const fn new(
        id: PeptideAtomIdV1,
        element: PeptideAtomElementV1,
        formal_charge: PeptideFormalChargeV1,
        stereochemistry: PeptideAtomStereochemistryV1,
    ) -> Self {
        Self {
            id,
            element,
            formal_charge,
            stereochemistry,
        }
    }

    #[must_use]
    pub const fn id(self) -> PeptideAtomIdV1 {
        self.id
    }

    #[must_use]
    pub const fn element(self) -> PeptideAtomElementV1 {
        self.element
    }

    #[must_use]
    pub const fn formal_charge(self) -> PeptideFormalChargeV1 {
        self.formal_charge
    }

    #[must_use]
    pub const fn stereochemistry(self) -> PeptideAtomStereochemistryV1 {
        self.stereochemistry
    }
}

/// A deterministic bond identity within one plan.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PeptideBondIdV1(usize);

impl PeptideBondIdV1 {
    #[must_use]
    pub const fn zero_based(self) -> usize {
        self.0
    }
}

/// A closed covalent bond order for a native peptide graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeptideBondOrderV1 {
    Single,
    Double,
    Aromatic,
}

/// Semantic ownership of one native peptide bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeptideBondRoleV1 {
    Backbone,
    PeptideLink,
    SideChain,
}

/// One immutable connectivity fact between two semantic atom identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeptideBondV1 {
    id: PeptideBondIdV1,
    start: PeptideAtomIdV1,
    end: PeptideAtomIdV1,
    order: PeptideBondOrderV1,
    role: PeptideBondRoleV1,
}

impl PeptideBondV1 {
    #[must_use]
    const fn new(
        id: PeptideBondIdV1,
        start: PeptideAtomIdV1,
        end: PeptideAtomIdV1,
        order: PeptideBondOrderV1,
        role: PeptideBondRoleV1,
    ) -> Self {
        Self {
            id,
            start,
            end,
            order,
            role,
        }
    }

    #[must_use]
    pub const fn id(self) -> PeptideBondIdV1 {
        self.id
    }

    #[must_use]
    pub const fn start(self) -> PeptideAtomIdV1 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> PeptideAtomIdV1 {
        self.end
    }

    #[must_use]
    pub const fn order(self) -> PeptideBondOrderV1 {
        self.order
    }

    #[must_use]
    pub const fn role(self) -> PeptideBondRoleV1 {
        self.role
    }
}

/// Immutable molecular semantics for a future document preparation adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeptideStructurePlanV1 {
    profile: FerrumPeptideProfileV1,
    atoms: Vec<PeptideStructureAtomV1>,
    bonds: Vec<PeptideBondV1>,
}

impl PeptideStructurePlanV1 {
    #[must_use]
    const fn new(
        profile: FerrumPeptideProfileV1,
        atoms: Vec<PeptideStructureAtomV1>,
        bonds: Vec<PeptideBondV1>,
    ) -> Self {
        Self {
            profile,
            atoms,
            bonds,
        }
    }

    #[must_use]
    pub const fn profile(&self) -> FerrumPeptideProfileV1 {
        self.profile
    }

    #[must_use]
    pub fn atoms(&self) -> &[PeptideStructureAtomV1] {
        &self.atoms
    }

    #[must_use]
    pub fn bonds(&self) -> &[PeptideBondV1] {
        &self.bonds
    }
}

/// Typed refusal or resource exhaustion from native peptide graph planning.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PeptideStructurePlanErrorV1 {
    #[error(
        "residue {residue} at position {position} is unsupported by native peptide structure \
         profile {profile}"
    )]
    UnsupportedResidue {
        position: usize,
        residue: ResidueCode,
        profile: &'static str,
    },
    #[error("native peptide structure planning could not reserve graph storage")]
    AllocationFailed,
}

/// Build direct molecular semantics for a prevalidated N-to-C peptide sequence.
pub fn build_peptide_structure_plan_v1(
    sequence: &PeptideSequence,
    profile: FerrumPeptideProfileV1,
) -> Result<PeptideStructurePlanV1, PeptideStructurePlanErrorV1> {
    let mut builder = PeptideStructurePlanBuilderV1::new(sequence.len())?;
    let mut previous_carbonyl = None;
    for (offset, residue) in sequence.residues().iter().copied().enumerate() {
        let position = PeptideResidueIndexV1(offset + 1);
        if !supports(profile, residue) {
            return Err(PeptideStructurePlanErrorV1::UnsupportedResidue {
                position: position.one_based(),
                residue,
                profile: profile.name(),
            });
        }
        let backbone =
            builder.add_backbone(position, residue, offset == 0, offset + 1 == sequence.len())?;
        if let Some(prior) = previous_carbonyl {
            builder.add_bond(
                prior,
                backbone.amino,
                PeptideBondOrderV1::Single,
                PeptideBondRoleV1::PeptideLink,
            )?;
        }
        builder.add_side_chain(position, residue, backbone.alpha)?;
        previous_carbonyl = Some(backbone.carbonyl);
    }
    Ok(PeptideStructurePlanV1::new(
        profile,
        builder.atoms,
        builder.bonds,
    ))
}

#[derive(Clone, Copy)]
struct PeptideBackboneV1 {
    amino: PeptideAtomIdV1,
    alpha: PeptideAtomIdV1,
    carbonyl: PeptideAtomIdV1,
}

struct PeptideStructurePlanBuilderV1 {
    atoms: Vec<PeptideStructureAtomV1>,
    bonds: Vec<PeptideBondV1>,
    next_bond: usize,
}

impl PeptideStructurePlanBuilderV1 {
    fn new(residue_count: usize) -> Result<Self, PeptideStructurePlanErrorV1> {
        let atom_capacity = residue_count
            .checked_mul(12)
            .ok_or(PeptideStructurePlanErrorV1::AllocationFailed)?;
        let bond_capacity = residue_count
            .checked_mul(13)
            .ok_or(PeptideStructurePlanErrorV1::AllocationFailed)?;
        let mut atoms = Vec::new();
        let mut bonds = Vec::new();
        atoms
            .try_reserve(atom_capacity)
            .map_err(|_| PeptideStructurePlanErrorV1::AllocationFailed)?;
        bonds
            .try_reserve(bond_capacity)
            .map_err(|_| PeptideStructurePlanErrorV1::AllocationFailed)?;
        Ok(Self {
            atoms,
            bonds,
            next_bond: 0,
        })
    }

    fn add_backbone(
        &mut self,
        residue: PeptideResidueIndexV1,
        code: ResidueCode,
        is_n_terminus: bool,
        is_c_terminus: bool,
    ) -> Result<PeptideBackboneV1, PeptideStructurePlanErrorV1> {
        let amino = self.add_atom(
            residue,
            PeptideAtomSiteV1::AminoNitrogen,
            PeptideAtomElementV1::Nitrogen,
            if is_n_terminus {
                PeptideFormalChargeV1::PositiveOne
            } else {
                PeptideFormalChargeV1::Neutral
            },
            PeptideAtomStereochemistryV1::Unspecified,
        )?;
        let alpha = self.add_atom(
            residue,
            PeptideAtomSiteV1::AlphaCarbon,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            alpha_stereo(code),
        )?;
        let carbonyl = self.add_atom(
            residue,
            PeptideAtomSiteV1::CarbonylCarbon,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
        )?;
        let oxygen = self.add_atom(
            residue,
            PeptideAtomSiteV1::CarbonylOxygen,
            PeptideAtomElementV1::Oxygen,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
        )?;
        self.add_bond(
            amino,
            alpha,
            PeptideBondOrderV1::Single,
            PeptideBondRoleV1::Backbone,
        )?;
        self.add_bond(
            alpha,
            carbonyl,
            PeptideBondOrderV1::Single,
            PeptideBondRoleV1::Backbone,
        )?;
        self.add_bond(
            carbonyl,
            oxygen,
            PeptideBondOrderV1::Double,
            PeptideBondRoleV1::Backbone,
        )?;
        if is_c_terminus {
            let oxide = self.add_atom(
                residue,
                PeptideAtomSiteV1::CarboxylateOxygen,
                PeptideAtomElementV1::Oxygen,
                PeptideFormalChargeV1::NegativeOne,
                PeptideAtomStereochemistryV1::Unspecified,
            )?;
            self.add_bond(
                carbonyl,
                oxide,
                PeptideBondOrderV1::Single,
                PeptideBondRoleV1::Backbone,
            )?;
        }
        Ok(PeptideBackboneV1 {
            amino,
            alpha,
            carbonyl,
        })
    }

    fn add_side_chain(
        &mut self,
        residue: PeptideResidueIndexV1,
        code: ResidueCode,
        alpha: PeptideAtomIdV1,
    ) -> Result<(), PeptideStructurePlanErrorV1> {
        let mut side_chain = SideChainBuilderV1::new(self, residue, alpha);
        match code {
            ResidueCode::Alanine => {
                side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
            }
            ResidueCode::Cysteine => {
                let carbon = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.sulfur_from(carbon)?;
            }
            ResidueCode::AsparticAcid => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let carbonyl =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.oxygen_double_from(carbonyl)?;
                side_chain.oxygen_negative_from(carbonyl)?;
            }
            ResidueCode::GlutamicAcid => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                let carbonyl =
                    side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.oxygen_double_from(carbonyl)?;
                side_chain.oxygen_negative_from(carbonyl)?;
            }
            ResidueCode::Phenylalanine => side_chain.phenyl(None)?,
            ResidueCode::Glycine => {}
            ResidueCode::Isoleucine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::TetrahedralS)?;
                side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
            }
            ResidueCode::Lysine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                let delta =
                    side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
                let epsilon =
                    side_chain.carbon_from(delta, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.nitrogen_positive_from(epsilon)?;
            }
            ResidueCode::Leucine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
            }
            ResidueCode::Methionine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                let sulfur = side_chain.sulfur_from(gamma)?;
                side_chain.carbon_from(sulfur, PeptideAtomStereochemistryV1::Unspecified)?;
            }
            ResidueCode::Asparagine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let carbonyl =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.oxygen_double_from(carbonyl)?;
                side_chain.nitrogen_from(carbonyl)?;
            }
            ResidueCode::Glutamine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                let carbonyl =
                    side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.oxygen_double_from(carbonyl)?;
                side_chain.nitrogen_from(carbonyl)?;
            }
            ResidueCode::Arginine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                let gamma =
                    side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                let delta =
                    side_chain.carbon_from(gamma, PeptideAtomStereochemistryV1::Unspecified)?;
                let nitrogen = side_chain.nitrogen_from(delta)?;
                let guanidino =
                    side_chain.carbon_from(nitrogen, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.nitrogen_double_positive_from(guanidino)?;
                side_chain.nitrogen_from(guanidino)?;
            }
            ResidueCode::Serine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.oxygen_from(beta)?;
            }
            ResidueCode::Threonine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::TetrahedralR)?;
                side_chain.oxygen_from(beta)?;
                side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
            }
            ResidueCode::Valine => {
                let beta = side_chain.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
                side_chain.carbon_from(beta, PeptideAtomStereochemistryV1::Unspecified)?;
            }
            ResidueCode::Tyrosine => side_chain.phenyl(Some(PeptideAtomElementV1::Oxygen))?,
            ResidueCode::Histidine | ResidueCode::Proline | ResidueCode::Tryptophan => {
                unreachable!("profile admission excludes this residue")
            }
        }
        Ok(())
    }

    fn add_atom(
        &mut self,
        residue: PeptideResidueIndexV1,
        site: PeptideAtomSiteV1,
        element: PeptideAtomElementV1,
        formal_charge: PeptideFormalChargeV1,
        stereochemistry: PeptideAtomStereochemistryV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atoms
            .try_reserve(1)
            .map_err(|_| PeptideStructurePlanErrorV1::AllocationFailed)?;
        let id = PeptideAtomIdV1 { residue, site };
        self.atoms.push(PeptideStructureAtomV1::new(
            id,
            element,
            formal_charge,
            stereochemistry,
        ));
        Ok(id)
    }

    fn add_bond(
        &mut self,
        start: PeptideAtomIdV1,
        end: PeptideAtomIdV1,
        order: PeptideBondOrderV1,
        role: PeptideBondRoleV1,
    ) -> Result<(), PeptideStructurePlanErrorV1> {
        self.bonds
            .try_reserve(1)
            .map_err(|_| PeptideStructurePlanErrorV1::AllocationFailed)?;
        let id = PeptideBondIdV1(self.next_bond);
        self.next_bond = self
            .next_bond
            .checked_add(1)
            .ok_or(PeptideStructurePlanErrorV1::AllocationFailed)?;
        self.bonds
            .push(PeptideBondV1::new(id, start, end, order, role));
        Ok(())
    }
}

struct SideChainBuilderV1<'a> {
    plan: &'a mut PeptideStructurePlanBuilderV1,
    residue: PeptideResidueIndexV1,
    next_site: u8,
    attachment: PeptideAtomIdV1,
}

impl<'a> SideChainBuilderV1<'a> {
    const fn new(
        plan: &'a mut PeptideStructurePlanBuilderV1,
        residue: PeptideResidueIndexV1,
        attachment: PeptideAtomIdV1,
    ) -> Self {
        Self {
            plan,
            residue,
            next_site: 1,
            attachment,
        }
    }

    fn atom_from(
        &mut self,
        parent: PeptideAtomIdV1,
        element: PeptideAtomElementV1,
        charge: PeptideFormalChargeV1,
        stereo: PeptideAtomStereochemistryV1,
        order: PeptideBondOrderV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        let site = PeptideAtomSiteV1::SideChain(self.next_site);
        self.next_site = self
            .next_site
            .checked_add(1)
            .ok_or(PeptideStructurePlanErrorV1::AllocationFailed)?;
        let id = self
            .plan
            .add_atom(self.residue, site, element, charge, stereo)?;
        self.plan
            .add_bond(parent, id, order, PeptideBondRoleV1::SideChain)?;
        Ok(id)
    }

    fn carbon(
        &mut self,
        stereo: PeptideAtomStereochemistryV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            self.attachment,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            stereo,
            PeptideBondOrderV1::Single,
        )
    }
    fn carbon_from(
        &mut self,
        parent: PeptideAtomIdV1,
        stereo: PeptideAtomStereochemistryV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            stereo,
            PeptideBondOrderV1::Single,
        )
    }
    fn sulfur_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Sulfur,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Single,
        )
    }
    fn oxygen_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Oxygen,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Single,
        )
    }
    fn oxygen_double_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Oxygen,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Double,
        )
    }
    fn oxygen_negative_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Oxygen,
            PeptideFormalChargeV1::NegativeOne,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Single,
        )
    }
    fn nitrogen_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Nitrogen,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Single,
        )
    }
    fn nitrogen_positive_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Nitrogen,
            PeptideFormalChargeV1::PositiveOne,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Single,
        )
    }
    fn nitrogen_double_positive_from(
        &mut self,
        parent: PeptideAtomIdV1,
    ) -> Result<PeptideAtomIdV1, PeptideStructurePlanErrorV1> {
        self.atom_from(
            parent,
            PeptideAtomElementV1::Nitrogen,
            PeptideFormalChargeV1::PositiveOne,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Double,
        )
    }

    fn phenyl(
        &mut self,
        phenol: Option<PeptideAtomElementV1>,
    ) -> Result<(), PeptideStructurePlanErrorV1> {
        let methylene = self.carbon(PeptideAtomStereochemistryV1::Unspecified)?;
        let first = self.atom_from(
            methylene,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Single,
        )?;
        let second = self.atom_from(
            first,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Aromatic,
        )?;
        let third = self.atom_from(
            second,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Aromatic,
        )?;
        let fourth = self.atom_from(
            third,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Aromatic,
        )?;
        let fifth = self.atom_from(
            fourth,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Aromatic,
        )?;
        let sixth = self.atom_from(
            fifth,
            PeptideAtomElementV1::Carbon,
            PeptideFormalChargeV1::Neutral,
            PeptideAtomStereochemistryV1::Unspecified,
            PeptideBondOrderV1::Aromatic,
        )?;
        self.plan.add_bond(
            sixth,
            first,
            PeptideBondOrderV1::Aromatic,
            PeptideBondRoleV1::SideChain,
        )?;
        if let Some(element) = phenol {
            self.atom_from(
                fourth,
                element,
                PeptideFormalChargeV1::Neutral,
                PeptideAtomStereochemistryV1::Unspecified,
                PeptideBondOrderV1::Single,
            )?;
        }
        Ok(())
    }
}

const fn supports(profile: FerrumPeptideProfileV1, residue: ResidueCode) -> bool {
    match profile {
        FerrumPeptideProfileV1::Native17ZwitterionicTermini => matches!(
            residue,
            ResidueCode::Alanine
                | ResidueCode::Cysteine
                | ResidueCode::AsparticAcid
                | ResidueCode::GlutamicAcid
                | ResidueCode::Phenylalanine
                | ResidueCode::Glycine
                | ResidueCode::Isoleucine
                | ResidueCode::Lysine
                | ResidueCode::Leucine
                | ResidueCode::Methionine
                | ResidueCode::Asparagine
                | ResidueCode::Glutamine
                | ResidueCode::Arginine
                | ResidueCode::Serine
                | ResidueCode::Threonine
                | ResidueCode::Valine
                | ResidueCode::Tyrosine
        ),
    }
}

const fn alpha_stereo(residue: ResidueCode) -> PeptideAtomStereochemistryV1 {
    match residue {
        ResidueCode::Glycine => PeptideAtomStereochemistryV1::Unspecified,
        ResidueCode::Cysteine => PeptideAtomStereochemistryV1::TetrahedralR,
        _ => PeptideAtomStereochemistryV1::TetrahedralS,
    }
}
