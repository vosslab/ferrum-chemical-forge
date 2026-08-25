//! Explicit E/Z carrier-mark operation geometry.

use ferrum_core::RecordId;
use serde::{Deserialize, Serialize};

use crate::{LineOp, Paint, PositiveFinite, RenderError, RenderPoint};

/// The stored native directional mark for one E/Z carrier bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoubleBondCarrierMarkDirectionV1 {
    /// Offset the accent to the left normal of the stored carrier orientation.
    Up,
    /// Offset the accent to the right normal of the stored carrier orientation.
    Down,
}

/// A renderer-neutral E/Z carrier accent with its durable central-double provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoubleBondCarrierMarkOp {
    accent_start: RenderPoint,
    accent_end: RenderPoint,
    width: PositiveFinite,
    paint: Paint,
    direction: DoubleBondCarrierMarkDirectionV1,
    central_double_bond: RecordId,
    z: i32,
}

impl DoubleBondCarrierMarkOp {
    /// Construct the short parallel accent for one already-selected carrier bond.
    ///
    /// The stored carrier orientation supplies the normal direction. The shared
    /// endpoint selects only where along that carrier the accent begins; no E/Z
    /// configuration or carrier is inferred from coordinates.
    pub fn from_carrier_line(
        carrier: &LineOp,
        shared_endpoint_is_start: bool,
        direction: DoubleBondCarrierMarkDirectionV1,
        central_double_bond: RecordId,
        z: i32,
    ) -> Result<Self, RenderError> {
        let start = carrier.start();
        let end = carrier.end();
        let dx = end.x() - start.x();
        let dy = end.y() - start.y();
        let length = dx.hypot(dy);
        if !length.is_finite() || length <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "E/Z carrier mark requires a nondegenerate carrier line".to_owned(),
            ));
        }
        let (near, far) = if shared_endpoint_is_start {
            (start, end)
        } else {
            (end, start)
        };
        let along_x = (far.x() - near.x()) / length;
        let along_y = (far.y() - near.y()) / length;
        let offset = carrier.width().get() * 2.0;
        if !offset.is_finite() || offset <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "E/Z carrier mark offset is not representable".to_owned(),
            ));
        }
        let sign = match direction {
            DoubleBondCarrierMarkDirectionV1::Up => 1.0,
            DoubleBondCarrierMarkDirectionV1::Down => -1.0,
        };
        let normal_x = -dy / length * sign;
        let normal_y = dx / length * sign;
        let accent_start = point_offset(near, along_x, along_y, normal_x, normal_y, 0.12, offset)?;
        let accent_end = point_offset(near, along_x, along_y, normal_x, normal_y, 0.42, offset)?;
        Ok(Self {
            accent_start,
            accent_end,
            width: carrier.width(),
            paint: carrier.paint().clone(),
            direction,
            central_double_bond,
            z,
        })
    }

    /// Return the geometry that every artifact backend paints as a thin line.
    #[must_use]
    pub fn accent_line(&self) -> LineOp {
        LineOp::new(
            self.accent_start,
            self.accent_end,
            self.width,
            self.paint.clone(),
            self.z,
        )
        .expect("validated carrier-mark geometry remains a line operation")
    }

    /// Return the first endpoint of the computed thin carrier accent.
    #[must_use]
    pub const fn accent_start(&self) -> RenderPoint {
        self.accent_start
    }

    /// Return the final endpoint of the computed thin carrier accent.
    #[must_use]
    pub const fn accent_end(&self) -> RenderPoint {
        self.accent_end
    }

    #[must_use]
    pub const fn direction(&self) -> DoubleBondCarrierMarkDirectionV1 {
        self.direction
    }
    #[must_use]
    pub const fn central_double_bond(&self) -> &RecordId {
        &self.central_double_bond
    }
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

fn point_offset(
    near: RenderPoint,
    along_x: f64,
    along_y: f64,
    normal_x: f64,
    normal_y: f64,
    along: f64,
    normal: f64,
) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(
        near.x() + along_x * along + normal_x * normal,
        near.y() + along_y * along + normal_y * normal,
    )
}
