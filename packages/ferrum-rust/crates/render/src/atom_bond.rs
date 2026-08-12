//! Atom-label and ordinary single-bond render-plan generation.

use std::collections::{HashMap, HashSet};

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::{Point2, Vector2};

use crate::{
    BatchSpace, FontFace, GlyphBounds, GlyphMetrics, LineOp, MoleculeRenderPlan, Paint,
    PositiveFinite, RenderBatch, RenderError, RenderIssue, RenderIssueKind, RenderOp, RenderPoint,
    RenderRevision, RenderTarget, TextOp, TextScript,
};

/// Complete atom-label presentation facts required by this render slice.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomLabelFontProfile {
    face: FontFace,
    size: PositiveFinite,
    paint: Paint,
}

impl AtomLabelFontProfile {
    /// Construct an exact label presentation profile without renderer defaults.
    #[must_use]
    pub const fn new(face: FontFace, size: PositiveFinite, paint: Paint) -> Self {
        Self { face, size, paint }
    }

    /// Return the exact requested face.
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    /// Return the exact requested text size.
    #[must_use]
    pub const fn size(&self) -> PositiveFinite {
        self.size
    }

    /// Return the exact requested label paint.
    #[must_use]
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }
}

/// Source facts that produce a structured atom label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomLabelFacts {
    element: String,
    formal_charge: i8,
    explicit_hydrogens: u8,
}

impl AtomLabelFacts {
    /// Construct validated atom-label facts.
    pub fn new(
        element: impl Into<String>,
        formal_charge: i8,
        explicit_hydrogens: u8,
    ) -> Result<Self, RenderError> {
        let element = element.into();
        let mut characters = element.chars();
        let Some(first) = characters.next() else {
            return Err(RenderError::InvalidRequest(
                "atom element must not be blank".to_owned(),
            ));
        };
        if !first.is_ascii_uppercase()
            || characters.clone().count() > 2
            || !characters.all(|character| character.is_ascii_lowercase())
        {
            return Err(RenderError::InvalidRequest(
                "atom element must use one uppercase letter followed by at most two lowercase letters"
                    .to_owned(),
            ));
        }
        Ok(Self {
            element,
            formal_charge,
            explicit_hydrogens,
        })
    }

    /// Return the declared element symbol.
    #[must_use]
    pub fn element(&self) -> &str {
        &self.element
    }

    /// Return the formal charge supplied by the source model.
    #[must_use]
    pub const fn formal_charge(&self) -> i8 {
        self.formal_charge
    }

    /// Return the source model's explicit hydrogen count.
    #[must_use]
    pub const fn explicit_hydrogens(&self) -> u8 {
        self.explicit_hydrogens
    }

    /// Return ordered source text segments before a metric provider lays them out.
    pub(crate) fn text_pieces(&self) -> Vec<(String, TextScript)> {
        let mut runs = vec![(self.element.clone(), TextScript::Baseline)];
        if self.explicit_hydrogens > 0 {
            runs.push(("H".to_owned(), TextScript::Baseline));
            if self.explicit_hydrogens > 1 {
                runs.push((self.explicit_hydrogens.to_string(), TextScript::Subscript));
            }
        }
        if self.formal_charge != 0 {
            let sign = if self.formal_charge > 0 { '+' } else { '-' };
            let magnitude = self.formal_charge.unsigned_abs();
            let charge = if magnitude == 1 {
                sign.to_string()
            } else {
                format!("{magnitude}{sign}")
            };
            runs.push((charge, TextScript::Superscript));
        }
        runs
    }
}

/// Deliberate visibility supplied by the authoritative source projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetVisibility {
    /// The target is part of this visible render projection.
    Visible,
    /// The target must remain absent with an explanatory non-fatal diagnostic.
    Hidden { reason: String },
}

impl TargetVisibility {
    fn issue(&self, noun: &str) -> Option<RenderIssueKind> {
        match self {
            Self::Visible => None,
            Self::Hidden { reason } if !reason.trim().is_empty() => {
                Some(RenderIssueKind::UnsupportedFeature {
                    feature: format!("invisible {noun}: {reason}"),
                })
            }
            Self::Hidden { .. } => Some(RenderIssueKind::UnsupportedFeature {
                feature: format!("invisible {noun}"),
            }),
        }
    }
}

/// Bond style carried by the source projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BondStyle {
    /// The sole supported bond style in this vertical slice.
    NormalSingle,
    /// A parallel double bond.
    Double,
    /// A parallel triple bond.
    Triple,
    /// An aromatic bond.
    Aromatic,
    /// A solid stereochemical wedge.
    SolidWedge,
    /// A hashed stereochemical wedge.
    HashedWedge,
    /// A dashed bond.
    Dashed,
}

