use super::*;
use crate::adapter_contract::{
    FERRUM_CHEM_BOND_DIRECTION_BEGINDASH, FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE,
    FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT, FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT,
    FERRUM_CHEM_BOND_DIRECTION_NONE, FERRUM_CHEM_BOND_DIRECTION_OTHER, FERRUM_CHEM_BOND_STEREO_ANY,
    FERRUM_CHEM_BOND_STEREO_CIS, FERRUM_CHEM_BOND_STEREO_E, FERRUM_CHEM_BOND_STEREO_NONE,
    FERRUM_CHEM_BOND_STEREO_OTHER, FERRUM_CHEM_BOND_STEREO_TRANS, FERRUM_CHEM_BOND_STEREO_Z,
    FERRUM_CHEM_CHIRAL_OTHER, FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW,
    FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW, FERRUM_CHEM_CHIRAL_UNSPECIFIED,
    FERRUM_CHEM_MOLECULE_FLAGS_NONE, FERRUM_CHEM_MOLECULE_RESERVED,
    FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE, FERRUM_CHEM_MOLECULE_WIRE_VERSION,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Coordinates, Point2, SdfProperty, SdfRecord};

fn aromatic_carbon() -> MolAtom {
    MolAtom::new(
        AtomicNumber::try_from(6).expect("carbon"),
        Some(-1),
        Some(13),
        Some(1),
        true,
    )
    .expect("valid atom")
}

fn graph() -> MolGraph {
    MolGraph::new(
        vec![aromatic_carbon(), aromatic_carbon()],
        vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
        Some(Coordinates::new(vec![
            Point2::new(1.0, 2.0).expect("finite"),
            Point2::new(-3.0, 4.0).expect("finite"),
        ])),
    )
    .expect("valid graph")
}

#[test]
fn request_codec_has_a_stable_golden_layout() {
    let bytes = encode_kekulize_request(&graph(), KekulizeOptions::default()).expect("encodes");
    let expected = [
        b'F', b'C', b'K', b'1', 1, 0, 0, 0, 2, 0, 0, 0, 100, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 6, 1,
        7, 0, 255, 255, 255, 255, 13, 0, 1, 0, 6, 1, 7, 0, 255, 255, 255, 255, 13, 0, 1, 0, 0, 0,
        0, 0, 1, 0, 0, 0, 4, 1, 0, 0,
    ];
    assert_eq!(bytes, expected);
}

#[test]
fn depiction_request_has_no_kekulization_controls() {
    let request = encode_depiction_request(&graph()).expect("depiction request encodes");
    assert_eq!(&request[8..12], &[0, 0, 0, 0]);
    assert_eq!(&request[12..16], &[1, 0, 0, 0]);
}

#[test]
fn direct_native_loader_failure_is_detail_free() {
    let hostile_path = "/private/ferrum/.dylibs/libferrum_chem.dylib: loader diagnostic";
    let error = match NativeChemEngine::load(Path::new(hostile_path)) {
        Ok(_) => panic!("a hostile nonexistent native-library path must not load"),
        Err(error) => error,
    };

    let reason = match &error {
        ChemistryError::NativeBoundary { reason } => reason,
        other => panic!("loader failure must remain a native-boundary error, got {other:?}"),
    };
    assert_eq!(reason, NATIVE_ADAPTER_BOUNDARY_REASON);
    for forbidden in [
        "/private/ferrum",
        ".dylibs",
        "libferrum_chem",
        "loader diagnostic",
    ] {
        assert!(
            !reason.contains(forbidden)
                && !error.to_string().contains(forbidden)
                && !format!("{error:?}").contains(forbidden),
            "public loader failure leaked private adapter detail {forbidden:?}"
        );
    }
}

