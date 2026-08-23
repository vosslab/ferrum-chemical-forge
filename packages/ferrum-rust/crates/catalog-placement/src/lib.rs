//! Renderer-preflighted placement for Ferrum-authored catalog recipes.
//!
//! Haworth entries stay out of this catalog: their native receipts retain
//! stereochemical display tokens which ordinary detached CDML cannot yet carry.
use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, DocumentFenceV1, DocumentSession,
    MoleculeInsertionAtomV1, MoleculeInsertionBondOrderV1, MoleculeInsertionBondV1,
    MoleculeInsertionV1, PendingCreateMolecule, PendingStandaloneHaworthV1, Point3V1,
    PresentationGesturePoint2V1, SessionOperationResultV1,
};
use ferrum_domain::{
    CatalogEntrySummaryV1, CatalogRecipeKindV1, catalog_entry_v1,
    haworth::{StandaloneDGlucoseHaworthRecipeV1, standalone_d_glucose_haworth_recipe_v1},
};
use ferrum_render::{
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct CatalogPlacementGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    key: String,
    entry: CatalogEntrySummaryV1,
}
#[derive(Clone, Debug)]
pub struct CatalogPlacementPreviewV1 {
    gesture: CatalogPlacementGestureV1,
    anchor: PresentationGesturePoint2V1,
    overlay: CatalogPlacementOverlayV1,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogPlacementOverlayV1 {
    pub atom_points: Vec<(f64, f64)>,
    pub bond_segments: Vec<(f64, f64, f64, f64)>,
}
#[derive(Debug)]
pub struct PreparedCatalogPlacementV1 {
    receipt: Option<CatalogReceiptV1>,
    identifier: String,
}
#[derive(Clone, Debug)]
pub struct CommittedCatalogPlacementV1 {
    identifier: String,
    result: SessionOperationResultV1,
}
impl CommittedCatalogPlacementV1 {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}
impl PreparedCatalogPlacementV1 {
    /// Expose only the renderer-issued candidate plan needed by catalog V2.
    /// Candidate CDML and its preflight proof stay private to this crate.
    #[must_use]
    pub fn render_plan(&self) -> Option<&DocumentRenderPlanV1> {
        self.receipt.as_ref().map(|receipt| &receipt.plan)
    }

    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPlacementCategoryV1 {
    UnknownKey,
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    RenderPreparation,
    SessionConflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPlacementRecoveryV1 {
    ChooseCatalogEntry,
    RefreshAndRestart,
    DocumentUnchanged,
}
#[derive(Clone, Debug, Error, PartialEq)]
pub enum CatalogPlacementErrorV1 {
    #[error("unknown Ferrum catalog key")]
    UnknownKey,
    #[error("catalog placement snapshot is stale")]
    StaleSnapshot,
    #[error("catalog placement handle belongs to another document session")]
    ForeignSession,
    #[error("catalog placement preview belongs to another gesture")]
    MismatchedPreview,
    #[error("catalog placement capability was already used")]
    ReplayedGesture,
    #[error("catalog placement anchor is not finite")]
    InvalidPoint,
    #[error("catalog candidate could not be rendered completely")]
    RenderPreparation,
    #[error("catalog placement commit was rejected by document session")]
    SessionConflict,
}
impl CatalogPlacementErrorV1 {
    pub const fn category(&self) -> CatalogPlacementCategoryV1 {
        match self {
            Self::UnknownKey => CatalogPlacementCategoryV1::UnknownKey,
            Self::StaleSnapshot => CatalogPlacementCategoryV1::StaleSnapshot,
            Self::ForeignSession => CatalogPlacementCategoryV1::ForeignSession,
            Self::MismatchedPreview => CatalogPlacementCategoryV1::MismatchedPreview,
            Self::ReplayedGesture => CatalogPlacementCategoryV1::ReplayedGesture,
            Self::InvalidPoint => CatalogPlacementCategoryV1::InvalidPoint,
            Self::RenderPreparation => CatalogPlacementCategoryV1::RenderPreparation,
            Self::SessionConflict => CatalogPlacementCategoryV1::SessionConflict,
        }
    }
    pub const fn recovery(&self) -> CatalogPlacementRecoveryV1 {
        match self {
            Self::UnknownKey => CatalogPlacementRecoveryV1::ChooseCatalogEntry,
            Self::InvalidPoint | Self::RenderPreparation => {
                CatalogPlacementRecoveryV1::DocumentUnchanged
            }
            _ => CatalogPlacementRecoveryV1::RefreshAndRestart,
        }
    }
}
#[derive(Debug)]
struct CatalogReceiptV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    key: String,
    identifier: String,
    pending: CatalogPendingV1,
    plan: DocumentRenderPlanV1,
}
#[derive(Debug)]
enum CatalogPendingV1 {
    Molecule(PendingCreateMolecule),
    StandaloneHaworth(PendingStandaloneHaworthV1),
}

impl CatalogPendingV1 {
    fn identifier(&self) -> &str {
        match self {
            Self::Molecule(pending) => pending.molecule_identifier().as_str(),
            Self::StandaloneHaworth(pending) => pending.molecule_identifier().as_str(),
        }
    }

