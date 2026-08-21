fn decode_coordinate_response(
    response: &[u8],
    expected_atom_count: usize,
) -> Result<Coordinates, ChemistryError> {
    if response.len() < COORDINATE_RESPONSE_HEADER_LENGTH {
        return Err(ChemistryError::TruncatedNativeResponse);
    }
    let mut reader = Reader::new(response);
    if reader.take(4).map_err(decode_error)? != COORDINATE_RESPONSE_MAGIC {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "coordinate response magic is not FCL1".to_owned(),
        });
    }
    if reader.u32().map_err(decode_error)? != 1 {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "unsupported coordinate response wire version".to_owned(),
        });
    }
    let status = reader.u32().map_err(decode_error)?;
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "unknown or inapplicable coordinate response result status".to_owned(),
        });
    }
    let detail_length =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let atom_count = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let detail =
        std::str::from_utf8(reader.take(detail_length).map_err(decode_error)?).map_err(|_| {
            ChemistryError::MalformedNativeResponse {
                reason: "coordinate response detail is not UTF-8".to_owned(),
            }
        })?;
    if status != FERRUM_CHEM_RESULT_OK {
        if atom_count != 0 || !reader.is_empty() {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: "failed coordinate response contains coordinate records".to_owned(),
            });
        }
        return Err(ChemistryError::CoordinateGenerationFailed {
            reason: detail.to_owned(),
        });
    }
    if !detail.is_empty() || atom_count != expected_atom_count {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "coordinate response does not match the input atom order".to_owned(),
        });
    }
    let expected_bytes = atom_count.checked_mul(COORDINATE_BYTES).ok_or_else(|| {
        ChemistryError::MalformedNativeResponse {
            reason: "coordinate response length overflows this platform".to_owned(),
        }
    })?;
    if response.len().saturating_sub(reader.cursor) != expected_bytes {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "coordinate response has truncated or trailing records".to_owned(),
        });
    }
    let mut points = Vec::with_capacity(atom_count);
    for _ in 0..atom_count {
        let x = f64::from_le_bytes(
            reader
                .take(8)
                .map_err(decode_error)?
                .try_into()
                .expect("fixed"),
        );
        let y = f64::from_le_bytes(
            reader
                .take(8)
                .map_err(decode_error)?
                .try_into()
                .expect("fixed"),
        );
        points.push(
            Point2::new(x, y).map_err(|_| ChemistryError::MalformedNativeResponse {
                reason: "coordinate response contains a non-finite point".to_owned(),
            })?,
        );
    }
    Ok(Coordinates::new(points))
}
fn adapter_error(error: AdapterError) -> ChemistryError {
    match error {
        AdapterError::NativeStatus { status }
            if u64::from(status) == FERRUM_CHEM_CALL_ALLOCATION_FAILURE =>
        {
            ChemistryError::ResourceExhausted {
                operation: "native adapter response",
            }
        }
        AdapterError::OperationUnavailable { operation } => {
            ChemistryError::OperationUnavailable { operation }
        }
        // Every remaining variant originates below the private FFI boundary.
        // In particular, `Load` may retain a platform loader string containing
        // an absolute path. Do not expose any adapter diagnostic, ABI fact,
        // capability bit, buffer length, pointer-adjacent state, or status.
        AdapterError::Load(_)
        | AdapterError::AbiMismatch { .. }
        | AdapterError::UnknownCapabilities { .. }
        | AdapterError::NullBuffer { .. }
        | AdapterError::BufferTooLarge { .. }
        | AdapterError::ResponseTooLarge { .. }
        | AdapterError::BufferFreeDidNotClear
        | AdapterError::NativeStatus { .. } => ChemistryError::NativeBoundary {
            reason: NATIVE_ADAPTER_BOUNDARY_REASON.to_owned(),
        },
    }
}

