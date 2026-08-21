use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const REQUIRED_MACROS: &[&str] = &[
    "FERRUM_CHEM_ADAPTER_ABI_VERSION",
    "FERRUM_CHEM_CALL_ALLOCATION_FAILURE",
    "FERRUM_CHEM_MAX_RESPONSE_BYTES",
    "FERRUM_CHEM_CAPABILITY_KEKULIZE",
    "FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE",
    "FERRUM_CHEM_CAPABILITY_GENERATE_2D",
    "FERRUM_CHEM_CAPABILITY_SMARTS",
    "FERRUM_CHEM_CAPABILITY_MOLFILE",
    "FERRUM_CHEM_CAPABILITY_SDF_WRITE",
    "FERRUM_CHEM_CAPABILITY_INCHI",
    "FERRUM_CHEM_CAPABILITY_SDF_READ",
    "FERRUM_CHEM_CAPABILITY_MOLFILE_READ",
    "FERRUM_CHEM_CAPABILITY_COMPOSITION",
    "FERRUM_CHEM_CAPABILITY_SMILES_WRITE",
    "FERRUM_CHEM_CAPABILITY_MOLFILE_TITLE",
    "FERRUM_CHEM_CAPABILITY_SMARTS_MATCH",
    "FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION",
    "FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES",
    "FERRUM_CHEM_SMARTS_MATCH_MAX_QUERY_BYTES",
    "FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS",
    "FERRUM_CHEM_SMARTS_MATCH_MAX_MATRIX_CELLS",
    "FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE",
    "FERRUM_CHEM_COMPOSITION_WIRE_VERSION",
    "FERRUM_CHEM_COMPOSITION_FLAGS_NONE",
    "FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES",
    "FERRUM_CHEM_COMPOSITION_ENTRY_BYTES",
    "FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES",
    "FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES",
    "FERRUM_CHEM_RESULT_OK",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_OK",
    "FERRUM_CHEM_RESULT_MALFORMED_REQUEST",
    "FERRUM_CHEM_RESULT_INVALID_MOLECULE",
    "FERRUM_CHEM_RESULT_DEPICTION_FAILURE",
    "FERRUM_CHEM_RESULT_RESOURCE_LIMIT",
    "FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE",
    "FERRUM_CHEM_RESULT_INTERNAL_FAILURE",
    "FERRUM_CHEM_SMILES_MAX_BYTES",
    "FERRUM_CHEM_MOLECULE_WIRE_VERSION",
    "FERRUM_CHEM_MOLECULE_FLAGS_NONE",
    "FERRUM_CHEM_MOLECULE_RESERVED",
    "FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE",
    "FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES",
    "FERRUM_CHEM_MOLECULE_ATOM_BYTES",
    "FERRUM_CHEM_MOLECULE_BOND_BYTES",
    "FERRUM_CHEM_COORDINATE_BYTES",
    "FERRUM_CHEM_GRAPH_WIRE_VERSION",
    "FERRUM_CHEM_GRAPH_FLAGS_NONE",
    "FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_GRAPH_ATOM_BYTES",
    "FERRUM_CHEM_GRAPH_BOND_BYTES",
    "FERRUM_CHEM_TEXT_WIRE_VERSION",
    "FERRUM_CHEM_TEXT_FLAGS_NONE",
    "FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES",
    "FERRUM_CHEM_SMILES_WRITE_MAX_BYTES",
    "FERRUM_CHEM_INCHI_WIRE_VERSION",
    "FERRUM_CHEM_INCHI_MODE_STANDARD",
    "FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN",
    "FERRUM_CHEM_INCHI_FLAGS_NONE",
    "FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_INCHI_MAX_BYTES",
    "FERRUM_CHEM_INCHI_KEY_BYTES",
    "FERRUM_CHEM_MOLBLOCK_WIRE_VERSION",
    "FERRUM_CHEM_MOLBLOCK_FORMAT_V2000",
    "FERRUM_CHEM_MOLBLOCK_FORMAT_V3000",
    "FERRUM_CHEM_MOLBLOCK_FLAGS_NONE",
    "FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION",
    "FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_SDF_WIRE_VERSION",
    "FERRUM_CHEM_SDF_FLAGS_NONE",
    "FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_SDF_RECORD_HEADER_BYTES",
    "FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES",
    "FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES",
    "FERRUM_CHEM_SDF_MAX_RECORDS",
    "FERRUM_CHEM_SDF_MAX_PROPERTIES",
    "FERRUM_CHEM_KEKULIZE_WIRE_VERSION",
    "FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS",
    "FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL",
    "FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE",
    "FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE",
    "FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS",
    "FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS",
    "FERRUM_CHEM_KEKULIZE_MAX_ATOMS",
    "FERRUM_CHEM_KEKULIZE_MAX_BONDS",
    "FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES",
    "FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES",
    "FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES",
    "FERRUM_CHEM_KEKULIZE_ATOM_BYTES",
    "FERRUM_CHEM_KEKULIZE_BOND_BYTES",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE",
    "FERRUM_CHEM_CHIRAL_UNSPECIFIED",
    "FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW",
    "FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW",
    "FERRUM_CHEM_CHIRAL_OTHER",
    "FERRUM_CHEM_BOND_STEREO_NONE",
    "FERRUM_CHEM_BOND_STEREO_ANY",
    "FERRUM_CHEM_BOND_STEREO_Z",
    "FERRUM_CHEM_BOND_STEREO_E",
    "FERRUM_CHEM_BOND_STEREO_CIS",
    "FERRUM_CHEM_BOND_STEREO_TRANS",
    "FERRUM_CHEM_BOND_STEREO_OTHER",
    "FERRUM_CHEM_BOND_DIRECTION_NONE",
    "FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE",
    "FERRUM_CHEM_BOND_DIRECTION_BEGINDASH",
    "FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT",
    "FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT",
    "FERRUM_CHEM_BOND_DIRECTION_OTHER",
];

