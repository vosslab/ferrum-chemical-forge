//! Closed semantic Telex V1 scalar contract.
//!
//! This closed maintained contract is verified against the immutable bundled
//! Telex Regular resource. It provides scalar eligibility only; the renderer
//! owns font bytes, glyph identifiers, metrics, outlines, and layout.

/// Immutable resource identity shared with the renderer.
pub const TELEX_REGULAR_RESOURCE_ID_V1: &str = "ferrum-telex-regular-v1";
/// SHA-256 of the bundled Telex resource verified by this contract.
pub const TELEX_REGULAR_SHA256_V1: &str =
    "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871";

/// Semantic capability of one Telex V1 scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelexScalarCapabilityV1 {
    /// A visible scalar with a non-zero glyph, positive advance, and outline.
    Outlined,
    /// A whitespace scalar with a non-zero glyph, positive advance, and no outline.
    WhitespaceAdvanceOnly,
    /// A structural line break, not a font glyph.
    LineFeed,
}

/// Closed source-independent reason a text segment is ineligible for Telex V1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelexTextExclusionV1 {
    BlankText,
    ForbiddenControlScalar { run_index: u32, scalar_index: u32 },
    UnsupportedScalar { run_index: u32, scalar_index: u32 },
    VisibleScalarWithoutOutline { run_index: u32, scalar_index: u32 },
    InvalidWhitespaceGlyph { run_index: u32, scalar_index: u32 },
    MissingAdvance { run_index: u32, scalar_index: u32 },
}

/// Classify a Unicode scalar against the closed Telex V1 contract.
#[must_use]
pub const fn classify_telex_scalar_v1(scalar: char) -> Option<TelexScalarCapabilityV1> {
    match scalar {
        '\n' => Some(TelexScalarCapabilityV1::LineFeed),
        ' ' | '\u{00a0}' => Some(TelexScalarCapabilityV1::WhitespaceAdvanceOnly),
        _ if in_outlined_ranges(scalar as u32) => Some(TelexScalarCapabilityV1::Outlined),
        _ => None,
    }
}

/// Validate ordered text segments without retaining a second representation.
pub fn validate_telex_text_segments_v1<'a>(
    segments: impl IntoIterator<Item = &'a str>,
) -> Result<(), TelexTextExclusionV1> {
    let mut has_visible_scalar = false;
    for (run_index, segment) in segments.into_iter().enumerate() {
        let run_index = u32::try_from(run_index).expect("bounded run index fits u32");
        if segment.is_empty() {
            return Err(TelexTextExclusionV1::BlankText);
        }
        for (index, scalar) in segment.chars().enumerate() {
            let scalar_index = u32::try_from(index).expect("bounded text index fits u32");
            if scalar.is_control() && scalar != '\n' {
                return Err(TelexTextExclusionV1::ForbiddenControlScalar {
                    run_index,
                    scalar_index,
                });
            }
            match classify_telex_scalar_v1(scalar) {
                Some(TelexScalarCapabilityV1::Outlined) => has_visible_scalar = true,
                Some(TelexScalarCapabilityV1::WhitespaceAdvanceOnly) if scalar.is_whitespace() => {}
                Some(TelexScalarCapabilityV1::WhitespaceAdvanceOnly) => {
                    return Err(TelexTextExclusionV1::InvalidWhitespaceGlyph {
                        run_index,
                        scalar_index,
                    });
                }
                Some(TelexScalarCapabilityV1::LineFeed) => {}
                None => {
                    return Err(TelexTextExclusionV1::UnsupportedScalar {
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
        Err(TelexTextExclusionV1::BlankText)
    }
}

const fn in_outlined_ranges(value: u32) -> bool {
    matches!(value,
        0x21..=0x7e | 0xa1..=0xb4 | 0xb6..=0xff | 0x127..=0x129 |
        0x131..=0x135 | 0x137..=0x138 | 0x140..=0x144 | 0x152..=0x154 |
        0x156..=0x159 | 0x160..=0x161 | 0x178 | 0x17d..=0x17e | 0x192 |
        0x2c6..=0x2c7 | 0x2d8..=0x2dd | 0x394 | 0x3a9 | 0x3bc | 0x3c0 |
        0x2013..=0x2014 | 0x2018..=0x201a | 0x201c..=0x201e |
        0x2020..=0x2022 | 0x2026 | 0x2030 | 0x2039..=0x203a | 0x2044 |
        0x20ac | 0x2122 | 0x2202 | 0x220f | 0x2211..=0x2212 | 0x221a |
        0x221e | 0x222b | 0x2248 | 0x2260 | 0x2264..=0x2265 | 0xf8ff |
        0xfb01..=0xfb02
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telex_contract_has_closed_visible_whitespace_and_line_break_semantics() {
        assert_eq!(
            classify_telex_scalar_v1('C'),
            Some(TelexScalarCapabilityV1::Outlined)
        );
        assert_eq!(
            classify_telex_scalar_v1(' '),
            Some(TelexScalarCapabilityV1::WhitespaceAdvanceOnly)
        );
        assert_eq!(
            classify_telex_scalar_v1('\n'),
            Some(TelexScalarCapabilityV1::LineFeed)
        );
        assert_eq!(classify_telex_scalar_v1('\u{1f642}'), None);
        assert_eq!(validate_telex_text_segments_v1(["C\nO"]), Ok(()));
        assert_eq!(
            validate_telex_text_segments_v1([" \u{a0}\n"]),
            Err(TelexTextExclusionV1::BlankText)
        );
        assert_eq!(
            validate_telex_text_segments_v1(["C\rO"]),
            Err(TelexTextExclusionV1::ForbiddenControlScalar {
                run_index: 0,
                scalar_index: 1,
            })
        );
        assert_eq!(
            validate_telex_text_segments_v1(["C\u{1f642}"]),
            Err(TelexTextExclusionV1::UnsupportedScalar {
                run_index: 0,
                scalar_index: 1,
            })
        );
    }
}
