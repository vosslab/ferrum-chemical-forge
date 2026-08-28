//! Descriptor-relative local-template admission.  Names are facts only; bytes
//! are read once from the accepted descriptor (ASVS 5.1.1, 5.2.1-5.2.2).
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::Read;
use std::os::fd::OwnedFd;
use std::path::{Component, Path};

use rustix::fs::{CWD, Dir, FileType, Mode, OFlags, fstat, openat};
use rustix::io::Errno;
use sha2::{Digest, Sha256};

use crate::TemplateCatalogErrorV1;
use crate::error::{TemplateCatalogRefusalCategoryV1 as Category, TemplateCatalogRefusalV1};

pub(crate) struct AdmittedUserFile {
    pub(crate) basename: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) digest: String,
}

pub(crate) fn scan_user_directory(
    directory: &Path,
    max_file: usize,
    max_entries: usize,
    max_candidates: usize,
    max_total: usize,
    max_refusals: usize,
) -> Result<(Vec<AdmittedUserFile>, Vec<TemplateCatalogRefusalV1>), TemplateCatalogErrorV1> {
    let mut refusals = Vec::new();
    let Some(fd) = open_directory(directory, &mut refusals)? else {
        return Ok((Vec::new(), refusals));
    };
    let mut stream = Dir::read_from(&fd).map_err(|error| {
        TemplateCatalogErrorV1::DirectoryOpen(std::io::Error::from_raw_os_error(
            error.raw_os_error(),
        ))
    })?;
    let mut names = BTreeSet::new();
    let mut candidate_count = 0u64;
    let mut suppressed_refusals = 0u64;
    while let Some(entry) = stream.read() {
        let entry = entry.map_err(|error| {
            TemplateCatalogErrorV1::DirectoryOpen(std::io::Error::from_raw_os_error(
                error.raw_os_error(),
            ))
        })?;
        let name = entry.file_name();
        if name.to_bytes() == b"." || name.to_bytes() == b".." {
            continue;
        }
        let Ok(name) = name.to_str().map(str::to_owned) else {
            if refusals.len() < max_refusals.saturating_sub(1) {
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::FilenameNonUtf8,
                    None,
                ));
            } else {
                suppressed_refusals = suppressed_refusals.saturating_add(1);
            }
            continue;
        };
        if !name.ends_with(".cdml") {
            continue;
        }
        candidate_count = candidate_count.saturating_add(1);
        if max_candidates > 0 && names.len() < max_candidates {
            names.insert(name);
        } else if max_candidates > 0 && names.last().is_some_and(|largest| name < *largest) {
            names.pop_last();
            names.insert(name);
        }
    }
    let candidate_overflow = candidate_count
        .saturating_sub(names.len() as u64)
        .saturating_add(suppressed_refusals);
    let mut files = Vec::new();
    files
        .try_reserve(max_entries)
        .map_err(|_| TemplateCatalogErrorV1::Allocation)?;
    let mut total = 0usize;
    for name in names {
        if files.len() >= max_entries {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::CatalogLimitExceeded,
                Some(name),
            ));
            continue;
        }
        let flags = OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let opened = match openat(&fd, OsStr::new(&name), flags, Mode::empty()) {
            Ok(value) => value,
            Err(Errno::LOOP) => {
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::CandidateSymlink,
                    Some(name),
                ));
                continue;
            }
            Err(_) => {
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::CandidateOpenFailed,
                    Some(name),
                ));
                continue;
            }
        };
        let stat = match fstat(&opened) {
            Ok(value) => value,
            Err(_) => {
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::CandidateOpenFailed,
                    Some(name),
                ));
                continue;
            }
        };
        if !FileType::from_raw_mode(stat.st_mode).is_file() {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::CandidateNotRegular,
                Some(name),
            ));
            continue;
        }
        if stat.st_size > max_file as _ {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::FileTooLarge,
                Some(name),
            ));
            continue;
        }
        let mut bytes = Vec::new();
        let reserve = usize::try_from(stat.st_size)
            .unwrap_or(max_file)
            .min(max_file)
            .saturating_add(1);
        if bytes.try_reserve(reserve).is_err() {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::CatalogLimitExceeded,
                Some(name),
            ));
            continue;
        }
        let mut file = std::fs::File::from(opened);
        let result = file
            .by_ref()
            .take((max_file as u64).saturating_add(1))
            .read_to_end(&mut bytes);
        if result.is_err() {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::CandidateReadFailed,
                Some(name),
            ));
            continue;
        }
        if bytes.len() > max_file {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::FileTooLarge,
                Some(name),
            ));
            continue;
        }
        if total.saturating_add(bytes.len()) > max_total {
            refusals.push(TemplateCatalogRefusalV1::new(
                Category::CatalogLimitExceeded,
                Some(name),
            ));
            continue;
        }
        total += bytes.len();
        let digest = hex_digest(Sha256::digest(&bytes).as_slice());
        files.push(AdmittedUserFile {
            basename: name,
            bytes,
            digest,
        });
    }
    Ok((
        files,
        bound_refusals(refusals, max_refusals, candidate_overflow),
    ))
}

