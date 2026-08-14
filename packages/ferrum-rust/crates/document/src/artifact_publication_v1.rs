//! Safe descriptor-relative publication of completed artifact bytes.
//!
//! This is intentionally a trusted-output-directory protocol: callers must ensure
//! that no same-UID or otherwise-authorized actor mutates the output directory
//! between Ferrum's final validation and `renameat`.  Portable macOS and Linux
//! `renameat` cannot impose an expected inode precondition, so this module refuses
//! aliases observed at both checks but does not claim to defeat a later hostile
//! hard-link swap.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::OwnedFd;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rustix::fs::{
    AtFlags, CWD, FileType, Mode, OFlags, fstat, fsync, openat, renameat, statat, unlinkat,
};
use rustix::io::Errno;
use thiserror::Error;

const TEMPORARY_ATTEMPTS: u8 = 16;
static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

/// A retained device/inode identity for an opened regular source file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RetainedRegularFileIdentityV1 {
    device: u64,
    inode: u64,
}

impl RetainedRegularFileIdentityV1 {
    /// Return the source file device identity.
    #[must_use]
    pub fn device(self) -> u64 {
        self.device
    }

    /// Return the source file inode identity.
    #[must_use]
    pub fn inode(self) -> u64 {
        self.inode
    }
}

/// An owned live descriptor for a regular source file.
pub struct RetainedSourceFileGuardV1 {
    file: File,
    identity: RetainedRegularFileIdentityV1,
}

impl fmt::Debug for RetainedSourceFileGuardV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetainedSourceFileGuardV1")
            .field("identity", &self.identity)
            .finish()
    }
}

impl RetainedSourceFileGuardV1 {
    /// Return the captured regular-file identity without exposing its descriptor.
    #[must_use]
    pub fn identity(&self) -> RetainedRegularFileIdentityV1 {
        self.identity
    }

    fn verify_live_identity(&self) -> Result<(), io::Error> {
        let current = identity_from_stat(fstat(&self.file).map_err(io::Error::from)?);
        if current == self.identity {
            Ok(())
        } else {
            Err(io::Error::other("retained source file identity changed"))
        }
    }
}

/// Retain an already-open regular source file until publication completes.
pub fn retain_regular_source_file_v1(
    opened_file: File,
) -> Result<RetainedSourceFileGuardV1, RetainedSourceIdentityErrorV1> {
    let metadata =
        fstat(&opened_file).map_err(|source| RetainedSourceIdentityErrorV1::Inspect {
            source: source.into(),
        })?;
    if !FileType::from_raw_mode(metadata.st_mode).is_file() {
        return Err(RetainedSourceIdentityErrorV1::NonRegular);
    }
    Ok(RetainedSourceFileGuardV1 {
        file: opened_file,
        identity: identity_from_stat(metadata),
    })
}

/// An owned completed artifact and destination for one publication operation.
pub struct ArtifactPublicationRequestV1 {
    destination: PathBuf,
    bytes: Vec<u8>,
    retained_source: Option<RetainedSourceFileGuardV1>,
}

impl fmt::Debug for ArtifactPublicationRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArtifactPublicationRequestV1")
            .field("destination", &self.destination)
            .field("byte_len", &self.bytes.len())
            .field("has_retained_source", &self.retained_source.is_some())
            .finish()
    }
}

impl ArtifactPublicationRequestV1 {
    /// Construct a request from a completed owned artifact.
    #[must_use]
    pub fn new(destination: PathBuf, bytes: Vec<u8>) -> Self {
        Self {
            destination,
            bytes,
            retained_source: None,
        }
    }

    /// Attach the live source descriptor used for observed-alias protection.
    #[must_use]
    pub fn with_retained_source(mut self, guard: RetainedSourceFileGuardV1) -> Self {
        self.retained_source = Some(guard);
        self
    }

    /// Return the immutable requested destination.
    #[must_use]
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Return the completed immutable artifact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Return the retained source identity, if this request has one.
    #[must_use]
    pub fn retained_source_identity(&self) -> Option<RetainedRegularFileIdentityV1> {
        self.retained_source
            .as_ref()
            .map(RetainedSourceFileGuardV1::identity)
    }
}

/// The immutable receipt for a completed or possibly completed replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactPublicationReceiptV1 {
    destination: PathBuf,
    retained_source: Option<RetainedRegularFileIdentityV1>,
}