#[test]
fn direct_smarts_boundary_failures_are_closed_and_detail_free() {
    let error = super::smarts_adapter_error(super::AdapterError::NativeStatus { status: u32::MAX });
    assert_eq!(
        error,
        ChemistryError::SmartsMatchUnavailable {
            reason: SmartsMatchUnavailableReason::NativeCallFailed,
        }
    );
    let rendered = format!("{error:?} {error}");
    for forbidden in ["FCQ1", "FQM1", "native detail", "libferrum_chem"] {
        assert!(
            !rendered.contains(forbidden),
            "SMARTS error leaked {forbidden:?}"
        );
    }
}

#[test]
fn complete_graph_request_preserves_codec_facts_and_omits_coordinates() {
    let atom = MolAtom::from_native(
        AtomicNumber::try_from(6).expect("carbon"),
        -1,
        13,
        1,
        false,
        AtomChirality::TetrahedralCw,
        2,
        true,
        7,
    )
    .expect("complete atom");
    let molecule = MolGraph::new(vec![atom], vec![], None).expect("complete graph");

    let request = graph_wire::encode(&molecule).expect("complete graph encodes");

    assert_eq!(&request[..4], b"FCG1");
    assert_eq!(
        request.len(),
        FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES + FERRUM_CHEM_GRAPH_ATOM_BYTES
    );
    assert_eq!(&request[20..24], &[6, 0, 1, 0]);
    assert_eq!(&request[24..28], &[7, 0, 0, 0]);
    assert_eq!(&request[28..32], &(-1_i32).to_le_bytes());
    assert_eq!(&request[32..36], &[13, 0, 1, 0]);
    assert_eq!(&request[36..44], &[2, 1, 0, 0, 7, 0, 0, 0]);
}

#[test]
fn molblock_request_binds_explicit_format_and_atom_aligned_coordinates() {
    let request = molblock_wire::encode(&graph(), MolblockVersion::V3000)
        .expect("coordinate-bearing graph encodes");

    assert_eq!(&request[..4], b"FCB1");
    assert_eq!(
        &request[4..8],
        &FERRUM_CHEM_MOLBLOCK_WIRE_VERSION.to_le_bytes()
    );
    assert_eq!(
        &request[8..12],
        &FERRUM_CHEM_MOLBLOCK_FORMAT_V3000.to_le_bytes()
    );
    assert_eq!(&request[12..16], &2_u32.to_le_bytes());
    assert_eq!(&request[16..20], &1_u32.to_le_bytes());
    assert_eq!(
        request.len(),
        FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES
            + 2 * FERRUM_CHEM_GRAPH_ATOM_BYTES
            + FERRUM_CHEM_GRAPH_BOND_BYTES
            + 2 * FERRUM_CHEM_COORDINATE_BYTES
    );
    assert_eq!(
        &request[request.len() - 32..request.len() - 24],
        &1.0_f64.to_le_bytes()
    );
    assert_eq!(
        &request[request.len() - 24..request.len() - 16],
        &2.0_f64.to_le_bytes()
    );
    assert_eq!(
        &request[request.len() - 16..request.len() - 8],
        &(-3.0_f64).to_le_bytes()
    );
    assert_eq!(&request[request.len() - 8..], &4.0_f64.to_le_bytes());
}