fn bound_refusals(
    mut refusals: Vec<TemplateCatalogRefusalV1>,
    max_refusals: usize,
    candidate_overflow: u64,
) -> Vec<TemplateCatalogRefusalV1> {
    let detailed_maximum = max_refusals.saturating_sub(1);
    let mut suppressed = candidate_overflow;
    if refusals.len() > detailed_maximum {
        for refusal in refusals.drain(detailed_maximum..) {
            suppressed = suppressed.saturating_add(refusal.occurrences());
        }
    }
    if suppressed > 0 && max_refusals > 0 {
        refusals.push(TemplateCatalogRefusalV1::aggregate_limit_exceeded(
            suppressed,
        ));
    }
    refusals
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(HEX[(byte >> 4) as usize] as char);
        value.push(HEX[(byte & 15) as usize] as char);
    }
    value
}

fn open_directory(
    path: &Path,
    refusals: &mut Vec<TemplateCatalogRefusalV1>,
) -> Result<Option<OwnedFd>, TemplateCatalogErrorV1> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = match openat(
        CWD,
        if path.is_absolute() {
            Path::new("/")
        } else {
            Path::new(".")
        },
        flags,
        Mode::empty(),
    ) {
        Ok(value) => value,
        Err(error) => {
            return Err(TemplateCatalogErrorV1::DirectoryOpen(
                std::io::Error::from_raw_os_error(error.raw_os_error()),
            ));
        }
    };
    let components: Vec<_> = path.components().collect();
    for (index, component) in components.iter().enumerate() {
        let value = match component {
            Component::Normal(value) => value,
            Component::ParentDir => OsStr::new(".."),
            Component::RootDir | Component::CurDir | Component::Prefix(_) => continue,
        };
        directory = match openat(&directory, value, flags, Mode::empty()) {
            Ok(value) => value,
            Err(Errno::NOENT) if index + 1 == components.len() => return Ok(None),
            Err(Errno::NOENT) => {
                return Err(TemplateCatalogErrorV1::DirectoryOpen(
                    std::io::Error::from_raw_os_error(Errno::NOENT.raw_os_error()),
                ));
            }
            Err(Errno::LOOP) => {
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::DirectorySymlink,
                    None,
                ));
                return Ok(None);
            }
            Err(Errno::NOTDIR) => {
                refusals.push(TemplateCatalogRefusalV1::new(
                    Category::DirectoryNotDirectory,
                    None,
                ));
                return Ok(None);
            }
            Err(error) => {
                return Err(TemplateCatalogErrorV1::DirectoryOpen(
                    std::io::Error::from_raw_os_error(error.raw_os_error()),
                ));
            }
        };
    }
    Ok(Some(directory))
}
