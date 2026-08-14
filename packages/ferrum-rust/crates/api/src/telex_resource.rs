//! Verified, immutable Telex Regular resource for Ferrum frontends.

use ferrum_render::{FerrumFontEnvironmentV1, FerrumFontId, RenderError};

/// Return exact, verified, embedded Telex Regular bytes and immutable resource facts.
///
/// This is the only API route for a frontend font resource. It accepts neither a
/// path nor a family selector, and copies bytes only after the compiled asset passes
/// the render crate's digest and metadata verification.
pub fn verified_telex_regular_v1() -> Result<VerifiedTelexRegularV1, RenderError> {
    let environment = FerrumFontEnvironmentV1::load()?;
    let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
    Ok(VerifiedTelexRegularV1 {
        resource_id: descriptor.id().resource_id(),
        bytes: descriptor.data().to_vec(),
        byte_length: descriptor.bytes(),
        sha256: descriptor.sha256().to_owned(),
        family: descriptor.family().to_owned(),
        postscript_name: descriptor.postscript_name().to_owned(),
    })
}

/// Immutable frontend copy of the only accepted V1 text face.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedTelexRegularV1 {
    resource_id: &'static str,
    bytes: Vec<u8>,
    byte_length: u64,
    sha256: String,
    family: String,
    postscript_name: String,
}

impl VerifiedTelexRegularV1 {
    /// Return the immutable resource identifier.
    #[must_use]
    pub const fn resource_id(&self) -> &'static str {
        self.resource_id
    }

    /// Return the verified font bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Return the exact byte length.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Return the lowercase SHA-256 digest.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Return the selected font family.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Return the selected PostScript name.
    #[must_use]
    pub fn postscript_name(&self) -> &str {
        &self.postscript_name
    }
}
