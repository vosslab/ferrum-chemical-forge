//! Immutable authored-coordinate anchor receipts for complete-root translation.

use super::{
    TopLevelRootSelectorV1, TopLevelTransformModeV1, TopLevelTransformV1, TypedDocument,
    TypedDocumentError,
};

/// One exact-source authored-coordinate anchor for a complete-root move.
#[derive(Clone, Debug, PartialEq)]
pub struct TopLevelTranslationAnchorV1 {
    selectors: Vec<TopLevelRootSelectorV1>,
    source_revision: u64,
    source_digest: [u8; 32],
    anchor_x: f64,
    anchor_y: f64,
}

impl TopLevelTranslationAnchorV1 {
    #[must_use]
    pub fn selectors(&self) -> &[TopLevelRootSelectorV1] {
        &self.selectors
    }

    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    #[must_use]
    pub const fn anchor(&self) -> (f64, f64) {
        (self.anchor_x, self.anchor_y)
    }
}

impl TypedDocument {
    /// Observe a complete-root lower-left anchor without changing retained state.
    pub(crate) fn top_level_translation_anchor_v1(
        &self,
        source_revision: u64,
        source_digest: [u8; 32],
        targets: Vec<TopLevelRootSelectorV1>,
    ) -> Result<TopLevelTranslationAnchorV1, TypedDocumentError> {
        let request = TopLevelTransformV1::new(
            targets,
            TopLevelTransformModeV1::Translate { dx: 0.0, dy: 0.0 },
        )?;
        super::typed_top_level_transform::validate_complete_bracket_selection(self, &request)?;
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
        let mut selectors = request.targets().to_vec();
        selectors.sort_by(|left, right| {
            (left.kind().local_name(), left.root_id().as_str())
                .cmp(&(right.kind().local_name(), right.root_id().as_str()))
        });
        Ok(TopLevelTranslationAnchorV1 {
            selectors,
            source_revision,
            source_digest,
            anchor_x,
            anchor_y,
        })
    }
}
