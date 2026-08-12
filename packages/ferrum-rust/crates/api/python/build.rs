use std::env;
use std::path::Path;

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
}
