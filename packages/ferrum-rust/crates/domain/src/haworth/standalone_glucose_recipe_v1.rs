//! Closed, source-owned native D-glucose Haworth recipes.
//!
//! These are drawing recipes, rather than a carbohydrate name parser.  They
//! intentionally describe only the four structures offered by the native tool.

use thiserror::Error;

use crate::haworth::HaworthPoint;

/// The four supported, named D-glucose Haworth drawings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneDGlucoseHaworthRecipeV1 {
    AlphaDGlucopyranose,
    BetaDGlucopyranose,
    AlphaDGlucofuranose,
    BetaDGlucofuranose,
}

/// One source-owned atom fact, retained in canonical recipe order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StandaloneHaworthAtomV1 {
    role: &'static str,
    element: &'static str,
    local: HaworthPoint,
}
impl StandaloneHaworthAtomV1 {
    #[must_use]
    pub const fn role(self) -> &'static str {
        self.role
    }
    #[must_use]
    pub const fn element(self) -> &'static str {
        self.element
    }
    #[must_use]
    pub const fn local(self) -> HaworthPoint {
        self.local
    }
}

/// Persisted presentation fact for one single chemical edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneHaworthBondTokenV1 {
    N1,
    Q1,
    W1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneHaworthPositionV1 {
    Front,
    Back,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandaloneHaworthBondV1 {
    start: usize,
    end: usize,
    token: StandaloneHaworthBondTokenV1,
    position: Option<StandaloneHaworthPositionV1>,
}
impl StandaloneHaworthBondV1 {
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }
    #[must_use]
    pub const fn token(self) -> StandaloneHaworthBondTokenV1 {
        self.token
    }
    #[must_use]
    pub const fn position(self) -> Option<StandaloneHaworthPositionV1> {
        self.position
    }
}

/// Immutable closed recipe receipt used by the document authoring boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct StandaloneDGlucoseHaworthReceiptV1 {
    recipe: StandaloneDGlucoseHaworthRecipeV1,
    atoms: Vec<StandaloneHaworthAtomV1>,
    bonds: Vec<StandaloneHaworthBondV1>,
}
impl StandaloneDGlucoseHaworthReceiptV1 {
    #[must_use]
    pub const fn recipe(&self) -> StandaloneDGlucoseHaworthRecipeV1 {
        self.recipe
    }
    #[must_use]
    pub fn atoms(&self) -> &[StandaloneHaworthAtomV1] {
        &self.atoms
    }
    #[must_use]
    pub fn bonds(&self) -> &[StandaloneHaworthBondV1] {
        &self.bonds
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum StandaloneDGlucoseHaworthErrorV1 {
    #[error("standalone Haworth geometry must remain finite")]
    Geometry,
}

/// Build a literal 40-point normal-orientation D-glucose Haworth recipe.
pub fn standalone_d_glucose_haworth_recipe_v1(
    recipe: StandaloneDGlucoseHaworthRecipeV1,
) -> Result<StandaloneDGlucoseHaworthReceiptV1, StandaloneDGlucoseHaworthErrorV1> {
    let pyranose = matches!(
        recipe,
        StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose
            | StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose
    );
    let beta = matches!(
        recipe,
        StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose
            | StandaloneDGlucoseHaworthRecipeV1::BetaDGlucofuranose
    );
    let point = |x, y| HaworthPoint { x, y };
    let mut atoms = if pyranose {
        vec![
            atom("O5", "O", point(-40.0, -40.0)),
            atom("C1", "C", point(40.0, -40.0)),
            atom("C2", "C", point(60.0, 0.0)),
            atom("C3", "C", point(20.0, 30.0)),
            atom("C4", "C", point(-20.0, 30.0)),
            atom("C5", "C", point(-60.0, 0.0)),
            atom("C6", "C", point(-90.0, -35.0)),
            atom(
                "O1",
                "O",
                if beta {
                    point(70.0, -80.0)
                } else {
                    point(40.0, 40.0)
                },
            ),
            atom("O2", "O", point(95.0, 25.0)),
            atom("O3", "O", point(20.0, 75.0)),
            atom("O4", "O", point(-30.0, 75.0)),
            atom("O6", "O", point(-125.0, -60.0)),
        ]
    } else {
        vec![
            atom("O4", "O", point(-30.0, -35.0)),
            atom("C1", "C", point(40.0, -25.0)),
            atom("C2", "C", point(55.0, 20.0)),
            atom("C3", "C", point(0.0, 40.0)),
            atom("C4", "C", point(-45.0, 10.0)),
            atom("C5", "C", point(-85.0, -20.0)),
            atom("C6", "C", point(-115.0, -55.0)),
            atom("O1", "O", point(70.0, if beta { -65.0 } else { 15.0 })),
            atom("O2", "O", point(90.0, 45.0)),
            atom("O3", "O", point(0.0, 85.0)),
            atom("O5", "O", point(-90.0, 25.0)),
            atom("O6", "O", point(-145.0, -80.0)),
        ]
    };
    if atoms
        .iter()
        .any(|fact| !fact.local.x.is_finite() || !fact.local.y.is_finite())
    {
        return Err(StandaloneDGlucoseHaworthErrorV1::Geometry);
    }
    let ring = if pyranose { 6 } else { 5 };
    let mut bonds = Vec::new();
    // Canonical O-C1-C2-C3-C4[-C5]-O cycle. q1 is the lower front edge;
    // the two directed shoulders point from the outer carbons toward that edge.
    for index in 0..ring {
        let (start, end) = if index == 3 {
            (4, 3)
        } else {
            (index, (index + 1) % ring)
        };
        let (token, position) = if (start, end) == (2, 3) {
            (
                StandaloneHaworthBondTokenV1::Q1,
                Some(StandaloneHaworthPositionV1::Front),
            )
        } else if matches!((start, end), (1, 2) | (4, 3)) {
            (
                StandaloneHaworthBondTokenV1::W1,
                Some(StandaloneHaworthPositionV1::Front),
            )
        } else {
            (
                StandaloneHaworthBondTokenV1::N1,
                Some(StandaloneHaworthPositionV1::Back),
            )
        };
        bonds.push(StandaloneHaworthBondV1 {
            start,
            end,
            token,
            position,
        });
    }
    if pyranose {
        bonds.extend([
            edge(5, 6),
            edge(1, 7),
            edge(2, 8),
            edge(3, 9),
            edge(4, 10),
            edge(6, 11),
        ]);
    } else {
        bonds.extend([
            edge(4, 5),
            edge(5, 6),
            edge(1, 7),
            edge(2, 8),
            edge(3, 9),
            edge(5, 10),
            edge(6, 11),
        ]);
    }
    // Both forms have exactly twelve edges: furanose has five ring + two chain + five OH.
    Ok(StandaloneDGlucoseHaworthReceiptV1 {
        recipe,
        atoms: std::mem::take(&mut atoms),
        bonds,
    })
}

const fn atom(
    role: &'static str,
    element: &'static str,
    local: HaworthPoint,
) -> StandaloneHaworthAtomV1 {
    StandaloneHaworthAtomV1 {
        role,
        element,
        local,
    }
}
const fn edge(start: usize, end: usize) -> StandaloneHaworthBondV1 {
    StandaloneHaworthBondV1 {
        start,
        end,
        token: StandaloneHaworthBondTokenV1::N1,
        position: None,
    }
}
