//! Renderer-neutral, page-scoped composition of already verified render roots.
//!
//! This model deliberately has no CDML, session, Qt, or wire-decoding knowledge.
//! Its caller has already authenticated the page, provenance, root identity, and
//! exact Telex layout from one document observation.

use std::collections::HashSet;
use std::ops::Deref;

use crate::{
    DocumentVectorRootV1, GlyphBounds, MoleculeMemberDepictionIssueV1, MoleculeRenderPlan, Paint,
    PresentationTextOp, RenderError, RenderPoint, RenderProvenance, TextOp,
};

/// Paintable molecule batches plus diagnostics owned by its direct-root molecule.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMoleculeRenderContentV1 {
    plan: MoleculeRenderPlan,
    member_issues: Vec<MoleculeMemberDepictionIssueV1>,
}

impl DocumentMoleculeRenderContentV1 {
    /// Preserve one molecule plan and all diagnostics owned by its durable members.
    #[must_use]
    pub const fn new(
        plan: MoleculeRenderPlan,
        member_issues: Vec<MoleculeMemberDepictionIssueV1>,
    ) -> Self {
        Self {
            plan,
            member_issues,
        }
    }

    /// Return paintable molecule batches.
    #[must_use]
    pub const fn plan(&self) -> &MoleculeRenderPlan {
        &self.plan
    }

    /// Return diagnostics for exact molecule members, including mixed paintable molecules.
    #[must_use]
    pub fn member_issues(&self) -> &[MoleculeMemberDepictionIssueV1] {
        &self.member_issues
    }
}

impl Deref for DocumentMoleculeRenderContentV1 {
    type Target = MoleculeRenderPlan;

    fn deref(&self) -> &Self::Target {
        &self.plan
    }
}

/// A finite, positive scene rectangle supplied as the physical output page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewportV1 {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl RenderViewportV1 {
    /// Construct an explicit finite page rectangle with positive extents.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, RenderError> {
        if ![x, y, width, height].into_iter().all(f64::is_finite) || width <= 0.0 || height <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "document page must use finite coordinates and positive extents".to_owned(),
            ));
        }
        Ok(Self {
            x: canonical_zero(x),
            y: canonical_zero(y),
            width: canonical_zero(width),
            height: canonical_zero(height),
        })
    }

    /// Return the horizontal scene origin.
    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }
    /// Return the vertical scene origin.
    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
    /// Return the finite positive scene width.
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }
    /// Return the finite positive scene height.
    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
}

/// One already-positioned verified Telex operation at a direct-root anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTextOpV1 {
    anchor: RenderPoint,
    operation: DocumentTextLayoutV1,
    bounds: GlyphBounds,
    background: Option<Paint>,
}

impl DocumentTextOpV1 {
    /// Wrap one verified fixed-content text operation, such as a plus sign.
    pub fn fixed(
        anchor: RenderPoint,
        operation: TextOp,
        bounds: GlyphBounds,
        background: Option<Paint>,
    ) -> Result<Self, RenderError> {
        Self::with_layout(
            anchor,
            DocumentTextLayoutV1::Fixed(operation),
            bounds,
            background,
        )
    }

    /// Wrap one verified multi-run presentation Text operation.
    pub fn presentation(
        anchor: RenderPoint,
        operation: PresentationTextOp,
        bounds: GlyphBounds,
        background: Option<Paint>,
    ) -> Result<Self, RenderError> {
        Self::with_layout(
            anchor,
            DocumentTextLayoutV1::Presentation(operation),
            bounds,
            background,
        )
    }

    fn with_layout(
        anchor: RenderPoint,
        operation: DocumentTextLayoutV1,
        bounds: GlyphBounds,
        background: Option<Paint>,
    ) -> Result<Self, RenderError> {
        let edges = [
            bounds.min_x(),
            bounds.min_y(),
            bounds.max_x(),
            bounds.max_y(),
        ];
        if !edges.into_iter().all(f64::is_finite)
            || bounds.min_x() >= bounds.max_x()
            || bounds.min_y() >= bounds.max_y()
        {
            return Err(RenderError::InvalidRequest(
                "document text bounds must be finite and nonempty".to_owned(),
            ));
        }
        Ok(Self {
            anchor,
            operation,
            bounds,
            background,
        })
    }

    /// Return the authored scene anchor.
    #[must_use]
    pub const fn anchor(&self) -> RenderPoint {
        self.anchor
    }
    /// Return the already verified Telex layout without reshaping it.
    #[must_use]
    pub fn operation(&self) -> &DocumentTextLayoutV1 {
        &self.operation
    }
    /// Return the anchor-local ink bounds.
    #[must_use]
    pub const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }
    /// Return the optional explicit background paint.
    #[must_use]
    pub fn background(&self) -> Option<&Paint> {
        self.background.as_ref()
    }
}

/// A preserved verified Telex layout for one direct-root text item.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentTextLayoutV1 {
    /// One fixed-content operation, currently used by a plus sign.
    Fixed(TextOp),
    /// One multi-run ordinary Text operation with preserved script layout.
    Presentation(PresentationTextOp),
}

/// One root that the current renderer can paint.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentRenderContentV1 {
    /// A complete existing molecule render plan.
    Molecule(DocumentMoleculeRenderContentV1),
    /// One complete existing text operation, including plus signs.
    Text(DocumentTextOpV1),
    /// One checked generic vector root in document-local paint order.
    Vector(DocumentVectorRootV1),
}

