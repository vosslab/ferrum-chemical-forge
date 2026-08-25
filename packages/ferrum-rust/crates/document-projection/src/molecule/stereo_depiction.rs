//! Resolved E/Z carrier-mark drawing facts for one molecule projection.

use ferrum_core::BondOrder;
use serde::Serialize;
use thiserror::Error;

use crate::DocumentObjectIdV1;

use super::BondProjectionV1;

/// The explicit native direction retained for one E/Z carrier mark.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoubleBondCarrierMarkV1 {
    /// Draw at the positive normal of the stored carrier orientation.
    Up,
    /// Draw at the negative normal of the stored carrier orientation.
    Down,
}

/// A resolved E/Z drawing fact linked to its carrier and central-double bonds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DoubleBondCarrierMarkProjectionV1 {
    carrier_bond: DocumentObjectIdV1,
    carrier_start: DocumentObjectIdV1,
    carrier_end: DocumentObjectIdV1,
    carrier_shared_endpoint: DocumentObjectIdV1,
    central_double_bond: DocumentObjectIdV1,
    mark: DoubleBondCarrierMarkV1,
}

/// Failure while resolving one typed carrier fact to immutable projection identities.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoubleBondCarrierMarkProjectionV1Error {
    #[error("stereo depiction references missing {role} bond at source index {index}")]
    MissingBond { role: &'static str, index: usize },
    #[error("stereo depiction {role} bond has no durable document identity")]
    MissingBondIdentity { role: &'static str },
    #[error("stereo depiction carrier endpoint has no durable document identity")]
    MissingCarrierEndpointIdentity,
    #[error("stereo depiction carrier bond is not a single bond")]
    CarrierIsNotSingle,
    #[error("stereo depiction central bond is not a double bond")]
    CentralIsNotDouble,
    #[error("stereo depiction carrier and central double bond do not share one endpoint")]
    InvalidCarrierAssociation,
}

impl DoubleBondCarrierMarkProjectionV1 {
    /// Resolve one admitted source-indexed depiction against projected molecule bonds.
    pub fn from_bond_indexes(
        bonds: &[BondProjectionV1],
        double_bond_index: usize,
        carrier_bond_index: usize,
        mark: DoubleBondCarrierMarkV1,
    ) -> Result<Self, DoubleBondCarrierMarkProjectionV1Error> {
        let central = bonds.get(double_bond_index).ok_or(
            DoubleBondCarrierMarkProjectionV1Error::MissingBond {
                role: "central double",
                index: double_bond_index,
            },
        )?;
        let carrier = bonds.get(carrier_bond_index).ok_or(
            DoubleBondCarrierMarkProjectionV1Error::MissingBond {
                role: "carrier",
                index: carrier_bond_index,
            },
        )?;
        if central.order() != Some(BondOrder::Double) {
            return Err(DoubleBondCarrierMarkProjectionV1Error::CentralIsNotDouble);
        }
        if carrier.order() != Some(BondOrder::Single) {
            return Err(DoubleBondCarrierMarkProjectionV1Error::CarrierIsNotSingle);
        }
        let central_double_bond = central.id().cloned().ok_or(
            DoubleBondCarrierMarkProjectionV1Error::MissingBondIdentity {
                role: "central double",
            },
        )?;
        let carrier_bond = carrier.id().cloned().ok_or(
            DoubleBondCarrierMarkProjectionV1Error::MissingBondIdentity { role: "carrier" },
        )?;
        let carrier_start = carrier
            .start()
            .object_id()
            .cloned()
            .ok_or(DoubleBondCarrierMarkProjectionV1Error::MissingCarrierEndpointIdentity)?;
        let carrier_end = carrier
            .end()
            .object_id()
            .cloned()
            .ok_or(DoubleBondCarrierMarkProjectionV1Error::MissingCarrierEndpointIdentity)?;
        let central_endpoints = [central.start().object_id(), central.end().object_id()];
        let shared = [carrier_start.clone(), carrier_end.clone()]
            .into_iter()
            .filter(|endpoint| {
                central_endpoints
                    .iter()
                    .flatten()
                    .any(|central| *central == endpoint)
            })
            .collect::<Vec<_>>();
        let [carrier_shared_endpoint] = shared.as_slice() else {
            return Err(DoubleBondCarrierMarkProjectionV1Error::InvalidCarrierAssociation);
        };
        Ok(Self {
            carrier_bond,
            carrier_start,
            carrier_end,
            carrier_shared_endpoint: carrier_shared_endpoint.clone(),
            central_double_bond,
            mark,
        })
    }

    #[must_use]
    pub const fn carrier_bond(&self) -> &DocumentObjectIdV1 {
        &self.carrier_bond
    }
    #[must_use]
    pub const fn carrier_start(&self) -> &DocumentObjectIdV1 {
        &self.carrier_start
    }
    #[must_use]
    pub const fn carrier_end(&self) -> &DocumentObjectIdV1 {
        &self.carrier_end
    }
    #[must_use]
    pub const fn carrier_shared_endpoint(&self) -> &DocumentObjectIdV1 {
        &self.carrier_shared_endpoint
    }
    #[must_use]
    pub const fn central_double_bond(&self) -> &DocumentObjectIdV1 {
        &self.central_double_bond
    }
    #[must_use]
    pub const fn mark(&self) -> DoubleBondCarrierMarkV1 {
        self.mark
    }
}
