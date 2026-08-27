//! Closed, stateless JSON operation protocol V1 DTO facade.
//!
//! The portable wire contract is grouped by stable responsibility while this
//! facade preserves the established `protocol::dto` Rust surface.

mod catalog_reaction_dto;
mod document_compact_group_attachment_v1;
mod document_general_dto;
mod document_interchange_dto;
mod document_molecule_diagnostics_dto;
mod document_observation_dto;
mod document_report_dto;
mod dto_errors;
mod operation_dto;
mod presentation_author_dto;

pub use catalog_reaction_dto::*;
pub use document_compact_group_attachment_v1::*;
pub use document_general_dto::*;
pub use document_interchange_dto::*;
pub use document_molecule_diagnostics_dto::*;
pub use document_observation_dto::*;
pub use document_report_dto::*;
pub use dto_errors::*;
pub use operation_dto::*;
pub use presentation_author_dto::*;

pub(super) use operation_dto::{MAX_ARTIFACT_BASE64_BYTES_V1, base64_encoded_len};
