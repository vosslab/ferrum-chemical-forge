//! Final bond-ink footprints and parallel-terminal optical clearance.

use crate::bond_style::BondStyle;
use crate::{PositiveFinite, RenderIssueKind};

use super::super::BondInkClearance;

/// Final ink reaching one clipped bond endpoint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BondInkFootprint {
    /// Radius of final ink at the endpoint along every direction (for example,
    /// a round line cap).
    endpoint_radius: f64,
    /// Half-width normal to the carrier axis at the endpoint.
    transverse_half_width: f64,
    /// Extra final ink extending beyond the clipped endpoint along the carrier axis.
    axial_overhang: f64,
    /// Final ink beginning inward from the clipped carrier endpoint.
    axial_retreat: f64,
}

impl BondInkFootprint {
    pub(super) fn axial_clip_reserve(self, clearance: f64) -> Result<f64, RenderIssueKind> {
        const ENDPOINT_WIDTH_GAP_FACTOR: f64 = 0.25;
        let endpoint_width = 2.0 * self.endpoint_radius.max(self.transverse_half_width);
        let optical_clearance = clearance.max(endpoint_width * ENDPOINT_WIDTH_GAP_FACTOR);
        let reserve =
            optical_clearance + self.endpoint_radius + self.axial_overhang - self.axial_retreat;
        validate_clip_distance(reserve)
    }

    pub(super) fn terminal_half_width(self) -> f64 {
        self.endpoint_radius.max(self.transverse_half_width)
    }
}

/// Directional endpoint envelopes for one final bond footprint.
#[derive(Clone, Copy)]
pub(super) struct EndpointBondInkFootprints {
    pub(super) first: BondInkFootprint,
    pub(super) second: BondInkFootprint,
}

impl EndpointBondInkFootprints {
    pub(super) const fn symmetric(footprint: BondInkFootprint) -> Self {
        Self {
            first: footprint,
            second: footprint,
        }
    }
}

/// Governing transverse final-ink envelope across both parallel-bond terminals.
///
/// Lane offsets may become asymmetric as depiction support grows. Measuring
/// the occupied interval rather than distance from the attachment axis keeps
/// optical clearance independent of that placement choice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ParallelBondTerminalEnvelope {
    width: PositiveFinite,
}

impl ParallelBondTerminalEnvelope {
    pub(super) fn from_lanes(
        lane_offsets: &[f64],
        footprints: EndpointBondInkFootprints,
    ) -> Result<Self, RenderIssueKind> {
        if lane_offsets.len() < 2 {
            return Err(RenderIssueKind::UnrenderableTarget {
                reason: "parallel-bond terminal envelope requires at least two lanes".to_owned(),
            });
        }
        let width = terminal_occupied_width(lane_offsets, footprints.first)?
            .max(terminal_occupied_width(lane_offsets, footprints.second)?);
        let width =
            PositiveFinite::new(width).map_err(|_| RenderIssueKind::UnrenderableTarget {
                reason: "parallel-bond terminal envelope is not representable".to_owned(),
            })?;
        Ok(Self { width })
    }

    pub(super) fn optical_clearance(
        self,
        base: BondInkClearance,
    ) -> Result<BondInkClearance, RenderIssueKind> {
        const TERMINAL_WIDTH_GAP_FACTOR: f64 = 0.25;
        // Stay materially inside the independent 1.75-stroke visual ceiling.
        // A boundary-equal vector distance can grow by one device pixel when
        // rasterized; 1.5 retains breathing room without backend dependence.
        const MAXIMUM_BASE_CLEARANCE_FACTOR: f64 = 1.5;

        let base_clearance = base.gap().get();
        let width_clearance = self.width.get() * TERMINAL_WIDTH_GAP_FACTOR;
        let maximum_clearance = base_clearance * MAXIMUM_BASE_CLEARANCE_FACTOR;
        let resolved = base_clearance.max(width_clearance.min(maximum_clearance));
        let gap =
            PositiveFinite::new(resolved).map_err(|_| RenderIssueKind::UnrenderableTarget {
                reason: "parallel-bond optical clearance is not representable".to_owned(),
            })?;
        Ok(BondInkClearance::new(gap))
    }

