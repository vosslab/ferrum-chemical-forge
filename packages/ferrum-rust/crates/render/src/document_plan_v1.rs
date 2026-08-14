//! Renderer-neutral, page-scoped composition of already verified render roots.
//!
//! This model deliberately has no CDML, session, Qt, or wire-decoding knowledge.
//! Its caller has already authenticated the page, provenance, root identity, and
//! exact Telex layout from one document observation.

use std::collections::HashSet;

use crate::{
    DocumentVectorRootV1, GlyphBounds, MoleculeRenderPlan, Paint, PresentationTextOp, RenderError,
    RenderPoint, RenderProvenance, TextOp,
};

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

/// An exact root identity, either durable or local to its immutable projection.
///
/// A projection-local key identifies a displayed root without claiming that the
/// root has an authored durable ID or can be used as a mutation selector.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DocumentRenderIdentityV1 {
    /// An authored durable root identity supplied by the authoritative caller.
    Durable(String),
    /// A projection-local root key supplied by the authoritative caller.
    ProjectionLocal(String),
}

impl DocumentRenderIdentityV1 {
    /// Construct a nonblank durable identity without changing its spelling.
    pub fn durable(value: impl Into<String>) -> Result<Self, RenderError> {
        Self::validated(value.into(), "durable document root identity").map(Self::Durable)
    }

    /// Construct a nonblank projection-local identity without inventing an ID.
    pub fn projection_local(value: impl Into<String>) -> Result<Self, RenderError> {
        Self::validated(value.into(), "projection-local document root identity")
            .map(Self::ProjectionLocal)
    }

    fn validated(value: String, description: &str) -> Result<String, RenderError> {
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(RenderError::InvalidRequest(format!(
                "{description} must be visible text without controls"
            )));
        }
        Ok(value)
    }

    /// Return the exact caller-issued identity spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Durable(value) | Self::ProjectionLocal(value) => value,
        }
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
    Molecule(MoleculeRenderPlan),
    /// One complete existing text operation, including plus signs.
    Text(DocumentTextOpV1),
    /// One checked generic vector root in document-local paint order.
    Vector(DocumentVectorRootV1),
}

/// A paintable direct-root item in strict document source order.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentRenderRootV1 {
    source_order: u32,
    identity: DocumentRenderIdentityV1,
    content: DocumentRenderContentV1,
}

impl DocumentRenderRootV1 {
    /// Construct one paintable direct-root item.
    #[must_use]
    pub const fn new(
        source_order: u32,
        identity: DocumentRenderIdentityV1,
        content: DocumentRenderContentV1,
    ) -> Self {
        Self {
            source_order,
            identity,
            content,
        }
    }

    /// Return the root's direct-child source order.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    /// Return the durable-or-local root identity.
    #[must_use]
    pub fn identity(&self) -> &DocumentRenderIdentityV1 {
        &self.identity
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
    source_order: u32,
    identity: DocumentRenderIdentityV1,
    feature: String,
}

impl DocumentRenderExclusionV1 {
    /// Record one named, intentionally unpainted direct root.
    pub fn new(
        source_order: u32,
        identity: DocumentRenderIdentityV1,
        feature: impl Into<String>,
    ) -> Result<Self, RenderError> {
        let feature = feature.into();
        if feature.trim().is_empty() || feature.chars().any(char::is_control) {
            return Err(RenderError::InvalidRequest(
                "document render exclusion must name a visible feature".to_owned(),
            ));
        }
        Ok(Self {
            source_order,
            identity,
            feature,
        })
    }

    /// Return the excluded root's direct-child source order.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    /// Return the exact durable-or-local root identity.
    #[must_use]
    pub fn identity(&self) -> &DocumentRenderIdentityV1 {
        &self.identity
    }
    /// Return the named renderer gap.
    #[must_use]
    pub fn feature(&self) -> &str {
        &self.feature
    }
}

/// One direct-root outcome in exact document source order.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentRenderOutcomeV1 {
    /// A root the current renderer can paint.
    Root(DocumentRenderRootV1),
    /// A named root intentionally omitted from this renderer slice.
    Exclusion(DocumentRenderExclusionV1),
}

impl DocumentRenderOutcomeV1 {
    /// Return the direct-child source order shared by all outcome kinds.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        match self {
            Self::Root(root) => root.source_order(),
            Self::Exclusion(exclusion) => exclusion.source_order(),
        }
    }

    /// Return the exact durable-or-local root identity.
    #[must_use]
    pub fn identity(&self) -> &DocumentRenderIdentityV1 {
        match self {
            Self::Root(root) => root.identity(),
            Self::Exclusion(exclusion) => exclusion.identity(),
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
        let mut identities = HashSet::new();
        let mut source_orders = HashSet::new();
        identities
            .try_reserve(outcomes.len())
            .map_err(|_| RenderError::ResourceExhausted)?;
        source_orders
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
                outcome.source_order(),
                outcome.identity(),
                &mut source_orders,
                &mut identities,
            )?;
            if let Some(previous) = previous
                && outcome.source_order() <= previous
            {
                return Err(RenderError::InvalidRequest(
                    "document render outcomes must have strictly increasing source order"
                        .to_owned(),
                ));
            }
            previous = Some(outcome.source_order());
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
    /// Return every paintable or excluded root in strict source order.
    #[must_use]
    pub fn outcomes(&self) -> &[DocumentRenderOutcomeV1] {
        &self.outcomes
    }
}

fn validate_root_key(
    source_order: u32,
    identity: &DocumentRenderIdentityV1,
    source_orders: &mut HashSet<u32>,
    identities: &mut HashSet<DocumentRenderIdentityV1>,
) -> Result<(), RenderError> {
    if !source_orders.insert(source_order) {
        return Err(RenderError::InvalidRequest(
            "document render roots and exclusions must have unique source orders".to_owned(),
        ));
    }
    if !identities.insert(identity.clone()) {
        return Err(RenderError::InvalidRequest(
            "document render roots and exclusions must have unique identities".to_owned(),
        ));
    }
    Ok(())
}

const fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
