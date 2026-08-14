//! Checked in-process replacement of selected molecule bond outcomes.

use std::collections::HashSet;

use ferrum_core::{RecordId, RecordKind};
use thiserror::Error;

use crate::{
    AuthoredDirectGlycosidicHaworthRenderPlanV1, DocumentRenderContentV1, DocumentRenderIdentityV1,
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, RenderTarget,
};

type TargetKey = (RecordId, u32);

/// An opaque in-memory composition; it intentionally has no wire representation.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderCompositeV1 {
    established: DocumentRenderPlanV1,
    replacement: BondReplacementV1,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BondReplacementV1 {
    root_identity: DocumentRenderIdentityV1,
    root_order: u32,
    selected_bonds: Vec<RenderTarget>,
    selected_keys: HashSet<TargetKey>,
    direct: AuthoredDirectGlycosidicHaworthRenderPlanV1,
}

impl DocumentRenderCompositeV1 {
    #[must_use]
    pub const fn page(&self) -> crate::RenderViewportV1 {
        self.established.page()
    }
    #[must_use]
    pub const fn provenance(&self) -> crate::RenderProvenance {
        self.established.provenance()
    }
}

/// Failures of the renderer-owned bond replacement seam.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DocumentBondReplacementErrorV1 {
    #[error("direct and established render provenance differ")]
    ProvenanceMismatch,
    #[error("authenticated root is absent, duplicated, or at a different order")]
    AuthenticatedRootMismatch,
    #[error("authenticated root is not a molecule")]
    RootIsNotMolecule,
    #[error("selected target is not a bond")]
    NonBondSelection,
    #[error("selected bonds are not complete molecule outcomes")]
    SelectedBondOutcomeMismatch,
    #[error("direct operations do not exactly match selected bond targets")]
    DirectOperationTargetMismatch,
    #[error("could not allocate checked replacement membership")]
    ResourceExhausted,
}

/// Validate and retain one whole-molecule selected-bond replacement.
pub fn compose_document_bond_replacement_v1(
    established: DocumentRenderPlanV1,
    authenticated_root: DocumentRenderIdentityV1,
    expected_root_order: u32,
    selected_bonds: Vec<RenderTarget>,
    direct: AuthoredDirectGlycosidicHaworthRenderPlanV1,
) -> Result<DocumentRenderCompositeV1, DocumentBondReplacementErrorV1> {
    if established.provenance() != direct.provenance() {
        return Err(DocumentBondReplacementErrorV1::ProvenanceMismatch);
    }
    let mut matching = established
        .outcomes()
        .iter()
        .filter_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root) if root.identity() == &authenticated_root => {
                Some(root)
            }
            _ => None,
        });
    let Some(root) = matching.next() else {
        return Err(DocumentBondReplacementErrorV1::AuthenticatedRootMismatch);
    };
    if matching.next().is_some() {
        return Err(DocumentBondReplacementErrorV1::AuthenticatedRootMismatch);
    }
    if root.source_order() != expected_root_order {
        return Err(DocumentBondReplacementErrorV1::AuthenticatedRootMismatch);
    }
    let DocumentRenderContentV1::Molecule(molecule) = root.content() else {
        return Err(DocumentBondReplacementErrorV1::RootIsNotMolecule);
    };
    if selected_bonds.is_empty() {
        return Err(DocumentBondReplacementErrorV1::SelectedBondOutcomeMismatch);
    }
    let mut selected_keys = HashSet::new();
    selected_keys
        .try_reserve(selected_bonds.len())
        .map_err(|_| DocumentBondReplacementErrorV1::ResourceExhausted)?;
    for target in &selected_bonds {
        if target.record_id().kind() != RecordKind::Bond {
            return Err(DocumentBondReplacementErrorV1::NonBondSelection);
        }
        if !selected_keys.insert(key(target)) {
            return Err(DocumentBondReplacementErrorV1::SelectedBondOutcomeMismatch);
        }
    }
    let mut outcomes = HashSet::new();
    let total = molecule
        .batches()
        .len()
        .checked_add(molecule.issues().len())
        .ok_or(DocumentBondReplacementErrorV1::ResourceExhausted)?;
    outcomes
        .try_reserve(total)
        .map_err(|_| DocumentBondReplacementErrorV1::ResourceExhausted)?;
    for target in molecule
        .batches()
        .iter()
        .map(|batch| batch.target())
        .chain(molecule.issues().iter().map(|issue| issue.target()))
    {
        if target.record_id().kind() == RecordKind::Bond && !outcomes.insert(key(target)) {
            return Err(DocumentBondReplacementErrorV1::SelectedBondOutcomeMismatch);
        }
    }
    if !selected_keys.iter().all(|value| outcomes.contains(value)) {
        return Err(DocumentBondReplacementErrorV1::SelectedBondOutcomeMismatch);
    }
    let mut direct_keys = HashSet::new();
    direct_keys
        .try_reserve(direct.operations().len())
        .map_err(|_| DocumentBondReplacementErrorV1::ResourceExhausted)?;
    for operation in direct.operations() {
        let target = (operation.bond().clone(), operation.authored_child_order());
        if !direct_keys.insert(target) {
            return Err(DocumentBondReplacementErrorV1::DirectOperationTargetMismatch);
        }
    }
    if direct_keys != selected_keys {
        return Err(DocumentBondReplacementErrorV1::DirectOperationTargetMismatch);
    }
    Ok(DocumentRenderCompositeV1 {
        established,
        replacement: BondReplacementV1 {
            root_identity: authenticated_root,
            root_order: expected_root_order,
            selected_bonds,
            selected_keys,
            direct,
        },
    })
}