#[test]
fn titled_molblock_request_wraps_exact_graph_and_validated_utf8_title() {
    let title = "authored \u{03b2}-lactam";
    let molecule = molblock_wire::encode(&graph(), MolblockVersion::V2000)
        .expect("plain molecule request encodes");
    let request = molblock_wire::encode_titled(&graph(), MolblockVersion::V2000, title)
        .expect("titled molecule request encodes");

    assert_eq!(&request[..4], b"FBT1");
    assert_eq!(
        &request[4..8],
        &FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION.to_le_bytes()
    );
    assert_eq!(&request[8..12], &(molecule.len() as u32).to_le_bytes());
    assert_eq!(&request[12..16], &(title.len() as u32).to_le_bytes());
    assert_eq!(
        &request[FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES
            ..FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES + molecule.len()],
        molecule
    );
    assert_eq!(&request[request.len() - title.len()..], title.as_bytes());
    molblock_wire::validate_output_title("authored \u{03b2}-lactam\nbody\n", title)
        .expect("exact returned title is accepted");
    molblock_wire::validate_output_title("\nbody\n", "")
        .expect("an exact returned empty title is accepted");
    assert!(matches!(
        molblock_wire::validate_output_title("substituted\nbody\n", title),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
    for invalid in ["nul\0title", "two\nlines", "carriage\rreturn"] {
        assert!(matches!(
            molblock_wire::encode_titled(&graph(), MolblockVersion::V2000, invalid),
            Err(ChemistryError::CodecFailed {
                codec: "molblock",
                ..
            })
        ));
    }
}

#[test]
fn molblock_request_rejects_a_graph_without_complete_coordinates() {
    let molecule = MolGraph::new(vec![aromatic_carbon()], vec![], None).expect("valid graph");

    assert!(matches!(
        molblock_wire::encode(&molecule, MolblockVersion::V2000),
        Err(ChemistryError::CodecFailed {
            codec: "molblock",
            ..
        })
    ));
}

#[test]
fn sdf_request_preserves_record_title_property_order_and_explicit_version() {
    let records = vec![
        SdfRecord::new(
            graph(),
            "first record",
            vec![
                SdfProperty::new("second", "line one\nline two").expect("valid property"),
                SdfProperty::new("first", "").expect("valid empty property"),
            ],
        )
        .expect("valid SDF record"),
        SdfRecord::new(graph(), "second record", Vec::new()).expect("valid record"),
    ];

    let request = sdf_wire::encode(&records, MolblockVersion::V3000).expect("SDF encodes");

    assert_eq!(&request[..4], b"FSD1");
    assert_eq!(&request[4..8], &FERRUM_CHEM_SDF_WIRE_VERSION.to_le_bytes());
    assert_eq!(&request[8..12], &2_u32.to_le_bytes());
    assert_eq!(
        &request[FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES + FERRUM_CHEM_SDF_RECORD_HEADER_BYTES..][..4],
        b"FCB1",
    );
    let first_molecule =
        &request[FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES + FERRUM_CHEM_SDF_RECORD_HEADER_BYTES..];
    assert_eq!(
        &first_molecule[8..12],
        &FERRUM_CHEM_MOLBLOCK_FORMAT_V3000.to_le_bytes(),
    );
    let second_name = request
        .windows("second".len())
        .position(|window| window == b"second")
        .expect("first property name is encoded");
    let first_name = request
        .windows("first".len())
        .rposition(|window| window == b"first")
        .expect("second property name is encoded");
    assert!(second_name < first_name);
}

#[test]
fn sdf_import_response_preserves_record_order_and_duplicate_property_names() {
    let mut response = Vec::from(*b"FSI1");
    put_u32(&mut response, FERRUM_CHEM_SDF_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_OK);
    put_u32(&mut response, 0);
    put_u32(&mut response, 2);
    put_u32(&mut response, FERRUM_CHEM_SDF_FLAGS_NONE);
    append_import_record(
        &mut response,
        "first",
        &[
            ("same", "one"),
            ("same", "two"),
            ("note", "line one\nline two"),
        ],
    );
    append_import_record(&mut response, "second", &[("empty", "")]);

    let records = sdf_import::decode(&response).expect("valid imported records decode");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].title(), "first");
    assert_eq!(records[1].title(), "second");
    assert_eq!(
        records[0].molecule().canonical_smiles(),
        "[13CH3][C@H](F)[C:9](=O)[O-]"
    );
    assert_eq!(
        records[0]
            .properties()
            .iter()
            .map(|property| (property.name(), property.value()))
            .collect::<Vec<_>>(),
        vec![
            ("same", "one"),
            ("same", "two"),
            ("note", "line one\nline two"),
        ],
    );
}