fn smarts_adapter_error(error: AdapterError) -> ChemistryError {
    let reason = match error {
        AdapterError::Load(_) => SmartsMatchUnavailableReason::RuntimeUnavailable,
        AdapterError::AbiMismatch { .. } => SmartsMatchUnavailableReason::AbiIncompatible,
        AdapterError::UnknownCapabilities { .. } | AdapterError::OperationUnavailable { .. } => {
            SmartsMatchUnavailableReason::CapabilityUnavailable
        }
        AdapterError::NativeStatus { .. }
        | AdapterError::NullBuffer { .. }
        | AdapterError::BufferTooLarge { .. }
        | AdapterError::ResponseTooLarge { .. }
        | AdapterError::BufferFreeDidNotClear => SmartsMatchUnavailableReason::NativeCallFailed,
    };
    ChemistryError::SmartsMatchUnavailable { reason }
}
fn encode_kekulize_request(
    molecule: &MolGraph,
    options: KekulizeOptions,
) -> Result<Vec<u8>, ChemistryError> {
    encode_graph_request(molecule, options_bits(options), options.max_backtracks())
}

fn encode_depiction_request(molecule: &MolGraph) -> Result<Vec<u8>, ChemistryError> {
    let policy = DepictionRequestOptions;
    encode_graph_request(
        molecule,
        policy.option_bits(),
        policy.parser_backtrack_sentinel(),
    )
}

fn encode_graph_request(
    molecule: &MolGraph,
    option_bits: u32,
    max_backtracks: u32,
) -> Result<Vec<u8>, ChemistryError> {
    let atom_count = checked_count(
        molecule.atoms().len(),
        FERRUM_CHEM_KEKULIZE_MAX_ATOMS,
        "atom count",
    )?;
    let bond_count = checked_count(
        molecule.bonds().len(),
        FERRUM_CHEM_KEKULIZE_MAX_BONDS,
        "bond count",
    )?;
    if max_backtracks == 0 || max_backtracks > FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS {
        return Err(ChemistryError::UnsupportedNativeRequest {
            reason: format!(
                "max_backtracks {} exceeds {FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS}",
                max_backtracks
            ),
        });
    }

    let capacity = REQUEST_HEADER_LENGTH
        .checked_add(usize::try_from(atom_count).expect("u32 fits usize") * ATOM_LENGTH)
        .and_then(|length| {
            length.checked_add(usize::try_from(bond_count).expect("u32 fits usize") * BOND_LENGTH)
        })
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "request length overflows this platform".to_owned(),
        })?;
    let mut output = Vec::with_capacity(capacity);
    output.extend_from_slice(&REQUEST_MAGIC);
    put_u32(&mut output, FERRUM_CHEM_KEKULIZE_WIRE_VERSION);
    put_u32(&mut output, option_bits);
    put_u32(&mut output, max_backtracks);
    put_u32(&mut output, atom_count);
    put_u32(&mut output, bond_count);
    debug_assert_eq!(output.len(), REQUEST_HEADER_LENGTH);

    for atom in molecule.atoms() {
        encode_atom(&mut output, atom);
    }
    for bond in molecule.bonds() {
        encode_bond(&mut output, bond)?;
    }
    Ok(output)
}

fn checked_count(count: usize, maximum: u32, name: &str) -> Result<u32, ChemistryError> {
    let count = u32::try_from(count).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: format!("{name} does not fit the adapter protocol"),
    })?;
    if count > maximum {
        return Err(ChemistryError::UnsupportedNativeRequest {
            reason: format!("{name} {count} exceeds {maximum}"),
        });
    }
    Ok(count)
}

fn options_bits(options: KekulizeOptions) -> u32 {
    (u32::from(options.clear_aromatic_flags()) * FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS)
        | (u32::from(options.canonical()) * FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL)
}

fn encode_atom(output: &mut Vec<u8>, atom: &MolAtom) {
    output.push(atom.atomic_number().get());
    output.push(u8::from(atom.is_aromatic()));
    let mut facts = 0_u32;
    if atom.formal_charge().is_some() {
        facts |= FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE;
    }
    if atom.isotope().is_some() {
        facts |= FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE;
    }
    if atom.explicit_hydrogens().is_some() {
        facts |= FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS;
    }
    put_u16(
        output,
        u16::try_from(facts).expect("generated fact constants fit u16"),
    );
    put_i32(output, atom.formal_charge().unwrap_or(0));
    put_u16(output, atom.isotope().unwrap_or(0));
    put_u16(output, atom.explicit_hydrogens().unwrap_or(0));
}

