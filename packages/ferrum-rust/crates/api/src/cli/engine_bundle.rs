//! Executable-relative lookup for the local Ferrum chemistry bundle.
//!
//! `build.sh` owns one sealed native closure at `build/runtime/engine-v1` beside
//! the staged `build/bin/ferrum` executable. The local CLI derives that closure
//! from its executable path, then validates its fixed manifest before loading
//! the adapter. It never consults a per-user install location, the current
//! directory, `PATH`, Python installations, or adapter environment variables.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use ferrum_chemistry::{ADAPTER_ABI_VERSION, NativeChemEngine};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::protocol::runtime::TrustedLibraryChemistryRuntimeV1;

const BUNDLE_MANIFEST_NAME: &str = "ferrum-engine-bundle-v1.json";
const ADAPTER_NAME: &str = "libferrum_chem.dylib";
const BUNDLE_SCHEMA: &str = "ferrum-engine-bundle-v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleManifestV1 {
    schema: String,
    target: String,
    adapter_abi_version: u32,
    adapter: String,
    members: Vec<BundleMemberV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleMemberV1 {
    path: String,
    sha256: String,
}

/// CLI engine bundle failure.
#[derive(Debug, Error)]
pub enum EngineBundleErrorV1 {
    /// The executable path cannot identify its adjacent local runtime.
    #[error("local Ferrum executable path has no build runtime parent: {0}")]
    ExecutablePathUnavailable(PathBuf),
    /// The executable-relative bundle is not a regular directory.
    #[error("local engine bundle is not a regular directory: {0}")]
    InvalidBundle(PathBuf),
    /// A required bundle member is absent, irregular, or symbolic.
    #[error("engine bundle member is not a regular non-symbolic file: {0}")]
    UnsafeMember(PathBuf),
    /// The bundle manifest is invalid for this executable.
    #[error("engine bundle manifest is invalid: {0}")]
    InvalidManifest(String),
    /// A filesystem operation prevented local bundle lookup.
    #[error("local engine bundle filesystem operation failed: {0}")]
    Io(#[from] io::Error),
    /// The validated native adapter could not load.
    #[error("validated local engine adapter could not load: {0}")]
    Chemistry(#[source] ferrum_chemistry::ChemistryError),
}

/// Resolve the staged local bundle into a trusted short-lived chemistry runtime.
pub fn active_runtime() -> Result<TrustedLibraryChemistryRuntimeV1, EngineBundleErrorV1> {
    Ok(TrustedLibraryChemistryRuntimeV1::from_trusted_library(
        active_library_path()?,
    ))
}

/// Resolve the staged local bundle and load its native chemistry engine.
pub fn active_native_engine() -> Result<NativeChemEngine, EngineBundleErrorV1> {
    let library_path = active_library_path()?;
    NativeChemEngine::load(&library_path).map_err(EngineBundleErrorV1::Chemistry)
}

fn active_library_path() -> Result<PathBuf, EngineBundleErrorV1> {
    let executable = std::env::current_exe()?;
    let bundle = local_bundle_path_from_executable(&executable)?;
    validate_bundle(&bundle)?;
    Ok(bundle.join(ADAPTER_NAME))
}

fn local_bundle_path_from_executable(executable: &Path) -> Result<PathBuf, EngineBundleErrorV1> {
    let bin_root = executable
        .parent()
        .ok_or_else(|| EngineBundleErrorV1::ExecutablePathUnavailable(executable.to_path_buf()))?;
    let build_root = bin_root
        .parent()
        .ok_or_else(|| EngineBundleErrorV1::ExecutablePathUnavailable(executable.to_path_buf()))?;
    Ok(build_root.join("runtime").join("engine-v1"))
}

fn validate_bundle(bundle: &Path) -> Result<(), EngineBundleErrorV1> {
    let metadata = fs::symlink_metadata(bundle)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(EngineBundleErrorV1::InvalidBundle(bundle.to_path_buf()));
    }
    let manifest_path = bundle.join(BUNDLE_MANIFEST_NAME);
    let manifest = read_regular_json::<BundleManifestV1>(&manifest_path)?;
    if manifest.schema != BUNDLE_SCHEMA
        || manifest.target != executable_target()
        || manifest.adapter_abi_version != ADAPTER_ABI_VERSION
        || manifest.adapter != ADAPTER_NAME
    {
        return Err(EngineBundleErrorV1::InvalidManifest(
            "schema, target, ABI version, or adapter name did not match".to_owned(),
        ));
    }
    validate_members(bundle, &manifest)
}

fn validate_members(bundle: &Path, manifest: &BundleManifestV1) -> Result<(), EngineBundleErrorV1> {
    if manifest.members.is_empty()
        || !manifest
            .members
            .iter()
            .any(|member| member.path == ADAPTER_NAME)
    {
        return Err(EngineBundleErrorV1::InvalidManifest(
            "manifest lacks adapter member".to_owned(),
        ));
    }
    let mut expected = std::collections::BTreeSet::from([BUNDLE_MANIFEST_NAME.to_owned()]);
    for member in &manifest.members {
        if !is_safe_component(&member.path)
            || !is_sha256(&member.sha256)
            || !expected.insert(member.path.clone())
        {
            return Err(EngineBundleErrorV1::InvalidManifest(
                "member path or digest is invalid".to_owned(),
            ));
        }
        let path = bundle.join(&member.path);
        ensure_regular_file(&path)?;
        if sha256_file(&path)? != member.sha256 {
            return Err(EngineBundleErrorV1::InvalidManifest(format!(
                "digest mismatch: {}",
                member.path
            )));
        }
    }
    let mut actual = std::collections::BTreeSet::new();
    for entry in fs::read_dir(bundle)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(EngineBundleErrorV1::UnsafeMember(path));
        }
        actual.insert(entry.file_name().to_string_lossy().into_owned());
    }
    if actual != expected {
        return Err(EngineBundleErrorV1::InvalidManifest(
            "bundle has missing or extra members".to_owned(),
        ));
    }
    Ok(())
}

fn ensure_regular_file(path: &Path) -> Result<(), EngineBundleErrorV1> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() && !metadata.file_type().is_symlink() {
        Ok(())
    } else {
        Err(EngineBundleErrorV1::UnsafeMember(path.to_path_buf()))
    }
}

fn read_regular_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, EngineBundleErrorV1> {
    ensure_regular_file(path)?;
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| EngineBundleErrorV1::InvalidManifest(error.to_string()))
}

fn open_no_follow(path: &Path) -> Result<File, EngineBundleErrorV1> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        Ok(OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)?)
    }
    #[cfg(not(unix))]
    {
        Ok(File::open(path)?)
    }
}

fn sha256_file(path: &Path) -> Result<String, EngineBundleErrorV1> {
    let mut file = open_no_follow(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_safe_component(value: &str) -> bool {
    let path = Path::new(value);
    path.components().count() == 1
        && matches!(path.components().next(), Some(Component::Normal(_)))
        && !value.is_empty()
}

fn executable_target() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

#[cfg(test)]
mod tests {
    use super::local_bundle_path_from_executable;

    #[test]
    fn staged_executable_resolves_only_its_sibling_runtime_bundle() {
        let executable = std::path::Path::new("/workspace/ferrum/build/bin/ferrum");
        assert_eq!(
            local_bundle_path_from_executable(executable).expect("staged executable path"),
            std::path::Path::new("/workspace/ferrum/build/runtime/engine-v1"),
        );
    }
}
