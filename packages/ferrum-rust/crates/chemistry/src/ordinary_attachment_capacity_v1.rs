//! Finite neutral capacity admission for one ordinary incoming attachment.

/// The closed V1 exterior attachment profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryAttachmentProfileV1 {
    /// One ordinary, non-aromatic single bond.
    NormalSingle,
}

impl OrdinaryAttachmentProfileV1 {
    const fn demand(self) -> u16 {
        match self {
            Self::NormalSingle => 1,
        }
    }
}

/// Chemical order of one existing incident bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryAttachmentBondOrderV1 {
    Single,
    Double,
    Triple,
    Aromatic,
    Unsupported,
}

impl OrdinaryAttachmentBondOrderV1 {
    const fn demand(self) -> Result<u16, OrdinaryAttachmentCapacityReasonV1> {
        match self {
            Self::Single => Ok(1),
            Self::Double => Ok(2),
            Self::Triple => Ok(3),
            Self::Aromatic => Err(OrdinaryAttachmentCapacityReasonV1::AromaticBond),
            Self::Unsupported => Err(OrdinaryAttachmentCapacityReasonV1::UnsupportedBondOrder),
        }
    }
}

/// Source-format-free anchor facts for capacity arithmetic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrdinaryAttachmentAnchorV1<'a> {
    pub element: &'a str,
    pub formal_charge: Option<i32>,
    pub explicit_hydrogens: Option<u16>,
    pub authored_valence: Option<u16>,
    pub multiplicity: Option<u16>,
    pub free_sites: Option<u16>,
    pub incident_bond_orders: &'a [OrdinaryAttachmentBondOrderV1],
}

/// Stable causes for an unavailable ordinary attachment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryAttachmentCapacityReasonV1 {
    ElementOutsideProfile,
    ChargeOutsideProfile,
    AuthoredCapacityOverride,
    RadicalOrMultiplicity,
    AromaticBond,
    UnsupportedBondOrder,
    DemandOverflow,
    ExceedsCapacity,
}

/// Stable next action paired with an unavailable reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryAttachmentCapacityRecoveryV1 {
    ChooseAnotherAtom,
    UseSupportedOrdinaryStructure,
    RemoveOrChangeAuthoredCapacityFact,
}

/// Auditable proof that exactly one profile bond fits at the anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrdinaryAttachmentCapacityAdmissionV1 {
    profile: OrdinaryAttachmentProfileV1,
    existing_demand: u16,
    added_demand: u16,
    resulting_demand: u16,
    capacity: u16,
}

impl OrdinaryAttachmentCapacityAdmissionV1 {
    #[must_use]
    pub const fn profile(&self) -> OrdinaryAttachmentProfileV1 {
        self.profile
    }
    #[must_use]
    pub const fn existing_demand(&self) -> u16 {
        self.existing_demand
    }
    #[must_use]
    pub const fn added_demand(&self) -> u16 {
        self.added_demand
    }
    #[must_use]
    pub const fn resulting_demand(&self) -> u16 {
        self.resulting_demand
    }
    #[must_use]
    pub const fn capacity(&self) -> u16 {
        self.capacity
    }
}

/// The complete result of neutral ordinary capacity admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryAttachmentCapacityOutcomeV1 {
    Admitted(OrdinaryAttachmentCapacityAdmissionV1),
    Unavailable {
        reason: OrdinaryAttachmentCapacityReasonV1,
        recovery: OrdinaryAttachmentCapacityRecoveryV1,
    },
}

/// Evaluate a finite neutral H/C/N/O attachment slot without mutation or allocation.
#[must_use]
pub fn admit_ordinary_attachment_capacity_v1(
    profile: OrdinaryAttachmentProfileV1,
    anchor: OrdinaryAttachmentAnchorV1<'_>,
) -> OrdinaryAttachmentCapacityOutcomeV1 {
    let capacity = match neutral_capacity(anchor.element) {
        Some(value) => value,
        None => return unavailable(OrdinaryAttachmentCapacityReasonV1::ElementOutsideProfile),
    };
    if anchor.formal_charge.is_some_and(|charge| charge != 0) {
        return unavailable(OrdinaryAttachmentCapacityReasonV1::ChargeOutsideProfile);
    }
    if anchor.authored_valence.is_some() || anchor.free_sites.is_some() {
        return unavailable(OrdinaryAttachmentCapacityReasonV1::AuthoredCapacityOverride);
    }
    if anchor.multiplicity.is_some() {
        return unavailable(OrdinaryAttachmentCapacityReasonV1::RadicalOrMultiplicity);
    }
    let mut existing_demand = anchor.explicit_hydrogens.unwrap_or(0);
    for order in anchor.incident_bond_orders {
        let Ok(demand) = order.demand() else {
            return unavailable(order.demand().expect_err("known unavailable order"));
        };
        let Some(total) = existing_demand.checked_add(demand) else {
            return unavailable(OrdinaryAttachmentCapacityReasonV1::DemandOverflow);
        };
        existing_demand = total;
    }
    let added_demand = profile.demand();
    let Some(resulting_demand) = existing_demand.checked_add(added_demand) else {
        return unavailable(OrdinaryAttachmentCapacityReasonV1::DemandOverflow);
    };
    if resulting_demand > capacity {
        return unavailable(OrdinaryAttachmentCapacityReasonV1::ExceedsCapacity);
    }
    OrdinaryAttachmentCapacityOutcomeV1::Admitted(OrdinaryAttachmentCapacityAdmissionV1 {
        profile,
        existing_demand,
        added_demand,
        resulting_demand,
        capacity,
    })
}

