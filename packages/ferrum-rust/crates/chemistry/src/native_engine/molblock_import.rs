//! Pre-native validation for one bounded V2000 or V3000 molblock.

use super::*;

pub(super) fn validate_input(input: &str) -> Result<(), ChemistryError> {
    if input.is_empty() {
        return invalid_input("must not be empty");
    }
    if input.as_bytes().contains(&0) {
        return invalid_input("must not contain NUL bytes");
    }
    if input.len() > MOLBLOCK_MAX_INPUT_BYTES {
        return invalid_input(&format!(
            "has {} bytes, above the {MOLBLOCK_MAX_INPUT_BYTES}-byte ABI limit",
            input.len()
        ));
    }
    if input.contains("$$$$") {
        return invalid_input("must not contain an SDF record delimiter");
    }

    let lines = input.lines().collect::<Vec<_>>();
    let Some(counts_line) = lines.get(3) else {
        return invalid_input("is missing its counts line");
    };
    let counts_line = counts_line.trim_end_matches([' ', '\t', '\r']);
    if !counts_line.ends_with("V2000") && !counts_line.ends_with("V3000") {
        return invalid_input("counts line must explicitly select V2000 or V3000");
    }

    let terminators = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == "M  END").then_some(index))
        .collect::<Vec<_>>();
    let [terminator] = terminators.as_slice() else {
        return invalid_input("must contain exactly one M  END terminator");
    };
    if lines[terminator + 1..].iter().any(|line| {
        !line
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
    }) {
        return invalid_input("must not contain data after M  END");
    }
    Ok(())
}

fn invalid_input<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::InvalidMolblockInput {
        reason: reason.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const V2000: &str = "ethane\n  Ferrum\ncomment\n  2  1  0  0  0  0  0  0  0  0999 V2000\n    0.0  0.0  0.0 C   0  0\n    1.0  0.0  0.0 C   0  0\n  1  2  1  0\nM  END\n";

    #[test]
    fn accepts_an_explicit_single_molblock_envelope() {
        assert_eq!(validate_input(V2000), Ok(()));
        assert_eq!(validate_input(&V2000.replace("V2000", "V3000")), Ok(()));
    }

    #[test]
    fn rejects_container_or_ambiguous_input_before_native_loading() {
        for (input, fragment) in [
            ("", "empty"),
            ("a\0b", "NUL"),
            (&V2000.replace("V2000", "V4000"), "V2000 or V3000"),
            (&format!("{V2000}$$$$\n"), "SDF"),
            (&format!("{V2000}M  END\n"), "exactly one"),
            (&format!("{V2000}extra\n"), "after M  END"),
        ] {
            let error = validate_input(input).expect_err("invalid molblock is rejected");
            assert!(error.to_string().contains(fragment), "{error}");
        }
    }
}