#[test]
fn sdf_import_rejects_invalid_input_and_hostile_response_envelopes() {
    assert!(matches!(
        sdf_import::validate_input(""),
        Err(ChemistryError::InvalidSdfInput { .. })
    ));
    assert!(matches!(
        sdf_import::validate_input("record\0$$$$"),
        Err(ChemistryError::InvalidSdfInput { .. })
    ));

    let mut response = Vec::from(*b"FSI1");
    put_u32(&mut response, FERRUM_CHEM_SDF_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_OK);
    put_u32(&mut response, 0);
    put_u32(&mut response, FERRUM_CHEM_SDF_MAX_RECORDS + 1);
    put_u32(&mut response, FERRUM_CHEM_SDF_FLAGS_NONE);
    assert!(matches!(
        sdf_import::decode(&response),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    response[16..20].copy_from_slice(&1_u32.to_le_bytes());
    append_import_record(&mut response, "record", &[]);
    response.push(0);
    assert_eq!(
        sdf_import::decode(&response),
        Err(ChemistryError::TrailingNativeResponse),
    );
}

#[test]
fn complete_graph_request_rejects_unreconstructable_other_enums() {
    let atom = MolAtom::from_native(
        AtomicNumber::try_from(6).expect("carbon"),
        0,
        0,
        0,
        false,
        AtomChirality::Other,
        0,
        false,
        0,
    )
    .expect("complete atom");
    let molecule = MolGraph::new(vec![atom], vec![], None).expect("complete graph");

    assert!(matches!(
        graph_wire::encode(&molecule),
        Err(ChemistryError::UnsupportedNativeRequest { .. })
    ));
}

#[test]
fn text_response_requires_one_bounded_utf8_line() {
    let mut response = Vec::from(*b"FCT1");
    put_u32(&mut response, FERRUM_CHEM_TEXT_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_OK);
    put_u32(&mut response, 0);
    put_u32(&mut response, 6);
    put_u32(&mut response, FERRUM_CHEM_TEXT_FLAGS_NONE);
    response.extend_from_slice(b"[#6]-O");

    assert_eq!(
        text_response::decode(&response, "SMARTS"),
        Ok("[#6]-O".to_owned())
    );

    response[16..20].copy_from_slice(&7_u32.to_le_bytes());
    response.push(b'\n');
    assert!(matches!(
        text_response::decode(&response, "SMARTS"),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn multiline_text_response_is_reserved_for_record_codecs() {
    let output = "Ferrum\n  RDKit          2D\n\n  0  0  0  0  0  0  0  0  0  0999 V2000\nM  END\n";
    let mut response = Vec::from(*b"FCT1");
    put_u32(&mut response, FERRUM_CHEM_TEXT_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_OK);
    put_u32(&mut response, 0);
    put_u32(
        &mut response,
        u32::try_from(output.len()).expect("small fixture"),
    );
    put_u32(&mut response, FERRUM_CHEM_TEXT_FLAGS_NONE);
    response.extend_from_slice(output.as_bytes());

    assert_eq!(
        text_response::decode_multiline(&response, "molblock", NativeTextOutputLimit::ADAPTER_MAXIMUM),
        Ok(output.to_owned())
    );
    assert!(matches!(
        text_response::decode(&response, "SMARTS"),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn failed_text_response_is_a_typed_codec_error() {
    let detail = b"not representable";
    let mut response = Vec::from(*b"FCT1");
    put_u32(&mut response, FERRUM_CHEM_TEXT_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE);
    put_u32(
        &mut response,
        u32::try_from(detail.len()).expect("small detail"),
    );
    put_u32(&mut response, 0);
    put_u32(&mut response, FERRUM_CHEM_TEXT_FLAGS_NONE);
    response.extend_from_slice(detail);

    assert!(matches!(
        text_response::decode(&response, "SMARTS"),
        Err(ChemistryError::CodecFailed {
            codec: "SMARTS",
            ..
        })
    ));
}

#[test]
fn bond_type_constants_preserve_the_v1_wire_vocabulary() {
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED, 0);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE, 1);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE, 2);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE, 3);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC, 4);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE, 5);
}

#[test]
fn fcm1_vocabulary_is_generated_from_the_public_header() {
    assert_eq!(FERRUM_CHEM_MOLECULE_WIRE_VERSION, 1);
    assert_eq!(FERRUM_CHEM_MOLECULE_FLAGS_NONE, 0);
    assert_eq!(FERRUM_CHEM_MOLECULE_RESERVED, 0);
    assert_eq!(FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE, u32::MAX);
    assert_eq!(FERRUM_CHEM_CHIRAL_UNSPECIFIED, 0);
    assert_eq!(FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW, 1);
    assert_eq!(FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW, 2);
    assert_eq!(FERRUM_CHEM_CHIRAL_OTHER, 3);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_NONE, 0);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_ANY, 1);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_Z, 2);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_E, 3);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_CIS, 4);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_TRANS, 5);
    assert_eq!(FERRUM_CHEM_BOND_STEREO_OTHER, 6);
    assert_eq!(FERRUM_CHEM_BOND_DIRECTION_NONE, 0);
    assert_eq!(FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE, 1);
    assert_eq!(FERRUM_CHEM_BOND_DIRECTION_BEGINDASH, 2);
    assert_eq!(FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT, 3);
    assert_eq!(FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT, 4);
    assert_eq!(FERRUM_CHEM_BOND_DIRECTION_OTHER, 5);
}

