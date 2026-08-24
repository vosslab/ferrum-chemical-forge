//! Closed atom-mark mutation intent retained by the document session.

use serde::Serialize;

/// Exact intent for one atom-mark operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomMarkActionV1 {
    /// Append one new authored mark.
    Add,
    /// Remove one matching authored mark.
    Remove,
}
