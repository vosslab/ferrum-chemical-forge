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
    DirectGlycosidic {
        receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
        anchor: Point3V1,
    },
    /// One supported standalone D-glucose Haworth recipe.
    StandaloneDGlucose {
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    },
}

impl CreateHaworthMoleculeV1 {
    #[must_use]
    pub const fn direct_glycosidic(
        receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
        anchor: Point3V1,
    ) -> Self {
        Self::DirectGlycosidic { receipt, anchor }
    }

    #[must_use]
    pub const fn standalone_d_glucose(
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    ) -> Self {
        Self::StandaloneDGlucose { recipe, anchor }
    }
}
