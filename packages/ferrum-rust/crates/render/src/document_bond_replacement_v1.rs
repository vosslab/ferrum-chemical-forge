//! Checked in-process replacement of selected molecule bond outcomes.

use std::collections::HashSet;

use ferrum_document_projection::DocumentObjectIdV1;
use thiserror::Error;

use crate::{
    AuthoredDirectGlycosidicHaworthRenderPlanV1, DocumentRenderContentV1, DocumentRenderOutcomeV1,
    DocumentRenderPlanV1, RenderTarget,
};

type TargetKey = (DocumentObjectIdV1, u32);

/// Durable selected-bond identity and its contractual paint order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondReplacementTargetV1 {
    target: RenderTarget,
    paint_order: u32,
}

impl BondReplacementTargetV1 {
    #[must_use]
    pub const fn new(target: RenderTarget, paint_order: u32) -> Self {
        Self {
            target,
            paint_order,
        }
    }
}

/// An opaque in-memory composition; it intentionally has no wire representation.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderCompositeV1 {
    established: DocumentRenderPlanV1,
    replacement: BondReplacementV1,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BondReplacementV1 {
    root_target: RenderTarget,
    root_paint_order: u32,
    selected_bonds: Vec<BondReplacementTargetV1>,
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
    authenticated_root: RenderTarget,
    expected_paint_order: u32,
    selected_bonds: Vec<BondReplacementTargetV1>,
    direct: AuthoredDirectGlycosidicHaworthRenderPlanV1,
) -> Result<DocumentRenderCompositeV1, DocumentBondReplacementErrorV1> {
    if established.provenance() != direct.provenance() {
        return Err(DocumentBondReplacementErrorV1::ProvenanceMismatch);
    }
    let mut matching = established
        .outcomes()
        .iter()
        .filter_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root) if root.target() == &authenticated_root => {
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
    if root.paint_order() != expected_paint_order {
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
        .map(BondReplacementTargetV1::from_batch)
        .chain(
            molecule
                .issues()
                .iter()
                .map(BondReplacementTargetV1::from_issue),
        )
    {
        if !outcomes.insert(key(&target)) {
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
        if !direct_keys.insert((operation.bond().clone(), operation.authored_child_order())) {
            return Err(DocumentBondReplacementErrorV1::DirectOperationTargetMismatch);
        }
    }
    if selected_keys != direct_keys {
        return Err(DocumentBondReplacementErrorV1::DirectOperationTargetMismatch);
    }
    Ok(DocumentRenderCompositeV1 {
        established,
        replacement: BondReplacementV1 {
            root_target: authenticated_root,
            root_paint_order: expected_paint_order,
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
    pub(crate) const fn root_target(&self) -> &RenderTarget {
        &self.root_target
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn root_paint_order(&self) -> u32 {
        self.root_paint_order
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
impl BondReplacementTargetV1 {
    fn from_batch(batch: &crate::RenderBatchV4) -> Self {
        Self::new(batch.target().clone(), batch.paint_order())
    }

    fn from_issue(issue: &crate::RenderIssue) -> Self {
        Self::new(issue.target().clone(), issue.paint_order())
    }
}

fn key(target: &BondReplacementTargetV1) -> TargetKey {
    (
        target.target.document_object_id().clone(),
        target.paint_order,
    )
}

#[cfg(test)]
mod tests {
    use ferrum_document_projection::DocumentObjectIdV1;

    use super::*;
    use crate::{
        BondRenderBatchV1, BondRenderOpV1, DocumentMoleculeRenderContentV1, LineOp,
        MoleculeRenderPlanV4, PositiveFinite, RenderBatchV4, RenderPaintV3, RenderPoint,
        RenderProvenance, RenderRevision, RenderViewportV1, Rgb24,
    };

    fn provenance() -> RenderProvenance {
        RenderProvenance::new(RenderRevision::new(7).expect("test revision"), [7; 32])
    }
    fn target(order: u8) -> RenderTarget {
        RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([order; 16]))
    }
    fn direct(bond: DocumentObjectIdV1, order: u32) -> AuthoredDirectGlycosidicHaworthRenderPlanV1 {
        AuthoredDirectGlycosidicHaworthRenderPlanV1::test_plan(
            provenance(),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
            vec![
                crate::authored_direct_glycosidic_haworth::
                    AuthoredDirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
                    bond,
                    authored_child_order: order,
                    endpoints: [
                        RenderPoint::new(0.0, 0.0).expect("point"),
                        RenderPoint::new(1.0, 0.0).expect("point"),
                    ],
                    width: PositiveFinite::new(1.0).expect("width"),
                },
            ],
        )
    }
    fn established(
        root_target: RenderTarget,
        bond: RenderTarget,
        paint_order: u32,
    ) -> DocumentRenderPlanV1 {
        let line = LineOp::new(
            RenderPoint::new(0.0, 0.0).expect("point"),
            RenderPoint::new(1.0, 0.0).expect("point"),
            PositiveFinite::new(1.0).expect("width"),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("paint")),
            0,
        )
        .expect("line");
        let molecule = MoleculeRenderPlanV4::new(
            provenance(),
            vec![RenderBatchV4::bond_target(
                bond,
                paint_order,
                BondRenderBatchV1::new(
                    crate::BondAttachmentAxisV1::new(
                        RenderPoint::new(0.0, 0.0).expect("point"),
                        RenderPoint::new(1.0, 0.0).expect("point"),
                    )
                    .expect("attachment axis"),
                    vec![BondRenderOpV1::Line(line)],
                )
                .expect("bond content"),
            )],
            vec![],
        )
        .expect("molecule");
        DocumentRenderPlanV1::new(
            provenance(),
            RenderViewportV1::new(0.0, 0.0, 10.0, 10.0).expect("page"),
            vec![DocumentRenderOutcomeV1::Root(
                crate::DocumentRenderRootV1::new(
                    root_target,
                    2,
                    DocumentRenderContentV1::Molecule(DocumentMoleculeRenderContentV1::new(
                        molecule,
                        Vec::new(),
                    )),
                ),
            )],
        )
        .expect("document")
    }

    #[test]
    fn replacement_requires_complete_bond_targets_and_exact_direct_keys() {
        let bond = target(3);
        let root = target(8);
        let bond_id = bond.document_object_id().clone();
        let result = compose_document_bond_replacement_v1(
            established(root.clone(), bond.clone(), 3),
            root.clone(),
            2,
            vec![BondReplacementTargetV1::new(bond.clone(), 3)],
            direct(bond_id, 3),
        )
        .expect("matching durable selection composes");
        assert_eq!(result.provenance(), provenance());

        let unmatched = target(1);
        let unmatched_id = unmatched.document_object_id().clone();
        let error = compose_document_bond_replacement_v1(
            established(root.clone(), bond, 3),
            root,
            2,
            vec![BondReplacementTargetV1::new(unmatched, 1)],
            direct(unmatched_id, 3),
        )
        .expect_err("unmatched durable target cannot be a replacement target");
        assert_eq!(
            error,
            DocumentBondReplacementErrorV1::SelectedBondOutcomeMismatch
        );
    }
}