impl ArtifactPublicationReceiptV1 {
    /// Return the published destination.
    #[must_use]
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Consume this receipt and return its owned destination.
    #[must_use]
    pub fn into_destination(self) -> PathBuf {
        self.destination
    }

    /// Return the source identity retained for this publication, if any.
    #[must_use]
    pub fn retained_source_identity(&self) -> Option<RetainedRegularFileIdentityV1> {
        self.retained_source
    }
}

/// Directory-entry durability status after a successful rename.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactPublicationDurabilityV1 {
    /// Data and containing directory synchronization succeeded.
    Confirmed,
    /// Replacement succeeded but the platform declined directory synchronization.
    DirectoryEntryUnconfirmed,
}

/// Publication result after a successful replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactPublicationOutcomeV1 {
    /// Data and directory entry durability were confirmed.
    ConfirmedDurable(ArtifactPublicationReceiptV1),
    /// Data was synchronized, but directory-entry durability is unavailable.
    DirectoryEntryUnconfirmed(ArtifactPublicationReceiptV1),
}

impl ArtifactPublicationOutcomeV1 {
    /// Return the immutable publication receipt.
    #[must_use]
    pub fn receipt(&self) -> &ArtifactPublicationReceiptV1 {
        match self {
            Self::ConfirmedDurable(receipt) | Self::DirectoryEntryUnconfirmed(receipt) => receipt,
        }
    }

    /// Return the exact durability status.
    #[must_use]
    pub fn durability(&self) -> ArtifactPublicationDurabilityV1 {
        match self {
            Self::ConfirmedDurable(_) => ArtifactPublicationDurabilityV1::Confirmed,
            Self::DirectoryEntryUnconfirmed(_) => {
                ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed
            }
        }
    }
}

/// Failure while establishing a retained source-file identity.
#[derive(Debug, Error)]
pub enum RetainedSourceIdentityErrorV1 {
    /// The opened descriptor could not be inspected.
    #[error("could not inspect retained source file: {source}")]
    Inspect {
        /// Underlying descriptor inspection failure.
        #[source]
        source: io::Error,
    },
    /// The opened descriptor is not a regular file.
    #[error("retained source file must be regular")]
    NonRegular,
}

/// A typed final-destination refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactDestinationRejectionV1 {
    /// The path does not name a final file.
    MissingFileName,
    /// A spelled parent component traverses a symlink.
    ParentTraversesSymlink,
    /// A spelled parent component is not a directory.
    ParentIsNotDirectory,
    /// The final entry is a symlink.
    FinalIsSymlink,
    /// The final entry exists but is not regular.
    FinalIsNotRegular,
    /// The final entry is an observed alias of the retained source descriptor.
    SourceAliasesDestination,
}

/// The pre-replacement phase that encountered an I/O failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactPrepublicationPhaseV1 {
    /// Opening the retained parent directory chain.
    OpenParent,
    /// The first final destination validation.
    ValidateBeforeTemporary,
    /// Reserving a same-directory temporary sibling.
    ReserveTemporary,
    /// Checking the newly reserved temporary file.
    ValidateTemporary,
    /// Writing or synchronizing the temporary file.
    WriteOrSyncTemporary,
    /// The final validation immediately before rename.
    ValidateBeforeRename,
    /// Calling `renameat` in the retained parent directory.
    Rename,
}

