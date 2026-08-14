//! Verification and ownership of Ferrum's one bundled molecule-label face.

#[cfg(test)]
use std::ffi::OsStr;
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Read;
#[cfg(test)]
use std::os::fd::OwnedFd;
#[cfg(test)]
use std::path::{Component, Path};
use std::sync::Arc;

#[cfg(test)]
use rustix::fs::{CWD, FileType, Mode, OFlags, fstat, openat};
#[cfg(test)]
use rustix::io::Errno;
use sha2::{Digest, Sha256};
use ttf_parser::{Face, name_id};

use crate::RenderError;

const TELEX_RESOURCE_ID: &str = "ferrum-telex-regular-v1";
const TELEX_BYTES: u64 = 38_940;
const TELEX_SHA256: &str = "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871";
const TELEX_FAMILY: &str = "Telex";
const TELEX_POSTSCRIPT_NAME: &str = "Telex-Regular";

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
    bytes: u64,
    sha256: String,
    family: String,
    postscript_name: String,
    data: Arc<[u8]>,
}

impl FontAssetDescriptor {
    /// Return the closed Ferrum face identifier.
    #[must_use]
    pub const fn id(&self) -> FerrumFontId {
        self.id
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

    /// Return the required font-family metadata for this closed resource.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Return the required PostScript face metadata for this closed resource.
    #[must_use]
    pub fn postscript_name(&self) -> &str {
        &self.postscript_name
    }

    /// Return the immutable, digest-verified bytes used by every font-engine consumer.
    #[must_use]
    pub fn data(&self) -> &Arc<[u8]> {
        &self.data
    }
}

/// A verified, immutable font resource environment for M12 measurements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FerrumFontEnvironmentV1 {
    telex: FontAssetDescriptor,
}

impl FerrumFontEnvironmentV1 {
    /// Load and verify the bundled Telex Regular asset before any font engine opens it.
    ///
    /// The authoritative resource is compiled into the Rust artifact, so an installed
    /// extension never depends on a source-tree path. It is still length, digest, and
    /// TrueType-metadata verified before the pure-Rust metrics engine consumes it.
    pub fn load() -> Result<Self, RenderError> {
        load_verified_bytes(
            FerrumFontId::TelexRegular,
            include_bytes!("../assets/fonts/Telex-Regular.ttf"),
        )
    }

    /// Return the verified descriptor for the requested closed font identifier.
    #[must_use]
    pub const fn descriptor(&self, id: FerrumFontId) -> &FontAssetDescriptor {
        match id {
            FerrumFontId::TelexRegular => &self.telex,
        }
    }

    #[cfg(test)]
    fn load_from_path(id: FerrumFontId, path: &Path) -> Result<Self, RenderError> {
        Self::load_from_path_with_hooks(id, path, || {}, || {})
    }

    #[cfg(test)]
    fn load_from_path_with_hooks<AfterParentOpen, AfterFinalOpen>(
        id: FerrumFontId,
        path: &Path,
        after_parent_open: AfterParentOpen,
        after_final_open: AfterFinalOpen,
    ) -> Result<Self, RenderError>
    where
        AfterParentOpen: FnOnce(),
        AfterFinalOpen: FnOnce(),
    {
        let (parent, name) = open_trusted_parent(path)?;
        after_parent_open();
        let fd = openat(
            &parent,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| final_open_error(path, error))?;
        after_final_open();
        load_verified_descriptor(id, fd)
    }

    #[cfg(test)]
    pub(crate) fn load_for_test(path: &Path) -> Result<Self, RenderError> {
        Self::load_from_path(FerrumFontId::TelexRegular, path)
    }

    #[cfg(test)]
    pub(crate) fn load_for_test_with_after_parent_open<Hook>(
        path: &Path,
        hook: Hook,
    ) -> Result<Self, RenderError>
    where
        Hook: FnOnce(),
    {
        Self::load_from_path_with_hooks(FerrumFontId::TelexRegular, path, hook, || {})
    }

    #[cfg(test)]
    pub(crate) fn load_for_test_with_after_final_open<Hook>(
        path: &Path,
        hook: Hook,
    ) -> Result<Self, RenderError>
    where
        Hook: FnOnce(),
    {
        Self::load_from_path_with_hooks(FerrumFontId::TelexRegular, path, || {}, hook)
    }
}

#[cfg(test)]
fn load_verified_descriptor(
    id: FerrumFontId,
    fd: OwnedFd,
) -> Result<FerrumFontEnvironmentV1, RenderError> {
    let descriptor_metadata = fstat(&fd).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not inspect verified Telex descriptor: {error}"
        ))
    })?;
    if !FileType::from_raw_mode(descriptor_metadata.st_mode).is_file()
        || descriptor_metadata.st_size as u64 != TELEX_BYTES
    {
        return Err(RenderError::InvalidRequest(
            "verified Telex descriptor has an unexpected file type or byte length".to_owned(),
        ));
    }
    let mut file = File::from(fd);
    let metadata = file.metadata().map_err(|error| {
        RenderError::InvalidRequest(format!("could not inspect verified Telex asset: {error}"))
    })?;
    if !metadata.is_file() || metadata.len() != TELEX_BYTES {
        return Err(RenderError::InvalidRequest(
            "verified Telex asset has an unexpected file type or byte length".to_owned(),
        ));
    }
    let mut bytes = Vec::with_capacity(TELEX_BYTES as usize);
    file.read_to_end(&mut bytes).map_err(|error| {
        RenderError::InvalidRequest(format!("could not read verified Telex asset: {error}"))
    })?;
    if bytes.len() as u64 != TELEX_BYTES {
        return Err(RenderError::InvalidRequest(
            "verified Telex asset changed while it was read".to_owned(),
        ));
    }
    load_verified_bytes(id, &bytes)
}

