//! Verified current molecule-label resource for Ferrum frontends.

use crate::{FerrumFontEnvironment, RenderError};

/// Return exact, verified, embedded Atkinson Hyperlegible Next Regular bytes and immutable resource facts.
///
/// This is the only API route for a frontend font resource. It accepts neither a
/// path nor a family selector, and copies bytes only after the compiled asset passes
/// the render crate's digest and metadata verification.
pub fn verified_molecule_label_font() -> Result<VerifiedMoleculeLabelFont, RenderError> {
    let environment = FerrumFontEnvironment::load()?;
    let descriptor = environment.molecule_label();
    Ok(VerifiedMoleculeLabelFont {
        resource_id: descriptor.resource_id(),
        bytes: descriptor.data().to_vec(),
        byte_length: descriptor.bytes(),
        sha256: descriptor.sha256().to_owned(),
        family: descriptor.family().to_owned(),
        postscript_name: descriptor.postscript_name().to_owned(),
    })
}

/// Immutable frontend copy of the current molecule-label font resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedMoleculeLabelFont {
    resource_id: &'static str,
    bytes: Vec<u8>,
    byte_length: u64,
    sha256: String,
    family: String,
    postscript_name: String,
}

impl VerifiedMoleculeLabelFont {
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