const ZERO_VALUE_MACROS: &[&str] = &[
    "FERRUM_CHEM_RESULT_OK",
    "FERRUM_CHEM_SMARTS_MATCH_STATUS_OK",
    "FERRUM_CHEM_MOLECULE_FLAGS_NONE",
    "FERRUM_CHEM_MOLECULE_RESERVED",
    "FERRUM_CHEM_GRAPH_FLAGS_NONE",
    "FERRUM_CHEM_TEXT_FLAGS_NONE",
    "FERRUM_CHEM_COMPOSITION_FLAGS_NONE",
    "FERRUM_CHEM_INCHI_FLAGS_NONE",
    "FERRUM_CHEM_MOLBLOCK_FLAGS_NONE",
    "FERRUM_CHEM_SDF_FLAGS_NONE",
    "FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED",
    "FERRUM_CHEM_CHIRAL_UNSPECIFIED",
    "FERRUM_CHEM_BOND_STEREO_NONE",
    "FERRUM_CHEM_BOND_DIRECTION_NONE",
];

fn main() {
    let header = PathBuf::from("native/include/ferrum_chem_adapter.h");
    println!("cargo:rerun-if-changed={}", header.display());
    let contents = fs::read_to_string(&header).expect("read Ferrum chemistry adapter header");
    let definitions = parse_definitions(&contents);
    let target_pointer_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
        .expect("Cargo provides target pointer width")
        .parse::<u32>()
        .expect("Cargo target pointer width is numeric");

    let mut generated = String::new();
    for name in REQUIRED_MACROS {
        let values = definitions
            .get(*name)
            .unwrap_or_else(|| panic!("public adapter header must define {name}"));
        assert_eq!(
            values.len(),
            1,
            "public adapter header must define exactly one numeric {name}"
        );
        let value = values[0];
        if ZERO_VALUE_MACROS.contains(name) {
            assert_eq!(
                value, 0,
                "public adapter header macro {name} must be the zero-valued FCM1 sentinel"
            );
        } else {
            assert_ne!(
                value, 0,
                "public adapter header macro {name} must be positive"
            );
        }
        if name.contains("_FACT_") {
            assert!(
                value <= u64::from(u16::MAX),
                "public adapter header macro {name} must fit u16"
            );
        }
        if name.ends_with("_BYTES") {
            assert_ne!(
                value, 0,
                "public adapter header macro {name} must be nonzero"
            );
        }
        if name.contains("_BOND_TYPE_") {
            assert!(
                value <= u64::from(u8::MAX),
                "public adapter header macro {name} must fit u8"
            );
        }
        let rust_type = if uses_usize(name) {
            assert!(
                value <= maximum_usize(target_pointer_width),
                "public adapter header macro {name} does not fit target usize"
            );
            "usize"
        } else if uses_u64(name) {
            "u64"
        } else {
            "u32"
        };
        if *name == "FERRUM_CHEM_ADAPTER_ABI_VERSION" {
            generated.push_str(&format!(
                "pub const ADAPTER_ABI_VERSION: {rust_type} = {value};\n"
            ));
        } else {
            generated.push_str(&format!(
                "pub(crate) const {name}: {rust_type} = {value};\n"
            ));
        }
    }
    validate_bond_type_codes(&definitions);

    let all_known_capabilities = REQUIRED_MACROS
        .iter()
        .copied()
        .filter(|name| name.starts_with("FERRUM_CHEM_CAPABILITY_"))
        .try_fold(0_u64, |mask, name| {
            let capability = definitions[name][0];
            assert!(
                capability.is_power_of_two(),
                "public adapter header capability {name} must be one bit"
            );
            assert_eq!(
                mask & capability,
                0,
                "public adapter header capability {name} overlaps another capability"
            );
            Ok::<u64, ()>(mask | capability)
        })
        .expect("adapter capability inventory is valid");
    generated.push_str(&format!(
        "pub(crate) const FERRUM_CHEM_ALL_KNOWN_CAPABILITIES: u64 = {all_known_capabilities};\n"
    ));

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"))
        .join("adapter_wire_constants.rs");
    fs::write(output, generated).expect("write generated adapter wire constants");
}