fn encode_bond(output: &mut Vec<u8>, bond: &MolBond) -> Result<(), ChemistryError> {
    let start =
        u32::try_from(bond.start()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
            reason: "bond start index does not fit the adapter protocol".to_owned(),
        })?;
    let end = u32::try_from(bond.end()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: "bond end index does not fit the adapter protocol".to_owned(),
    })?;
    put_u32(output, start);
    put_u32(output, end);
    output.push(match bond.order() {
        BondOrder::Single => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE),
        BondOrder::Double => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE),
        BondOrder::Triple => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE),
        BondOrder::Aromatic => wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC),
        BondOrder::Quadruple => {
            return Err(ChemistryError::UnsupportedNativeRequest {
                reason: "quadruple bonds are not representable by adapter wire version 1"
                    .to_owned(),
            });
        }
    });
    output.push(u8::from(bond.is_aromatic()));
    put_u16(output, 0);
    Ok(())
}

fn wire_bond_type(value: u32) -> u8 {
    u8::try_from(value).expect("generated bond-type constant fits u8")
}

struct DecodedResponse {
    status: u32,
    detail: String,
    echoed_options: u32,
    echoed_max_backtracks: u32,
    atoms: Vec<MolAtom>,
    bonds: Vec<MolBond>,
}

fn decode_response(response: &[u8]) -> Result<DecodedResponse, DecodeFailure> {
    if response.len() < RESPONSE_HEADER_LENGTH {
        return Err(DecodeFailure::Truncated);
    }
    let mut reader = Reader::new(response);
    if reader.take(4)? != RESPONSE_MAGIC {
        return Err(DecodeFailure::Malformed("response magic is not FCR1"));
    }
    if reader.u32()? != FERRUM_CHEM_KEKULIZE_WIRE_VERSION {
        return Err(DecodeFailure::Malformed(
            "unsupported response wire version",
        ));
    }
    let status = reader.u32()?;
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_DEPICTION_FAILURE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return Err(DecodeFailure::Malformed("unknown response result status"));
    }
    let detail_length = usize::try_from(reader.u32()?).expect("u32 fits usize");
    if detail_length > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES {
        return Err(DecodeFailure::Malformed(
            "response detail exceeds protocol maximum",
        ));
    }
    let echoed_options = reader.u32()?;
    if echoed_options & !OPTION_MASK != 0 {
        return Err(DecodeFailure::Malformed(
            "response echoed reserved option bits",
        ));
    }
    let echoed_max_backtracks = reader.u32()?;
    if status == FERRUM_CHEM_RESULT_OK {
        if echoed_max_backtracks == 0 || echoed_max_backtracks > FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS
        {
            return Err(DecodeFailure::Malformed(
                "response echoed invalid max_backtracks",
            ));
        }
    } else if echoed_options != 0 || echoed_max_backtracks != 0 {
        return Err(DecodeFailure::Malformed(
            "error response includes request option echoes",
        ));
    }
    let atom_count = reader.u32()?;
    let bond_count = reader.u32()?;
    if atom_count > FERRUM_CHEM_KEKULIZE_MAX_ATOMS || bond_count > FERRUM_CHEM_KEKULIZE_MAX_BONDS {
        return Err(DecodeFailure::Malformed(
            "response count exceeds protocol maximum",
        ));
    }
    let detail = std::str::from_utf8(reader.take(detail_length)?)
        .map_err(|_| DecodeFailure::Malformed("response detail is not UTF-8"))?
        .to_owned();
    if status != FERRUM_CHEM_RESULT_OK && (atom_count != 0 || bond_count != 0) {
        return Err(DecodeFailure::Malformed("error response includes topology"));
    }

    let required_records = u64::from(atom_count)
        .checked_mul(u64::try_from(ATOM_LENGTH).expect("record length fits u64"))
        .and_then(|length| {
            length.checked_add(
                u64::from(bond_count) * u64::try_from(BOND_LENGTH).expect("record length fits u64"),
            )
        })
        .ok_or(DecodeFailure::Malformed("response record length overflows"))?;
    let remaining = response
        .len()
        .checked_sub(reader.cursor)
        .ok_or(DecodeFailure::Truncated)?;
    let remaining = u64::try_from(remaining).expect("usize fits u64");
    if remaining < required_records {
        return Err(DecodeFailure::Truncated);
    }
    if remaining > required_records {
        return Err(DecodeFailure::Trailing);
    }

    let mut atoms = Vec::with_capacity(usize::try_from(atom_count).expect("u32 fits usize"));
    for _ in 0..atom_count {
        atoms.push(decode_atom(&mut reader)?);
    }
    let mut bonds = Vec::with_capacity(usize::try_from(bond_count).expect("u32 fits usize"));
    for _ in 0..bond_count {
        bonds.push(decode_bond(&mut reader)?);
    }
    if !reader.is_empty() {
        return Err(DecodeFailure::Trailing);
    }
    Ok(DecodedResponse {
        status,
        detail,
        echoed_options,
        echoed_max_backtracks,
        atoms,
        bonds,
    })
}