/// Failure while publishing a generic artifact.
#[derive(Debug, Error)]
pub enum ArtifactPublicationErrorV1 {
    /// The destination was refused before a temporary existed.
    #[error("cannot publish artifact to {destination}: {reason:?}")]
    RejectedDestination {
        /// Requested destination.
        destination: PathBuf,
        /// Typed reason for refusal.
        reason: ArtifactDestinationRejectionV1,
    },
    /// A pre-replacement I/O failure occurred and temporary cleanup succeeded.
    #[error(
        "could not publish artifact to {destination} before replacement during {phase:?}: {source}"
    )]
    NotPublished {
        /// Requested destination.
        destination: PathBuf,
        /// Failed pre-replacement phase.
        phase: ArtifactPrepublicationPhaseV1,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// A pre-replacement I/O failure occurred and temporary cleanup failed.
    #[error(
        "could not publish artifact to {destination} before replacement during {phase:?}: \
         {source}; temporary cleanup failed: {cleanup}"
    )]
    NotPublishedTemporaryMayRemain {
        /// Requested destination.
        destination: PathBuf,
        /// Failed pre-replacement phase.
        phase: ArtifactPrepublicationPhaseV1,
        /// Underlying I/O failure.
        source: io::Error,
        /// Temporary cleanup failure.
        cleanup: io::Error,
    },
    /// A typed final refusal occurred after a temporary existed and cleanup failed.
    #[error(
        "destination {destination} became invalid before replacement: {reason:?}; \
         temporary cleanup failed: {cleanup}"
    )]
    RejectedDestinationTemporaryMayRemain {
        /// Requested destination.
        destination: PathBuf,
        /// Typed reason for refusal.
        reason: ArtifactDestinationRejectionV1,
        /// Temporary cleanup failure.
        cleanup: io::Error,
    },
    /// Random temporary-name generation failed.
    #[error("could not generate a temporary sibling for {destination}: {source}")]
    TemporaryName {
        /// Requested destination.
        destination: PathBuf,
        /// Random-source failure.
        #[source]
        source: getrandom::Error,
    },
    /// Bounded same-directory temporary-name attempts all collided.
    #[error("could not reserve a unique temporary sibling for {destination}")]
    TemporaryNameExhausted {
        /// Requested destination.
        destination: PathBuf,
    },
    /// Rename may have completed, but directory synchronization failed.
    #[error(
        "artifact replacement may have completed at {receipt:?}, \
         but directory synchronization failed: {source}"
    )]
    PossiblyPublished {
        /// Receipt for the completed rename.
        receipt: ArtifactPublicationReceiptV1,
        /// Directory synchronization failure.
        #[source]
        source: io::Error,
    },
}

/// Publish completed artifact bytes to one concrete destination.
pub fn publish_artifact_v1(
    request: ArtifactPublicationRequestV1,
) -> Result<ArtifactPublicationOutcomeV1, ArtifactPublicationErrorV1> {
    publish_artifact_impl(request, |_| {}, directory_sync)
}

fn publish_artifact_impl<H, S>(
    request: ArtifactPublicationRequestV1,
    mut phase_hook: H,
    mut synchronize_directory: S,
) -> Result<ArtifactPublicationOutcomeV1, ArtifactPublicationErrorV1>
where
    H: FnMut(ArtifactPrepublicationPhaseV1),
    S: FnMut(&OwnedFd) -> io::Result<ArtifactPublicationDurabilityV1>,
{
    let ArtifactPublicationRequestV1 {
        destination,
        bytes,
        retained_source,
    } = request;
    let destination_name = destination.file_name().ok_or_else(|| {
        rejected(
            &destination,
            ArtifactDestinationRejectionV1::MissingFileName,
        )
    })?;
    phase_hook(ArtifactPrepublicationPhaseV1::OpenParent);
    let directory = open_trusted_parent(&destination)?;
    phase_hook(ArtifactPrepublicationPhaseV1::ValidateBeforeTemporary);
    validate_destination(
        &directory,
        destination_name,
        retained_source.as_ref(),
        &destination,
    )
    .map_err(|failure| {
        validation_error(
            &destination,
            ArtifactPrepublicationPhaseV1::ValidateBeforeTemporary,
            failure,
        )
    })?;

    phase_hook(ArtifactPrepublicationPhaseV1::ReserveTemporary);
    let (temporary_name, temporary_fd) = reserve_temporary(&directory, &destination)?;
    phase_hook(ArtifactPrepublicationPhaseV1::ValidateTemporary);
    if let Err(source) = validate_temporary(&temporary_fd) {
        return Err(cleanup_io_error(
            &destination,
            ArtifactPrepublicationPhaseV1::ValidateTemporary,
            source,
            &directory,
            &temporary_name,
        ));
    }
    let mut temporary = File::from(temporary_fd);
    phase_hook(ArtifactPrepublicationPhaseV1::WriteOrSyncTemporary);
    if let Err(source) = temporary
        .write_all(&bytes)
        .and_then(|_| temporary.sync_all())
    {
        return Err(cleanup_io_error(
            &destination,
            ArtifactPrepublicationPhaseV1::WriteOrSyncTemporary,
            source,
            &directory,
            &temporary_name,
        ));
    }
    drop(temporary);

    phase_hook(ArtifactPrepublicationPhaseV1::ValidateBeforeRename);
    match validate_destination(
        &directory,
        destination_name,
        retained_source.as_ref(),
        &destination,
    ) {
        Ok(()) => {}
        Err(ValidationFailure::Rejected(reason)) => {
            return match remove_temporary(&directory, &temporary_name) {
                Ok(()) => Err(rejected(&destination, reason)),
                Err(cleanup) => Err(
                    ArtifactPublicationErrorV1::RejectedDestinationTemporaryMayRemain {
                        destination,
                        reason,
                        cleanup,
                    },
                ),
            };
        }
        Err(ValidationFailure::Io(source)) => {
            return Err(cleanup_io_error(
                &destination,
                ArtifactPrepublicationPhaseV1::ValidateBeforeRename,
                source,
                &directory,
                &temporary_name,
            ));
        }
    }

    phase_hook(ArtifactPrepublicationPhaseV1::Rename);
    if let Err(source) = renameat(&directory, &temporary_name, &directory, destination_name) {
        return Err(cleanup_io_error(
            &destination,
            ArtifactPrepublicationPhaseV1::Rename,
            source.into(),
            &directory,
            &temporary_name,
        ));
    }
    let receipt = ArtifactPublicationReceiptV1 {
        destination,
        retained_source: retained_source
            .as_ref()
            .map(RetainedSourceFileGuardV1::identity),
    };
    match synchronize_directory(&directory) {
        Ok(ArtifactPublicationDurabilityV1::Confirmed) => {
            Ok(ArtifactPublicationOutcomeV1::ConfirmedDurable(receipt))
        }
        Ok(ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed) => Ok(
            ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(receipt),
        ),
        Err(source) => Err(ArtifactPublicationErrorV1::PossiblyPublished { receipt, source }),
    }
}

