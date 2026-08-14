//! Bounded FCS1 isotope-aware composition response decoding.

use std::cmp::Ordering;

use super::*;
use crate::{
    CompositionElementKey, ElementCount, ElementMassPercentage, MoleculeComposition,
    composition::{format_formula, hill_order},
};

const MAGIC: [u8; 4] = *b"FCS1";

pub(super) fn decode(
    response: &[u8],
    request_atom_count: usize,
) -> Result<MoleculeComposition, ChemistryError> {
    if response.len() < FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES {
        return Err(ChemistryError::TruncatedNativeResponse);
    }
    let mut reader = Reader::new(response);
    if reader.take(4).map_err(decode_error)? != MAGIC {
        return malformed("FCS1 response magic is invalid");
    }
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_COMPOSITION_WIRE_VERSION {
        return malformed("FCS1 wire version is unsupported");
    }
    let status = reader.u32().map_err(decode_error)?;
    let detail_length = usize::try_from(reader.u32().map_err(decode_error)?)
        .expect("u32 fits every supported usize");
    let formula_length = usize::try_from(reader.u32().map_err(decode_error)?)
        .expect("u32 fits every supported usize");
    let entry_count = usize::try_from(reader.u32().map_err(decode_error)?)
        .expect("u32 fits every supported usize");
    let flags = reader.u32().map_err(decode_error)?;
    let reserved = reader.u32().map_err(decode_error)?;
    let net_charge = reader.i64().map_err(decode_error)?;
    let average_bits = reader.u64().map_err(decode_error)?;
    let exact_bits = reader.u64().map_err(decode_error)?;

    validate_declared_lengths(
        response,
        request_atom_count,
        detail_length,
        formula_length,
        entry_count,
    )?;
    if flags != FERRUM_CHEM_COMPOSITION_FLAGS_NONE || reserved != 0 {
        return malformed("FCS1 flags or reserved header field are nonzero");
    }
    let detail = response_text(
        reader.take(detail_length).map_err(decode_error)?,
        "FCS1 detail",
    )?;
    let formula_text = response_text(
        reader.take(formula_length).map_err(decode_error)?,
        "FCS1 formula",
    )?;
    if status != FERRUM_CHEM_RESULT_OK {
        validate_error_shape(
            status,
            detail,
            formula_text,
            entry_count,
            net_charge,
            average_bits,
            exact_bits,
        )?;
        return Err(ChemistryError::NativeRejected {
            status,
            reason: try_owned(detail, "composition error detail")?,
        });
    }
    if !detail.is_empty() || formula_text.is_empty() || entry_count == 0 {
        return malformed("successful FCS1 response has invalid text or entry fields");
    }
    let average_mass = f64::from_bits(average_bits);
    let exact_mass = f64::from_bits(exact_bits);
    if !average_mass.is_finite()
        || average_mass <= 0.0
        || !exact_mass.is_finite()
        || exact_mass <= 0.0
    {
        return malformed("successful FCS1 response has invalid masses");
    }
    decode_success(
        &mut reader,
        formula_text,
        entry_count,
        net_charge,
        average_mass,
        exact_mass,
    )
}

