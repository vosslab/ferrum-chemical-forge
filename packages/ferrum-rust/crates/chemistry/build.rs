use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const REQUIRED_MACROS: &[&str] = &[
    "FERRUM_CHEM_ADAPTER_ABI_VERSION",
    "FERRUM_CHEM_RESULT_OK",
    "FERRUM_CHEM_RESULT_MALFORMED_REQUEST",
    "FERRUM_CHEM_RESULT_INVALID_MOLECULE",
    "FERRUM_CHEM_RESULT_KEKULIZE_FAILURE",
    "FERRUM_CHEM_RESULT_INTERNAL_FAILURE",
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
        if *name != "FERRUM_CHEM_RESULT_OK" && *name != "FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED"
        {
            assert_ne!(
                value, 0,
                "public adapter header macro {name} must be positive"
            );
        }
        if name.contains("_FACT_") {
            assert!(
                value <= u32::from(u16::MAX),
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
                value <= u32::from(u8::MAX),
                "public adapter header macro {name} must fit u8"
            );
        }
        let rust_type = if uses_usize(name) {
            assert!(
                u64::from(value) <= maximum_usize(target_pointer_width),
                "public adapter header macro {name} does not fit target usize"
            );
            "usize"
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

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"))
        .join("adapter_wire_constants.rs");
    fs::write(output, generated).expect("write generated adapter wire constants");
}

fn validate_bond_type_codes(definitions: &BTreeMap<String, Vec<u32>>) {
    let mut codes = BTreeMap::<u32, &str>::new();
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
    name.ends_with("_BYTES") || name == "FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES"
}

fn maximum_usize(pointer_width: u32) -> u64 {
    match pointer_width {
        16 => u64::from(u16::MAX),
        32 => u64::from(u32::MAX),
        64 => u64::MAX,
        _ => panic!("unsupported Cargo target pointer width {pointer_width}"),
    }
}

fn parse_definitions(contents: &str) -> BTreeMap<String, Vec<u32>> {
    let mut definitions = BTreeMap::<String, Vec<u32>>::new();
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

fn parse_unsigned_u_literal(value: &str) -> Option<u32> {
    let digits = value.strip_suffix('U')?;
    if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        digits.parse().ok()
    } else {
        None
    }
}