fn decode_atom(reader: &mut Reader<'_>) -> Result<MolAtom, DecodeFailure> {
    let atomic_number = AtomicNumber::try_from(reader.u8()?)
        .map_err(|_| DecodeFailure::Malformed("response has an unsupported atomic number"))?;
    let aromatic = bool_from_byte(reader.u8()?, "atom aromatic flag")?;
    let facts = u32::from(reader.u16()?);
    if facts & !FACT_MASK != 0 {
        return Err(DecodeFailure::Malformed(
            "response atom has reserved presence bits",
        ));
    }
    let formal_charge_value = reader.i32()?;
    let isotope_value = reader.u16()?;
    let hydrogens_value = reader.u16()?;
    let formal_charge = optional_fact(
        facts,
        FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE,
        formal_charge_value,
        "formal charge",
    )?;
    let isotope = optional_fact(
        facts,
        FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE,
        isotope_value,
        "isotope",
    )?;
    let explicit_hydrogens = optional_fact(
        facts,
        FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS,
        hydrogens_value,
        "explicit hydrogens",
    )?;
    MolAtom::new(
        atomic_number,
        formal_charge,
        isotope,
        explicit_hydrogens,
        aromatic,
    )
    .map_err(|_| DecodeFailure::Malformed("response atom facts violate MolGraph invariants"))
}

fn optional_fact<T: Eq + Default>(
    flags: u32,
    flag: u32,
    value: T,
    name: &'static str,
) -> Result<Option<T>, DecodeFailure> {
    if flags & flag != 0 {
        Ok(Some(value))
    } else if value == T::default() {
        Ok(None)
    } else {
        Err(DecodeFailure::Malformed(match name {
            "formal charge" => "absent formal charge has a nonzero value",
            "isotope" => "absent isotope has a nonzero value",
            _ => "absent explicit hydrogens has a nonzero value",
        }))
    }
}

fn decode_bond(reader: &mut Reader<'_>) -> Result<MolBond, DecodeFailure> {
    let start = usize::try_from(reader.u32()?).expect("u32 fits usize");
    let end = usize::try_from(reader.u32()?).expect("u32 fits usize");
    let order = match u32::from(reader.u8()?) {
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE => BondOrder::Single,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE => BondOrder::Double,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE => BondOrder::Triple,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC => BondOrder::Aromatic,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED | FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE => {
            return Err(DecodeFailure::Malformed(
                "response has an unsupported bond type",
            ));
        }
        _ => {
            return Err(DecodeFailure::Malformed(
                "response has an unsupported bond type",
            ));
        }
    };
    let aromatic = bool_from_byte(reader.u8()?, "bond aromatic flag")?;
    if reader.u16()? != 0 {
        return Err(DecodeFailure::Malformed(
            "response bond reserved field is nonzero",
        ));
    }
    if order == BondOrder::Aromatic && !aromatic {
        return Err(DecodeFailure::Malformed(
            "aromatic bond type lacks aromatic flag",
        ));
    }
    if aromatic && matches!(order, BondOrder::Triple) {
        return Err(DecodeFailure::Malformed("aromatic triple bond is invalid"));
    }
    Ok(MolBond::new(start, end, order, aromatic))
}

fn bool_from_byte(value: u8, name: &'static str) -> Result<bool, DecodeFailure> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DecodeFailure::Malformed(name)),
    }
}

