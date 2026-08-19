//! Fixed-root installation and lookup for native Ferrum chemistry bundles.
//!
//! Cargo installs the executable but cannot carry a relocatable dynamic-library
//! closure. This module accepts an explicit bundle directory, validates its
//! fixed manifest, and publishes an active bundle record without searching the
//! current directory, `PATH`, Python installations, or adapter environment variables.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ferrum_chemistry::ADAPTER_ABI_VERSION;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::protocol::runtime::TrustedLibraryChemistryRuntimeV1;

const BUNDLE_MANIFEST_NAME: &str = "ferrum-engine-bundle-v1.json";
const ACTIVE_RECORD_NAME: &str = "active-v1.json";
const ADAPTER_NAME: &str = "libferrum_chem.dylib";
const BUNDLE_SCHEMA: &str = "ferrum-engine-bundle-v1";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BundleManifestV1 {
    schema: String,
    target: String,
    adapter_abi_version: u32,
    adapter: String,
    members: Vec<BundleMemberV1>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BundleMemberV1 {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ActiveBundleV1 {
    schema: String,
    bundle_id: String,
}

/// User-facing result of a fixed-root engine lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EngineStatusV1 {
    /// No active bundle has been installed.
    NotInstalled,
    /// The named active bundle passed its fixed-manifest validation.
    Ready,
    /// An active record exists but it cannot safely supply an adapter.
    Invalid,
}

/// CLI engine bundle failure.
#[derive(Debug, Error)]
pub enum EngineBundleErrorV1 {
    /// The caller did not supply a regular bundle directory.
    #[error("engine bundle is not a regular directory: {0}")]
    InvalidBundle(PathBuf),
    /// A required bundle member is absent, irregular, or symbolic.
    #[error("engine bundle member is not a regular non-symbolic file: {0}")]
    UnsafeMember(PathBuf),
    /// The bundle manifest is invalid for this executable.
    #[error("engine bundle manifest is invalid: {0}")]
    InvalidManifest(String),
    /// A filesystem operation prevented installation or lookup.
    #[error("engine bundle filesystem operation failed: {0}")]
    Io(#[from] io::Error),
    /// The per-user fixed application-data root could not be determined.
    #[error("Ferrum application-data root is unavailable")]
    ApplicationDataUnavailable,
}

/// Install an explicitly named bundle at Ferrum's fixed application-data root.
pub(crate) fn install_bundle(source: &Path) -> Result<(), EngineBundleErrorV1> {
    install_bundle_at(source, &application_data_root()?)
}

/// Write a stable plain-text engine status for the CLI.
pub(crate) fn write_status(stdout: &mut dyn Write) -> Result<(), EngineBundleErrorV1> {
    let status = match application_data_root() {
        Ok(root) => status_at(&root),
        Err(_) => EngineStatusV1::Invalid,
    };
    let text = match status {
        EngineStatusV1::NotInstalled => "not-installed\n",
        EngineStatusV1::Ready => "ready\n",
        EngineStatusV1::Invalid => "invalid\n",
    };
    stdout
        .write_all(text.as_bytes())
        .map_err(EngineBundleErrorV1::Io)
}

/// Resolve the active bundle into a trusted short-lived chemistry runtime.
pub fn active_runtime() -> Result<TrustedLibraryChemistryRuntimeV1, EngineBundleErrorV1> {
    active_runtime_at(&application_data_root()?)
}

fn application_data_root() -> Result<PathBuf, EngineBundleErrorV1> {
    let home = std::env::var_os("HOME").ok_or(EngineBundleErrorV1::ApplicationDataUnavailable)?;
    #[cfg(target_os = "macos")]
    let root = PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Ferrum")
        .join("engine-v1");
    #[cfg(not(target_os = "macos"))]
    let root = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("ferrum")
        .join("engine-v1");
    Ok(root)
}

fn install_bundle_at(source: &Path, root: &Path) -> Result<(), EngineBundleErrorV1> {
    fs::create_dir_all(root.join("bundles"))?;
    let bundle_id = fresh_bundle_id()?;
    let staging = root.join("bundles").join(format!(".{bundle_id}.staging"));
    copy_bundle(source, &staging)?;
    validate_bundle(&staging)?;
    sync_tree(&staging)?;
    let destination = root.join("bundles").join(&bundle_id);
    fs::rename(&staging, &destination)?;
    let active = ActiveBundleV1 {
        schema: BUNDLE_SCHEMA.to_owned(),
        bundle_id,
    };
    publish_active_record(root, &active)
}

fn active_runtime_at(root: &Path) -> Result<TrustedLibraryChemistryRuntimeV1, EngineBundleErrorV1> {
    let active_path = root.join(ACTIVE_RECORD_NAME);
    let active = read_regular_json::<ActiveBundleV1>(&active_path)?;
    if active.schema != BUNDLE_SCHEMA || !is_safe_component(&active.bundle_id) {
        return Err(EngineBundleErrorV1::InvalidManifest(
            "active bundle record is not valid for Ferrum engine V1".to_owned(),
        ));
    }
    let bundle = root.join("bundles").join(active.bundle_id);
    validate_bundle(&bundle)?;
    Ok(TrustedLibraryChemistryRuntimeV1::from_trusted_library(
        bundle.join(ADAPTER_NAME),
    ))
}

fn status_at(root: &Path) -> EngineStatusV1 {
    let active_path = root.join(ACTIVE_RECORD_NAME);
    if !active_path.exists() {
        return EngineStatusV1::NotInstalled;
    }
    match active_runtime_at(root) {
        Ok(_) => EngineStatusV1::Ready,
        Err(_) => EngineStatusV1::Invalid,
    }
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

fn copy_bundle(source: &Path, destination: &Path) -> Result<(), EngineBundleErrorV1> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.is_file() && !metadata.file_type().is_symlink() {
            let mut input = open_no_follow(&source_path)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination_path)?;
            io::copy(&mut input, &mut output)?;
            output.sync_all()?;
        } else {
            return Err(EngineBundleErrorV1::UnsafeMember(source_path));
        }
    }
    Ok(())
}