    fn candidate_observation(&self) -> Option<ferrum_document::SessionDocumentObservationV1> {
        match self {
            Self::Molecule(pending) => pending.candidate_observation_v1(),
            Self::StandaloneHaworth(pending) => pending.candidate_observation_v1(),
        }
    }
}
#[derive(Clone, Copy)]
struct Recipe {
    elements: &'static [&'static str],
    edges: &'static [(usize, usize, &'static str)],
    shape: Shape,
}
#[derive(Clone, Copy)]
enum Shape {
    Ring,
    Purine,
}
const S3: &[(usize, usize, &str)] = &[(0, 1, "n1"), (1, 2, "n1"), (2, 0, "n1")];
const S4: &[(usize, usize, &str)] = &[(0, 1, "n1"), (1, 2, "n1"), (2, 3, "n1"), (3, 0, "n1")];
const S5: &[(usize, usize, &str)] = &[
    (0, 1, "n1"),
    (1, 2, "n1"),
    (2, 3, "n1"),
    (3, 4, "n1"),
    (4, 0, "n1"),
];
const S6: &[(usize, usize, &str)] = &[
    (0, 1, "n1"),
    (1, 2, "n1"),
    (2, 3, "n1"),
    (3, 4, "n1"),
    (4, 5, "n1"),
    (5, 0, "n1"),
];
const B6: &[(usize, usize, &str)] = &[
    (0, 1, "n2"),
    (1, 2, "n1"),
    (2, 3, "n2"),
    (3, 4, "n1"),
    (4, 5, "n2"),
    (5, 0, "n1"),
];
const H5: &[(usize, usize, &str)] = &[
    (0, 1, "n1"),
    (1, 2, "n2"),
    (2, 3, "n1"),
    (3, 4, "n2"),
    (4, 0, "n1"),
];
const PURINE: &[(usize, usize, &str)] = &[
    (0, 1, "n1"),
    (1, 2, "n2"),
    (2, 3, "n1"),
    (3, 4, "n1"),
    (4, 5, "n2"),
    (5, 6, "n1"),
    (6, 7, "n2"),
    (7, 8, "n1"),
    (8, 3, "n2"),
    (8, 0, "n1"),
];
fn recipe(kind: CatalogRecipeKindV1) -> Recipe {
    match kind {
        CatalogRecipeKindV1::Benzene => Recipe {
            elements: &["C", "C", "C", "C", "C", "C"],
            edges: B6,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclopropane => Recipe {
            elements: &["C", "C", "C"],
            edges: S3,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclobutane => Recipe {
            elements: &["C", "C", "C", "C"],
            edges: S4,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclopentane => Recipe {
            elements: &["C", "C", "C", "C", "C"],
            edges: S5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclohexane => Recipe {
            elements: &["C", "C", "C", "C", "C", "C"],
            edges: S6,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Thiophene => Recipe {
            elements: &["S", "C", "C", "C", "C"],
            edges: H5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Furan => Recipe {
            elements: &["O", "C", "C", "C", "C"],
            edges: H5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Pyrrole => Recipe {
            elements: &["N", "C", "C", "C", "C"],
            edges: H5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Purine => Recipe {
            elements: &["N", "C", "N", "C", "C", "N", "C", "N", "C"],
            edges: PURINE,
            shape: Shape::Purine,
        },
        CatalogRecipeKindV1::HaworthBiomolecule(_) => {
            unreachable!("Haworth catalog recipes use the literal depiction compiler")
        }
    }
}
fn capability_error(error: AuthoringCapabilityAccessErrorV1) -> CatalogPlacementErrorV1 {
    match error {
        AuthoringCapabilityAccessErrorV1::ForeignSession => CatalogPlacementErrorV1::ForeignSession,
        AuthoringCapabilityAccessErrorV1::Replayed => CatalogPlacementErrorV1::ReplayedGesture,
    }
}
fn fence(
    session: &DocumentSession,
    expected: DocumentFenceV1,
) -> Result<(), CatalogPlacementErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CatalogPlacementErrorV1::SessionConflict)?;
    (snapshot.revision() == expected.revision() && snapshot.digest() == &expected.digest())
        .then_some(())
        .ok_or(CatalogPlacementErrorV1::StaleSnapshot)
}
pub fn begin_catalog_placement_v1(
    session: &DocumentSession,
    expected: DocumentFenceV1,
    key: &str,
) -> Result<CatalogPlacementGestureV1, CatalogPlacementErrorV1> {
    fence(session, expected)?;
    Ok(CatalogPlacementGestureV1 {
        capability: session.authoring_capability_issuer_v1().issue(),
        fence: expected,
        key: key.to_owned(),
        entry: catalog_entry_v1(key).ok_or(CatalogPlacementErrorV1::UnknownKey)?,
    })
}
pub fn preview_catalog_placement_v1(
    session: &DocumentSession,
    gesture: &CatalogPlacementGestureV1,
    anchor: PresentationGesturePoint2V1,
) -> Result<CatalogPlacementPreviewV1, CatalogPlacementErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(CatalogPlacementErrorV1::ForeignSession);
    }
    fence(session, gesture.fence)?;
    Ok(CatalogPlacementPreviewV1 {
        gesture: gesture.clone(),
        anchor,
        overlay: match gesture.entry.recipe() {
            CatalogRecipeKindV1::HaworthBiomolecule(value) => haworth_overlay(value, anchor)?,
            kind => overlay(recipe(kind), anchor)?,
        },
    })
}
impl CatalogPlacementPreviewV1 {
    pub fn overlay(&self) -> &CatalogPlacementOverlayV1 {
        &self.overlay
    }
}
pub fn prepare_catalog_placement_v1(
    session: &mut DocumentSession,
    gesture: &CatalogPlacementGestureV1,
    preview: &CatalogPlacementPreviewV1,
) -> Result<PreparedCatalogPlacementV1, CatalogPlacementErrorV1> {
    let issuer = session.authoring_capability_issuer_v1();
    if !gesture.capability.belongs_to(&issuer) || !preview.gesture.capability.belongs_to(&issuer) {
        return Err(CatalogPlacementErrorV1::ForeignSession);
    }
    if !gesture
        .capability
        .same_capability(&preview.gesture.capability)
        || gesture.key != preview.gesture.key
    {
        return Err(CatalogPlacementErrorV1::MismatchedPreview);
    }
    fence(session, gesture.fence)?;
    let pending = match gesture.entry.recipe() {
        CatalogRecipeKindV1::HaworthBiomolecule(value) => CatalogPendingV1::StandaloneHaworth(
            session
                .prepare_create_standalone_haworth_v1(
                    gesture.fence.revision(),
                    value,
                    Point3V1::new(preview.anchor.x(), preview.anchor.y(), 0.0)
                        .map_err(|_| CatalogPlacementErrorV1::InvalidPoint)?,
                )
                .map_err(|_| CatalogPlacementErrorV1::SessionConflict)?,
        ),
        kind => {
            let recipe = recipe(kind);
            let molecule = lower_recipe(gesture.entry, recipe, preview.anchor)?;
            CatalogPendingV1::Molecule(
                session
                    .prepare_create_molecule_v1(gesture.fence.revision(), &molecule)
                    .map_err(|_| CatalogPlacementErrorV1::SessionConflict)?,
            )
        }
    };
    let identifier = pending.identifier().to_owned();
    let observation = pending
        .candidate_observation()
        .ok_or(CatalogPlacementErrorV1::RenderPreparation)?;
    let render = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&render)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|o| matches!(o, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(CatalogPlacementErrorV1::RenderPreparation);
    }
    Ok(PreparedCatalogPlacementV1 {
        identifier: identifier.clone(),
        receipt: Some(CatalogReceiptV1 {
            capability: gesture.capability.clone(),
            fence: gesture.fence,
            key: gesture.key.clone(),
            identifier,
            pending,
            plan,
        }),
    })
}
pub fn commit_catalog_placement_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCatalogPlacementV1,
) -> Result<CommittedCatalogPlacementV1, CatalogPlacementErrorV1> {
    let (capability, receipt_fence) = {
        let receipt = prepared
            .receipt
            .as_ref()
            .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
        if catalog_entry_v1(&receipt.key).is_none()
            || receipt.identifier != prepared.identifier
            || receipt.identifier != receipt.pending.identifier()
            || receipt
                .plan
                .outcomes()
                .iter()
                .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
        {
            return Err(CatalogPlacementErrorV1::RenderPreparation);
        }
        (receipt.capability.clone(), receipt.fence)
    };
    let issuer = session.authoring_capability_issuer_v1();
    if !capability.belongs_to(&issuer) {
        return Err(CatalogPlacementErrorV1::ForeignSession);
    }
    let claim = capability
        .claim_for_commit(&issuer)
        .map_err(capability_error)?;
    fence(session, receipt_fence)?;
    let result = match &mut prepared
        .receipt
        .as_mut()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?
        .pending
    {
        CatalogPendingV1::Molecule(pending) => session
            .commit_create_molecule(receipt_fence.revision(), pending)
            .map_err(|_| CatalogPlacementErrorV1::SessionConflict),
        CatalogPendingV1::StandaloneHaworth(pending) => session
            .commit_create_standalone_haworth_v1(receipt_fence.revision(), pending)
            .map_err(|_| CatalogPlacementErrorV1::SessionConflict),
    }?;
    let receipt = prepared
        .receipt
        .take()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
    claim.consume();
    Ok(CommittedCatalogPlacementV1 {
        identifier: receipt.identifier,
        result,
    })
}
fn local(recipe: Recipe) -> Vec<(f64, f64)> {
    match recipe.shape {
        Shape::Ring => {
            let n = recipe.elements.len() as f64;
            let radius = 20.0 / (std::f64::consts::PI / n).sin();
            (0..recipe.elements.len())
                .map(|i| {
                    let a = 2.0 * std::f64::consts::PI * i as f64 / n - std::f64::consts::FRAC_PI_2;
                    (radius * a.cos(), radius * a.sin())
                })
                .collect()
        }
        Shape::Purine => vec![
            (32.360_679_774_997_9, -45.84313896971147),
            (0.0, -69.3545490614104),
            (-32.360_679_774_997_9, -45.84313896971147),
            (-20.0, -7.80087831790533),
            (-40.0, 26.84013783347222),
            (-20.0, 61.48115398484977),
            (20.0, 61.48115398484977),
            (40.0, 26.84013783347222),
            (20.0, -7.80087831790533),
        ],
    }
}
fn overlay(
    recipe: Recipe,
    anchor: PresentationGesturePoint2V1,
) -> Result<CatalogPlacementOverlayV1, CatalogPlacementErrorV1> {
    if !anchor.x().is_finite() || !anchor.y().is_finite() {
        return Err(CatalogPlacementErrorV1::InvalidPoint);
    }
    let atom_points = local(recipe)
        .into_iter()
        .map(|(x, y)| (x + anchor.x(), y + anchor.y()))
        .collect::<Vec<_>>();
    let bond_segments = recipe
        .edges
        .iter()
        .map(|(s, e, _)| {
            let a = atom_points[*s];
            let b = atom_points[*e];
            (a.0, a.1, b.0, b.1)
        })
        .collect();
    Ok(CatalogPlacementOverlayV1 {
        atom_points,
        bond_segments,
    })
}

/// Lower catalog geometry into the public document insertion grammar. Durable
/// identities are intentionally absent: `DocumentSession` owns their allocation.
fn lower_recipe(
    entry: CatalogEntrySummaryV1,
    recipe: Recipe,
    anchor: PresentationGesturePoint2V1,
) -> Result<MoleculeInsertionV1, CatalogPlacementErrorV1> {
    let overlay = overlay(recipe, anchor)?;
    let atoms = recipe
        .elements
        .iter()
        .zip(&overlay.atom_points)
        .map(|(element, (x, y))| {
            MoleculeInsertionAtomV1::new(
                *element,
                Point3V1::new(*x, *y, 0.0).map_err(|_| CatalogPlacementErrorV1::InvalidPoint)?,
                None,
                None,
                None,
            )
            .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = recipe
        .edges
        .iter()
        .map(|(start, end, token)| {
            let order = match *token {
                "n1" => MoleculeInsertionBondOrderV1::Single,
                "n2" => MoleculeInsertionBondOrderV1::Double,
                _ => return Err(CatalogPlacementErrorV1::RenderPreparation),
            };
            Ok(MoleculeInsertionBondV1::new(*start, *end, order))
        })
        .collect::<Result<Vec<_>, _>>()?;
    MoleculeInsertionV1::new(atoms, bonds)
        .and_then(|molecule| molecule.with_name(entry.label()))
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)
}
fn haworth_overlay(
    kind: StandaloneDGlucoseHaworthRecipeV1,
    anchor: PresentationGesturePoint2V1,
) -> Result<CatalogPlacementOverlayV1, CatalogPlacementErrorV1> {
    if !anchor.x().is_finite() || !anchor.y().is_finite() {
        return Err(CatalogPlacementErrorV1::InvalidPoint);
    }
    let receipt = standalone_d_glucose_haworth_recipe_v1(kind)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    let atom_points = receipt
        .atoms()
        .iter()
        .map(|fact| {
            let point = fact.local();
            (point.x + anchor.x(), point.y + anchor.y())
        })
        .collect::<Vec<_>>();
    let bond_segments = receipt
        .bonds()
        .iter()
        .map(|fact| {
            let first = atom_points[fact.start()];
            let second = atom_points[fact.end()];
            (first.0, first.1, second.0, second.1)
        })
        .collect();
    Ok(CatalogPlacementOverlayV1 {
        atom_points,
        bond_segments,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\"/>";
    fn strictly_cross(left: (f64, f64, f64, f64), right: (f64, f64, f64, f64)) -> bool {
        fn side(ax: f64, ay: f64, bx: f64, by: f64, px: f64, py: f64) -> f64 {
            (bx - ax) * (py - ay) - (by - ay) * (px - ax)
        }
        let (ax, ay, bx, by) = left;
        let (cx, cy, dx, dy) = right;
        let first = side(ax, ay, bx, by, cx, cy);
        let second = side(ax, ay, bx, by, dx, dy);
        let third = side(cx, cy, dx, dy, ax, ay);
        let fourth = side(cx, cy, dx, dy, bx, by);
        first * second < 0.0 && third * fourth < 0.0
    }
    fn current(s: &DocumentSession) -> DocumentFenceV1 {
        let q = s.snapshot().expect("snapshot");
        DocumentFenceV1::new(q.revision(), *q.digest())
    }
    #[test]
    fn each_curated_system_recipe_has_topology_geometry_and_preflighted_commit() {
        for (key, elements, edges) in [
            (
                "system/rings/benzene",
                &["C", "C", "C", "C", "C", "C"][..],
                B6,
            ),
            ("system/rings/cyclopropane", &["C", "C", "C"][..], S3),
            ("system/rings/cyclobutane", &["C", "C", "C", "C"][..], S4),
            (
                "system/rings/cyclopentane",
                &["C", "C", "C", "C", "C"][..],
                S5,
            ),
            (
                "system/rings/cyclohexane",
                &["C", "C", "C", "C", "C", "C"][..],
                S6,
            ),
            (
                "system/heterocycles/thiophene",
                &["S", "C", "C", "C", "C"][..],
                H5,
            ),
            (
                "system/heterocycles/furan",
                &["O", "C", "C", "C", "C"][..],
                H5,
            ),
            (
                "system/heterocycles/pyrrole",
                &["N", "C", "C", "C", "C"][..],
                H5,
            ),
            (
                "system/heterocycles/purine",
                &["N", "C", "N", "C", "C", "N", "C", "N", "C"][..],
                PURINE,
            ),
        ] {
            let mut s = DocumentSession::load(EMPTY).expect("session");
            let g = begin_catalog_placement_v1(&s, current(&s), key).expect("gesture");
            let authored = recipe(g.entry.recipe());
            assert_eq!(authored.elements, elements, "{key} elements");
            assert_eq!(authored.edges, edges, "{key} bonds and orders");
            let p = preview_catalog_placement_v1(
                &s,
                &g,
                PresentationGesturePoint2V1::new(100.0, 50.0).expect("point"),
            )
            .expect("preview");
            assert_eq!(
                (
                    p.overlay().atom_points.len(),
                    p.overlay().bond_segments.len()
                ),
                (elements.len(), edges.len())
            );
            let centroid = p
                .overlay()
                .atom_points
                .iter()
                .fold((0.0, 0.0), |sum, point| (sum.0 + point.0, sum.1 + point.1));
            assert!(
                (centroid.0 / elements.len() as f64 - 100.0).abs() < 0.001,
                "{key} x centroid"
            );
            assert!(
                (centroid.1 / elements.len() as f64 - 50.0).abs() < 0.001,
                "{key} y centroid"
            );
            assert!(p.overlay().bond_segments.iter().all(|(x, y, u, v)| {
                x.is_finite()
                    && y.is_finite()
                    && u.is_finite()
                    && v.is_finite()
                    && (((u - x).powi(2) + (v - y).powi(2)).sqrt() - 40.0).abs() < 0.001
            }));
            for (left_index, left) in edges.iter().enumerate() {
                for (right_index, right) in edges.iter().enumerate().skip(left_index + 1) {
                    if [left.0, left.1].contains(&right.0) || [left.0, left.1].contains(&right.1) {
                        continue;
                    }
                    assert!(
                        !strictly_cross(
                            p.overlay().bond_segments[left_index],
                            p.overlay().bond_segments[right_index],
                        ),
                        "{key} bonds {left_index} and {right_index} cross"
                    );
                }
            }
            let mut r = prepare_catalog_placement_v1(&mut s, &g, &p).expect("prepare");
            let c = commit_catalog_placement_v1(&mut s, &mut r).expect("commit");
            assert!(c.identifier().starts_with("ferrum-molecule-v1-"));
            assert!(
                c.result()
                    .observation()
                    .snapshot()
                    .cdml()
                    .contains(c.identifier())
            );
            assert!(
                c.result()
                    .observation()
                    .snapshot()
                    .cdml()
                    .contains(&format!("name=\"{}\"", g.entry.label()))
            );
            assert!(s.undo(1).is_ok());
        }
    }
    #[test]
    fn catalog_uses_document_owned_ids_and_discarded_candidates_do_not_advance_them() {
        let mut session = DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><opaque id=\"ferrum-molecule-v1-0\"/><opaque id=\"ferrum-atom-v1-0\"/><opaque id=\"ferrum-bond-v1-0\"/></cdml>",
        )
        .expect("source");
        let anchor = PresentationGesturePoint2V1::new(0.0, 0.0).expect("anchor");
        let first = begin_catalog_placement_v1(&session, current(&session), "system/rings/benzene")
            .expect("gesture");
        let first_preview =
            preview_catalog_placement_v1(&session, &first, anchor).expect("preview");
        let discarded =
            prepare_catalog_placement_v1(&mut session, &first, &first_preview).expect("candidate");
        let discarded_identifier = discarded.identifier().to_owned();
        drop(discarded);

        let second =
            begin_catalog_placement_v1(&session, current(&session), "system/rings/benzene")
                .expect("gesture");
        let second_preview =
            preview_catalog_placement_v1(&session, &second, anchor).expect("preview");
        let mut prepared = prepare_catalog_placement_v1(&mut session, &second, &second_preview)
            .expect("candidate");
        assert_eq!(prepared.identifier(), discarded_identifier);
        let committed = commit_catalog_placement_v1(&mut session, &mut prepared).expect("commit");
        assert!(committed.identifier().starts_with("ferrum-molecule-v1-"));
        let ordinary = lower_recipe(
            catalog_entry_v1("system/rings/cyclopropane").expect("catalog entry"),
            recipe(CatalogRecipeKindV1::Cyclopropane),
            anchor,
        )
        .expect("ordinary molecule");
        let pending = session
            .prepare_create_molecule_v1(current(&session).revision(), &ordinary)
            .expect("next candidate");
        assert_ne!(
            pending.molecule_identifier().as_str(),
            committed.identifier()
        );
    }

    #[test]
    fn capability_fences_identical_foreign_sessions_and_preserves_owner_retry() {
        let mut owner = DocumentSession::load(EMPTY).expect("owner session");
        let mut foreign = DocumentSession::load(EMPTY).expect("foreign session");
        let gesture = begin_catalog_placement_v1(&owner, current(&owner), "system/rings/benzene")
            .expect("owner gesture");
        let anchor = PresentationGesturePoint2V1::new(20.0, 30.0).expect("anchor");

        assert!(matches!(
            preview_catalog_placement_v1(&foreign, &gesture, anchor),
            Err(CatalogPlacementErrorV1::ForeignSession)
        ));
        let preview = preview_catalog_placement_v1(&owner, &gesture, anchor).expect("preview");
        assert!(matches!(
            prepare_catalog_placement_v1(&mut foreign, &gesture, &preview),
            Err(CatalogPlacementErrorV1::ForeignSession)
        ));
        let mut prepared =
            prepare_catalog_placement_v1(&mut owner, &gesture, &preview).expect("owner prepared");
        assert!(matches!(
            commit_catalog_placement_v1(&mut foreign, &mut prepared),
            Err(CatalogPlacementErrorV1::ForeignSession)
        ));
        assert_eq!(foreign.snapshot().expect("foreign snapshot").revision(), 0);
        commit_catalog_placement_v1(&mut owner, &mut prepared).expect("owner retry commits");
        assert!(matches!(
            commit_catalog_placement_v1(&mut owner, &mut prepared),
            Err(CatalogPlacementErrorV1::ReplayedGesture)
        ));
    }
}