    #[cfg(test)]
    const fn width(self) -> PositiveFinite {
        self.width
    }
}

pub(super) fn final_ink_footprints(
    style: &BondStyle,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
) -> Result<EndpointBondInkFootprints, RenderIssueKind> {
    let symmetric = final_ink_footprint(style, stroke_width, wedge_width)?;
    match style {
        // Directed wedge lowering emits a narrow tip at source endpoint one
        // and its full width only at endpoint two. Reserving the base radius
        // at both ends detached the tip from its intended atom character.
        BondStyle::SolidWedge | BondStyle::HashedWedge | BondStyle::HaworthFrontWedge => {
            Ok(EndpointBondInkFootprints {
                first: BondInkFootprint {
                    endpoint_radius: 0.0,
                    transverse_half_width: 0.0,
                    axial_overhang: 0.0,
                    axial_retreat: if matches!(style, BondStyle::HashedWedge) {
                        stroke_width.get() / 2.0
                    } else {
                        0.0
                    },
                },
                second: symmetric,
            })
        }
        _ => Ok(EndpointBondInkFootprints::symmetric(symmetric)),
    }
}

pub(super) fn final_ink_footprint(
    style: &BondStyle,
    stroke_width: PositiveFinite,
    wedge_width: PositiveFinite,
) -> Result<BondInkFootprint, RenderIssueKind> {
    let endpoint_radius = match style {
        // A wavy bond starts and ends on its carrier axis. Its endpoint clip
        // needs only the round-cap radius there; the later lateral amplitude
        // belongs to complete-plan collision admission, not label clearance.
        BondStyle::Wavy => stroke_width.get() / 2.0,
        BondStyle::HaworthFrontStroke => wedge_width.get() / 2.0,
        _ => 0.0,
    };
    let transverse_half_width = match style {
        BondStyle::SolidWedge | BondStyle::HashedWedge | BondStyle::HaworthFrontWedge => {
            wedge_width.get() / 2.0
        }
        BondStyle::Bold => stroke_width.get(),
        BondStyle::Wavy | BondStyle::HaworthFrontStroke => 0.0,
        _ => stroke_width.get() / 2.0,
    };
    let axial_overhang = match style {
        // Each terminal hashed-wedge stroke is perpendicular to the carrier,
        // so its butt-capped stroke extends half a line width along the axis.
        BondStyle::HashedWedge => stroke_width.get() / 2.0,
        // q1 pads its emitted centerline 0.35w toward each label; its separate
        // round-cap radius is represented above.
        BondStyle::HaworthFrontStroke => wedge_width.get() * 0.35,
        // The filled Haworth-front wedge extends its base exactly 0.25w past
        // the already-clipped carrier endpoint.
        BondStyle::HaworthFrontWedge => wedge_width.get() * 0.25,
        _ => 0.0,
    };
    if endpoint_radius.is_finite()
        && endpoint_radius >= 0.0
        && transverse_half_width.is_finite()
        && transverse_half_width >= 0.0
        && axial_overhang.is_finite()
        && axial_overhang >= 0.0
    {
        Ok(BondInkFootprint {
            endpoint_radius,
            transverse_half_width,
            axial_overhang,
            axial_retreat: 0.0,
        })
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "final bond ink footprint is not representable".to_owned(),
        })
    }
}

