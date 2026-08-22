//! Canonical Ferrum CDML XML vocabulary identity.

use xot::{NameId, Xot};

/// The only namespace accepted for ordinary Ferrum CDML documents.
pub(crate) const CDML_NAMESPACE: &str = "urn:ferrum:cdml";

/// Return whether one expanded XML name is the requested Ferrum CDML core name.
#[must_use]
pub(crate) fn is_ferrum_cdml_name(local_name: &str, namespace: &str, expected: &str) -> bool {
    local_name == expected && namespace == CDML_NAMESPACE
}

/// Return whether one expanded XML name is the Ferrum CDML document root.
#[must_use]
pub(crate) fn is_ferrum_cdml_root(local_name: &str, namespace: &str) -> bool {
    is_ferrum_cdml_name(local_name, namespace, "cdml")
}

/// Construct an authored Ferrum CDML element name.
pub(crate) fn ferrum_cdml_element_name(tree: &mut Xot, local_name: &str) -> NameId {
    let namespace = tree.add_namespace(CDML_NAMESPACE);
    tree.add_name_ns(local_name, namespace)
}
