use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Coordinates, Point2};

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
fn bond_type_constants_preserve_the_v1_wire_vocabulary() {
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED, 0);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE, 1);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE, 2);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE, 3);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC, 4);
    assert_eq!(FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE, 5);
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
        FERRUM_CHEM_RESULT_KEKULIZE_FAILURE,
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
fn smiles_response_rejects_unknown_status_and_oversized_declared_fields() {
    let mut unknown_status = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    unknown_status[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        decode_smiles_response(&unknown_status),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut oversized_detail = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    oversized_detail[12..16]
        .copy_from_slice(&(FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES as u32 + 1).to_le_bytes());
    assert!(matches!(
        decode_smiles_response(&oversized_detail),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut oversized_smiles = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    oversized_smiles[16..20]
        .copy_from_slice(&(FERRUM_CHEM_SMILES_MAX_BYTES as u32 + 1).to_le_bytes());
    assert!(matches!(
        decode_smiles_response(&oversized_smiles),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));

    let mut oversized_atoms = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    oversized_atoms[20..24].copy_from_slice(&(FERRUM_CHEM_KEKULIZE_MAX_ATOMS + 1).to_le_bytes());
    assert!(matches!(
        decode_smiles_response(&oversized_atoms),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn smiles_contract_rejects_nul_input_and_response_text() {
    assert!(matches!(
        validate_smiles_input("C\0O"),
        Err(ChemistryError::InvalidSmilesInput { .. })
    ));
    let oversized = "C".repeat(FERRUM_CHEM_SMILES_MAX_BYTES + 1);
    assert!(matches!(
        validate_smiles_input(&oversized),
        Err(ChemistryError::InvalidSmilesInput { .. })
    ));

    let mut detail_nul = smiles_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE, "x", "", 0);
    detail_nul[24] = 0;
    assert!(matches!(
        decode_smiles_response(&detail_nul),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
    let mut canonical_nul = smiles_response(FERRUM_CHEM_RESULT_OK, "", "C", 1);
    canonical_nul[24] = 0;
    assert!(matches!(
        decode_smiles_response(&canonical_nul),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn hostile_fcs1_responses_release_the_native_owner_exactly_once() {
    let fixture = HostileSmilesAdapter::build();
    let engine =
        NativeChemEngine::load(fixture.library_path()).expect("hostile test adapter loads");
    for (index, selector) in ["A", "B", "C", "D", "E", "F", "G", "H"]
        .into_iter()
        .enumerate()
    {
        assert!(matches!(
            engine.smiles_to_2d(selector),
            Err(ChemistryError::MalformedNativeResponse { .. })
        ));
        let count = engine
            .smiles_to_2d("Q")
            .expect("counter probe is a valid FCS1 response")
            .canonical_smiles()
            .to_owned();
        let expected = char::from_u32(u32::from(b'A') + u32::try_from(index * 2).expect("small"))
            .expect("ASCII count marker")
            .to_string();
        assert_eq!(
            count, expected,
            "{selector} response must release exactly once"
        );
    }
}

struct HostileSmilesAdapter {
    directory: PathBuf,
    library: PathBuf,
}

impl HostileSmilesAdapter {
    fn build() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "ferrum-hostile-fcs1-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&directory).expect("create hostile adapter directory");
        let source = directory.join("hostile_adapter.c");
        let library = directory.join(hostile_library_name());
        fs::write(&source, HOSTILE_FCS1_ADAPTER).expect("write hostile adapter source");
        let mut compiler = Command::new("cc");
        hostile_shared_library_flags(&mut compiler);
        let output = compiler
            .arg(&source)
            .args(["-o"])
            .arg(&library)
            .output()
            .expect("run C compiler for hostile adapter");
        assert!(
            output.status.success(),
            "compile hostile adapter: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { directory, library }
    }

    fn library_path(&self) -> &Path {
        &self.library
    }
}

impl Drop for HostileSmilesAdapter {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[cfg(target_os = "macos")]
fn hostile_shared_library_flags(compiler: &mut Command) {
    compiler.arg("-dynamiclib");
}

#[cfg(target_os = "linux")]
fn hostile_shared_library_flags(compiler: &mut Command) {
    compiler.args(["-shared", "-fPIC"]);
}

fn hostile_library_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "libhostile_fcs1.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "libhostile_fcs1.so"
    }
}

const HOSTILE_FCS1_ADAPTER: &str = r#"
#include <stdint.h>
typedef struct { uint8_t *data; uint64_t len; } owner;
static uint8_t output[64]; static uint32_t releases;
static void u32le(uint32_t offset, uint32_t value) {
  output[offset]=value; output[offset+1]=value>>8; output[offset+2]=value>>16; output[offset+3]=value>>24;
}
static void fcs1(uint32_t status, uint32_t detail, uint32_t smiles, uint32_t atoms, uint64_t *len) {
  output[0]='F'; output[1]='C'; output[2]='S'; output[3]='1'; u32le(4,1); u32le(8,status);
  u32le(12,detail); u32le(16,smiles); u32le(20,atoms); *len=24;
}
uint32_t ferrum_chem_abi_version(void) { return 3; }
uint64_t ferrum_chem_capabilities_v1(void) { return 7; }
uint32_t ferrum_chem_kekulize_v1(const uint8_t *r,uint64_t n,owner *o) { (void)r;(void)n;o->data=0;o->len=0;return 0; }
uint32_t ferrum_chem_generate_2d_v1(const uint8_t *r,uint64_t n,owner *o) { (void)r;(void)n;o->data=0;o->len=0;return 0; }
uint32_t ferrum_chem_smiles_to_2d_v1(const uint8_t *r,uint64_t n,owner *o) {
  uint64_t len; uint8_t kind = n ? r[0] : 0; fcs1(0,0,0,0,&len);
  if (kind=='A') fcs1(99,0,0,0,&len);
  if (kind=='B') { fcs1(2,1,0,0,&len); output[24]=0xff; len=25; }
  if (kind=='C') { fcs1(2,1,0,0,&len); output[24]=0; len=25; }
  if (kind=='D') fcs1(2,4097,0,0,&len);
  if (kind=='E') fcs1(0,0,1048577,0,&len);
  if (kind=='F') fcs1(0,0,1,1000001,&len);
  if (kind=='G') { fcs1(2,0,1,0,&len); output[24]='C'; len=25; }
  if (kind=='H') { fcs1(0,0,1,1,&len); output[24]='C'; len=25; }
  if (kind=='Q') { fcs1(0,0,1,0,&len); output[24]=(uint8_t)('A'+releases-1); len=25; }
  o->data=output; o->len=len; return 0;
}
void ferrum_chem_owned_buffer_free_v1(owner *o) { releases++; o->data=0; o->len=0; }
"#;

#[test]
fn response_semantics_reject_aromatic_contract_mutations() {
    let input = graph();
    let options = KekulizeOptions::default();
    let mut wrong_order = success_response(&input, options);
    let bond_offset = RESPONSE_HEADER_LENGTH + input.atoms().len() * ATOM_LENGTH;
    wrong_order[bond_offset + 8] = wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC);
    assert!(matches!(
        finish_response(
            &input,
            options,
            decode_response(&wrong_order).expect("decode")
        ),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
    let mut wrong_atom_flag = success_response(&input, options);
    wrong_atom_flag[RESPONSE_HEADER_LENGTH + 1] = 0;
    assert!(matches!(
        finish_response(
            &input,
            options,
            decode_response(&wrong_atom_flag).expect("decode")
        ),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

#[test]
fn clear_aromatic_flags_requires_cleared_atom_and_bond_flags() {
    let input = graph();
    let options = KekulizeOptions::new(true, true, 100).expect("valid options");
    assert!(
        finish_response(
            &input,
            options,
            decode_response(&success_response(&input, options)).expect("decode")
        )
        .is_ok()
    );
    let mut wrong_bond_flag = success_response(&input, options);
    let bond_offset = RESPONSE_HEADER_LENGTH + input.atoms().len() * ATOM_LENGTH;
    wrong_bond_flag[bond_offset + 9] = 1;
    assert!(matches!(
        finish_response(
            &input,
            options,
            decode_response(&wrong_bond_flag).expect("decode")
        ),
        Err(ChemistryError::MalformedNativeResponse { .. })
    ));
}

fn success_response(graph: &MolGraph, options: KekulizeOptions) -> Vec<u8> {
    let request = encode_kekulize_request(graph, options).expect("request");
    let mut response = Vec::new();
    response.extend_from_slice(&RESPONSE_MAGIC);
    put_u32(&mut response, FERRUM_CHEM_KEKULIZE_WIRE_VERSION);
    put_u32(&mut response, FERRUM_CHEM_RESULT_OK);
    put_u32(&mut response, 0);
    put_u32(&mut response, options_bits(options));
    put_u32(&mut response, options.max_backtracks());
    put_u32(
        &mut response,
        u32::try_from(graph.atoms().len()).expect("count"),
    );
    put_u32(
        &mut response,
        u32::try_from(graph.bonds().len()).expect("count"),
    );
    response.extend_from_slice(&request[REQUEST_HEADER_LENGTH..]);
    for index in 0..graph.bonds().len() {
        if graph.bonds()[index].order() == BondOrder::Aromatic {
            let offset = RESPONSE_HEADER_LENGTH
                + graph.atoms().len() * ATOM_LENGTH
                + index * BOND_LENGTH
                + 8;
            response[offset] = wire_bond_type(FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE);
        }
    }
    if options.clear_aromatic_flags() {
        for index in 0..graph.atoms().len() {
            response[RESPONSE_HEADER_LENGTH + index * ATOM_LENGTH + 1] = 0;
        }
        for index in 0..graph.bonds().len() {
            if graph.bonds()[index].order() == BondOrder::Aromatic {
                let offset = RESPONSE_HEADER_LENGTH
                    + graph.atoms().len() * ATOM_LENGTH
                    + index * BOND_LENGTH
                    + 9;
                response[offset] = 0;
            }
        }
    }
    response
}

fn error_response(status: u32, detail: &str) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&RESPONSE_MAGIC);
    put_u32(&mut response, FERRUM_CHEM_KEKULIZE_WIRE_VERSION);
    put_u32(&mut response, status);
    put_u32(
        &mut response,
        u32::try_from(detail.len()).expect("detail count"),
    );
    for _ in 0..4 {
        put_u32(&mut response, 0);
    }
    response.extend_from_slice(detail.as_bytes());
    response
}

fn coordinate_error_response(status: u32, detail: &str) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&COORDINATE_RESPONSE_MAGIC);
    put_u32(&mut response, 1);
    put_u32(&mut response, status);
    put_u32(
        &mut response,
        u32::try_from(detail.len()).expect("detail count"),
    );
    put_u32(&mut response, 0);
    response.extend_from_slice(detail.as_bytes());
    response
}

fn smiles_response(status: u32, detail: &str, canonical_smiles: &str, atom_count: u32) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&SMILES_RESPONSE_MAGIC);
    put_u32(&mut response, 1);
    put_u32(&mut response, status);
    put_u32(
        &mut response,
        u32::try_from(detail.len()).expect("test detail length fits protocol"),
    );
    put_u32(
        &mut response,
        u32::try_from(canonical_smiles.len()).expect("test SMILES length fits protocol"),
    );
    put_u32(&mut response, atom_count);
    response.extend_from_slice(detail.as_bytes());
    response.extend_from_slice(canonical_smiles.as_bytes());
    for _ in 0..atom_count {
        response.extend_from_slice(&0.0_f64.to_le_bytes());
        response.extend_from_slice(&0.0_f64.to_le_bytes());
    }
    response
}