fn publish_active_record(root: &Path, active: &ActiveBundleV1) -> Result<(), EngineBundleErrorV1> {
    let temporary = root.join(format!(".{ACTIVE_RECORD_NAME}.{}.tmp", fresh_bundle_id()?));
    let bytes = serde_json::to_vec(active)
        .map_err(|error| EngineBundleErrorV1::InvalidManifest(error.to_string()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(temporary, root.join(ACTIVE_RECORD_NAME))?;
    Ok(())
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

fn sync_tree(directory: &Path) -> Result<(), EngineBundleErrorV1> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        ensure_regular_file(&entry.path())?;
        File::open(entry.path())?.sync_all()?;
    }
    File::open(directory)?.sync_all()?;
    Ok(())
}

fn fresh_bundle_id() -> Result<String, EngineBundleErrorV1> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| EngineBundleErrorV1::InvalidManifest(error.to_string()))?
        .as_nanos();
    Ok(format!("{nanos:032x}"))
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
    use std::fs;

    use super::{
        ADAPTER_NAME, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA, EngineStatusV1, install_bundle_at,
        status_at,
    };

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("ferrum-engine-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temporary directory");
        path
    }

    #[test]
    fn fixed_root_install_publishes_a_validated_bundle() {
        let source = temporary_directory("source");
        let root = temporary_directory("root");
        fs::write(source.join(ADAPTER_NAME), b"fixture adapter").expect("write adapter");
        let manifest = serde_json::json!({
            "schema": BUNDLE_SCHEMA,
            "target": super::executable_target(),
            "adapter_abi_version": ferrum_chemistry::ADAPTER_ABI_VERSION,
            "adapter": ADAPTER_NAME,
            "members": [{"path": ADAPTER_NAME, "sha256": super::sha256_file(&source.join(ADAPTER_NAME)).expect("digest")}],
        });
        fs::write(
            source.join(BUNDLE_MANIFEST_NAME),
            serde_json::to_vec(&manifest).expect("manifest"),
        )
        .expect("write manifest");
        install_bundle_at(&source, &root).expect("install trusted fixture bundle");
        assert_eq!(status_at(&root), EngineStatusV1::Ready);
        let _ = fs::remove_dir_all(source);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn bundle_with_wrong_abi_is_not_published() {
        let source = temporary_directory("bad-source");
        let root = temporary_directory("bad-root");
        fs::write(source.join(ADAPTER_NAME), b"fixture adapter").expect("write adapter");
        let manifest = serde_json::json!({
            "schema": BUNDLE_SCHEMA,
            "target": super::executable_target(),
            "adapter_abi_version": 0,
            "adapter": ADAPTER_NAME,
            "members": [{"path": ADAPTER_NAME, "sha256": super::sha256_file(&source.join(ADAPTER_NAME)).expect("digest")}],
        });
        fs::write(
            source.join(BUNDLE_MANIFEST_NAME),
            serde_json::to_vec(&manifest).expect("manifest"),
        )
        .expect("write manifest");
        assert!(install_bundle_at(&source, &root).is_err());
        assert_eq!(status_at(&root), EngineStatusV1::NotInstalled);
        let _ = fs::remove_dir_all(source);
        let _ = fs::remove_dir_all(root);
    }
}