fn validate_declared_lengths(
    response: &[u8],
    request_atom_count: usize,
    detail_length: usize,
    formula_length: usize,
    entry_count: usize,
) -> Result<(), ChemistryError> {
    if detail_length > FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES {
        return malformed("FCS1 detail exceeds its ABI limit");
    }
    if formula_length > FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES {
        return malformed("FCS1 formula exceeds its ABI limit");
    }
    let maximum_entries = request_atom_count.checked_add(1).ok_or_else(|| {
        ChemistryError::MalformedNativeResponse {
            reason: "FCS1 request atom bound overflows this platform".to_owned(),
        }
    })?;
    if entry_count > maximum_entries {
        return malformed("FCS1 entry count exceeds the request-derived bound");
    }
    let entry_bytes = entry_count
        .checked_mul(FERRUM_CHEM_COMPOSITION_ENTRY_BYTES)
        .ok_or_else(|| ChemistryError::MalformedNativeResponse {
            reason: "FCS1 entry byte length overflows this platform".to_owned(),
        })?;
    let declared = FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES
        .checked_add(detail_length)
        .and_then(|length| length.checked_add(formula_length))
        .and_then(|length| length.checked_add(entry_bytes))
        .ok_or_else(|| ChemistryError::MalformedNativeResponse {
            reason: "FCS1 total byte length overflows this platform".to_owned(),
        })?;
    if declared > FERRUM_CHEM_MAX_RESPONSE_BYTES || response.len() != declared {
        return malformed("FCS1 response is truncated, trailing, or above the global limit");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_error_shape(
    status: u32,
    detail: &str,
    formula: &str,
    entry_count: usize,
    net_charge: i64,
    average_bits: u64,
    exact_bits: u64,
) -> Result<(), ChemistryError> {
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_RESOURCE_LIMIT
            | FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return malformed("FCS1 status is unsupported");
    }
    if detail.is_empty()
        || !formula.is_empty()
        || entry_count != 0
        || net_charge != 0
        || average_bits != 0
        || exact_bits != 0
    {
        return malformed("failed FCS1 response contains success data");
    }
    Ok(())
}

fn decode_success(
    reader: &mut Reader<'_>,
    formula_text: &str,
    entry_count: usize,
    net_charge: i64,
    average_mass: f64,
    exact_mass: f64,
) -> Result<MoleculeComposition, ChemistryError> {
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(entry_count)
        .map_err(|_| resource("composition element counts"))?;
    let mut contributions = Vec::new();
    contributions
        .try_reserve_exact(entry_count)
        .map_err(|_| resource("composition mass entries"))?;
    let mut previous = None;
    let mut total_count = 0_u64;
    let mut contribution_total = 0.0_f64;
    for _ in 0..entry_count {
        let atomic_number = AtomicNumber::try_from(reader.u8().map_err(decode_error)?)
            .map_err(|_| malformed_error("FCS1 entry has an invalid atomic number"))?;
        let isotope_present = reader.u8().map_err(decode_error)?;
        if reader.u16().map_err(decode_error)? != 0 {
            return malformed("FCS1 entry reserved field is nonzero");
        }
        let isotope_value = reader.u16().map_err(decode_error)?;
        if reader.u16().map_err(decode_error)? != 0 {
            return malformed("FCS1 entry reserved field is nonzero");
        }
        let isotope = match (isotope_present, isotope_value) {
            (0, 0) => None,
            (1, value) if value != 0 => Some(value),
            _ => return malformed("FCS1 entry has invalid isotope fields"),
        };
        let count = reader.u64().map_err(decode_error)?;
        let contribution = reader.f64().map_err(decode_error)?;
        if count == 0 || !contribution.is_finite() || contribution <= 0.0 {
            return malformed("FCS1 entry has an invalid count or mass contribution");
        }
        let key = CompositionElementKey::new(atomic_number, isotope);
        if previous.is_some_and(|prior| hill_order(prior, key) != Ordering::Less) {
            return malformed("FCS1 entries are duplicate or outside canonical formula order");
        }
        previous = Some(key);
        total_count = total_count
            .checked_add(count)
            .ok_or_else(|| malformed_error("FCS1 element count sum overflows u64"))?;
        contribution_total += contribution;
        if !contribution_total.is_finite() {
            return malformed("FCS1 mass contribution sum is not finite");
        }
        counts.push(ElementCount::new(key, count));
        contributions.push((key, contribution));
    }
    if !reader.is_empty() {
        return Err(ChemistryError::TrailingNativeResponse);
    }
    if total_count == 0 || contribution_total.to_bits() != average_mass.to_bits() {
        return malformed("FCS1 entries do not reproduce the declared average mass");
    }
    let expected_formula = format_formula(
        &counts,
        net_charge,
        FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES,
    )
    .map_err(|()| resource("composition formula"))?;
    if expected_formula != formula_text {
        return malformed("FCS1 formula disagrees with its ordered entries and charge");
    }
    let formula = try_owned(formula_text, "composition formula")?;
    let mut percentages = Vec::new();
    percentages
        .try_reserve_exact(entry_count)
        .map_err(|_| resource("composition mass percentages"))?;
    for (key, contribution) in contributions {
        let percentage = contribution / contribution_total * 100.0;
        if !percentage.is_finite() || percentage <= 0.0 {
            return malformed("FCS1 mass percentage is not finite and positive");
        }
        percentages.push(ElementMassPercentage::new(key, contribution, percentage));
    }
    Ok(MoleculeComposition::new(
        formula,
        net_charge,
        average_mass,
        exact_mass,
        counts,
        percentages,
    ))
}

fn response_text<'a>(bytes: &'a [u8], field: &str) -> Result<&'a str, ChemistryError> {
    if bytes.contains(&0) {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: format!("{field} contains NUL"),
        });
    }
    std::str::from_utf8(bytes).map_err(|_| ChemistryError::MalformedNativeResponse {
        reason: format!("{field} is not UTF-8"),
    })
}

fn try_owned(value: &str, operation: &'static str) -> Result<String, ChemistryError> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| resource(operation))?;
    owned.push_str(value);
    Ok(owned)
}

