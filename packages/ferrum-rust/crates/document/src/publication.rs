//! Descriptor-relative atomic publication for authoritative CDML snapshots.
//!
//! The publisher accepts a caller-owned path only long enough to acquire its parent
//! directory. Every mutation after that point uses the retained directory descriptor.
//! This Unix implementation requires a concrete directory at every parent component:
//! callers that spell a path through a symbolic-link parent must provide its physical
//! directory spelling instead. That deliberate contract keeps the trusted descriptor
//! chain auditable on macOS and Linux.

use std::fs::File;
use std::io::{self, Write};
use std::os::fd::OwnedFd;
use std::path::{Component, Path};
use std::sync::atomic::{AtomicU64, Ordering};

use rustix::fs::{AtFlags, CWD, FileType, Mode, OFlags, fsync, openat, renameat, statat, unlinkat};
use rustix::io::Errno;

use super::session::DocumentSessionError;

const TEMPORARY_ATTEMPTS: u8 = 16;
static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

/// Whether the directory entry replacement received platform-supported confirmation.
///
/// Data is synchronized before replacement. Directory confirmation is attempted through
/// the same retained descriptor; platforms that decline it report the completed
/// replacement without overstating directory-entry durability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationDurability {
    /// Both data and the containing directory entry were synchronized.
    Confirmed,
    /// The replacement succeeded, but directory-entry durability is not confirmed.
    DirectoryEntryUnconfirmed,
}

pub(crate) fn publish_snapshot(
    path: &Path,
    cdml: &str,
) -> Result<PublicationDurability, DocumentSessionError> {
    publish_snapshot_with_after_parent_open(path, cdml, || {})
}

#[cfg(test)]
pub(crate) fn publish_snapshot_with_after_parent_open<H>(
    path: &Path,
    cdml: &str,
    after_parent_open: H,
) -> Result<PublicationDurability, DocumentSessionError>
where
    H: FnOnce(),
{
    publish_snapshot_after_parent_open(path, cdml, after_parent_open)
}

#[cfg(not(test))]
fn publish_snapshot_with_after_parent_open<H>(
    path: &Path,
    cdml: &str,
    after_parent_open: H,
) -> Result<PublicationDurability, DocumentSessionError>
where
    H: FnOnce(),
{
    publish_snapshot_after_parent_open(path, cdml, after_parent_open)
}

fn publish_snapshot_after_parent_open<H>(
    path: &Path,
    cdml: &str,
    after_parent_open: H,
) -> Result<PublicationDurability, DocumentSessionError>
where
    H: FnOnce(),
{
    validate_destination_name(path)?;
    let directory = open_trusted_parent(path)?;
    after_parent_open();
    validate_destination_in(&directory, path)?;

    let (temporary_name, temporary_fd) = reserve_temporary_in(&directory, path)?;
    let mut file = File::from(temporary_fd);
    if let Err(source) = file
        .write_all(cdml.as_bytes())
        .and_then(|_| file.sync_all())
    {
        return Err(before_replacement_error(
            path,
            source,
            &directory,
            &temporary_name,
        ));
    }
    drop(file);

    // Recheck through the descriptor immediately before replacement. `renameat`
    // replaces the entry rather than following it, so neither a final-link swap nor
    // renaming the visible parent path can redirect the data.
    if let Err(error) = validate_destination_in(&directory, path) {
        return match remove_temporary_in(&directory, &temporary_name) {
            Ok(()) => Err(error),
            Err(cleanup) => Err(DocumentSessionError::ReplacementRejectedWithCleanup {
                path: path.to_path_buf(),
                reason: error.to_string(),
                cleanup,
            }),
        };
    }
    let destination_name = destination_name(path)?;
    if let Err(source) =
        renameat(&directory, &temporary_name, &directory, destination_name).map_err(io::Error::from)
    {
        return Err(before_replacement_error(
            path,
            source,
            &directory,
            &temporary_name,
        ));
    }
    confirm_publication(&directory).map_err(|source| {
        DocumentSessionError::PublishPossiblyCompleted {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn validate_destination_name(path: &Path) -> Result<(), DocumentSessionError> {
    if path.file_name().is_none() {
        return Err(DocumentSessionError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination must name a file",
        });
    }
    Ok(())
}

fn destination_name(path: &Path) -> Result<&std::ffi::OsStr, DocumentSessionError> {
    path.file_name()
        .ok_or_else(|| DocumentSessionError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination must name a file",
        })
}

fn open_trusted_parent(path: &Path) -> Result<OwnedFd, DocumentSessionError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let directory_flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = openat(
        CWD,
        if parent.is_absolute() {
            Path::new("/")
        } else {
            Path::new(".")
        },
        directory_flags,
        Mode::empty(),
    )
    .map_err(|error| parent_open_error(path, error))?;

    for component in parent.components() {
        let component = match component {
            Component::Normal(component) => component,
            Component::ParentDir => std::ffi::OsStr::new(".."),
            Component::RootDir | Component::CurDir | Component::Prefix(_) => continue,
        };
        directory = openat(&directory, component, directory_flags, Mode::empty())
            .map_err(|error| parent_open_error(path, error))?;
    }
    Ok(directory)
}

fn parent_open_error(path: &Path, error: Errno) -> DocumentSessionError {
    match error {
        Errno::LOOP => DocumentSessionError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination parent must not traverse a symbolic link",
        },
        Errno::NOTDIR => DocumentSessionError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination parent must be a directory",
        },
        _ => DocumentSessionError::PublishNotStarted {
            path: path.to_path_buf(),
            source: error.into(),
        },
    }
}

