use super::*;
use crate::UnavailableChemEngine;

fn text_response(output: &[u8]) -> Vec<u8> {
    let mut response = Vec::from(*b"FCT1");
    put_u32(&mut response, FERRUM_CHEM_TEXT_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_OK);
    put_u32(&mut response, 0);
    put_u32(
        &mut response,
        u32::try_from(output.len()).expect("small output fixture"),
    );
    put_u32(&mut response, FERRUM_CHEM_TEXT_FLAGS_NONE);
    response.extend_from_slice(output);
    response
}

#[test]
fn canonical_smiles_decoder_accepts_one_printable_ascii_line() {
    let response = text_response(b"[13CH3][NH3+]");

    assert_eq!(
        text_response::decode_smiles(&response),
        Ok("[13CH3][NH3+]".to_owned())
    );
}

#[test]
fn canonical_smiles_decoder_rejects_whitespace_non_ascii_and_oversize() {
    for output in [b"C C".as_slice(), b"C\tC", b"C\nC", &[b'C', 0xc2, 0xb5]] {
        assert!(matches!(
            text_response::decode_smiles(&text_response(output)),
            Err(ChemistryError::MalformedNativeResponse { .. })
        ));
    }

    let mut response = text_response(b"C");
    let oversized =
        u32::try_from(FERRUM_CHEM_SMILES_WRITE_MAX_BYTES + 1).expect("ABI maximum fits u32");
    response[16..20].copy_from_slice(&oversized.to_le_bytes());
    assert!(matches!(
        text_response::decode_smiles(&response),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn smiles_output_limit_fills_the_bounded_text_response_envelope() {
    assert_eq!(
        NATIVE_SMILES_MAX_OUTPUT_BYTES + FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES,
        FERRUM_CHEM_MAX_RESPONSE_BYTES
    );
}

#[test]
fn unavailable_engine_reports_only_the_missing_smiles_writer() {
    let molecule = MolGraph::new(
        vec![
            MolAtom::new(
                AtomicNumber::from_symbol("C").expect("carbon"),
                None,
                None,
                None,
                false,
            )
            .expect("atom"),
        ],
        Vec::new(),
        None,
    )
    .expect("graph");

    assert!(matches!(
        UnavailableChemEngine.molecule_to_smiles(&molecule),
        Err(ChemistryError::OperationUnavailable {
            operation: "molecule_to_smiles"
        })
    ));
}