impl BondStyle {
    fn unsupported_name(self) -> Option<&'static str> {
        match self {
            Self::NormalSingle => None,
            Self::Double => Some("double bond"),
            Self::Triple => Some("triple bond"),
            Self::Aromatic => Some("aromatic bond"),
            Self::SolidWedge => Some("solid wedge bond"),
            Self::HashedWedge => Some("hashed wedge bond"),
            Self::Dashed => Some("dashed bond"),
        }
    }
}

/// An atom with explicit source identity, finite position, and label facts.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomRenderTarget {
    target: RenderTarget,
    position: RenderPoint,
    label: AtomLabelFacts,
    visibility: TargetVisibility,
}

impl AtomRenderTarget {
    /// Construct a valid atom target for this render slice.
    pub fn new(
        target: RenderTarget,
        position: RenderPoint,
        label: AtomLabelFacts,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if target.record_id().kind() != RecordKind::Atom {
            return Err(RenderError::InvalidRequest(
                "atom render target requires an atom RecordId".to_owned(),
            ));
        }
        Ok(Self {
            target,
            position,
            label,
            visibility,
        })
    }

    /// Return the durable target and source order.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }
}

/// A bond with explicit endpoint atom identities and source style facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondRenderTarget {
    target: RenderTarget,
    first_atom: RecordId,
    second_atom: RecordId,
    style: BondStyle,
    visibility: TargetVisibility,
}

impl BondRenderTarget {
    /// Construct a valid bond target for this render slice.
    pub fn new(
        target: RenderTarget,
        first_atom: RecordId,
        second_atom: RecordId,
        style: BondStyle,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if target.record_id().kind() != RecordKind::Bond {
            return Err(RenderError::InvalidRequest(
                "bond render target requires a bond RecordId".to_owned(),
            ));
        }
        if first_atom.kind() != RecordKind::Atom || second_atom.kind() != RecordKind::Atom {
            return Err(RenderError::InvalidRequest(
                "bond endpoints require atom RecordIds".to_owned(),
            ));
        }
        Ok(Self {
            target,
            first_atom,
            second_atom,
            style,
            visibility,
        })
    }

    /// Return the durable target and source order.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }
}

/// A complete, order-explicit request for atom labels and ordinary single bonds.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomBondRenderRequest {
    revision: RenderRevision,
    atoms: Vec<AtomRenderTarget>,
    bonds: Vec<BondRenderTarget>,
    font: AtomLabelFontProfile,
    line_width: PositiveFinite,
    line_paint: Paint,
}

impl AtomBondRenderRequest {
    /// Construct a request whose target identities and source orders are unique.
    pub fn new(
        revision: RenderRevision,
        atoms: Vec<AtomRenderTarget>,
        bonds: Vec<BondRenderTarget>,
        font: AtomLabelFontProfile,
        line_width: PositiveFinite,
        line_paint: Paint,
    ) -> Result<Self, RenderError> {
        let mut identifiers = HashSet::new();
        let mut source_orders = HashSet::new();
        for target in atoms
            .iter()
            .map(AtomRenderTarget::target)
            .chain(bonds.iter().map(BondRenderTarget::target))
        {
            if !identifiers.insert(target.record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate targets".to_owned(),
                ));
            }
            if !source_orders.insert(target.source_order()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate source order".to_owned(),
                ));
            }
        }
        Ok(Self {
            revision,
            atoms,
            bonds,
            font,
            line_width,
            line_paint,
        })
    }
}

/// Build the total ordered batch-or-issue partition for this render slice.
pub fn build_atom_bond_plan<M: GlyphMetrics>(
    request: &AtomBondRenderRequest,
    metrics: &M,
) -> Result<MoleculeRenderPlan, RenderError> {
    let mut batches = Vec::new();
    let mut issues = Vec::new();
    let mut atoms = HashMap::new();

    for atom in &request.atoms {
        let target = atom.target.clone();
        let outcome = atom.visibility.issue("atom target").map_or_else(
            || build_atom_batch(atom, &request.font, metrics),
            |kind| Ok(Err(kind)),
        );
        match outcome? {
            Ok((batch, bounds)) => {
                atoms.insert(
                    target.record_id().clone(),
                    AtomGeometry {
                        position: render_point_to_geometry(atom.position)?,
                        bounds,
                    },
                );
                batches.push(batch);
            }
            Err(kind) => issues.push(RenderIssue::new(target, kind)?),
        }
    }

    for bond in &request.bonds {
        let target = bond.target.clone();
        let outcome = if let Some(kind) = bond.visibility.issue("bond target") {
            Err(kind)
        } else if let Some(style) = bond.style.unsupported_name() {
            Err(RenderIssueKind::UnsupportedFeature {
                feature: style.to_owned(),
            })
        } else {
            build_bond_batch(bond, &atoms, request.line_width, request.line_paint.clone())
        };
        match outcome {
            Ok(batch) => batches.push(batch),
            Err(kind) => issues.push(RenderIssue::new(target, kind)?),
        }
    }

    batches.sort_by_key(|batch| batch.target().source_order());
    issues.sort_by_key(|issue| issue.target().source_order());
    MoleculeRenderPlan::new(request.revision, batches, issues)
}