/// Validate immutable bytes from either the compiled resource or a hardened test fixture.
fn load_verified_bytes(
    id: FerrumFontId,
    bytes: &[u8],
) -> Result<FerrumFontEnvironmentV1, RenderError> {
    if bytes.len() as u64 != TELEX_BYTES {
        return Err(RenderError::InvalidRequest(
            "verified Telex asset has an unexpected byte length".to_owned(),
        ));
    }
    let digest = hex_digest(Sha256::digest(bytes).as_slice());
    if digest != TELEX_SHA256 {
        return Err(RenderError::InvalidRequest(
            "verified Telex asset has an unexpected SHA-256 digest".to_owned(),
        ));
    }
    verify_face_metadata(bytes)?;
    Ok(FerrumFontEnvironmentV1 {
        telex: FontAssetDescriptor {
            id,
            bytes: TELEX_BYTES,
            sha256: TELEX_SHA256.to_owned(),
            family: TELEX_FAMILY.to_owned(),
            postscript_name: TELEX_POSTSCRIPT_NAME.to_owned(),
            data: Arc::from(bytes),
        },
    })
}

fn verify_face_metadata(bytes: &[u8]) -> Result<(), RenderError> {
    let face = Face::parse(bytes, 0).map_err(|error| {
        RenderError::InvalidRequest(format!("could not parse verified Telex bytes: {error}"))
    })?;
    let family = face
        .names()
        .into_iter()
        .filter(|name| name.name_id == name_id::FAMILY)
        .find_map(|name| name.to_string());
    let postscript_name = face
        .names()
        .into_iter()
        .filter(|name| name.name_id == name_id::POST_SCRIPT_NAME)
        .find_map(|name| name.to_string());
    if family.as_deref() != Some(TELEX_FAMILY)
        || postscript_name.as_deref() != Some(TELEX_POSTSCRIPT_NAME)
        || face.tables().kern.is_some()
    {
        return Err(RenderError::InvalidRequest(
            "verified Telex asset has unexpected face metadata".to_owned(),
        ));
    }
    Ok(())
}

/// Open every parent through a retained descriptor chain and return the final name.
///
/// The root directory descriptor and every child descriptor use `NOFOLLOW`; after a
/// parent is open, a visible-path replacement cannot change the parent used for the
/// final `openat`. This is the only packaged-resource path boundary in the renderer.
#[cfg(test)]
fn open_trusted_parent(path: &Path) -> Result<(OwnedFd, &OsStr), RenderError> {
    if !path.is_absolute() {
        return Err(RenderError::InvalidRequest(
            "verified Telex asset path must be absolute".to_owned(),
        ));
    }
    let name = path.file_name().ok_or_else(|| {
        RenderError::InvalidRequest("verified Telex asset path must name a file".to_owned())
    })?;
    let parent = path.parent().ok_or_else(|| {
        RenderError::InvalidRequest("verified Telex asset path must have a parent".to_owned())
    })?;
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = openat(CWD, Path::new("/"), flags, Mode::empty()).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not open verified Telex resource root: {error}"
        ))
    })?;
    for component in parent.components() {
        match component {
            Component::Normal(component) => {
                directory = openat(&directory, component, flags, Mode::empty())
                    .map_err(|error| parent_open_error(path, error))?;
            }
            Component::Prefix(_) | Component::RootDir => {}
            Component::CurDir | Component::ParentDir => {
                return Err(RenderError::InvalidRequest(
                    "verified Telex asset path must not contain traversal components".to_owned(),
                ));
            }
        }
    }
    Ok((directory, name))
}

#[cfg(test)]
fn parent_open_error(path: &Path, error: Errno) -> RenderError {
    match error {
        Errno::LOOP => RenderError::InvalidRequest(format!(
            "verified Telex resource parent must not traverse a symbolic link: {}",
            path.display()
        )),
        Errno::NOTDIR => RenderError::InvalidRequest(format!(
            "verified Telex resource parent must be a directory: {}",
            path.display()
        )),
        _ => RenderError::InvalidRequest(format!(
            "could not open verified Telex resource parent {}: {error}",
            path.display()
        )),
    }
}

#[cfg(test)]
fn final_open_error(path: &Path, error: Errno) -> RenderError {
    match error {
        Errno::LOOP => RenderError::InvalidRequest(format!(
            "verified Telex resource must not be a symbolic link: {}",
            path.display()
        )),
        _ => RenderError::InvalidRequest(format!("verified Telex asset is unavailable: {error}")),
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