impl DocumentRenderCompositeV1 {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn established(&self) -> &DocumentRenderPlanV1 {
        &self.established
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn replacement(&self) -> &BondReplacementV1 {
        &self.replacement
    }
}
impl BondReplacementV1 {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn root_identity(&self) -> &DocumentRenderIdentityV1 {
        &self.root_identity
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn root_order(&self) -> u32 {
        self.root_order
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn selected_keys(&self) -> &HashSet<TargetKey> {
        &self.selected_keys
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn direct(&self) -> &AuthoredDirectGlycosidicHaworthRenderPlanV1 {
        &self.direct
    }
}
fn key(target: &RenderTarget) -> TargetKey {
    (target.record_id().clone(), target.source_order())
}

#[cfg(test)]
mod tests {
    use ferrum_core::{Identifier, RecordId};

    use super::*;
    use crate::{
        BatchSpace, LineOp, MoleculeRenderPlan, Paint, PositiveFinite, RenderBatch, RenderOp,
        RenderPoint, RenderProvenance, RenderRevision, RenderViewportV1, Rgb24,
    };

    fn provenance() -> RenderProvenance {
        RenderProvenance::new(RenderRevision::new(7).expect("test revision"), [7; 32])
    }
    fn target(kind: RecordKind, name: &str, order: u32) -> RenderTarget {
        let id = Identifier::new(name).expect("test identifier");
        RenderTarget::new(RecordId::from_source(kind, &id), order)
    }
    fn direct(target: &RenderTarget) -> AuthoredDirectGlycosidicHaworthRenderPlanV1 {
        AuthoredDirectGlycosidicHaworthRenderPlanV1::test_plan(
            provenance(),
            Paint::rgb24(Rgb24::new("000000").expect("test paint")),
            vec![
                crate::authored_direct_glycosidic_haworth::
                    AuthoredDirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
                    bond: target.record_id().clone(),
                    authored_child_order: target.source_order(),
                    endpoints: [
                        RenderPoint::new(0.0, 0.0).expect("point"),
                        RenderPoint::new(1.0, 0.0).expect("point"),
                    ],
                    width: PositiveFinite::new(1.0).expect("width"),
                },
            ],
        )
    }
    fn established(bond: RenderTarget) -> DocumentRenderPlanV1 {
        let line = LineOp::new(
            RenderPoint::new(0.0, 0.0).expect("point"),
            RenderPoint::new(1.0, 0.0).expect("point"),
            PositiveFinite::new(1.0).expect("width"),
            Paint::rgb24(Rgb24::new("000000").expect("paint")),
            0,
        )
        .expect("line");
        let molecule = MoleculeRenderPlan::new(
            provenance(),
            vec![
                RenderBatch::new(bond, BatchSpace::Scene, vec![RenderOp::Line(line)])
                    .expect("batch"),
            ],
            vec![],
        )
        .expect("molecule");
        DocumentRenderPlanV1::new(
            provenance(),
            RenderViewportV1::new(0.0, 0.0, 10.0, 10.0).expect("page"),
            vec![DocumentRenderOutcomeV1::Root(
                crate::DocumentRenderRootV1::new(
                    2,
                    DocumentRenderIdentityV1::durable("molecule").expect("root"),
                    DocumentRenderContentV1::Molecule(molecule),
                ),
            )],
        )
        .expect("document")
    }

    #[test]
    fn replacement_requires_complete_bond_targets_and_exact_direct_keys() {
        let bond = target(RecordKind::Bond, "b1", 3);
        let result = compose_document_bond_replacement_v1(
            established(bond.clone()),
            DocumentRenderIdentityV1::durable("molecule").expect("root"),
            2,
            vec![bond.clone()],
            direct(&bond),
        )
        .expect("matching durable selection composes");
        assert_eq!(result.provenance(), provenance());

        let atom = target(RecordKind::Atom, "a1", 1);
        let error = compose_document_bond_replacement_v1(
            established(bond),
            DocumentRenderIdentityV1::durable("molecule").expect("root"),
            2,
            vec![atom.clone()],
            direct(&atom),
        )
        .expect_err("atom cannot be a replacement target");
        assert_eq!(error, DocumentBondReplacementErrorV1::NonBondSelection);
    }
}
