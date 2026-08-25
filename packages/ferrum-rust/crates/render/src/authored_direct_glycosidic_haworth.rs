//! Private-profile lowering of a checked, durable authored Haworth depiction.
//!
//! This deliberately parallels, rather than converts into, the M14 source-spec
//! route: its target order is the durable document child order.

use std::collections::HashSet;

use ferrum_core::RecordId;
use ferrum_document_projection::DocumentObjectIdV1;
use ferrum_domain::haworth::{
    AuthoredDirectGlycosidicHaworthBondRoleV1, AuthoredDirectGlycosidicHaworthDepictionV1,
    DirectGlycosidicHaworthBondStyleV1, DirectGlycosidicHaworthPositionV1,
};

use crate::direct_glycosidic_haworth::{
    extended_direct_base, padded_direct_q, rounded_direct_wedge,
};
use crate::{Paint, PositiveFinite, RenderError, RenderPoint, RenderProvenance};

const MAX_TARGETS: usize = 14;

/// Exact input for the durable authored direct-profile route.
#[derive(Clone, Debug)]
pub struct AuthoredDirectGlycosidicHaworthRenderRequestV1<'a> {
    provenance: RenderProvenance,
    depiction: &'a AuthoredDirectGlycosidicHaworthDepictionV1,
    canonical_bond_targets: &'a [crate::RenderTarget],
    paint: Paint,
    line_width: PositiveFinite,
    wedge_width: PositiveFinite,
}

impl<'a> AuthoredDirectGlycosidicHaworthRenderRequestV1<'a> {
    /// Construct a synchronous request; the resulting plan retains no borrow.
    #[must_use]
    pub fn new(
        provenance: RenderProvenance,
        depiction: &'a AuthoredDirectGlycosidicHaworthDepictionV1,
        canonical_bond_targets: &'a [crate::RenderTarget],
        paint: Paint,
        line_width: PositiveFinite,
        wedge_width: PositiveFinite,
    ) -> Self {
        Self {
            provenance,
            depiction,
            canonical_bond_targets,
            paint,
            line_width,
            wedge_width,
        }
    }
}

/// One non-wire direct operation keyed by durable child identity and order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AuthoredDirectGlycosidicHaworthDrawOpV1 {
    OrdinaryLine {
        bond: DocumentObjectIdV1,
        authored_child_order: u32,
        endpoints: [RenderPoint; 2],
        width: PositiveFinite,
    },
    HaworthFrontStroke {
        bond: DocumentObjectIdV1,
        authored_child_order: u32,
        endpoints: [RenderPoint; 2],
        width: PositiveFinite,
    },
    RoundedFrontWedge {
        bond: DocumentObjectIdV1,
        authored_child_order: u32,
        tip: RenderPoint,
        base: RenderPoint,
        width: PositiveFinite,
        commands: Vec<crate::direct_glycosidic_haworth::DirectGlycosidicHaworthPathCommandV1>,
    },
}

impl AuthoredDirectGlycosidicHaworthDrawOpV1 {
    pub(crate) const fn bond(&self) -> &DocumentObjectIdV1 {
        match self {
            Self::OrdinaryLine { bond, .. }
            | Self::HaworthFrontStroke { bond, .. }
            | Self::RoundedFrontWedge { bond, .. } => bond,
        }
    }

    pub(crate) const fn authored_child_order(&self) -> u32 {
        match self {
            Self::OrdinaryLine {
                authored_child_order,
                ..
            }
            | Self::HaworthFrontStroke {
                authored_child_order,
                ..
            }
            | Self::RoundedFrontWedge {
                authored_child_order,
                ..
            } => *authored_child_order,
        }
    }
}

/// Owned private renderer plan with its own explicit paint order.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthRenderPlanV1 {
    provenance: RenderProvenance,
    paint: Paint,
    operations: Vec<AuthoredDirectGlycosidicHaworthDrawOpV1>,
}

impl AuthoredDirectGlycosidicHaworthRenderPlanV1 {
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn paint(&self) -> &Paint {
        &self.paint
    }
    pub(crate) fn operations(&self) -> &[AuthoredDirectGlycosidicHaworthDrawOpV1] {
        &self.operations
    }
    #[cfg(test)]
    pub(crate) fn test_plan(
        provenance: RenderProvenance,
        paint: Paint,
        operations: Vec<AuthoredDirectGlycosidicHaworthDrawOpV1>,
    ) -> Self {
        Self {
            provenance,
            paint,
            operations,
        }
    }
}

