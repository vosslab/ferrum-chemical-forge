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

use ferrum_render_contract::{MOLECULE_LABEL_RESOURCE_ID, MOLECULE_LABEL_SHA256};

const MOLECULE_LABEL_BYTES: u64 = 65_068;
const MOLECULE_LABEL_FAMILY: &str = "Atkinson Hyperlegible Next";
const MOLECULE_LABEL_POSTSCRIPT_NAME: &str = "AtkinsonHyperlegibleNext-Regular";

/// Immutable facts identifying a verified bundled font asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontAssetDescriptor {
    resource_id: &'static str,
    bytes: u64,
    sha256: String,
    family: String,
    postscript_name: String,
    data: Arc<[u8]>,
}

impl FontAssetDescriptor {
    /// Return the stable serialized resource identifier.
    #[must_use]
    pub const fn resource_id(&self) -> &'static str {
        self.resource_id
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

/// The verified, immutable font resources selected for current renderer roles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FerrumFontEnvironment {
    molecule_label: FontAssetDescriptor,
}

impl FerrumFontEnvironment {
    /// Load and verify the bundled Atkinson Hyperlegible Next Regular asset before any font engine opens it.
    ///
    /// The authoritative resource is compiled into the Rust artifact, so an installed
    /// extension never depends on a source-tree path. It is still length, digest, and
    /// TrueType-metadata verified before the pure-Rust metrics engine consumes it.
    pub fn load() -> Result<Self, RenderError> {
        load_verified_bytes(include_bytes!(
            "../assets/fonts/atkinson_hyperlegible_next/ttf/atkinson_hyperlegible_next_regular.ttf"
        ))
    }

    /// Return the verified resource selected for molecule-label rendering.
    ///
    /// This method is the single selection boundary for Ferrum's current label
    /// face. Consumers depend on the role while the descriptor retains the exact
    /// resource identity required for deterministic plans and frontend replay.
    #[must_use]
    pub const fn molecule_label(&self) -> &FontAssetDescriptor {
        &self.molecule_label
    }

    #[cfg(test)]
    fn load_from_path(path: &Path) -> Result<Self, RenderError> {
        Self::load_from_path_with_hooks(path, || {}, || {})
    }

    #[cfg(test)]
    fn load_from_path_with_hooks<AfterParentOpen, AfterFinalOpen>(
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
        load_verified_descriptor(fd)
    }

    #[cfg(test)]
    pub(crate) fn load_for_test(path: &Path) -> Result<Self, RenderError> {
        Self::load_from_path(path)
    }

    #[cfg(test)]
    pub(crate) fn load_for_test_with_after_parent_open<Hook>(
        path: &Path,
        hook: Hook,
    ) -> Result<Self, RenderError>
    where
        Hook: FnOnce(),
    {
        Self::load_from_path_with_hooks(path, hook, || {})
    }

    #[cfg(test)]
    pub(crate) fn load_for_test_with_after_final_open<Hook>(
        path: &Path,
        hook: Hook,
    ) -> Result<Self, RenderError>
    where
        Hook: FnOnce(),
    {
        Self::load_from_path_with_hooks(path, || {}, hook)
    }
}

#[cfg(test)]
fn load_verified_descriptor(fd: OwnedFd) -> Result<FerrumFontEnvironment, RenderError> {
    let descriptor_metadata = fstat(&fd).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not inspect verified Atkinson Hyperlegible Next descriptor: {error}"
        ))
    })?;
    if !FileType::from_raw_mode(descriptor_metadata.st_mode).is_file()
        || descriptor_metadata.st_size as u64 != MOLECULE_LABEL_BYTES
    {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next descriptor has an unexpected file type or byte length".to_owned(),
        ));
    }
    let mut file = File::from(fd);
    let metadata = file.metadata().map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not inspect verified Atkinson Hyperlegible Next asset: {error}"
        ))
    })?;
    if !metadata.is_file() || metadata.len() != MOLECULE_LABEL_BYTES {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset has an unexpected file type or byte length"
                .to_owned(),
        ));
    }
    let mut bytes = Vec::with_capacity(MOLECULE_LABEL_BYTES as usize);
    file.read_to_end(&mut bytes).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not read verified Atkinson Hyperlegible Next asset: {error}"
        ))
    })?;
    if bytes.len() as u64 != MOLECULE_LABEL_BYTES {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset changed while it was read".to_owned(),
        ));
    }
    load_verified_bytes(&bytes)
}

/// Validate immutable bytes from either the compiled resource or a hardened test fixture.
fn load_verified_bytes(bytes: &[u8]) -> Result<FerrumFontEnvironment, RenderError> {
    if bytes.len() as u64 != MOLECULE_LABEL_BYTES {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset has an unexpected byte length".to_owned(),
        ));
    }
    let digest = hex_digest(Sha256::digest(bytes).as_slice());
    if digest != MOLECULE_LABEL_SHA256 {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset has an unexpected SHA-256 digest".to_owned(),
        ));
    }
    verify_face_metadata(bytes)?;
    Ok(FerrumFontEnvironment {
        molecule_label: FontAssetDescriptor {
            resource_id: MOLECULE_LABEL_RESOURCE_ID,
            bytes: MOLECULE_LABEL_BYTES,
            sha256: MOLECULE_LABEL_SHA256.to_owned(),
            family: MOLECULE_LABEL_FAMILY.to_owned(),
            postscript_name: MOLECULE_LABEL_POSTSCRIPT_NAME.to_owned(),
            data: Arc::from(bytes),
        },
    })
}

fn verify_face_metadata(bytes: &[u8]) -> Result<(), RenderError> {
    let face = Face::parse(bytes, 0).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not parse verified Atkinson Hyperlegible Next bytes: {error}"
        ))
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
    if family.as_deref() != Some(MOLECULE_LABEL_FAMILY)
        || postscript_name.as_deref() != Some(MOLECULE_LABEL_POSTSCRIPT_NAME)
        || face.tables().kern.is_some()
    {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset has unexpected face metadata".to_owned(),
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
            "verified Atkinson Hyperlegible Next asset path must be absolute".to_owned(),
        ));
    }
    let name = path.file_name().ok_or_else(|| {
        RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset path must name a file".to_owned(),
        )
    })?;
    let parent = path.parent().ok_or_else(|| {
        RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset path must have a parent".to_owned(),
        )
    })?;
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = openat(CWD, Path::new("/"), flags, Mode::empty()).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not open verified Atkinson Hyperlegible Next resource root: {error}"
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
                    "verified Atkinson Hyperlegible Next asset path must not contain traversal components".to_owned(),
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
            "verified Atkinson Hyperlegible Next resource parent must not traverse a symbolic link: {}",
            path.display()
        )),
        Errno::NOTDIR => RenderError::InvalidRequest(format!(
            "verified Atkinson Hyperlegible Next resource parent must be a directory: {}",
            path.display()
        )),
        _ => RenderError::InvalidRequest(format!(
            "could not open verified Atkinson Hyperlegible Next resource parent {}: {error}",
            path.display()
        )),
    }
}

#[cfg(test)]
fn final_open_error(path: &Path, error: Errno) -> RenderError {
    match error {
        Errno::LOOP => RenderError::InvalidRequest(format!(
            "verified Atkinson Hyperlegible Next resource must not be a symbolic link: {}",
            path.display()
        )),
        _ => RenderError::InvalidRequest(format!(
            "verified Atkinson Hyperlegible Next asset is unavailable: {error}"
        )),
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