fn terminal_occupied_width(
    lane_offsets: &[f64],
    footprint: BondInkFootprint,
) -> Result<f64, RenderIssueKind> {
    let half_width = footprint.terminal_half_width();
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for offset in lane_offsets {
        if !offset.is_finite() {
            return Err(RenderIssueKind::UnrenderableTarget {
                reason: "parallel-bond lane offset is not representable".to_owned(),
            });
        }
        minimum = minimum.min(offset - half_width);
        maximum = maximum.max(offset + half_width);
    }
    let width = maximum - minimum;
    if width.is_finite() && width > 0.0 {
        Ok(width)
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "parallel-bond terminal width is not representable".to_owned(),
        })
    }
}

fn validate_clip_distance(distance: f64) -> Result<f64, RenderIssueKind> {
    if distance.is_finite() && distance >= 0.0 {
        Ok(distance)
    } else {
        Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond endpoint clip distance is not representable".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_terminal_envelope_is_independent_of_axis_translation() {
        let footprint = final_ink_footprint(
            &BondStyle::Double,
            PositiveFinite::new(1.0).expect("stroke width"),
            PositiveFinite::new(5.0).expect("wedge width"),
        )
        .expect("double-bond footprint");
        let footprints = EndpointBondInkFootprints::symmetric(footprint);
        let centered = ParallelBondTerminalEnvelope::from_lanes(&[-3.0, 3.0], footprints)
            .expect("centered envelope");
        let translated = ParallelBondTerminalEnvelope::from_lanes(&[-1.0, 5.0], footprints)
            .expect("translated envelope");
        assert_eq!(centered.width().get(), 7.0);
        assert_eq!(translated.width(), centered.width());
    }

    #[test]
    fn parallel_terminal_clearance_is_bounded_and_scale_equivariant() {
        let base = BondInkClearance::new(PositiveFinite::new(0.75).expect("base clearance"));
        let footprint = final_ink_footprint(
            &BondStyle::Double,
            PositiveFinite::new(1.0).expect("stroke width"),
            PositiveFinite::new(5.0).expect("wedge width"),
        )
        .expect("double-bond footprint");
        let footprints = EndpointBondInkFootprints::symmetric(footprint);
        let double = ParallelBondTerminalEnvelope::from_lanes(&[-3.0, 3.0], footprints)
            .expect("double-bond envelope")
            .optical_clearance(base)
            .expect("double-bond clearance")
            .gap()
            .get();
        let triple = ParallelBondTerminalEnvelope::from_lanes(&[-4.2, 0.0, 4.2], footprints)
            .expect("triple-bond envelope")
            .optical_clearance(base)
            .expect("triple-bond clearance")
            .gap()
            .get();
        assert_eq!(double, base.gap().get() * 1.5);
        assert_eq!(triple, double);

        let scaled_footprint = final_ink_footprint(
            &BondStyle::Double,
            PositiveFinite::new(2.0).expect("scaled stroke width"),
            PositiveFinite::new(10.0).expect("scaled wedge width"),
        )
        .expect("scaled double-bond footprint");
        let scaled = ParallelBondTerminalEnvelope::from_lanes(
            &[-6.0, 6.0],
            EndpointBondInkFootprints::symmetric(scaled_footprint),
        )
        .expect("scaled double-bond envelope")
        .optical_clearance(BondInkClearance::new(
            PositiveFinite::new(base.gap().get() * 2.0).expect("scaled clearance"),
        ))
        .expect("scaled double-bond clearance")
        .gap()
        .get();
        assert_eq!(scaled, double * 2.0);
    }

    #[test]
    fn wide_endpoint_uses_a_quarter_width_optical_gap_floor() {
        let wide = BondInkFootprint {
            endpoint_radius: 2.0,
            transverse_half_width: 0.0,
            axial_overhang: 1.4,
            axial_retreat: 0.0,
        };
        let ordinary = BondInkFootprint {
            endpoint_radius: 0.0,
            transverse_half_width: 0.4,
            axial_overhang: 0.0,
            axial_retreat: 0.0,
        };
        assert_eq!(wide.axial_clip_reserve(0.75), Ok(4.4));
        assert_eq!(ordinary.axial_clip_reserve(0.75), Ok(0.75));
    }
}
