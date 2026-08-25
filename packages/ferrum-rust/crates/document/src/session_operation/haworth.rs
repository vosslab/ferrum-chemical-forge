//! Closed semantic intent for Haworth molecule authoring.

use ferrum_domain::haworth::{
    DirectGlycosidicHaworthAuthoringReceiptV1, StandaloneDGlucoseHaworthRecipeV1,
};

use crate::Point3V1;

/// One document-owned Haworth creation request.
///
/// The variants retain only chemistry facts validated by their source routes and
/// the resolved page anchor. Identity allocation, renderer admission, and
/// mutation remain owned by the generic session transition lifecycle.
#[derive(Clone, Debug, PartialEq)]
pub enum CreateHaworthMoleculeV1 {
    /// A validated two-ring direct-glycosidic Haworth source.
    DirectGlycosidic(Box<DirectGlycosidicHaworthPayloadV1>),
    /// One supported standalone D-glucose Haworth recipe.
    StandaloneDGlucose {
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    },
}

/// The direct-glycosidic inputs retained by a Haworth-create operation.
///
/// This payload is boxed at the enum boundary because a validated receipt is
/// substantially larger than the other Haworth authoring variants.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthPayloadV1 {
    receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
    anchor: Point3V1,
}

impl DirectGlycosidicHaworthPayloadV1 {
    fn new(receipt: DirectGlycosidicHaworthAuthoringReceiptV1, anchor: Point3V1) -> Self {
        Self { receipt, anchor }
    }

    pub(crate) fn into_parts(self) -> (DirectGlycosidicHaworthAuthoringReceiptV1, Point3V1) {
        (self.receipt, self.anchor)
    }
}

impl CreateHaworthMoleculeV1 {
    #[must_use]
    pub fn direct_glycosidic(
        receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
        anchor: Point3V1,
    ) -> Self {
        Self::DirectGlycosidic(Box::new(DirectGlycosidicHaworthPayloadV1::new(
            receipt, anchor,
        )))
    }

    #[must_use]
    pub const fn standalone_d_glucose(
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    ) -> Self {
        Self::StandaloneDGlucose { recipe, anchor }
    }
}
