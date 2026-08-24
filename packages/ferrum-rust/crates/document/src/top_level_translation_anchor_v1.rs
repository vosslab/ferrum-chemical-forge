//! Private authored-coordinate anchor calculation for complete-root translation.

use super::{TopLevelRootSelectorV1, TopLevelRootTranslationV1, TypedDocument, TypedDocumentError};

impl TypedDocument {
    /// Calculate a complete-root lower-left anchor without exposing it externally.
    pub(crate) fn top_level_translation_anchor_v1(
        &self,
        targets: Vec<TopLevelRootSelectorV1>,
    ) -> Result<(f64, f64), TypedDocumentError> {
        let request = TopLevelRootTranslationV1::new(targets, 0.0, 0.0)?;
        super::typed_top_level_transform::validate_complete_bracket_selection(
            self,
            request.common_transform(),
        )?;
        let geometries =
            super::typed_top_level_transform::resolve_geometries(self, request.targets())?;
        let anchor_x = geometries
            .iter()
            .map(|geometry| geometry.bounds.0)
            .reduce(f64::min)
            .expect("validated complete-root geometry is nonempty");
        let anchor_y = geometries
            .iter()
            .map(|geometry| geometry.bounds.1)
            .reduce(f64::min)
            .expect("validated complete-root geometry is nonempty");
        Ok((anchor_x, anchor_y))
    }
}
