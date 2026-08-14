//! Bounded local-file admission for untrusted molblock text.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use ferrum_chemistry::MOLBLOCK_MAX_INPUT_BYTES;
use thiserror::Error;

/// Read one local molblock using the exact ABI-4 operation byte ceiling.
///
/// Symbolic links are allowed for this read-only import route. Security depends on
/// the opened handle being a regular file, not on a race-prone pre-open path check.
/// The reader consumes at most the operation limit plus one sentinel byte.
pub fn read_molblock_file_v1(path: &Path) -> Result<String, MolblockSourceErrorV1> {
    let file = File::open(path).map_err(|source| MolblockSourceErrorV1::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let metadata = file
        .metadata()
        .map_err(|source| MolblockSourceErrorV1::Read {
            path: path.to_path_buf(),
            source,
        })?;
    if !metadata.is_file() {
        return Err(MolblockSourceErrorV1::NonRegularFile {
            path: path.to_path_buf(),
        });
    }
    read_bounded_utf8(file, path, MOLBLOCK_MAX_INPUT_BYTES)
}

fn read_bounded_utf8(
    reader: impl Read,
    path: &Path,
    maximum_bytes: usize,
) -> Result<String, MolblockSourceErrorV1> {
    let sentinel_limit = u64::try_from(maximum_bytes)
        .map_err(|_| MolblockSourceErrorV1::LimitUnrepresentable { maximum_bytes })?
        .checked_add(1)
        .ok_or(MolblockSourceErrorV1::LimitUnrepresentable { maximum_bytes })?;
    let mut bytes = Vec::new();
    reader
        .take(sentinel_limit)
        .read_to_end(&mut bytes)
        .map_err(|source| MolblockSourceErrorV1::Read {
            path: path.to_path_buf(),
            source,
        })?;
    if bytes.len() > maximum_bytes {
        return Err(MolblockSourceErrorV1::ByteLimitExceeded {
            path: path.to_path_buf(),
            limit: maximum_bytes,
            observed_at_least: bytes.len(),
        });
    }
    String::from_utf8(bytes).map_err(|source| MolblockSourceErrorV1::Utf8 {
        path: path.to_path_buf(),
        source,
    })
}

/// Failure while admitting one local molblock before native parsing.
#[derive(Debug, Error)]
pub enum MolblockSourceErrorV1 {
    /// The file could not be opened, inspected, or read.
    #[error("could not read molblock file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The opened handle is not a regular file.
    #[error("molblock input is not a regular file: {path}")]
    NonRegularFile { path: PathBuf },
    /// The ABI operation limit cannot be represented with one sentinel byte.
    #[error("molblock byte limit {maximum_bytes} cannot represent a sentinel byte")]
    LimitUnrepresentable { maximum_bytes: usize },
    /// The file exceeded the exact native operation byte ceiling.
    #[error("molblock file {path} exceeds {limit} bytes (observed at least {observed_at_least})")]
    ByteLimitExceeded {
        path: PathBuf,
        limit: usize,
        observed_at_least: usize,
    },
    /// The admitted bytes are not UTF-8 molblock text.
    #[error("molblock file is not UTF-8: {path}")]
    Utf8 {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn bounded_reader_reports_the_first_byte_beyond_its_limit() {
        let error = read_bounded_utf8(Cursor::new(b"abcd"), Path::new("sample.mol"), 3)
            .expect_err("four bytes must exceed a three-byte test limit");

        assert!(matches!(
            error,
            MolblockSourceErrorV1::ByteLimitExceeded {
                limit: 3,
                observed_at_least: 4,
                ..
            }
        ));
    }

    #[test]
    fn bounded_reader_rejects_non_utf8_before_native_parsing() {
        let error = read_bounded_utf8(Cursor::new([0xff]), Path::new("sample.mol"), 4)
            .expect_err("invalid UTF-8 must fail at admission");

        assert!(matches!(error, MolblockSourceErrorV1::Utf8 { .. }));
    }
}