#[test]
fn response_codec_rejects_truncated_and_trailing_bytes() {
    assert!(matches!(
        decode_response(&[0; 3]),
        Err(DecodeFailure::Truncated)
    ));
    let mut response = success_response(&graph(), KekulizeOptions::default());
    response.push(0);
    assert!(matches!(
        decode_response(&response),
        Err(DecodeFailure::Trailing)
    ));
}

#[test]
fn response_codec_preflights_declared_records_before_allocating() {
    let mut response = success_response(&graph(), KekulizeOptions::default());
    response[24..28].copy_from_slice(&FERRUM_CHEM_KEKULIZE_MAX_ATOMS.to_le_bytes());
    assert!(matches!(
        decode_response(&response),
        Err(DecodeFailure::Truncated)
    ));
}

#[test]
fn response_identity_validation_preserves_coordinates_and_optional_facts() {
    let input = graph();
    let decoded =
        decode_response(&success_response(&input, KekulizeOptions::default())).expect("decode");
    let output = finish_response(&input, KekulizeOptions::default(), decoded).expect("accept");
    assert_eq!(output.coordinates(), input.coordinates());
    assert_eq!(output.atoms()[0].formal_charge(), Some(-1));
    assert_eq!(output.atoms()[0].isotope(), Some(13));
    assert_eq!(output.atoms()[0].explicit_hydrogens(), Some(1));
}

#[test]
fn kekulize_failure_is_typed() {
    let decoded = decode_response(&error_response(
        FERRUM_CHEM_RESULT_DEPICTION_FAILURE,
        "cannot assign bonds",
    ))
    .expect("decode");
    assert_eq!(
        finish_response(&graph(), KekulizeOptions::default(), decoded),
        Err(ChemistryError::KekulizationFailed {
            reason: "cannot assign bonds".to_owned()
        })
    );
}

