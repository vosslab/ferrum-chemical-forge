//! Versioned resource policy for ordinary decoded local-document operations.
//!
//! This profile is an operational desktop/CLI allocation envelope, not a file-
//! format validity rule or a promise that every historical document fits. A
//! future population or resource model can add a new profile without silently
//! changing the meaning of V1.

use std::path::Path;

use crate::artifact_publication_v1::{RetainedRegularFileIdentityV1, RetainedSourceFileGuardV1};
use crate::{
    CdmlIngressBudgetV1, CdsvgIngressBudgetV1, DocumentIngressErrorV1, DocumentIngressFormatV1,
    DocumentSession, XmlInputBudgetV1, load_document_file_for_publication_with_budget,
};

/// Stable identifier for the first ordinary local-CDML admission profile.
pub const LOCAL_CDML_INGRESS_PROFILE_V1: &str = "ferrum-local-cdml-ingress-v1";

/// Maximum uncompressed UTF-8 CDML source bytes under the local V1 profile.
pub const LOCAL_CDML_SOURCE_UTF8_BYTES_V1: usize = 16 * 1024 * 1024;

/// Stable identifier for ordinary decoded CD-SVG admission.
pub const LOCAL_DECODED_CDSVG_INGRESS_PROFILE_V1: &str = "ferrum-local-decoded-cdsvg-ingress-v1";

/// Return the immutable V1 policy for uncompressed local CDML or bounded stdin.
///
/// The 16 MiB source envelope is the dominant allocation guard. Independent
/// structural ceilings stop compact adversarial XML from trading source bytes
/// for excessive tree fan-out, attributes, text retention, or parser depth.
/// CD-SVG and compressed containers deliberately require separate profiles.
#[must_use]
pub const fn local_cdml_ingress_format_v1() -> DocumentIngressFormatV1 {
    DocumentIngressFormatV1::Cdml(CdmlIngressBudgetV1 {
        xml: XmlInputBudgetV1 {
            max_utf8_bytes: LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
            max_elements: 262_144,
            max_depth: 64,
            max_attributes: 1_048_576,
            max_text_bytes: 8 * 1024 * 1024,
        },
    })
}

/// Return the immutable V1 profile for a decoded UTF-8 SVG CDML container.
///
/// The wrapper and selected normalized payload are independently bounded. This
/// profile deliberately does not sniff extensions or admit compressed SVG.
#[must_use]
pub const fn local_decoded_cdsvg_ingress_format_v1() -> DocumentIngressFormatV1 {
    DocumentIngressFormatV1::Cdsvg(CdsvgIngressBudgetV1 {
        wrapper: XmlInputBudgetV1 {
            max_utf8_bytes: LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
            max_elements: 262_144,
            max_depth: 64,
            max_attributes: 1_048_576,
            max_text_bytes: 8 * 1024 * 1024,
        },
        payload: XmlInputBudgetV1 {
            max_utf8_bytes: LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
            max_elements: 262_144,
            max_depth: 64,
            max_attributes: 1_048_576,
            max_text_bytes: 8 * 1024 * 1024,
        },
    })
}

/// Admit one ordinary local, uncompressed CDML file through the immutable V1 profile.
///
/// This is the product-facing file boundary for desktop and other local callers.
/// Callers choose the path, while Rust owns the complete named resource policy.
///
/// # Errors
///
/// Returns [`DocumentIngressErrorV1`] when the path, source bytes, XML, typed CDML,
/// or revision-zero session does not satisfy the profile.
pub fn load_local_cdml_file_v1(path: &Path) -> Result<DocumentSession, DocumentIngressErrorV1> {
    Ok(prepare_local_cdml_file_v1(path)?.0)
}

/// Admit one local CDML file and return its descriptor-derived live-origin identity.
///
/// The identity is captured from the exact regular descriptor that supplied the
/// admitted bytes. It is a local process receipt fact for editor tab
/// de-duplication, rather than document metadata or a path-normalization API.
pub fn prepare_local_cdml_file_v1(
    path: &Path,
) -> Result<(DocumentSession, RetainedRegularFileIdentityV1), DocumentIngressErrorV1> {
    let (session, source) = prepare_local_cdml_file_with_origin_v1(path)?;
    Ok((session, source.identity()))
}

/// Admit one ordinary local CDML file and retain its exact opened source descriptor.
///
/// This narrow desktop route lets an editor retain an opaque source guard for a
/// later descriptor-relative publication.  Identity-only callers should keep
/// using [`prepare_local_cdml_file_v1`].
///
/// # Errors
///
/// Returns [`DocumentIngressErrorV1`] when the local input does not satisfy the
/// immutable local CDML profile.
pub fn prepare_local_cdml_file_with_origin_v1(
    path: &Path,
) -> Result<(DocumentSession, RetainedSourceFileGuardV1), DocumentIngressErrorV1> {
    let admitted =
        load_document_file_for_publication_with_budget(path, local_cdml_ingress_format_v1())?;
    Ok(admitted.into_parts())
}

/// Admit one decoded local CD-SVG file and retain its exact source descriptor.
///
/// Successful session state is derived only from the canonical embedded CDML;
/// no SVG wrapper data survives this boundary.
///
/// # Errors
///
/// Returns [`DocumentIngressErrorV1`] when the local source or either CD-SVG
/// envelope does not satisfy the immutable V1 profile.
pub fn prepare_local_decoded_cdsvg_file_v1(
    path: &Path,
) -> Result<(DocumentSession, RetainedRegularFileIdentityV1), DocumentIngressErrorV1> {
    let (session, source) = prepare_local_decoded_cdsvg_file_with_origin_v1(path)?;
    Ok((session, source.identity()))
}

/// Admit one decoded local CD-SVG file and retain its exact wrapper descriptor.
///
/// The session remains canonical embedded CDML while the retained descriptor
/// protects the opened SVG wrapper from becoming an export destination.  The
/// descriptor is intentionally not a CD-SVG export contract.
///
/// # Errors
///
/// Returns [`DocumentIngressErrorV1`] when the local source or its selected
/// CD-SVG payload does not satisfy the immutable profile.
pub fn prepare_local_decoded_cdsvg_file_with_origin_v1(
    path: &Path,
) -> Result<(DocumentSession, RetainedSourceFileGuardV1), DocumentIngressErrorV1> {
    let admitted = load_document_file_for_publication_with_budget(
        path,
        local_decoded_cdsvg_ingress_format_v1(),
    )?;
    Ok(admitted.into_parts())
}
