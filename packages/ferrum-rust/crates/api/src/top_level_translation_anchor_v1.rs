//! Private-adapter request for authored complete-root translation anchors.

use ferrum_document::{
    DocumentSession, DocumentSessionError, TopLevelRootSelectorV1, TopLevelTranslationAnchorV1,
};

/// Observe one immutable complete-root translation anchor at an exact revision.
pub fn observe_top_level_translation_anchor_v1(
    session: &DocumentSession,
    expected_revision: u64,
    targets: Vec<TopLevelRootSelectorV1>,
) -> Result<TopLevelTranslationAnchorV1, DocumentSessionError> {
    session.observe_top_level_translation_anchor_v1(expected_revision, targets)
}
