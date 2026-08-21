//! Renderer-preflighted placement for Ferrum-authored catalog recipes.
//!
//! Haworth entries stay out of this catalog: their native receipts retain
//! stereochemical display tokens which ordinary detached CDML cannot yet carry.
use ferrum_document::{
    DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1, SessionOperationResultV1,
};
use ferrum_domain::{
    CatalogEntrySummaryV1, CatalogRecipeKindV1, catalog_entry_v1,
    haworth::{
        StandaloneDGlucoseHaworthRecipeV1, StandaloneHaworthBondTokenV1,
        StandaloneHaworthPositionV1, standalone_d_glucose_haworth_recipe_v1,
    },
};
use ferrum_render::{
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct CatalogPlacementGestureV1 {
    origin: u64,
    nonce: u64,
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
    origin: u64,
    nonce: u64,
    fence: DocumentFenceV1,
    key: String,
    identifier: String,
    candidate: String,
    digest: [u8; 32],
    plan: DocumentRenderPlanV1,
    contract: ferrum_render_contract::PreflightedDocumentRenderV1,
}
#[derive(Clone, Debug)]
struct Namespace {
    root: String,
    atoms: Vec<String>,
    bonds: Vec<String>,
}
impl Namespace {
    fn new(recipe: Recipe, index: u64) -> Self {
        let root = format!("ferrum-catalog-{}-{index}", recipe.slug);
        Self {
            atoms: (1..=recipe.elements.len())
                .map(|n| format!("{root}-a{n}"))
                .collect(),
            bonds: (1..=recipe.edges.len())
                .map(|n| format!("{root}-b{n}"))
                .collect(),
            root,
        }
    }
    fn ids(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.root.as_str())
            .chain(self.atoms.iter().map(String::as_str))
            .chain(self.bonds.iter().map(String::as_str))
    }
}
#[derive(Clone, Copy)]
struct Recipe {
    slug: &'static str,
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
            slug: "benzene",
            elements: &["C", "C", "C", "C", "C", "C"],
            edges: B6,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclopropane => Recipe {
            slug: "cyclopropane",
            elements: &["C", "C", "C"],
            edges: S3,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclobutane => Recipe {
            slug: "cyclobutane",
            elements: &["C", "C", "C", "C"],
            edges: S4,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclopentane => Recipe {
            slug: "cyclopentane",
            elements: &["C", "C", "C", "C", "C"],
            edges: S5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Cyclohexane => Recipe {
            slug: "cyclohexane",
            elements: &["C", "C", "C", "C", "C", "C"],
            edges: S6,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Thiophene => Recipe {
            slug: "thiophene",
            elements: &["S", "C", "C", "C", "C"],
            edges: H5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Furan => Recipe {
            slug: "furan",
            elements: &["O", "C", "C", "C", "C"],
            edges: H5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Pyrrole => Recipe {
            slug: "pyrrole",
            elements: &["N", "C", "C", "C", "C"],
            edges: H5,
            shape: Shape::Ring,
        },
        CatalogRecipeKindV1::Purine => Recipe {
            slug: "purine",
            elements: &["N", "C", "N", "C", "C", "N", "C", "N", "C"],
            edges: PURINE,
            shape: Shape::Purine,
        },
        CatalogRecipeKindV1::HaworthBiomolecule(_) => {
            unreachable!("Haworth catalog recipes use the literal depiction compiler")
        }
    }
}
fn consumed() -> &'static Mutex<HashSet<(u64, u64)>> {
    static V: OnceLock<Mutex<HashSet<(u64, u64)>>> = OnceLock::new();
    V.get_or_init(|| Mutex::new(HashSet::new()))
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
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Ok(CatalogPlacementGestureV1 {
        origin: session.bridge_session_origin_v1(),
        nonce: NEXT.fetch_add(1, Ordering::Relaxed),
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
    if gesture.origin != session.bridge_session_origin_v1() {
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
    if gesture.origin != session.bridge_session_origin_v1()
        || preview.gesture.origin != session.bridge_session_origin_v1()
    {
        return Err(CatalogPlacementErrorV1::ForeignSession);
    }
    if gesture.nonce != preview.gesture.nonce || gesture.key != preview.gesture.key {
        return Err(CatalogPlacementErrorV1::MismatchedPreview);
    }
    if consumed()
        .lock()
        .expect("catalog lock")
        .contains(&(gesture.origin, gesture.nonce))
    {
        return Err(CatalogPlacementErrorV1::ReplayedGesture);
    }
    fence(session, gesture.fence)?;
    let source = session
        .snapshot()
        .map_err(|_| CatalogPlacementErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    let (ns, candidate) = match gesture.entry.recipe() {
        CatalogRecipeKindV1::HaworthBiomolecule(value) => {
            let receipt = standalone_d_glucose_haworth_recipe_v1(value)
                .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
            let ns = fresh_haworth(session, &receipt, gesture.nonce);
            let candidate = append_haworth(&source, gesture.entry, &receipt, &ns, preview.anchor)?;
            (ns, candidate)
        }
        kind => {
            let recipe = recipe(kind);
            let ns = fresh(session, recipe, gesture.nonce);
            let candidate = append(&source, gesture.entry, recipe, &ns, preview.anchor)?;
            (ns, candidate)
        }
    };
    let contract = ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    let candidate_session = DocumentSession::load(&candidate)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
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
    let digest = *candidate_session
        .snapshot()
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?
        .digest();
    Ok(PreparedCatalogPlacementV1 {
        identifier: ns.root.clone(),
        receipt: Some(CatalogReceiptV1 {
            origin: gesture.origin,
            nonce: gesture.nonce,
            fence: gesture.fence,
            key: gesture.key.clone(),
            identifier: ns.root,
            candidate,
            digest,
            plan,
            contract,
        }),
    })
}
pub fn commit_catalog_placement_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCatalogPlacementV1,
) -> Result<CommittedCatalogPlacementV1, CatalogPlacementErrorV1> {
    let receipt = prepared
        .receipt
        .as_ref()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
    if receipt.origin != session.bridge_session_origin_v1() {
        return Err(CatalogPlacementErrorV1::ForeignSession);
    }
    if consumed()
        .lock()
        .expect("catalog lock")
        .contains(&(receipt.origin, receipt.nonce))
    {
        return Err(CatalogPlacementErrorV1::ReplayedGesture);
    }
    fence(session, receipt.fence)?;
    if catalog_entry_v1(&receipt.key).is_none()
        || receipt.identifier != prepared.identifier
        || receipt.contract.source() != receipt.candidate
        || receipt
            .plan
            .outcomes()
            .iter()
            .any(|o| matches!(o, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(CatalogPlacementErrorV1::RenderPreparation);
    }
    let check = DocumentSession::load(&receipt.candidate)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    if *check
        .snapshot()
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?
        .digest()
        != receipt.digest
    {
        return Err(CatalogPlacementErrorV1::RenderPreparation);
    }
    let result = session.commit_complete_cdml_transaction_v1(receipt.fence, &receipt.candidate);
    let result = result.map_err(|_| CatalogPlacementErrorV1::SessionConflict)?;
    let receipt = prepared
        .receipt
        .take()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
    consumed()
        .lock()
        .expect("catalog lock")
        .insert((receipt.origin, receipt.nonce));
    Ok(CommittedCatalogPlacementV1 {
        identifier: receipt.identifier,
        result,
    })
}
fn fresh(session: &DocumentSession, recipe: Recipe, nonce: u64) -> Namespace {
    let mut index = nonce;
    loop {
        let candidate = Namespace::new(recipe, index);
        if candidate
            .ids()
            .all(|id| !session.contains_durable_id_v1(id))
        {
            return candidate;
        }
        index += 1;
    }
}
fn fresh_haworth(
    session: &DocumentSession,
    receipt: &ferrum_domain::haworth::StandaloneDGlucoseHaworthReceiptV1,
    nonce: u64,
) -> Namespace {
    let mut index = nonce;
    loop {
        let root = format!("ferrum-catalog-d-glucose-haworth-{index}");
        let candidate = Namespace {
            atoms: (1..=receipt.atoms().len())
                .map(|n| format!("{root}-a{n}"))
                .collect(),
            bonds: (1..=receipt.bonds().len())
                .map(|n| format!("{root}-b{n}"))
                .collect(),
            root,
        };
        if candidate
            .ids()
            .all(|id| !session.contains_durable_id_v1(id))
        {
            return candidate;
        }
        index += 1;
    }
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
fn append(
    source: &str,
    entry: CatalogEntrySummaryV1,
    recipe: Recipe,
    ns: &Namespace,
    anchor: PresentationGesturePoint2V1,
) -> Result<String, CatalogPlacementErrorV1> {
    let view = overlay(recipe, anchor)?;
    let atoms = view
        .atom_points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            format!(
                "<atom id=\"{}\" name=\"{}\"><point x=\"{}\" y=\"{}\"/></atom>",
                ns.atoms[i], recipe.elements[i], p.0, p.1
            )
        })
        .collect::<String>();
    let bonds = recipe
        .edges
        .iter()
        .enumerate()
        .map(|(i, (s, e, kind))| {
            format!(
                "<bond id=\"{}\" start=\"{}\" end=\"{}\" type=\"{}\"/>",
                ns.bonds[i], ns.atoms[*s], ns.atoms[*e], kind
            )
        })
        .collect::<String>();
    let root = format!(
        "<molecule id=\"{}\" name=\"{}\">{atoms}{bonds}</molecule>",
        ns.root,
        entry.label()
    );
    if let Some(close) = source.rfind("</cdml") {
        Ok(format!("{}{}{}", &source[..close], root, &source[close..]))
    } else {
        let close = source
            .rfind("/>")
            .filter(|i| source[*i + 2..].trim().is_empty())
            .ok_or(CatalogPlacementErrorV1::RenderPreparation)?;
        Ok(format!("{}>{root}</cdml>", &source[..close]))
    }
}
fn append_haworth(
    source: &str,
    entry: CatalogEntrySummaryV1,
    receipt: &ferrum_domain::haworth::StandaloneDGlucoseHaworthReceiptV1,
    ns: &Namespace,
    anchor: PresentationGesturePoint2V1,
) -> Result<String, CatalogPlacementErrorV1> {
    let view = haworth_overlay(receipt.recipe(), anchor)?;
    let atoms = receipt
        .atoms()
        .iter()
        .enumerate()
        .map(|(index, fact)| {
            let point = view.atom_points[index];
            format!(
                "<atom id=\"{}\" name=\"{}\"><point x=\"{}\" y=\"{}\"/></atom>",
                ns.atoms[index],
                fact.element(),
                point.0,
                point.1
            )
        })
        .collect::<String>();
    let bonds = receipt
        .bonds()
        .iter()
        .enumerate()
        .map(|(index, fact)| {
            let token = match fact.token() {
                StandaloneHaworthBondTokenV1::N1 => "n1",
                StandaloneHaworthBondTokenV1::Q1 => "q1",
                StandaloneHaworthBondTokenV1::W1 => "w1",
            };
            let position = match fact.position() {
                None => "",
                Some(StandaloneHaworthPositionV1::Front) => " position=\"front\"",
                Some(StandaloneHaworthPositionV1::Back) => " position=\"back\"",
            };
            format!(
                "<bond id=\"{}\" start=\"{}\" end=\"{}\" type=\"{}\"{position}/>",
                ns.bonds[index],
                ns.atoms[fact.start()],
                ns.atoms[fact.end()],
                token
            )
        })
        .collect::<String>();
    let root = format!(
        "<molecule id=\"{}\" name=\"{}\">{atoms}{bonds}</molecule>",
        ns.root,
        entry.label()
    );
    if let Some(close) = source.rfind("</cdml") {
        Ok(format!("{}{}{}", &source[..close], root, &source[close..]))
    } else {
        let close = source
            .rfind("/>")
            .filter(|index| source[*index + 2..].trim().is_empty())
            .ok_or(CatalogPlacementErrorV1::RenderPreparation)?;
        Ok(format!("{}>{root}</cdml>", &source[..close]))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const EMPTY: &str = "<cdml/>";
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
        for (key, elements, edges, label) in [
            (
                "system/rings/benzene",
                &["C", "C", "C", "C", "C", "C"][..],
                B6,
                "Benzene",
            ),
            (
                "system/rings/cyclopropane",
                &["C", "C", "C"][..],
                S3,
                "Cyclopropane",
            ),
            (
                "system/rings/cyclobutane",
                &["C", "C", "C", "C"][..],
                S4,
                "Cyclobutane",
            ),
            (
                "system/rings/cyclopentane",
                &["C", "C", "C", "C", "C"][..],
                S5,
                "Cyclopentane",
            ),
            (
                "system/rings/cyclohexane",
                &["C", "C", "C", "C", "C", "C"][..],
                S6,
                "Cyclohexane",
            ),
            (
                "system/heterocycles/thiophene",
                &["S", "C", "C", "C", "C"][..],
                H5,
                "Thiophene",
            ),
            (
                "system/heterocycles/furan",
                &["O", "C", "C", "C", "C"][..],
                H5,
                "Furan",
            ),
            (
                "system/heterocycles/pyrrole",
                &["N", "C", "C", "C", "C"][..],
                H5,
                "Pyrrole",
            ),
            (
                "system/heterocycles/purine",
                &["N", "C", "N", "C", "C", "N", "C", "N", "C"][..],
                PURINE,
                "Purine",
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
            assert!(
                c.result()
                    .observation()
                    .snapshot()
                    .cdml()
                    .contains(&format!("name=\"{label}\""))
            );
            assert!(s.undo(1).is_ok());
        }
    }
    #[test]
    fn largest_recipe_reserves_final_atom_and_bond_identifiers_before_assembly() {
        let s = DocumentSession::load(
            "<cdml><opaque id=\"ferrum-catalog-purine-1-a9\"/><opaque id=\"ferrum-catalog-purine-1-b10\"/></cdml>",
        )
        .expect("source");
        let authored = recipe(CatalogRecipeKindV1::Purine);
        let ns = fresh(&s, authored, 1);
        assert_eq!(ns.root, "ferrum-catalog-purine-2");
        assert!(ns.ids().all(|id| !s.contains_durable_id_v1(id)));
        let candidate = append(
            s.snapshot().expect("snapshot").cdml(),
            catalog_entry_v1("system/heterocycles/purine").expect("entry"),
            authored,
            &ns,
            PresentationGesturePoint2V1::new(0.0, 0.0).expect("anchor"),
        )
        .expect("candidate");
        assert!(candidate.contains("ferrum-catalog-purine-2-a9"));
        assert!(candidate.contains("ferrum-catalog-purine-2-b10"));
    }
    #[test]
    fn reservation_covers_every_emitted_identifier() {
        let s =
            DocumentSession::load("<cdml><opaque id=\"ferrum-catalog-thiophene-1-a1\"/></cdml>")
                .expect("source");
        let ns = fresh(&s, recipe(CatalogRecipeKindV1::Thiophene), 1);
        assert_eq!(ns.root, "ferrum-catalog-thiophene-2");
        assert!(ns.ids().all(|id| !s.contains_durable_id_v1(id)));
    }
}
