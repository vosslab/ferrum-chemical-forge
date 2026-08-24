//! Shared bounded-work limits for candidate chemistry admission.

/// Maximum atom and group vertices in one candidate root.
pub const DOCUMENT_CHEMISTRY_MAX_VERTICES_V1: usize = 256;
/// Maximum bonds in one candidate root.
pub const DOCUMENT_CHEMISTRY_MAX_BONDS_V1: usize = 512;
/// Maximum disconnected graph components in one candidate root.
pub const DOCUMENT_CHEMISTRY_MAX_COMPONENTS_V1: usize = 64;