fn finish_response(
    input: &MolGraph,
    options: KekulizeOptions,
    response: DecodedResponse,
) -> Result<MolGraph, ChemistryError> {
    match response.status {
        FERRUM_CHEM_RESULT_OK => {
            if response.echoed_options != options_bits(options)
                || response.echoed_max_backtracks != options.max_backtracks()
            {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: "response did not echo the submitted options".to_owned(),
                });
            }
        }
        FERRUM_CHEM_RESULT_DEPICTION_FAILURE => {
            return Err(ChemistryError::KekulizationFailed {
                reason: response.detail,
            });
        }
        FERRUM_CHEM_RESULT_MALFORMED_REQUEST
        | FERRUM_CHEM_RESULT_INVALID_MOLECULE
        | FERRUM_CHEM_RESULT_INTERNAL_FAILURE => {
            return Err(ChemistryError::NativeRejected {
                status: response.status,
                reason: response.detail,
            });
        }
        _ => unreachable!("decoded response status is known"),
    }
    if !response.detail.is_empty() {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "successful response contains diagnostic detail".to_owned(),
        });
    }
    validate_output_semantics(input, &response.atoms, &response.bonds, options)?;
    MolGraph::new(response.atoms, response.bonds, input.coordinates().cloned()).map_err(|error| {
        ChemistryError::MalformedNativeResponse {
            reason: format!("response graph violates Ferrum invariants: {error}"),
        }
    })
}

fn validate_output_semantics(
    input: &MolGraph,
    output_atoms: &[MolAtom],
    output_bonds: &[MolBond],
    options: KekulizeOptions,
) -> Result<(), ChemistryError> {
    if input.atoms().len() != output_atoms.len() || input.bonds().len() != output_bonds.len() {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "response changed graph topology counts".to_owned(),
        });
    }
    for (index, (original, returned)) in input.atoms().iter().zip(output_atoms).enumerate() {
        if original.atomic_number() != returned.atomic_number()
            || original.formal_charge() != returned.formal_charge()
            || original.isotope() != returned.isotope()
            || original.explicit_hydrogens() != returned.explicit_hydrogens()
        {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed immutable atom facts at index {index}"),
            });
        }
        if options.clear_aromatic_flags() {
            if original.is_aromatic() && returned.is_aromatic() {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!("response retained aromatic atom flag at index {index}"),
                });
            }
            if !original.is_aromatic() && returned.is_aromatic() {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!("response changed non-aromatic atom flag at index {index}"),
                });
            }
        } else if original.is_aromatic() != returned.is_aromatic() {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed atom aromatic flag at index {index}"),
            });
        }
    }
    for (index, (original, returned)) in input.bonds().iter().zip(output_bonds).enumerate() {
        if original.start() != returned.start() || original.end() != returned.end() {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed bond endpoints at index {index}"),
            });
        }
        if original.order() == BondOrder::Aromatic {
            if !matches!(returned.order(), BondOrder::Single | BondOrder::Double) {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!("response did not kekulize aromatic bond at index {index}"),
                });
            }
            let expected_aromatic = !options.clear_aromatic_flags();
            if returned.is_aromatic() != expected_aromatic {
                return Err(ChemistryError::MalformedNativeResponse {
                    reason: format!(
                        "response has wrong aromatic flag for kekulized bond at index {index}"
                    ),
                });
            }
        } else if original.order() != returned.order()
            || original.is_aromatic() != returned.is_aromatic()
        {
            return Err(ChemistryError::MalformedNativeResponse {
                reason: format!("response changed non-aromatic bond at index {index}"),
            });
        }
    }
    Ok(())
}

fn decode_error(error: DecodeFailure) -> ChemistryError {
    match error {
        DecodeFailure::Malformed(reason) => ChemistryError::MalformedNativeResponse {
            reason: reason.to_owned(),
        },
        DecodeFailure::Truncated => ChemistryError::TruncatedNativeResponse,
        DecodeFailure::Trailing => ChemistryError::TrailingNativeResponse,
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecodeFailure {
    Malformed(&'static str),
    Truncated,
    Trailing,
}
#[cfg(test)]
mod native_engine_tests;
