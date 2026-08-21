#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn hostile_fcm1_responses_release_the_native_owner_exactly_once() {
    let fixture = HostileSmilesAdapter::build();
    let engine =
        NativeChemEngine::load(fixture.library_path()).expect("hostile test adapter loads");
    for (index, selector) in ["A", "B", "C", "D", "E", "F", "G", "H"]
        .into_iter()
        .enumerate()
    {
        assert!(matches!(
            engine.smiles_to_molecule(selector),
            Err(ChemistryError::MalformedNativeResponse { .. })
        ));
        let count = engine
            .smiles_to_molecule("Q")
            .expect("counter probe is a valid FCM1 response")
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
            "ferrum-hostile-fcm1-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&directory).expect("create hostile adapter directory");
        let source = directory.join("hostile_adapter.c");
        let library = directory.join(hostile_library_name());
        fs::write(&source, HOSTILE_FCM1_ADAPTER).expect("write hostile adapter source");
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
        "libhostile_fcm1.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "libhostile_fcm1.so"
    }
}

const HOSTILE_FCM1_ADAPTER: &str = r#"
#include <stdint.h>
typedef struct { uint8_t *data; uint64_t len; } owner;
static uint8_t output[128]; static uint32_t releases;
static void u32le(uint32_t offset, uint32_t value) {
  output[offset]=value; output[offset+1]=value>>8; output[offset+2]=value>>16; output[offset+3]=value>>24;
}
static void fcm1(uint32_t status, uint32_t detail, uint32_t smiles, uint32_t atoms, uint32_t bonds, uint64_t *len) {
  output[0]='F'; output[1]='C'; output[2]='M'; output[3]='1'; u32le(4,1); u32le(8,status);
  u32le(12,detail); u32le(16,smiles); u32le(20,atoms); u32le(24,bonds); u32le(28,0); *len=32;
}
uint32_t ferrum_chem_abi_version(void) { return 5; }
uint64_t ferrum_chem_capabilities_v1(void) { return 7; }
uint32_t ferrum_chem_kekulize_v1(const uint8_t *r,uint64_t n,owner *o) { (void)r;(void)n;o->data=0;o->len=0;return 0; }
uint32_t ferrum_chem_generate_2d_v1(const uint8_t *r,uint64_t n,owner *o) { (void)r;(void)n;o->data=0;o->len=0;return 0; }
uint32_t ferrum_chem_smiles_to_molecule_v1(const uint8_t *r,uint64_t n,owner *o) {
  uint64_t len; uint8_t kind = n ? r[0] : 0; fcm1(0,0,0,0,0,&len);
  if (kind=='A') fcm1(99,0,0,0,0,&len);
  if (kind=='B') { fcm1(2,1,0,0,0,&len); output[32]=0xff; len=33; }
  if (kind=='C') { fcm1(2,1,0,0,0,&len); output[32]=0; len=33; }
  if (kind=='D') fcm1(2,4097,0,0,0,&len);
  if (kind=='E') fcm1(0,0,1048577,0,0,&len);
  if (kind=='F') fcm1(0,0,1,1000001,0,&len);
  if (kind=='G') { fcm1(2,0,1,0,0,&len); output[32]='C'; len=33; }
  if (kind=='H') { fcm1(0,0,1,1,0,&len); output[32]='C'; len=33; }
  if (kind=='Q') { fcm1(0,0,1,0,0,&len); output[32]=(uint8_t)('A'+releases-1); len=33; }
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
    assert!(finish_response(
        &input,
        options,
        decode_response(&success_response(&input, options)).expect("decode")
    )
    .is_ok());
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
    put_u32(&mut response, FERRUM_CHEM_MOLECULE_WIRE_VERSION);
    put_u32(&mut response, status);
    put_u32(
        &mut response,
        u32::try_from(detail.len()).expect("detail count"),
    );
    put_u32(&mut response, FERRUM_CHEM_MOLECULE_FLAGS_NONE);
    response.extend_from_slice(detail.as_bytes());
    response
}

