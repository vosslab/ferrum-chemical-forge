//! Validation for the historical compact carbohydrate notation, version one.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use super::semantic::{
    BackboneToken, FootnoteFamily, FootnoteKey, LegacyCompactSugarCodeV1, SugarPosition,
    SugarPrefix, SugarSeries,
};

/// A stable, user-facing invalid-input diagnostic for the sugar syntax API.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LegacyCompactSugarCodeV1Error {
    /// A field was absent, malformed, or scientifically incompatible.
    #[error("invalid sugar code {field}: {reason}")]
    InvalidInput {
        /// Name of the user-visible field that needs correction.
        field: &'static str,
        /// Concise reason that does not expose parser implementation details.
        reason: String,
    },
}

fn invalid(field: &'static str, reason: impl Into<String>) -> LegacyCompactSugarCodeV1Error {
    LegacyCompactSugarCodeV1Error::InvalidInput {
        field,
        reason: reason.into(),
    }
}

pub(super) fn parse_legacy_compact_v1(
    input: &str,
) -> Result<LegacyCompactSugarCodeV1, LegacyCompactSugarCodeV1Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(invalid("code", "must not be empty"));
    }
    let (body, raw_footnotes) = split_footnote_block(trimmed)?;
    let positions = parse_body(body)?;
    let prefix = determine_prefix(body);
    let series = determine_series(body, prefix)?;
    let footnotes = parse_footnotes(raw_footnotes, body, prefix)?;
    Ok(LegacyCompactSugarCodeV1::new(
        body.to_owned(),
        prefix,
        series,
        positions,
        footnotes,
    ))
}

fn split_footnote_block(
    input: &str,
) -> Result<(&str, Option<&str>), LegacyCompactSugarCodeV1Error> {
    let Some(open) = input.find('[') else {
        return Ok((input, None));
    };
    if open == 0 {
        return Err(invalid("footnotes", "must follow a non-empty code body"));
    }
    if !input.ends_with(']')
        || input[open + 1..input.len() - 1].contains('[')
        || input[open + 1..input.len() - 1].contains(']')
    {
        return Err(invalid(
            "footnotes",
            "must use one closing, non-nested bracket block",
        ));
    }
    Ok((&input[..open], Some(&input[open + 1..input.len() - 1])))
}

fn parse_body(body: &str) -> Result<Vec<SugarPosition>, LegacyCompactSugarCodeV1Error> {
    let count = body.chars().count();
    if count < 3 {
        return Err(invalid(
            "code",
            "body must contain at least three backbone positions",
        ));
    }
    if body == "MRK" || body == "MLK" {
        return Err(invalid(
            "code",
            "a 3-ketose prefix must include a terminal position",
        ));
    }
    let mut positions = Vec::with_capacity(count);
    for (offset, character) in body.chars().enumerate() {
        let position = u8::try_from(offset + 1)
            .map_err(|_| invalid("code", "supports at most 255 backbone positions"))?;
        if character == 'D' && usize::from(position) != count - 1 {
            return Err(invalid(
                "code",
                "D-series marker is only valid at the penultimate backbone position",
            ));
        }
        let token = BackboneToken::from_char(character, position).ok_or_else(|| {
            invalid(
                "code",
                format!("'{character}' is not valid at backbone position {position}"),
            )
        })?;
        positions.push(SugarPosition { position, token });
    }
    Ok(positions)
}

fn determine_prefix(body: &str) -> SugarPrefix {
    if body.starts_with("MRK") || body.starts_with("MLK") || body.chars().nth(2) == Some('K') {
        SugarPrefix::ThreeKeto
    } else if body.starts_with("MK") || body.chars().nth(1) == Some('K') {
        SugarPrefix::Keto
    } else {
        SugarPrefix::Aldo
    }
}

fn determine_series(
    body: &str,
    prefix: SugarPrefix,
) -> Result<SugarSeries, LegacyCompactSugarCodeV1Error> {
    let penultimate = body
        .chars()
        .nth(body.chars().count() - 2)
        .expect("a previously validated code has at least three positions");
    match penultimate {
        'D' => Ok(SugarSeries::D),
        'L' => Ok(SugarSeries::L),
        _ if is_meso_form(body, prefix) => Ok(SugarSeries::Meso),
        _ => Err(invalid(
            "code",
            "requires a D or L series marker at the penultimate position",
        )),
    }
}

fn is_meso_form(body: &str, prefix: SugarPrefix) -> bool {
    match prefix {
        SugarPrefix::Aldo => {
            body.chars().count() == 3 && body.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
        }
        SugarPrefix::Keto => body.chars().count() == 3,
        SugarPrefix::ThreeKeto => body.chars().count() == 5,
    }
}