fn resource(operation: &'static str) -> ChemistryError {
    ChemistryError::ResourceExhausted { operation }
}

fn malformed_error(reason: &'static str) -> ChemistryError {
    ChemistryError::MalformedNativeResponse {
        reason: reason.to_owned(),
    }
}

fn malformed<T>(reason: &'static str) -> Result<T, ChemistryError> {
    Err(malformed_error(reason))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn success_response(
        formula: &str,
        charge: i64,
        exact_mass: f64,
        entries: &[(u8, Option<u16>, u64, f64)],
    ) -> Vec<u8> {
        let average_mass = entries.iter().map(|entry| entry.3).sum::<f64>();
        let mut response = Vec::new();
        response.extend_from_slice(&MAGIC);
        response.extend_from_slice(&FERRUM_CHEM_COMPOSITION_WIRE_VERSION.to_le_bytes());
        response.extend_from_slice(&FERRUM_CHEM_RESULT_OK.to_le_bytes());
        response.extend_from_slice(&0_u32.to_le_bytes());
        response.extend_from_slice(
            &u32::try_from(formula.len())
                .expect("test formula fits")
                .to_le_bytes(),
        );
        response.extend_from_slice(
            &u32::try_from(entries.len())
                .expect("test entries fit")
                .to_le_bytes(),
        );
        response.extend_from_slice(&FERRUM_CHEM_COMPOSITION_FLAGS_NONE.to_le_bytes());
        response.extend_from_slice(&0_u32.to_le_bytes());
        response.extend_from_slice(&charge.to_le_bytes());
        response.extend_from_slice(&average_mass.to_le_bytes());
        response.extend_from_slice(&exact_mass.to_le_bytes());
        response.extend_from_slice(formula.as_bytes());
        for (atomic_number, isotope, count, contribution) in entries {
            response.push(*atomic_number);
            response.push(u8::from(isotope.is_some()));
            response.extend_from_slice(&0_u16.to_le_bytes());
            response.extend_from_slice(&isotope.unwrap_or(0).to_le_bytes());
            response.extend_from_slice(&0_u16.to_le_bytes());
            response.extend_from_slice(&count.to_le_bytes());
            response.extend_from_slice(&contribution.to_le_bytes());
        }
        response
    }

    #[test]
    fn hill_formatter_matches_isotope_and_charge_examples() {
        let carbon = CompositionElementKey::new(AtomicNumber::try_from(6).expect("carbon"), None);
        let carbon_13 =
            CompositionElementKey::new(AtomicNumber::try_from(6).expect("carbon"), Some(13));
        let hydrogen =
            CompositionElementKey::new(AtomicNumber::try_from(1).expect("hydrogen"), None);
        let entries = [
            ElementCount::new(carbon, 1),
            ElementCount::new(carbon_13, 1),
            ElementCount::new(hydrogen, 6),
        ];

        assert_eq!(
            format_formula(&entries, 1, usize::MAX).expect("formula"),
            "C[13C]H6+"
        );
        assert_eq!(
            format_formula(&entries, -2, usize::MAX).expect("formula"),
            "C[13C]H6-2"
        );
    }

    #[test]
    fn decoder_retains_one_nonduplicating_mass_basis() {
        let response = success_response(
            "CH4",
            0,
            16.031_300_128,
            &[(6, None, 1, 12.011), (1, None, 4, 4.032)],
        );

        let composition = decode(&response, 1).expect("valid methane composition");

        assert_eq!(composition.formula(), "CH4");
        assert_eq!(composition.element_counts()[0].count(), 1);
        assert_eq!(composition.element_counts()[1].count(), 4);
        assert!(
            composition.mass_percentages()[0].percentage()
                > composition.mass_percentages()[1].percentage()
        );
    }

    #[test]
    fn decoder_enforces_request_bound_and_exact_hill_order() {
        let response = success_response(
            "C[13C]H6+",
            1,
            31.0,
            &[
                (6, None, 1, 12.011),
                (6, Some(13), 1, 13.003),
                (1, None, 6, 6.048),
            ],
        );
        let bounded = decode(&response, 2).expect("two atoms permit one inferred-H key");
        assert_eq!(bounded.formula(), "C[13C]H6+");

        assert!(matches!(
            decode(&response, 1),
            Err(ChemistryError::MalformedNativeResponse { .. })
        ));
        let out_of_order =
            success_response("CH4", 0, 16.0, &[(1, None, 4, 4.0), (6, None, 1, 12.0)]);
        assert!(matches!(
            decode(&out_of_order, 1),
            Err(ChemistryError::MalformedNativeResponse { .. })
        ));
    }
}