/// Lower durable authored facts without recreating a source-local depiction spec.
pub fn lower_authored_direct_glycosidic_haworth_v1(
    request: AuthoredDirectGlycosidicHaworthRenderRequestV1<'_>,
) -> Result<AuthoredDirectGlycosidicHaworthRenderPlanV1, RenderError> {
    let bonds = request.depiction.canonical_bonds();
    if bonds.is_empty()
        || bonds.len() > MAX_TARGETS
        || request.canonical_bond_targets.len() != bonds.len()
    {
        return Err(RenderError::InvalidRequest(
            "authored direct depiction has invalid canonical bond targets".to_owned(),
        ));
    }
    let mut operations = Vec::new();
    operations
        .try_reserve(bonds.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    let mut seen = HashSet::new();
    seen.try_reserve(bonds.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    // The direct profile, not document child order, owns this tiered paint order.
    for tier in 0..3 {
        for (index, fact) in bonds.iter().enumerate() {
            let style = fact.token();
            let selected_tier = match tier {
                0 => {
                    fact.role() == AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge
                        || style == DirectGlycosidicHaworthBondStyleV1::N1
                }
                1 => style == DirectGlycosidicHaworthBondStyleV1::Q1,
                _ => style == DirectGlycosidicHaworthBondStyleV1::W1,
            };
            if !selected_tier {
                continue;
            }
            if !seen.insert((fact.bond().clone(), fact.authored_child_order())) {
                return Err(RenderError::InvalidRequest(
                    "authored direct depiction repeats a bond target".to_owned(),
                ));
            }
            let points = endpoints(request.depiction, fact.endpoints())?;
            let order = fact.authored_child_order();
            let bond = request.canonical_bond_targets[index]
                .document_object_id()
                .clone();
            match style {
                DirectGlycosidicHaworthBondStyleV1::N1 => {
                    operations.push(AuthoredDirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
                        bond,
                        authored_child_order: order,
                        endpoints: points,
                        width: request.line_width,
                    })
                }
                DirectGlycosidicHaworthBondStyleV1::Q1
                    if fact.haworth_position()
                        == Some(DirectGlycosidicHaworthPositionV1::Front) =>
                {
                    operations.push(
                        AuthoredDirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke {
                            bond,
                            authored_child_order: order,
                            endpoints: padded_direct_q(points, request.wedge_width)?,
                            width: request.wedge_width,
                        },
                    )
                }
                DirectGlycosidicHaworthBondStyleV1::W1
                    if fact.haworth_position()
                        == Some(DirectGlycosidicHaworthPositionV1::Front) =>
                {
                    let [tip, base] = points;
                    let base = extended_direct_base(tip, base, request.wedge_width)?;
                    let commands =
                        rounded_direct_wedge(tip, base, request.line_width, request.wedge_width)?;
                    operations.push(AuthoredDirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge {
                        bond,
                        authored_child_order: order,
                        tip,
                        base,
                        width: request.wedge_width,
                        commands,
                    });
                }
                _ => {
                    return Err(RenderError::InvalidRequest(
                        "authored direct depiction has invalid Haworth style facts".to_owned(),
                    ));
                }
            }
        }
    }
    if operations.len() != bonds.len() {
        return Err(RenderError::InvalidRequest(
            "authored direct depiction does not partition bonds".to_owned(),
        ));
    }
    Ok(AuthoredDirectGlycosidicHaworthRenderPlanV1 {
        provenance: request.provenance,
        paint: request.paint,
        operations,
    })
}

fn endpoints(
    depiction: &AuthoredDirectGlycosidicHaworthDepictionV1,
    ids: &[RecordId; 2],
) -> Result<[RenderPoint; 2], RenderError> {
    let coordinates = depiction.coordinates();
    let first = coordinates.get(&ids[0]).ok_or_else(missing)?;
    let second = coordinates.get(&ids[1]).ok_or_else(missing)?;
    Ok([
        RenderPoint::new(first.x, first.y)?,
        RenderPoint::new(second.x, second.y)?,
    ])
}

fn missing() -> RenderError {
    RenderError::InvalidRequest("authored direct depiction is incomplete".to_owned())
}