fn neutral_capacity(element: &str) -> Option<u16> {
    match element {
        "H" => Some(1),
        "C" => Some(4),
        "N" => Some(3),
        "O" => Some(2),
        _ => None,
    }
}

const fn unavailable(
    reason: OrdinaryAttachmentCapacityReasonV1,
) -> OrdinaryAttachmentCapacityOutcomeV1 {
    let recovery = match reason {
        OrdinaryAttachmentCapacityReasonV1::AuthoredCapacityOverride
        | OrdinaryAttachmentCapacityReasonV1::RadicalOrMultiplicity => {
            OrdinaryAttachmentCapacityRecoveryV1::RemoveOrChangeAuthoredCapacityFact
        }
        OrdinaryAttachmentCapacityReasonV1::ExceedsCapacity => {
            OrdinaryAttachmentCapacityRecoveryV1::ChooseAnotherAtom
        }
        OrdinaryAttachmentCapacityReasonV1::ElementOutsideProfile
        | OrdinaryAttachmentCapacityReasonV1::ChargeOutsideProfile
        | OrdinaryAttachmentCapacityReasonV1::AromaticBond
        | OrdinaryAttachmentCapacityReasonV1::UnsupportedBondOrder
        | OrdinaryAttachmentCapacityReasonV1::DemandOverflow => {
            OrdinaryAttachmentCapacityRecoveryV1::UseSupportedOrdinaryStructure
        }
    };
    OrdinaryAttachmentCapacityOutcomeV1::Unavailable { reason, recovery }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor<'a>(
        element: &'a str,
        hydrogens: Option<u16>,
        orders: &'a [OrdinaryAttachmentBondOrderV1],
    ) -> OrdinaryAttachmentAnchorV1<'a> {
        OrdinaryAttachmentAnchorV1 {
            element,
            formal_charge: None,
            explicit_hydrogens: hydrogens,
            authored_valence: None,
            multiplicity: None,
            free_sites: None,
            incident_bond_orders: orders,
        }
    }

    #[test]
    fn ordinary_carbon_and_oxygen_slots_admit_exact_neutral_demand() {
        let carbon = admit_ordinary_attachment_capacity_v1(
            OrdinaryAttachmentProfileV1::NormalSingle,
            anchor("C", Some(1), &[OrdinaryAttachmentBondOrderV1::Double]),
        );
        let oxygen = admit_ordinary_attachment_capacity_v1(
            OrdinaryAttachmentProfileV1::NormalSingle,
            anchor("O", None, &[OrdinaryAttachmentBondOrderV1::Single]),
        );
        assert!(matches!(
            carbon,
            OrdinaryAttachmentCapacityOutcomeV1::Admitted(admission)
                if admission.existing_demand() == 3 && admission.resulting_demand() == 4
        ));
        assert!(matches!(
            oxygen,
            OrdinaryAttachmentCapacityOutcomeV1::Admitted(admission)
                if admission.capacity() == 2 && admission.resulting_demand() == 2
        ));
    }

    #[test]
    fn saturated_or_outside_profile_anchor_returns_closed_unavailability() {
        let saturated = admit_ordinary_attachment_capacity_v1(
            OrdinaryAttachmentProfileV1::NormalSingle,
            anchor("C", Some(4), &[]),
        );
        let aromatic = admit_ordinary_attachment_capacity_v1(
            OrdinaryAttachmentProfileV1::NormalSingle,
            anchor("C", None, &[OrdinaryAttachmentBondOrderV1::Aromatic]),
        );
        assert!(matches!(
            saturated,
            OrdinaryAttachmentCapacityOutcomeV1::Unavailable {
                reason: OrdinaryAttachmentCapacityReasonV1::ExceedsCapacity,
                recovery: OrdinaryAttachmentCapacityRecoveryV1::ChooseAnotherAtom,
            }
        ));
        assert!(matches!(
            aromatic,
            OrdinaryAttachmentCapacityOutcomeV1::Unavailable {
                reason: OrdinaryAttachmentCapacityReasonV1::AromaticBond,
                ..
            }
        ));
    }
}
