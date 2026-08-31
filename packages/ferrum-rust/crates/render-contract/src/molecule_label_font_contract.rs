//! Closed semantic molecule-label scalar contract.
//!
//! This closed maintained contract is verified against the immutable bundled
//! Atkinson Hyperlegible Next Regular resource. It provides scalar eligibility
//! only; the renderer owns font bytes, glyph identifiers, metrics, outlines,
//! and layout.

/// Immutable resource identity shared with the renderer.
pub const MOLECULE_LABEL_RESOURCE_ID: &str = "ferrum-atkinson-hyperlegible-next-regular-2.001";
/// SHA-256 of the bundled molecule-label resource verified by this contract.
pub const MOLECULE_LABEL_SHA256: &str =
    "88ed5c31a71584c7772963b02d04bef1eb7e3d2e9c8b9cb204339b1f82cf432c";

/// Semantic capability of one molecule-label scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoleculeLabelScalarCapability {
    /// A visible scalar with a non-zero glyph, positive advance, and outline.
    Outlined,
    /// A whitespace scalar with a non-zero glyph, positive advance, and no outline.
    WhitespaceAdvanceOnly,
    /// A structural line break, not a font glyph.
    LineFeed,
}

/// Closed source-independent reason a text segment is ineligible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoleculeLabelTextExclusion {
    BlankText,
    ForbiddenControlScalar { run_index: u32, scalar_index: u32 },
    UnsupportedScalar { run_index: u32, scalar_index: u32 },
    VisibleScalarWithoutOutline { run_index: u32, scalar_index: u32 },
    InvalidWhitespaceGlyph { run_index: u32, scalar_index: u32 },
    MissingAdvance { run_index: u32, scalar_index: u32 },
}

/// Classify a Unicode scalar against the closed molecule-label contract.
#[must_use]
pub const fn classify_molecule_label_scalar(scalar: char) -> Option<MoleculeLabelScalarCapability> {
    match scalar {
        '\n' => Some(MoleculeLabelScalarCapability::LineFeed),
        ' ' | '\u{00a0}' | '\u{2009}' => Some(MoleculeLabelScalarCapability::WhitespaceAdvanceOnly),
        _ if in_outlined_ranges(scalar as u32) => Some(MoleculeLabelScalarCapability::Outlined),
        _ => None,
    }
}

/// Validate ordered text segments without retaining a second representation.
pub fn validate_molecule_label_text_segments<'a>(
    segments: impl IntoIterator<Item = &'a str>,
) -> Result<(), MoleculeLabelTextExclusion> {
    let mut has_visible_scalar = false;
    for (run_index, segment) in segments.into_iter().enumerate() {
        let run_index = u32::try_from(run_index).expect("bounded run index fits u32");
        if segment.is_empty() {
            return Err(MoleculeLabelTextExclusion::BlankText);
        }
        for (index, scalar) in segment.chars().enumerate() {
            let scalar_index = u32::try_from(index).expect("bounded text index fits u32");
            if scalar.is_control() && scalar != '\n' {
                return Err(MoleculeLabelTextExclusion::ForbiddenControlScalar {
                    run_index,
                    scalar_index,
                });
            }
            match classify_molecule_label_scalar(scalar) {
                Some(MoleculeLabelScalarCapability::Outlined) => has_visible_scalar = true,
                Some(MoleculeLabelScalarCapability::WhitespaceAdvanceOnly)
                    if scalar.is_whitespace() => {}
                Some(MoleculeLabelScalarCapability::WhitespaceAdvanceOnly) => {
                    return Err(MoleculeLabelTextExclusion::InvalidWhitespaceGlyph {
                        run_index,
                        scalar_index,
                    });
                }
                Some(MoleculeLabelScalarCapability::LineFeed) => {}
                None => {
                    return Err(MoleculeLabelTextExclusion::UnsupportedScalar {
                        run_index,
                        scalar_index,
                    });
                }
            }
        }
    }
    if has_visible_scalar {
        Ok(())
    } else {
        Err(MoleculeLabelTextExclusion::BlankText)
    }
}

const fn in_outlined_ranges(value: u32) -> bool {
    matches!(value,
        0x21..=0x7e | 0xa1..=0xac | 0xae..=0xb4 | 0xb6..=0x107 |
        0x10a..=0x113 | 0x116..=0x11b | 0x11e..=0x123 | 0x126..=0x127 |
        0x12a..=0x12b | 0x12e..=0x133 | 0x136..=0x137 | 0x139..=0x13e |
        0x141..=0x148 | 0x150..=0x155 | 0x158..=0x15b | 0x15e..=0x165 |
        0x16a..=0x16b | 0x16e..=0x17e | 0x192 | 0x218..=0x21b | 0x237 |
        0x2c6..=0x2c7 | 0x2c9 | 0x2d8..=0x2dd | 0x394 | 0x3a9 | 0x3bc |
        0x3c0 | 0x1e80..=0x1e85 | 0x1e9e | 0x1ef2..=0x1ef3 |
        0x2013..=0x2014 | 0x2018..=0x201a | 0x201c..=0x201e |
        0x2020..=0x2022 | 0x2026 | 0x2030 | 0x2039..=0x203a | 0x2044 |
        0x20ac | 0x20b9 | 0x2113 | 0x2122 | 0x212e | 0x2202 | 0x220f |
        0x2211..=0x2212 | 0x2215 | 0x2219..=0x221a | 0x221e | 0x222b |
        0x2248 | 0x2260 | 0x2264..=0x2265 | 0x25ca | 0x266a
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn molecule_label_contract_has_closed_visible_whitespace_and_line_break_semantics() {
        assert_eq!(
            classify_molecule_label_scalar('C'),
            Some(MoleculeLabelScalarCapability::Outlined)
        );
        assert_eq!(
            classify_molecule_label_scalar(' '),
            Some(MoleculeLabelScalarCapability::WhitespaceAdvanceOnly)
        );
        assert_eq!(
            classify_molecule_label_scalar('\n'),
            Some(MoleculeLabelScalarCapability::LineFeed)
        );
        assert_eq!(classify_molecule_label_scalar('\u{1f642}'), None);
        assert_eq!(validate_molecule_label_text_segments(["C\nO"]), Ok(()));
        assert_eq!(
            validate_molecule_label_text_segments([" \u{a0}\n"]),
            Err(MoleculeLabelTextExclusion::BlankText)
        );
        assert_eq!(
            validate_molecule_label_text_segments(["C\rO"]),
            Err(MoleculeLabelTextExclusion::ForbiddenControlScalar {
                run_index: 0,
                scalar_index: 1,
            })
        );
        assert_eq!(
            validate_molecule_label_text_segments(["C\u{1f642}"]),
            Err(MoleculeLabelTextExclusion::UnsupportedScalar {
                run_index: 0,
                scalar_index: 1,
            })
        );
    }
}