#[derive(Debug)]
enum ValidationFailure {
    Rejected(ArtifactDestinationRejectionV1),
    Io(io::Error),
}

fn validation_error(
    destination: &Path,
    phase: ArtifactPrepublicationPhaseV1,
    failure: ValidationFailure,
) -> ArtifactPublicationErrorV1 {
    match failure {
        ValidationFailure::Rejected(reason) => rejected(destination, reason),
        ValidationFailure::Io(source) => ArtifactPublicationErrorV1::NotPublished {
            destination: destination.to_path_buf(),
            phase,
            source,
        },
    }
}

fn rejected(
    destination: &Path,
    reason: ArtifactDestinationRejectionV1,
) -> ArtifactPublicationErrorV1 {
    ArtifactPublicationErrorV1::RejectedDestination {
        destination: destination.to_path_buf(),
        reason,
    }
}

fn open_trusted_parent(destination: &Path) -> Result<OwnedFd, ArtifactPublicationErrorV1> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = openat(
        CWD,
        if parent.is_absolute() {
            Path::new("/")
        } else {
            Path::new(".")
        },
        flags,
        Mode::empty(),
    )
    .map_err(|error| parent_error(destination, error))?;
    for component in parent.components() {
        let component = match component {
            Component::Normal(value) => value,
            Component::ParentDir => OsStr::new(".."),
            Component::RootDir | Component::CurDir | Component::Prefix(_) => continue,
        };
        directory = openat(&directory, component, flags, Mode::empty())
            .map_err(|error| parent_error(destination, error))?;
    }
    Ok(directory)
}

fn parent_error(destination: &Path, error: Errno) -> ArtifactPublicationErrorV1 {
    match error {
        Errno::LOOP => rejected(
            destination,
            ArtifactDestinationRejectionV1::ParentTraversesSymlink,
        ),
        Errno::NOTDIR => rejected(
            destination,
            ArtifactDestinationRejectionV1::ParentIsNotDirectory,
        ),
        _ => ArtifactPublicationErrorV1::NotPublished {
            destination: destination.to_path_buf(),
            phase: ArtifactPrepublicationPhaseV1::OpenParent,
            source: error.into(),
        },
    }
}