fn smiles_response(status: u32, detail: &str, canonical_smiles: &str, atom_count: u32) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&MOLECULE_RESPONSE_MAGIC);
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

fn fcm1_header(
    status: u32,
    detail_length: u32,
    smiles_length: u32,
    atom_count: u32,
    bond_count: u32,
) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&MOLECULE_RESPONSE_MAGIC);
    put_u32(&mut response, 1);
    put_u32(&mut response, status);
    put_u32(&mut response, detail_length);
    put_u32(&mut response, smiles_length);
    put_u32(&mut response, atom_count);
    put_u32(&mut response, bond_count);
    put_u32(&mut response, 0);
    response
}

fn fcm1_molecule_response() -> Vec<u8> {
    let smiles = "[13CH3][C@H](F)[C:9](=O)[O-]";
    let mut response = fcm1_header(FERRUM_CHEM_RESULT_OK, 0, smiles.len() as u32, 6, 5);
    response.extend_from_slice(smiles.as_bytes());
    for (number, aromatic, chirality, charge, isotope, hydrogens, radical, no_implicit, map) in [
        (6, 0, 0, 0, 13, 3, 0, 0, 0),
        (6, 0, 1, 0, 0, 1, 0, 0, 0),
        (9, 0, 0, 0, 0, 0, 0, 0, 0),
        (6, 0, 0, 0, 0, 0, 0, 0, 9),
        (8, 0, 0, 0, 0, 0, 0, 0, 0),
        (8, 0, 0, -1, 0, 0, 0, 1, 0),
    ] {
        response.extend_from_slice(&[
            number,
            aromatic,
            chirality,
            FERRUM_CHEM_MOLECULE_RESERVED as u8,
        ]);
        put_i32(&mut response, charge);
        put_u16(&mut response, isotope);
        put_u16(&mut response, hydrogens);
        response.extend_from_slice(&[radical, no_implicit]);
        put_u16(&mut response, FERRUM_CHEM_MOLECULE_RESERVED as u16);
        put_u32(&mut response, map);
    }
    for (start, end, order) in [(0, 1, 1), (1, 2, 1), (1, 3, 1), (3, 4, 2), (3, 5, 1)] {
        put_u32(&mut response, start);
        put_u32(&mut response, end);
        response.extend_from_slice(&[
            order,
            FERRUM_CHEM_MOLECULE_RESERVED as u8,
            FERRUM_CHEM_BOND_STEREO_NONE as u8,
            FERRUM_CHEM_BOND_DIRECTION_NONE as u8,
        ]);
        put_u32(&mut response, FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE);
        put_u32(&mut response, FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE);
        put_u32(&mut response, FERRUM_CHEM_MOLECULE_RESERVED);
    }
    for index in 0..6 {
        response.extend_from_slice(&(index as f64).to_le_bytes());
        response.extend_from_slice(&0.0_f64.to_le_bytes());
    }
    response
}

fn append_import_record(response: &mut Vec<u8>, title: &str, properties: &[(&str, &str)]) {
    let molecule = fcm1_molecule_response();
    put_u32(
        response,
        u32::try_from(molecule.len()).expect("fixture molecule fits u32"),
    );
    put_u32(
        response,
        u32::try_from(title.len()).expect("fixture title fits u32"),
    );
    put_u32(
        response,
        u32::try_from(properties.len()).expect("fixture property count fits u32"),
    );
    put_u32(response, FERRUM_CHEM_SDF_FLAGS_NONE);
    response.extend_from_slice(&molecule);
    response.extend_from_slice(title.as_bytes());
    for (name, value) in properties {
        put_u32(
            response,
            u32::try_from(name.len()).expect("fixture property name fits u32"),
        );
        put_u32(
            response,
            u32::try_from(value.len()).expect("fixture property value fits u32"),
        );
        response.extend_from_slice(name.as_bytes());
        response.extend_from_slice(value.as_bytes());
    }
}
