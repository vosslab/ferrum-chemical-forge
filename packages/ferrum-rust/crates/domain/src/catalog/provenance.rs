use std::fmt;

/// Immutable origin information for a catalog payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogProvenance {
    source_location: &'static str,
    revision: &'static str,
    license: &'static str,
    import_record: &'static str,
    sha256: &'static str,
}

impl CatalogProvenance {
    pub const fn new(
        source_location: &'static str,
        revision: &'static str,
        license: &'static str,
        import_record: &'static str,
        sha256: &'static str,
    ) -> Self {
        Self {
            source_location,
            revision,
            license,
            import_record,
            sha256,
        }
    }

    pub const fn source_location(&self) -> &'static str {
        self.source_location
    }

    pub const fn revision(&self) -> &'static str {
        self.revision
    }

    pub const fn license(&self) -> &'static str {
        self.license
    }

    pub const fn import_record(&self) -> &'static str {
        self.import_record
    }

    pub const fn sha256(&self) -> &'static str {
        self.sha256
    }

    fn validate_hash_shape(&self) -> Result<(), CatalogError> {
        if self.sha256.len() == 64 && self.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            Ok(())
        } else {
            Err(CatalogError::InvalidSha256 {
                source_location: self.source_location,
            })
        }
    }
}

/// A validated catalog payload and its immutable provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedCatalog<T> {
    entries: Box<[T]>,
    provenance: CatalogProvenance,
}

impl<T> VerifiedCatalog<T> {
    pub fn verify(
        source: &'static str,
        provenance: CatalogProvenance,
        entries: impl Into<Box<[T]>>,
    ) -> Result<Self, CatalogError> {
        provenance.validate_hash_shape()?;
        let actual = sha256_hex(source.as_bytes());
        if actual != provenance.sha256 {
            return Err(CatalogError::ChecksumMismatch {
                source_location: provenance.source_location,
                expected: provenance.sha256,
                actual,
            });
        }
        Ok(Self {
            entries: entries.into(),
            provenance,
        })
    }

    pub fn entries(&self) -> &[T] {
        &self.entries
    }

    pub const fn provenance(&self) -> &CatalogProvenance {
        &self.provenance
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogError {
    InvalidSha256 {
        source_location: &'static str,
    },
    ChecksumMismatch {
        source_location: &'static str,
        expected: &'static str,
        actual: String,
    },
    MalformedRecord {
        catalog: &'static str,
        line: usize,
    },
    InvalidField {
        catalog: &'static str,
        line: usize,
        field: &'static str,
    },
    DuplicateIdentifier {
        catalog: &'static str,
        identifier: String,
    },
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSha256 { source_location } => {
                write!(formatter, "invalid SHA-256 metadata for {source_location}")
            }
            Self::ChecksumMismatch {
                source_location,
                expected,
                actual,
            } => write!(
                formatter,
                "checksum mismatch for {source_location}: expected {expected}, got {actual}"
            ),
            Self::MalformedRecord { catalog, line } => {
                write!(
                    formatter,
                    "malformed {catalog} catalog record at line {line}"
                )
            }
            Self::InvalidField {
                catalog,
                line,
                field,
            } => write!(
                formatter,
                "invalid {field} in {catalog} catalog at line {line}"
            ),
            Self::DuplicateIdentifier {
                catalog,
                identifier,
            } => {
                write!(
                    formatter,
                    "duplicate {catalog} catalog identifier: {identifier}"
                )
            }
        }
    }
}

impl std::error::Error for CatalogError {}

pub(crate) fn records(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source.lines().enumerate().filter_map(|(index, line)| {
        let trimmed = line.trim();
        (!trimmed.is_empty() && !trimmed.starts_with('#')).then_some((index + 1, trimmed))
    })
}

pub(crate) fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// Minimal, dependency-free SHA-256 for catalog payload integrity checks.
fn sha256_hex(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut padded = input.to_vec();
    padded.push(0x80);
    while !(padded.len() + 8).is_multiple_of(64) {
        padded.push(0);
    }
    padded.extend_from_slice(&((input.len() as u64) * 8).to_be_bytes());
    let mut hash = INITIAL;
    for chunk in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().unwrap());
        }
        for index in 16..64 {
            let sigma0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let sigma1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(sigma0)
                .wrapping_add(words[index - 7])
                .wrapping_add(sigma1);
        }
        let mut state = hash;
        for (index, constant) in K.iter().enumerate() {
            let sigma1 =
                state[4].rotate_right(6) ^ state[4].rotate_right(11) ^ state[4].rotate_right(25);
            let choice = (state[4] & state[5]) ^ (!state[4] & state[6]);
            let temporary1 = state[7]
                .wrapping_add(sigma1)
                .wrapping_add(choice)
                .wrapping_add(*constant)
                .wrapping_add(words[index]);
            let sigma0 =
                state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22);
            let majority = (state[0] & state[1]) ^ (state[0] & state[2]) ^ (state[1] & state[2]);
            let temporary2 = sigma0.wrapping_add(majority);
            state[7] = state[6];
            state[6] = state[5];
            state[5] = state[4];
            state[4] = state[3].wrapping_add(temporary1);
            state[3] = state[2];
            state[2] = state[1];
            state[1] = state[0];
            state[0] = temporary1.wrapping_add(temporary2);
        }
        for (destination, value) in hash.iter_mut().zip(state) {
            *destination = destination.wrapping_add(value);
        }
    }
    hash.iter().map(|word| format!("{word:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{CatalogError, CatalogProvenance, VerifiedCatalog, sha256_hex};

    #[test]
    fn sha256_matches_standard_empty_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn verified_catalog_rejects_modified_payload() {
        let provenance = CatalogProvenance::new(
            "test",
            "r1",
            "CC0-1.0",
            "test fixture",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        assert!(matches!(
            VerifiedCatalog::<()>::verify("changed", provenance, []),
            Err(CatalogError::ChecksumMismatch { .. })
        ));
    }
}
