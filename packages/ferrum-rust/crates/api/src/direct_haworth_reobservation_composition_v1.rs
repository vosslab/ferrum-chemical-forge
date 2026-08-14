//! Authentication boundary for composing one re-observed direct Haworth profile.

use ferrum_document::ReobservedDirectHaworthV1;
use ferrum_render::DocumentRenderCompositeV1;

use crate::{
    DirectHaworthDocumentCompositionErrorV1,
    direct_haworth_document_composition_v1::compose_authenticated_direct_haworth_document_v1,
};

/// Compose the opaque selective replacement authenticated from a current re-observation.
///
/// This accepts no session or source text. The enclosed immutable observation and
/// re-authenticated durable profile are the sole authority for this native route.
pub fn compose_reobserved_direct_haworth_document_v1(
    reobserved: &ReobservedDirectHaworthV1,
) -> Result<DocumentRenderCompositeV1, DirectHaworthDocumentCompositionErrorV1> {
    compose_authenticated_direct_haworth_document_v1(
        reobserved.observation(),
        reobserved.molecule(),
        reobserved.root_order(),
        reobserved.atom_identifiers(),
        reobserved.bond_facts(),
        reobserved.authored_depiction(),
    )
}
