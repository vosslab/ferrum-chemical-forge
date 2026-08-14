use std::collections::BTreeSet;

use crate::{paper_size_catalog_v1, paper_size_v1};

#[test]
fn catalog_entries_have_unique_names_and_physical_fixed_dimensions() {
    let mut names = BTreeSet::new();
    for entry in paper_size_catalog_v1() {
        assert!(!entry.name().is_empty() && names.insert(entry.name()));
        if let Some(dimensions) = entry.dimensions() {
            assert!(
                dimensions.width().is_finite()
                    && dimensions.width() > 0.0
                    && dimensions.height().is_finite()
                    && dimensions.height() > 0.0
            );
        }
    }
}

#[test]
fn exact_lookup_distinguishes_fixed_and_custom_cdml_sizes() {
    let a4 = paper_size_v1("A4").expect("A4 is a recognized CDML paper size");
    let letter = paper_size_v1("Letter").expect("Letter is a recognized CDML paper size");

    assert_eq!(
        a4.dimensions().map(|value| (value.width(), value.height())),
        Some((210.0, 297.0))
    );
    assert_eq!(
        letter
            .dimensions()
            .map(|value| (value.width(), value.height())),
        Some((215.9, 279.4))
    );
    assert!(paper_size_v1("custom").is_some_and(|value| value.dimensions().is_none()));
    assert!(paper_size_v1("a4").is_none());
}