/// A paintable direct-root item in explicit renderer paint order.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderRootV1 {
    target: crate::RenderTarget,
    paint_order: u32,
    content: DocumentRenderContentV1,
}

impl DocumentRenderRootV1 {
    /// Construct one paintable direct-root item.
    #[must_use]
    pub const fn new(
        target: crate::RenderTarget,
        paint_order: u32,
        content: DocumentRenderContentV1,
    ) -> Self {
        Self {
            target,
            paint_order,
            content,
        }
    }

    /// Return the durable root target.
    #[must_use]
    pub const fn target(&self) -> &crate::RenderTarget {
        &self.target
    }
    /// Return the root's explicit renderer paint order.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }
    /// Return the renderer-owned content.
    #[must_use]
    pub const fn content(&self) -> &DocumentRenderContentV1 {
        &self.content
    }
}

/// A valid root intentionally omitted because its operation has no renderer equivalent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRenderExclusionV1 {
    target: crate::RenderTarget,
    paint_order: u32,
    feature: String,
}

impl DocumentRenderExclusionV1 {
    /// Record one named, intentionally unpainted direct root.
    pub fn new(
        target: crate::RenderTarget,
        paint_order: u32,
        feature: impl Into<String>,
    ) -> Result<Self, RenderError> {
        let feature = feature.into();
        if feature.trim().is_empty() || feature.chars().any(char::is_control) {
            return Err(RenderError::InvalidRequest(
                "document render exclusion must name a visible feature".to_owned(),
            ));
        }
        Ok(Self {
            target,
            paint_order,
            feature,
        })
    }

    /// Return the durable excluded root target.
    #[must_use]
    pub const fn target(&self) -> &crate::RenderTarget {
        &self.target
    }
    /// Return the excluded root's explicit renderer paint order.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }
    /// Return the named renderer gap.
    #[must_use]
    pub fn feature(&self) -> &str {
        &self.feature
    }
}

/// One direct-root outcome in explicit renderer paint order.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentRenderOutcomeV1 {
    /// A root the current renderer can paint.
    Root(DocumentRenderRootV1),
    /// A named root intentionally omitted from this renderer slice.
    Exclusion(DocumentRenderExclusionV1),
}

impl DocumentRenderOutcomeV1 {
    /// Return the explicit paint order shared by all outcome kinds.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        match self {
            Self::Root(root) => root.paint_order(),
            Self::Exclusion(exclusion) => exclusion.paint_order(),
        }
    }

    /// Return the durable target shared by all outcome kinds.
    #[must_use]
    pub const fn target(&self) -> &crate::RenderTarget {
        match self {
            Self::Root(root) => root.target(),
            Self::Exclusion(exclusion) => exclusion.target(),
        }
    }
}

/// Immutable renderer-neutral plan for one physical document page.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderPlanV1 {
    provenance: RenderProvenance,
    page: RenderViewportV1,
    outcomes: Vec<DocumentRenderOutcomeV1>,
}

impl DocumentRenderPlanV1 {
    /// Construct a page plan from one observation's authenticated render facts.
    pub fn new(
        provenance: RenderProvenance,
        page: RenderViewportV1,
        outcomes: Vec<DocumentRenderOutcomeV1>,
    ) -> Result<Self, RenderError> {
        let mut targets = HashSet::new();
        let mut paint_orders = HashSet::new();
        targets
            .try_reserve(outcomes.len())
            .map_err(|_| RenderError::ResourceExhausted)?;
        paint_orders
            .try_reserve(outcomes.len())
            .map_err(|_| RenderError::ResourceExhausted)?;
        let mut previous = None;
        for outcome in &outcomes {
            if let DocumentRenderOutcomeV1::Root(root) = outcome
                && let DocumentRenderContentV1::Molecule(plan) = root.content()
                && plan.provenance() != provenance
            {
                return Err(RenderError::InvalidRequest(
                    "document molecule root provenance differs from the page plan".to_owned(),
                ));
            }
            validate_root_key(
                outcome.paint_order(),
                outcome.target(),
                &mut paint_orders,
                &mut targets,
            )?;
            if let Some(previous) = previous
                && outcome.paint_order() <= previous
            {
                return Err(RenderError::InvalidRequest(
                    "document render outcomes must have strictly increasing paint order".to_owned(),
                ));
            }
            previous = Some(outcome.paint_order());
        }
        Ok(Self {
            provenance,
            page,
            outcomes,
        })
    }

    /// Return the exact source observation provenance.
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    /// Return the physical page rectangle.
    #[must_use]
    pub const fn page(&self) -> RenderViewportV1 {
        self.page
    }
    /// Return every paintable or excluded root in strict paint order.
    #[must_use]
    pub fn outcomes(&self) -> &[DocumentRenderOutcomeV1] {
        &self.outcomes
    }
}

fn validate_root_key(
    paint_order: u32,
    target: &crate::RenderTarget,
    paint_orders: &mut HashSet<u32>,
    targets: &mut HashSet<ferrum_document_projection::DocumentObjectIdV1>,
) -> Result<(), RenderError> {
    if !paint_orders.insert(paint_order) {
        return Err(RenderError::InvalidRequest(
            "document render roots and exclusions must have unique paint orders".to_owned(),
        ));
    }
    if !targets.insert(target.document_object_id().clone()) {
        return Err(RenderError::InvalidRequest(
            "document render roots and exclusions must have unique durable targets".to_owned(),
        ));
    }
    Ok(())
}

const fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
