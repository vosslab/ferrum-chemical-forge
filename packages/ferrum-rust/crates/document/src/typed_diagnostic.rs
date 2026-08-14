//! Non-demoting structural diagnostics attached to recognized CDML records.

use super::TypedClass;

/// A non-demoting structural problem found on a recognized record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedDiagnosticKind {
    /// A required child slot was absent.
    MissingChild,
    /// A child exceeded the class's maximum cardinality and stayed opaque.
    ExcessChild,
}

/// One diagnostic attached to a typed record without changing its class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedDiagnostic {
    pub(crate) kind: TypedDiagnosticKind,
    pub(crate) child_class: TypedClass,
    pub(crate) message: String,
}

impl TypedDiagnostic {
    /// Return the stable problem category.
    #[must_use]
    pub fn kind(&self) -> TypedDiagnosticKind {
        self.kind
    }

    /// Return the child slot involved.
    #[must_use]
    pub fn child_class(&self) -> TypedClass {
        self.child_class
    }

    /// Return the human-readable diagnostic.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
