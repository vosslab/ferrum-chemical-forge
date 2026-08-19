//! Document chemistry admission for untrusted SDF text.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use ferrum_chemistry::SDF_MAX_INPUT_BYTES;
use thiserror::Error;

/// Read one local SDF using the exact ABI-4 operation byte ceiling.
///
/// Symbolic links are allowed for this read-only import route. Security depends on
/// the opened handle being a regular file. At most the operation limit plus one
/// sentinel byte is retained before native parsing begins.
pub fn read_sdf_file_v1(path: &Path) -> Result<String, SdfSourceErrorV1> {
    let file = File::open(path).map_err(|source| SdfSourceErrorV1::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let metadata = file.metadata().map_err(|source| SdfSourceErrorV1::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() {
        return Err(SdfSourceErrorV1::NonRegularFile {
            path: path.to_path_buf(),
        });
    }
    read_bounded_utf8(file, path, SDF_MAX_INPUT_BYTES)
}

fn read_bounded_utf8(
    reader: impl Read,
    path: &Path,
    maximum_bytes: usize,
) -> Result<String, SdfSourceErrorV1> {
    let sentinel_limit = u64::try_from(maximum_bytes)
        .map_err(|_| SdfSourceErrorV1::LimitUnrepresentable { maximum_bytes })?
        .checked_add(1)
        .ok_or(SdfSourceErrorV1::LimitUnrepresentable { maximum_bytes })?;
    let mut bytes = Vec::new();
    reader
        .take(sentinel_limit)
        .read_to_end(&mut bytes)
        .map_err(|source| SdfSourceErrorV1::Read {
            path: path.to_path_buf(),
            source,
        })?;
    if bytes.len() > maximum_bytes {
        return Err(SdfSourceErrorV1::ByteLimitExceeded {
            path: path.to_path_buf(),
            limit: maximum_bytes,
            observed_at_least: bytes.len(),
        });
    }
    String::from_utf8(bytes).map_err(|source| SdfSourceErrorV1::Utf8 {
        path: path.to_path_buf(),
        source,
    })
}

/// Failure while admitting one local SDF before native parsing.
#[derive(Debug, Error)]
pub enum SdfSourceErrorV1 {
    /// The file could not be opened, inspected, or read.
    #[error("could not read SDF file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The opened handle is not a regular file.
    #[error("SDF input is not a regular file: {path}")]
    NonRegularFile { path: PathBuf },
    /// The ABI operation limit cannot be represented with one sentinel byte.
    #[error("SDF byte limit {maximum_bytes} cannot represent a sentinel byte")]
    LimitUnrepresentable { maximum_bytes: usize },
    /// The file exceeded the exact native operation byte ceiling.
    #[error("SDF file {path} exceeds {limit} bytes (observed at least {observed_at_least})")]
    ByteLimitExceeded {
        path: PathBuf,
        limit: usize,
        observed_at_least: usize,
    },
    /// The admitted bytes are not UTF-8 SDF text.
    #[error("SDF file is not UTF-8: {path}")]
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
    fn bounded_reader_reports_only_the_first_byte_beyond_the_limit() {
        let error = read_bounded_utf8(Cursor::new(b"abcd"), Path::new("sample.sdf"), 3)
            .expect_err("four bytes must exceed a three-byte test limit");
        assert!(matches!(
            error,
            SdfSourceErrorV1::ByteLimitExceeded {
                limit: 3,
                observed_at_least: 4,
                ..
            }
        ));
    }

    #[test]
    fn bounded_reader_rejects_non_utf8_before_native_parsing() {
        let error = read_bounded_utf8(Cursor::new([0xff]), Path::new("sample.sdf"), 4)
            .expect_err("invalid UTF-8 must fail at admission");
        assert!(matches!(error, SdfSourceErrorV1::Utf8 { .. }));
    }
}