fn validate_destination(
    directory: &OwnedFd,
    name: &OsStr,
    retained_source: Option<&RetainedSourceFileGuardV1>,
    _destination: &Path,
) -> Result<(), ValidationFailure> {
    if let Some(source) = retained_source {
        source
            .verify_live_identity()
            .map_err(ValidationFailure::Io)?;
    }
    let metadata = match statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(metadata) => metadata,
        Err(Errno::NOENT) => return Ok(()),
        Err(source) => return Err(ValidationFailure::Io(source.into())),
    };
    let file_type = FileType::from_raw_mode(metadata.st_mode);
    if file_type.is_symlink() {
        return Err(ValidationFailure::Rejected(
            ArtifactDestinationRejectionV1::FinalIsSymlink,
        ));
    }
    if !file_type.is_file() {
        return Err(ValidationFailure::Rejected(
            ArtifactDestinationRejectionV1::FinalIsNotRegular,
        ));
    }
    if let Some(source) = retained_source
        && identity_from_stat(metadata) == source.identity()
    {
        return Err(ValidationFailure::Rejected(
            ArtifactDestinationRejectionV1::SourceAliasesDestination,
        ));
    }
    Ok(())
}

fn reserve_temporary(
    directory: &OwnedFd,
    destination: &Path,
) -> Result<(OsString, OwnedFd), ArtifactPublicationErrorV1> {
    for _ in 0..TEMPORARY_ATTEMPTS {
        let temporary_name = temporary_sibling(destination)?;
        match openat(
            directory,
            &temporary_name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        ) {
            Ok(file) => return Ok((temporary_name, file)),
            Err(Errno::EXIST) => continue,
            Err(source) => {
                return Err(ArtifactPublicationErrorV1::NotPublished {
                    destination: destination.to_path_buf(),
                    phase: ArtifactPrepublicationPhaseV1::ReserveTemporary,
                    source: source.into(),
                });
            }
        }
    }
    Err(ArtifactPublicationErrorV1::TemporaryNameExhausted {
        destination: destination.to_path_buf(),
    })
}

fn temporary_sibling(destination: &Path) -> Result<OsString, ArtifactPublicationErrorV1> {
    let mut random = [0_u8; 16];
    getrandom::fill(&mut random).map_err(|source| ArtifactPublicationErrorV1::TemporaryName {
        destination: destination.to_path_buf(),
        source,
    })?;
    let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
    Ok(format!(
        ".ferrum-{random:032x}-{sequence:016x}.tmp",
        random = u128::from_le_bytes(random)
    )
    .into())
}

fn validate_temporary(temporary: &OwnedFd) -> Result<(), io::Error> {
    let metadata = fstat(temporary).map_err(io::Error::from)?;
    if FileType::from_raw_mode(metadata.st_mode).is_file() {
        Ok(())
    } else {
        Err(io::Error::other("reserved temporary is not a regular file"))
    }
}

fn cleanup_io_error(
    destination: &Path,
    phase: ArtifactPrepublicationPhaseV1,
    source: io::Error,
    directory: &OwnedFd,
    temporary_name: &OsStr,
) -> ArtifactPublicationErrorV1 {
    match remove_temporary(directory, temporary_name) {
        Ok(()) => ArtifactPublicationErrorV1::NotPublished {
            destination: destination.to_path_buf(),
            phase,
            source,
        },
        Err(cleanup) => ArtifactPublicationErrorV1::NotPublishedTemporaryMayRemain {
            destination: destination.to_path_buf(),
            phase,
            source,
            cleanup,
        },
    }
}

fn remove_temporary(directory: &OwnedFd, temporary_name: &OsStr) -> io::Result<()> {
    unlinkat(directory, temporary_name, AtFlags::empty()).map_err(io::Error::from)
}

fn directory_sync(directory: &OwnedFd) -> io::Result<ArtifactPublicationDurabilityV1> {
    match fsync(directory) {
        Ok(()) => Ok(ArtifactPublicationDurabilityV1::Confirmed),
        #[cfg(target_os = "macos")]
        Err(Errno::INVAL) => Ok(ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed),
        Err(error) => Err(error.into()),
    }
}

fn identity_from_stat(metadata: rustix::fs::Stat) -> RetainedRegularFileIdentityV1 {
    RetainedRegularFileIdentityV1 {
        device: metadata.st_dev as u64,
        inode: metadata.st_ino,
    }
}

#[cfg(test)]
pub(crate) fn publish_artifact_with_test_seams_v1<H, S>(
    request: ArtifactPublicationRequestV1,
    phase_hook: H,
    directory_sync: S,
) -> Result<ArtifactPublicationOutcomeV1, ArtifactPublicationErrorV1>
where
    H: FnMut(ArtifactPrepublicationPhaseV1),
    S: FnMut(&OwnedFd) -> io::Result<ArtifactPublicationDurabilityV1>,
{
    publish_artifact_impl(request, phase_hook, directory_sync)
}