fn validate_bond_type_codes(definitions: &BTreeMap<String, Vec<u64>>) {
    let mut codes = BTreeMap::<u64, &str>::new();
    for name in REQUIRED_MACROS
        .iter()
        .copied()
        .filter(|name| name.contains("_BOND_TYPE_"))
    {
        let value = definitions[name][0];
        assert!(
            codes.insert(value, name).is_none(),
            "public adapter header bond-type macros must have distinct values"
        );
    }
}

fn uses_usize(name: &str) -> bool {
    name.ends_with("_BYTES")
        || name == "FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES"
        || name == "FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES"
}

fn uses_u64(name: &str) -> bool {
    name == "FERRUM_CHEM_CALL_ALLOCATION_FAILURE" || name.starts_with("FERRUM_CHEM_CAPABILITY_")
}

fn maximum_usize(pointer_width: u32) -> u64 {
    match pointer_width {
        16 => u64::from(u16::MAX),
        32 => u64::from(u32::MAX),
        64 => u64::MAX,
        _ => panic!("unsupported Cargo target pointer width {pointer_width}"),
    }
}

fn parse_definitions(contents: &str) -> BTreeMap<String, Vec<u64>> {
    let mut definitions = BTreeMap::<String, Vec<u64>>::new();
    for line in contents.lines() {
        let mut fields = line.split_ascii_whitespace();
        let (Some("#define"), Some(name), Some(value), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if let Some(value) = parse_unsigned_u_literal(value) {
            definitions.entry(name.to_owned()).or_default().push(value);
        }
    }
    definitions
}

fn parse_unsigned_u_literal(value: &str) -> Option<u64> {
    let digits = value.trim_end_matches(['U', 'L']);
    if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()
    } else if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        digits.parse().ok()
    } else {
        None
    }
}
