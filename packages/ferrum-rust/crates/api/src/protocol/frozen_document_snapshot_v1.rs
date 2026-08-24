//! Request-owned admission for detached document snapshot operations.
//!
//! A snapshot's caller revision is source provenance.  The detached session
//! created to evaluate its CDML always begins at revision zero.

use ferrum_document::{
    DocumentSession, SessionDocumentObservationV1, load_document_utf8_bytes_with_budget,
    local_cdml_ingress_format_v1,
};

/// Verified source provenance and an immutable observation from one request-local session.
pub(super) struct FrozenDocumentSnapshotV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    session: DocumentSession,
    observation: SessionDocumentObservationV1,
}

/// Operation-neutral failures while admitting one frozen source snapshot.
pub(super) enum FrozenDocumentSnapshotAdmissionErrorV1 {
    MalformedDigest(&'static str),
    DigestMismatch,
    DocumentAdmission(String),
    DocumentInvalid(String),
    Internal(String),
}

impl FrozenDocumentSnapshotV1 {
    /// Verify one frozen source snapshot and admit it into a detached session.
    pub(super) fn admit(
        source_cdml: &str,
        source_revision: u64,
        source_digest_hex: &str,
    ) -> Result<Self, FrozenDocumentSnapshotAdmissionErrorV1> {
        let source_digest = parse_lowercase_sha256(source_digest_hex)?;
        let session = load_document_utf8_bytes_with_budget(
            source_cdml.as_bytes(),
            local_cdml_ingress_format_v1(),
        )
        .map_err(|error| {
            FrozenDocumentSnapshotAdmissionErrorV1::DocumentAdmission(error.to_string())
        })?;
        let snapshot = session
            .snapshot()
            .map_err(|error| FrozenDocumentSnapshotAdmissionErrorV1::Internal(error.to_string()))?;
        if snapshot.digest() != &source_digest {
            return Err(FrozenDocumentSnapshotAdmissionErrorV1::DigestMismatch);
        }
        let observation = session.observe(0).map_err(|_| {
            FrozenDocumentSnapshotAdmissionErrorV1::DocumentInvalid(
                "document observation was refused".to_owned(),
            )
        })?;
        Ok(Self {
            source_revision,
            source_digest,
            session,
            observation,
        })
    }

    #[must_use]
    pub(super) fn observation(&self) -> &SessionDocumentObservationV1 {
        &self.observation
    }

    #[must_use]
    pub(super) fn session(&self) -> &DocumentSession {
        &self.session
    }

    #[must_use]
    pub(super) const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    #[must_use]
    pub(super) const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
}

fn parse_lowercase_sha256(value: &str) -> Result<[u8; 32], FrozenDocumentSnapshotAdmissionErrorV1> {
    if value.len() != 64 {
        return Err(FrozenDocumentSnapshotAdmissionErrorV1::MalformedDigest(
            "snapshot.digest_hex must contain 64 hexadecimal characters",
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair).expect("digest bytes are sized as ASCII pairs");
        if !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(FrozenDocumentSnapshotAdmissionErrorV1::MalformedDigest(
                "snapshot.digest_hex must contain lowercase hexadecimal characters",
            ));
        }
        digest[index] = u8::from_str_radix(text, 16).expect("validated hexadecimal pair");
    }
    Ok(digest)
}
