//! Closed recipe and preview geometry for Ferrum-authored catalog placement.
//!
//! The document session owns candidate construction, renderer admission, and
//! atomic mutation for both ordinary molecule and standalone Haworth requests.
use ferrum_document::{
    CatalogMoleculePlacementGestureV1, CatalogMoleculePlacementRefusalV1,
    CatalogMoleculePlacementRequestV1, DocumentFenceV1, DocumentSession, MoleculeInsertionAtomV1,
    MoleculeInsertionBondOrderV1, MoleculeInsertionBondV1, MoleculeInsertionV1, Point3V1,
    PresentationGesturePoint2V1,
};
use ferrum_domain::{
    CatalogEntrySummaryV1, CatalogRecipeKindV1, catalog_entry_v1,
    haworth::{StandaloneDGlucoseHaworthRecipeV1, standalone_d_glucose_haworth_recipe_v1},
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct CatalogPlacementGestureV1 {
    placement: CatalogMoleculePlacementGestureV1,
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
fn document_refusal(error: CatalogMoleculePlacementRefusalV1) -> CatalogPlacementErrorV1 {
    match error {
        CatalogMoleculePlacementRefusalV1::StaleSnapshot => CatalogPlacementErrorV1::StaleSnapshot,
        CatalogMoleculePlacementRefusalV1::ForeignSession => {
            CatalogPlacementErrorV1::ForeignSession
        }
        CatalogMoleculePlacementRefusalV1::ReplayedGesture => {
            CatalogPlacementErrorV1::ReplayedGesture
        }
        CatalogMoleculePlacementRefusalV1::RendererAdmission => {
            CatalogPlacementErrorV1::RenderPreparation
        }
        CatalogMoleculePlacementRefusalV1::SessionConflict => {
            CatalogPlacementErrorV1::SessionConflict
        }
    }
}
pub fn begin_catalog_placement_v1(
    session: &DocumentSession,
    expected: DocumentFenceV1,
    key: &str,
) -> Result<CatalogPlacementGestureV1, CatalogPlacementErrorV1> {
    let placement = session
        .begin_catalog_molecule_placement_v1(expected)
        .map_err(document_refusal)?;
    Ok(CatalogPlacementGestureV1 {
        placement,
        key: key.to_owned(),
        entry: catalog_entry_v1(key).ok_or(CatalogPlacementErrorV1::UnknownKey)?,
    })
}
pub fn preview_catalog_placement_v1(
    session: &DocumentSession,
    gesture: &CatalogPlacementGestureV1,
    anchor: PresentationGesturePoint2V1,
) -> Result<CatalogPlacementPreviewV1, CatalogPlacementErrorV1> {
    session
        .validate_catalog_molecule_placement_v1(&gesture.placement)
        .map_err(document_refusal)?;
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
/// Resolve the exact document request represented by one catalog preview.
pub fn resolve_catalog_placement_v1(
    gesture: &CatalogPlacementGestureV1,
    preview: &CatalogPlacementPreviewV1,
) -> Result<CatalogMoleculePlacementRequestV1, CatalogPlacementErrorV1> {
    if !gesture
        .placement
        .same_gesture_v1(&preview.gesture.placement)
        || gesture.key != preview.gesture.key
    {
        return Err(CatalogPlacementErrorV1::MismatchedPreview);
    }
    let request = match gesture.entry.recipe() {
        CatalogRecipeKindV1::HaworthBiomolecule(value) => {
            CatalogMoleculePlacementRequestV1::StandaloneHaworth {
                recipe: value,
                anchor: Point3V1::new(preview.anchor.x(), preview.anchor.y(), 0.0)
                    .map_err(|_| CatalogPlacementErrorV1::InvalidPoint)?,
            }
        }
        kind => {
            let recipe = recipe(kind);
            let molecule = lower_recipe(gesture.entry, recipe, preview.anchor)?;
            CatalogMoleculePlacementRequestV1::Molecule(molecule)
        }
    };
    Ok(request)
}

/// Return the document-owned capability carried by a catalog selection.
#[must_use]
pub fn catalog_molecule_placement_gesture_v1(
    gesture: &CatalogPlacementGestureV1,
) -> &CatalogMoleculePlacementGestureV1 {
    &gesture.placement
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
    fn current(s: &DocumentSession) -> DocumentFenceV1 {
        let q = s.snapshot().expect("snapshot");
        DocumentFenceV1::new(q.revision(), *q.digest())
    }

    fn pending(
        session: &mut DocumentSession,
        gesture: &CatalogPlacementGestureV1,
        preview: &CatalogPlacementPreviewV1,
    ) -> ferrum_document::PendingCatalogMoleculePlacementV1 {
        let request = resolve_catalog_placement_v1(gesture, preview).expect("catalog request");
        session
            .prepare_catalog_molecule_placement_v1(
                catalog_molecule_placement_gesture_v1(gesture),
                request,
            )
            .expect("document pending")
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
        let discarded = pending(&mut session, &first, &first_preview);
        let discarded_identifier = discarded.identifier().to_owned();
        drop(discarded);

        let second =
            begin_catalog_placement_v1(&session, current(&session), "system/rings/benzene")
                .expect("gesture");
        let second_preview =
            preview_catalog_placement_v1(&session, &second, anchor).expect("preview");
        let mut prepared = pending(&mut session, &second, &second_preview);
        assert_eq!(prepared.identifier(), discarded_identifier);
        let committed_identifier = prepared.identifier().to_owned();
        session
            .commit_catalog_molecule_placement_v1(&mut prepared)
            .expect("commit");
        assert!(committed_identifier.starts_with("ferrum-molecule-v1-"));
        let ordinary = lower_recipe(
            catalog_entry_v1("system/rings/cyclopropane").expect("catalog entry"),
            recipe(CatalogRecipeKindV1::Cyclopropane),
            anchor,
        )
        .expect("ordinary molecule");
        let pending = session
            .prepare_admitted_molecule_insertion_v1(current(&session).revision(), &ordinary)
            .expect("next candidate");
        assert_ne!(pending.molecule_identifier().as_str(), committed_identifier);
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
            foreign.prepare_catalog_molecule_placement_v1(
                catalog_molecule_placement_gesture_v1(&gesture),
                resolve_catalog_placement_v1(&gesture, &preview).expect("request"),
            ),
            Err(CatalogMoleculePlacementRefusalV1::ForeignSession)
        ));
        let mut prepared = pending(&mut owner, &gesture, &preview);
        assert!(matches!(
            foreign.commit_catalog_molecule_placement_v1(&mut prepared),
            Err(CatalogMoleculePlacementRefusalV1::ForeignSession)
        ));
        assert_eq!(foreign.snapshot().expect("foreign snapshot").revision(), 0);
        owner
            .commit_catalog_molecule_placement_v1(&mut prepared)
            .expect("owner retry commits");
        assert!(matches!(
            owner.commit_catalog_molecule_placement_v1(&mut prepared),
            Err(CatalogMoleculePlacementRefusalV1::ReplayedGesture)
        ));
    }
}
