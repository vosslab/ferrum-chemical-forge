use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const REQUIRED_MACROS: &[&str] = &[
    "FERRUM_CHEM_ADAPTER_ABI_VERSION",
    "FERRUM_CHEM_CALL_ALLOCATION_FAILURE",
    "FERRUM_CHEM_MAX_RESPONSE_BYTES",
];

const CAPABILITY_PREFIX: &str = "FERRUM_CHEM_CAPABILITY_";

fn main() {
    let header = PathBuf::from("../chemistry/native/include/ferrum_chem_adapter.h");
    println!("cargo:rerun-if-changed={}", header.display());
    let source = fs::read_to_string(&header).expect("read public Ferrum chemistry adapter header");
    let definitions = parse_definitions(&source);
    let mut generated = String::new();
    for name in REQUIRED_MACROS {
        let value = definitions
            .get(*name)
            .unwrap_or_else(|| panic!("public adapter header must define {name}"));
        generated.push_str(&format!("pub const {name}: u64 = {value};\n"));
    }
    let capability_names = definitions
        .keys()
        .filter(|name| name.starts_with(CAPABILITY_PREFIX))
        .collect::<Vec<_>>();
    assert!(
        !capability_names.is_empty(),
        "public adapter header must define at least one ABI-4 capability"
    );
    let mut all_known_capabilities = 0_u64;
    for name in capability_names {
        let value = definitions[name];
        assert!(
            value.is_power_of_two(),
            "public adapter header capability {name} must be one bit"
        );
        assert_eq!(
            all_known_capabilities & value,
            0,
            "public adapter header capability {name} overlaps another capability"
        );
        all_known_capabilities |= value;
        generated.push_str(&format!("pub const {name}: u64 = {value};\n"));
    }
    let capability_expression = definitions
        .keys()
        .filter(|name| name.starts_with(CAPABILITY_PREFIX))
        .cloned()
        .collect::<Vec<_>>()
        .join(" | ");
    generated.push_str(&format!(
        "pub const FERRUM_CHEM_ALL_KNOWN_CAPABILITIES: u64 = {capability_expression};\n"
    ));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"))
        .join("adapter_abi_constants.rs");
    fs::write(output, generated).expect("write generated adapter ABI constants");
}

fn parse_definitions(source: &str) -> BTreeMap<String, u64> {
    source
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_ascii_whitespace();
            let (Some("#define"), Some(name), Some(value), None) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return None;
            };
            parse_unsigned_literal(value).map(|number| (name.to_owned(), number))
        })
        .collect()
}

fn parse_unsigned_literal(value: &str) -> Option<u64> {
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