#[test]
fn coordinate_response_rejects_an_unknown_result_status() {
    assert!(matches!(
        decode_coordinate_response(
            &coordinate_error_response(u32::MAX, "unrecognized status"),
            graph().atoms().len()
        ),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn fcm1_rejects_unknown_status_and_oversized_declared_fields() {
    let mut unknown_status = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    unknown_status[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        fcm1::decode(&unknown_status),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut oversized_detail = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    oversized_detail[12..16]
        .copy_from_slice(&(FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES as u32 + 1).to_le_bytes());
    assert!(matches!(
        fcm1::decode(&oversized_detail),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut oversized_smiles = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    oversized_smiles[16..20]
        .copy_from_slice(&(FERRUM_CHEM_SMILES_MAX_BYTES as u32 + 1).to_le_bytes());
    assert!(matches!(
        fcm1::decode(&oversized_smiles),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut oversized_atoms = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    oversized_atoms[20..24].copy_from_slice(&(FERRUM_CHEM_KEKULIZE_MAX_ATOMS + 1).to_le_bytes());
    assert!(matches!(
        fcm1::decode(&oversized_atoms),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn fcm1_rejects_nul_input_and_response_text() {
    assert!(matches!(
        fcm1::validate_input("C\0O"),
        Err(ChemistryError::InvalidSmilesInput { .. })
    ));
    let oversized = "C".repeat(FERRUM_CHEM_SMILES_MAX_BYTES + 1);
    assert!(matches!(
        fcm1::validate_input(&oversized),
        Err(ChemistryError::InvalidSmilesInput { .. })
    ));

    let mut detail_nul = fcm1_header(FERRUM_CHEM_RESULT_INVALID_MOLECULE, 1, 0, 0, 0);
    detail_nul.push(0);
    assert!(matches!(
        fcm1::decode(&detail_nul),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
    let mut canonical_nul = fcm1_header(FERRUM_CHEM_RESULT_OK, 0, 1, 0, 0);
    canonical_nul.push(0);
    assert!(matches!(
        fcm1::decode(&canonical_nul),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn fcm1_preserves_complete_ordered_molecule_facts() {
    let response = fcm1_molecule_response();
    let molecule = fcm1::decode(&response).expect("complete FCM1 molecule decodes");
    assert_eq!(molecule.canonical_smiles(), "[13CH3][C@H](F)[C:9](=O)[O-]");
    let graph = molecule.molecule();
    assert_eq!(graph.atoms().len(), 6);
    assert_eq!(graph.bonds().len(), 5);
    assert_eq!(graph.atoms()[0].isotope(), Some(13));
    assert_eq!(graph.atoms()[1].chirality(), AtomChirality::TetrahedralCw);
    assert_eq!(graph.atoms()[3].atom_map_number(), Some(9));
    assert_eq!(graph.atoms()[5].formal_charge(), Some(-1));
    assert_eq!(graph.bonds()[1].order(), BondOrder::Single);
    assert_eq!(
        graph
            .coordinates()
            .expect("complete coordinates")
            .points()
            .len(),
        6
    );
}

#[test]
fn fcm1_rejects_reserved_and_invalid_graph_semantics() {
    let mut reserved = fcm1_molecule_response();
    let atom_offset = 32 + "[13CH3][C@H](F)[C:9](=O)[O-]".len();
    reserved[atom_offset + 3] = 1;
    assert!(matches!(
        fcm1::decode(&reserved),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut duplicate = fcm1_molecule_response();
    let bond_offset = atom_offset + 6 * FERRUM_CHEM_MOLECULE_ATOM_BYTES;
    duplicate[bond_offset + FERRUM_CHEM_MOLECULE_BOND_BYTES
        ..bond_offset + FERRUM_CHEM_MOLECULE_BOND_BYTES + 4]
        .copy_from_slice(&0_u32.to_le_bytes());
    duplicate[bond_offset + FERRUM_CHEM_MOLECULE_BOND_BYTES + 4
        ..bond_offset + FERRUM_CHEM_MOLECULE_BOND_BYTES + 8]
        .copy_from_slice(&1_u32.to_le_bytes());
    assert!(matches!(
        fcm1::decode(&duplicate),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    for direction in [
        FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE,
        FERRUM_CHEM_BOND_DIRECTION_BEGINDASH,
    ] {
        let mut directed_double = fcm1_molecule_response();
        directed_double[bond_offset + 8] = 2;
        directed_double[bond_offset + 11] = direction as u8;
        assert!(matches!(
            fcm1::decode(&directed_double),
            Err(ChemistryError::MalformedNativeResponse { .. })
        ));
    }
}