fn validate_destination_in(directory: &OwnedFd, path: &Path) -> Result<(), DocumentSessionError> {
    let destination_name = destination_name(path)?;
    match statat(directory, destination_name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(metadata) if FileType::from_raw_mode(metadata.st_mode).is_file() => Ok(()),
        Ok(metadata) if FileType::from_raw_mode(metadata.st_mode).is_symlink() => {
            Err(DocumentSessionError::InvalidDestination {
                path: path.to_path_buf(),
                reason: "destination must not be a symbolic link",
            })
        }
        Ok(_) => Err(DocumentSessionError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination exists but is not a regular file",
        }),
        Err(Errno::NOENT) => Ok(()),
        Err(source) => Err(DocumentSessionError::PublishNotStarted {
            path: path.to_path_buf(),
            source: source.into(),
        }),
    }
}

fn reserve_temporary_in(
    directory: &OwnedFd,
    path: &Path,
) -> Result<(std::ffi::OsString, OwnedFd), DocumentSessionError> {
    let file_name = destination_name(path)?;
    for _ in 0..TEMPORARY_ATTEMPTS {
        let temporary_name = temporary_sibling(file_name, path)?;
        match openat(
            directory,
            &temporary_name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        ) {
            Ok(file) => return Ok((temporary_name, file)),
            Err(Errno::EXIST) => continue,
            Err(source) => {
                return Err(DocumentSessionError::PublishNotStarted {
                    path: path.to_path_buf(),
                    source: source.into(),
                });
            }
        }
    }
    Err(DocumentSessionError::TemporaryNameExhausted {
        path: path.to_path_buf(),
    })
}

fn temporary_sibling(
    file_name: &std::ffi::OsStr,
    destination: &Path,
) -> Result<std::ffi::OsString, DocumentSessionError> {
    let mut random = [0_u8; 16];
    getrandom::fill(&mut random).map_err(|source| DocumentSessionError::TemporaryName {
        path: destination.to_path_buf(),
        detail: source.to_string(),
    })?;
    let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
    let random = u128::from_le_bytes(random);
    Ok(format!(
        ".{}.ferrum-{random:032x}-{sequence:016x}.tmp",
        file_name.to_string_lossy()
    )
    .into())
}

fn remove_temporary_in(directory: &OwnedFd, temporary_name: &std::ffi::OsStr) -> io::Result<()> {
    unlinkat(directory, temporary_name, AtFlags::empty()).map_err(io::Error::from)
}

fn before_replacement_error(
    destination: &Path,
    source: io::Error,
    directory: &OwnedFd,
    temporary_name: &std::ffi::OsStr,
) -> DocumentSessionError {
    match remove_temporary_in(directory, temporary_name) {
        Ok(()) => DocumentSessionError::PublishNotStarted {
            path: destination.to_path_buf(),
            source,
        },
        Err(cleanup) => DocumentSessionError::PublishNotStartedWithCleanup {
            path: destination.to_path_buf(),
            source,
            cleanup,
        },
    }
}

fn confirm_publication(directory: &OwnedFd) -> io::Result<PublicationDurability> {
    match fsync(directory) {
        Ok(()) => Ok(PublicationDurability::Confirmed),
        #[cfg(target_os = "macos")]
        Err(Errno::INVAL) => Ok(PublicationDurability::DirectoryEntryUnconfirmed),
        Err(error) => Err(error.into()),
    }
}