struct AtomGeometry {
    position: Point2,
    bounds: GlyphBounds,
}

fn build_atom_batch<M: GlyphMetrics>(
    atom: &AtomRenderTarget,
    font: &AtomLabelFontProfile,
    metrics: &M,
) -> Result<Result<(RenderBatch, GlyphBounds), RenderIssueKind>, RenderError> {
    let layout = match metrics.layout_atom_label(&atom.label, font) {
        Ok(layout) => layout,
        Err(error) => {
            return Ok(Err(RenderIssueKind::UnrenderableTarget {
                reason: format!("atom label metrics unavailable: {error}"),
            }));
        }
    };
    let operation = TextOp::new(
        RenderPoint::new(0.0, 0.0)?,
        layout.runs().to_vec(),
        font.face.clone(),
        font.size,
        font.paint.clone(),
        0,
    )?;
    let batch = RenderBatch::new(
        atom.target.clone(),
        BatchSpace::AtomLocal {
            anchor: atom.position,
        },
        vec![RenderOp::Text(operation)],
    )?;
    Ok(Ok((batch, layout.bounds())))
}

fn build_bond_batch(
    bond: &BondRenderTarget,
    atoms: &HashMap<RecordId, AtomGeometry>,
    width: PositiveFinite,
    paint: Paint,
) -> Result<RenderBatch, RenderIssueKind> {
    let Some(first) = atoms.get(&bond.first_atom) else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "first bond endpoint has no renderable visible atom label".to_owned(),
        });
    };
    let Some(second) = atoms.get(&bond.second_atom) else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "second bond endpoint has no renderable visible atom label".to_owned(),
        });
    };
    let vector = second.position - first.position;
    let length = vector.length();
    if !length.is_finite() || length == 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond endpoints are coincident or not representable".to_owned(),
        });
    }
    let direction = Vector2::new(vector.x() / length, vector.y() / length).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("bond direction is not representable: {error}"),
        }
    })?;
    let first_clip = clip_distance(first.bounds, direction)?;
    let second_clip = clip_distance(second.bounds, negated(direction)?)?;
    let remaining_length = length - first_clip - second_clip;
    if !remaining_length.is_finite() || remaining_length <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "label clipping leaves no positive visible bond segment".to_owned(),
        });
    }
    let start = first
        .position
        .offset(direction, first_clip)
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond start is not representable: {error}"),
        })?;
    let end = second
        .position
        .offset(negated(direction)?, second_clip)
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond end is not representable: {error}"),
        })?;
    let start = geometry_to_render_point(start)?;
    let end = geometry_to_render_point(end)?;
    let line = LineOp::new(start, end, width, paint, 0).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("clipped bond is not renderable: {error}"),
        }
    })?;
    RenderBatch::new(
        bond.target.clone(),
        BatchSpace::Scene,
        vec![RenderOp::Line(line)],
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond batch is not renderable: {error}"),
    })
}

fn clip_distance(bounds: GlyphBounds, direction: Vector2) -> Result<f64, RenderIssueKind> {
    let x = if direction.x() > 0.0 {
        bounds.max_x() / direction.x()
    } else if direction.x() < 0.0 {
        bounds.min_x() / direction.x()
    } else {
        f64::INFINITY
    };
    let y = if direction.y() > 0.0 {
        bounds.max_y() / direction.y()
    } else if direction.y() < 0.0 {
        bounds.min_y() / direction.y()
    } else {
        f64::INFINITY
    };
    let distance = x.min(y);
    if !distance.is_finite() || distance <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "glyph clipping distance is not finite and positive".to_owned(),
        });
    }
    Ok(distance)
}

fn negated(vector: Vector2) -> Result<Vector2, RenderIssueKind> {
    Vector2::new(-vector.x(), -vector.y()).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond direction is not representable: {error}"),
    })
}

fn render_point_to_geometry(point: RenderPoint) -> Result<Point2, RenderError> {
    Point2::new(point.x(), point.y()).map_err(|error| {
        RenderError::InvalidRequest(format!("render point is invalid geometry: {error}"))
    })
}

fn geometry_to_render_point(point: Point2) -> Result<RenderPoint, RenderIssueKind> {
    RenderPoint::new(point.x(), point.y()).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("derived bond point is not finite: {error}"),
    })
}