fn parse_footnotes(
    raw: Option<&str>,
    body: &str,
    prefix: SugarPrefix,
) -> Result<BTreeMap<FootnoteKey, String>, LegacyCompactSugarCodeV1Error> {
    let marker_positions = body
        .chars()
        .enumerate()
        .filter_map(|(offset, character)| character.is_ascii_digit().then_some((offset + 1) as u8))
        .collect::<BTreeSet<_>>();
    let Some(raw) = raw else {
        if marker_positions.is_empty() {
            return Ok(BTreeMap::new());
        }
        return Err(invalid(
            "footnotes",
            "every position marker needs a declaration",
        ));
    };
    if marker_positions.is_empty() {
        return Err(invalid(
            "footnotes",
            "require at least one position marker in the code body",
        ));
    }
    if raw.trim().is_empty() {
        return Err(invalid("footnotes", "must not be empty"));
    }

    let mut declared = BTreeMap::new();
    let mut last_position = 0_u8;
    for entry in split_entries(raw)? {
        let (key_text, value) = entry
            .split_once('=')
            .ok_or_else(|| invalid("footnotes", format!("'{entry}' must use key=value syntax")))?;
        let key = parse_key(key_text.trim())?;
        let value = value.trim();
        if value.is_empty() {
            return Err(invalid(
                "footnotes",
                format!("{} has no value", key_text.trim()),
            ));
        }
        if key.position < last_position {
            return Err(invalid(
                "footnotes",
                "must be ordered by ascending backbone position",
            ));
        }
        last_position = key.position;
        if !marker_positions.contains(&key.position) {
            return Err(invalid(
                "footnotes",
                format!("{} does not reference a position marker", key.position),
            ));
        }
        if declared.insert(key, value.to_owned()).is_some() {
            return Err(invalid(
                "footnotes",
                format!("{} is declared more than once", key_text.trim()),
            ));
        }
    }
    validate_footnote_families(&declared, &marker_positions, body, prefix)?;
    complete_side_pairs(&mut declared);
    Ok(declared)
}

fn split_entries(raw: &str) -> Result<Vec<&str>, LegacyCompactSugarCodeV1Error> {
    let mut entries = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0_usize;
    for (offset, character) in raw.char_indices() {
        match character {
            '(' => depth += 1,
            ')' if depth == 0 => return Err(invalid("footnotes", "contains an unmatched ')'")),
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let entry = raw[start..offset].trim();
                if entry.is_empty() {
                    return Err(invalid("footnotes", "contains an empty entry"));
                }
                entries.push(entry);
                start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(invalid("footnotes", "contains an unclosed '('"));
    }
    let tail = raw[start..].trim();
    if tail.is_empty() {
        return Err(invalid("footnotes", "contains an empty entry"));
    }
    entries.push(tail);
    Ok(entries)
}

fn parse_key(text: &str) -> Result<FootnoteKey, LegacyCompactSugarCodeV1Error> {
    let mut characters = text.chars();
    let digit = characters
        .next()
        .filter(char::is_ascii_digit)
        .ok_or_else(|| invalid("footnotes", format!("'{text}' is not a valid footnote key")))?;
    let family = match characters.next() {
        None => FootnoteFamily::Plain,
        Some('C') if characters.next().is_none() => FootnoteFamily::CarbonState,
        Some('L') if characters.next().is_none() => FootnoteFamily::Left,
        Some('R') if characters.next().is_none() => FootnoteFamily::Right,
        _ => {
            return Err(invalid(
                "footnotes",
                format!("'{text}' is not a valid footnote key"),
            ));
        }
    };
    Ok(FootnoteKey {
        position: digit
            .to_digit(10)
            .expect("an ASCII digit has a base-10 value") as u8,
        family,
    })
}

fn validate_footnote_families(
    declared: &BTreeMap<FootnoteKey, String>,
    marker_positions: &BTreeSet<u8>,
    body: &str,
    prefix: SugarPrefix,
) -> Result<(), LegacyCompactSugarCodeV1Error> {
    for position in marker_positions {
        let families = declared
            .keys()
            .filter(|key| key.position == *position)
            .map(|key| key.family)
            .collect::<BTreeSet<_>>();
        if families.is_empty() {
            return Err(invalid(
                "footnotes",
                format!("position marker {position} has no declaration"),
            ));
        }
        let whole_position = families.contains(&FootnoteFamily::Plain)
            || families.contains(&FootnoteFamily::CarbonState);
        let side_specific =
            families.contains(&FootnoteFamily::Left) || families.contains(&FootnoteFamily::Right);
        if whole_position && side_specific {
            return Err(invalid(
                "footnotes",
                format!(
                    "position marker {position} cannot mix whole-position and side declarations"
                ),
            ));
        }
        if families.contains(&FootnoteFamily::Plain) && is_chiral(*position, body, prefix) {
            return Err(invalid(
                "footnotes",
                format!(
                    "position marker {position} is chiral and needs side or carbon-state declarations"
                ),
            ));
        }
    }
    Ok(())
}

fn is_chiral(position: u8, body: &str, prefix: SugarPrefix) -> bool {
    let length = body.chars().count() as u8;
    match prefix {
        SugarPrefix::Aldo => (2..length).contains(&position),
        SugarPrefix::Keto => (3..length).contains(&position),
        SugarPrefix::ThreeKeto => position == 2 || (4..length).contains(&position),
    }
}

fn complete_side_pairs(declared: &mut BTreeMap<FootnoteKey, String>) {
    let positions = declared
        .keys()
        .map(|key| key.position)
        .collect::<BTreeSet<_>>();
    for position in positions {
        let left = FootnoteKey {
            position,
            family: FootnoteFamily::Left,
        };
        let right = FootnoteKey {
            position,
            family: FootnoteFamily::Right,
        };
        if declared.contains_key(&left) && !declared.contains_key(&right) {
            declared.insert(right, "H".to_owned());
        }
        if declared.contains_key(&right) && !declared.contains_key(&left) {
            declared.insert(left, "H".to_owned());
        }
    }
}
