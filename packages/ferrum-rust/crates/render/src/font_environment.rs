//! Verification and ownership of Ferrum's one bundled molecule-label face.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::RenderError;

const TELEX_RESOURCE_ID: &str = "ferrum-telex-regular-v1";
const TELEX_BYTES: u64 = 38_940;
const TELEX_SHA256: &str = "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871";

/// The only face accepted by the V1 molecule-label contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FerrumFontId {
    /// The vendored Telex Regular OpenType face.
    TelexRegular,
}

impl FerrumFontId {
    /// Return the stable serialized resource identifier.
    #[must_use]
    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::TelexRegular => TELEX_RESOURCE_ID,
        }
    }
}

/// Immutable facts identifying a verified bundled font asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontAssetDescriptor {
    id: FerrumFontId,
    path: PathBuf,
    bytes: u64,
    sha256: String,
}

impl FontAssetDescriptor {
    /// Return the closed Ferrum face identifier.
    #[must_use]
    pub const fn id(&self) -> FerrumFontId {
        self.id
    }

    /// Return the verified canonical asset path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the expected byte length.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Return the expected lowercase SHA-256 digest.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// A verified, immutable font resource environment for M12 measurements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FerrumFontEnvironmentV1 {
    telex: FontAssetDescriptor,
}

impl FerrumFontEnvironmentV1 {
    /// Load and verify the bundled Telex Regular asset before any font engine opens it.
    pub fn load() -> Result<Self, RenderError> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/Telex-Regular.ttf");
        Self::load_from_path(FerrumFontId::TelexRegular, &path)
    }

    /// Return the verified descriptor for the requested closed font identifier.
    #[must_use]
    pub const fn descriptor(&self, id: FerrumFontId) -> &FontAssetDescriptor {
        match id {
            FerrumFontId::TelexRegular => &self.telex,
        }
    }

    fn load_from_path(id: FerrumFontId, path: &Path) -> Result<Self, RenderError> {
        let canonical = path.canonicalize().map_err(|error| {
            RenderError::InvalidRequest(format!("verified Telex asset is unavailable: {error}"))
        })?;
        let metadata = std::fs::metadata(&canonical).map_err(|error| {
            RenderError::InvalidRequest(format!("could not inspect verified Telex asset: {error}"))
        })?;
        if !metadata.is_file() || metadata.len() != TELEX_BYTES {
            return Err(RenderError::InvalidRequest(
                "verified Telex asset has an unexpected file type or byte length".to_owned(),
            ));
        }
        let bytes = std::fs::read(&canonical).map_err(|error| {
            RenderError::InvalidRequest(format!("could not read verified Telex asset: {error}"))
        })?;
        let digest = hex_digest(Sha256::digest(&bytes).as_slice());
        if digest != TELEX_SHA256 {
            return Err(RenderError::InvalidRequest(
                "verified Telex asset has an unexpected SHA-256 digest".to_owned(),
            ));
        }
        Ok(Self {
            telex: FontAssetDescriptor {
                id,
                path: canonical,
                bytes: TELEX_BYTES,
                sha256: TELEX_SHA256.to_owned(),
            },
        })
    }

    #[cfg(test)]
    pub(crate) fn load_for_test(path: &Path) -> Result<Self, RenderError> {
        Self::load_from_path(FerrumFontId::TelexRegular, path)
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    result
}
