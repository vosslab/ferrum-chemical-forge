use std::env;
use std::fs;
use std::path::Path;

fn adapter_abi_version() -> u32 {
    let raw_value = env::var("FERRUM_CHEM_ADAPTER_ABI_VERSION")
        .expect("FERRUM_CHEM_ADAPTER_ABI_VERSION must be supplied by the native-wheel builder");
    let abi_version = raw_value
        .parse::<u32>()
        .expect("FERRUM_CHEM_ADAPTER_ABI_VERSION must be an unsigned integer");
    if abi_version == 0 {
        panic!("FERRUM_CHEM_ADAPTER_ABI_VERSION must be at least 1");
    }
    abi_version
}

fn main() {
    let library_directory = env::var("FERRUM_CHEM_LIB_DIR")
        .expect("FERRUM_CHEM_LIB_DIR must name the directory containing libferrum_chem.dylib");
    if !Path::new(&library_directory)
        .join("libferrum_chem.dylib")
        .is_file()
    {
        panic!("FERRUM_CHEM_LIB_DIR does not contain libferrum_chem.dylib: {library_directory}");
    }
    println!("cargo:rustc-link-search=native={library_directory}");
    println!("cargo:rustc-link-lib=dylib=ferrum_chem");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/.libs");
    println!("cargo:rerun-if-env-changed=FERRUM_CHEM_LIB_DIR");
    println!("cargo:rerun-if-env-changed=FERRUM_CHEM_ADAPTER_ABI_VERSION");

    let abi_version = adapter_abi_version();
    let generated_source = format!("const SUPPORTED_ADAPTER_ABI_VERSION: u32 = {abi_version};\n");
    let generated_path = Path::new(&env::var("OUT_DIR").expect("Cargo must set OUT_DIR"))
        .join("ferrum_chem_adapter_abi.rs");
    fs::write(&generated_path, generated_source)
        .expect("Cargo must permit writing generated adapter ABI configuration");
}
