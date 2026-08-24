//! Closed recipe lowering for Ferrum-authored catalog placement.
//!
//! The document session owns candidate construction, renderer admission, and
//! atomic mutation for both ordinary molecule and standalone Haworth requests.
use ferrum_document::{
    CatalogMoleculePlacementContentV1, CatalogMoleculePlacementV1, CatalogPlacementKeyV1,
    DocumentBondOrderV1, MoleculeInsertionAtomV1, MoleculeInsertionBondV1, MoleculeInsertionV1,
    Point3V1, PresentationGesturePoint2V1,
};
use ferrum_domain::{CatalogEntrySummaryV1, CatalogRecipeKindV1, catalog_entry_v1};
use thiserror::Error;

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
/// Resolve a catalog key and finite anchor into one closed document semantic operation.
pub fn resolve_catalog_molecule_placement_v1(
    key: &str,
    anchor: PresentationGesturePoint2V1,
) -> Result<CatalogMoleculePlacementV1, CatalogPlacementErrorV1> {
    let entry = catalog_entry_v1(key).ok_or(CatalogPlacementErrorV1::UnknownKey)?;
    let content = match entry.recipe() {
        CatalogRecipeKindV1::HaworthBiomolecule(value) => {
            CatalogMoleculePlacementContentV1::StandaloneHaworth(value)
        }
        kind => {
            let recipe = recipe(kind);
            let molecule = lower_recipe(entry, recipe, anchor)?;
            CatalogMoleculePlacementContentV1::Molecule(molecule)
        }
    };
    Ok(CatalogMoleculePlacementV1::new(
        CatalogPlacementKeyV1::new(key.to_owned())
            .map_err(|_| CatalogPlacementErrorV1::UnknownKey)?,
        anchor,
        content,
    ))
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
/// Lower catalog geometry into the public document insertion grammar. Durable
/// identities are intentionally absent: `DocumentSession` owns their allocation.
fn lower_recipe(
    entry: CatalogEntrySummaryV1,
    recipe: Recipe,
    anchor: PresentationGesturePoint2V1,
) -> Result<MoleculeInsertionV1, CatalogPlacementErrorV1> {
    if !anchor.x().is_finite() || !anchor.y().is_finite() {
        return Err(CatalogPlacementErrorV1::InvalidPoint);
    }
    let atom_points = local(recipe)
        .into_iter()
        .map(|(x, y)| (x + anchor.x(), y + anchor.y()))
        .collect::<Vec<_>>();
    let atoms = recipe
        .elements
        .iter()
        .zip(&atom_points)
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
                "n1" => DocumentBondOrderV1::Single,
                "n2" => DocumentBondOrderV1::Double,
                _ => return Err(CatalogPlacementErrorV1::RenderPreparation),
            };
            Ok(MoleculeInsertionBondV1::new(*start, *end, order))
        })
        .collect::<Result<Vec<_>, _>>()?;
    MoleculeInsertionV1::new(atoms, bonds)
        .and_then(|molecule| molecule.with_name(entry.label()))
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)
}
