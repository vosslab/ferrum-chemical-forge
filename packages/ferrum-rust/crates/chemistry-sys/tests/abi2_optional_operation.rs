//! ABI-2 adapters without the optional depiction symbol remain usable.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ferrum_chemistry_sys::{AdapterError, ChemistryAdapter};

const ABI_VERSION: u32 = 2;

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn graphmol_only_abi2_adapter_keeps_kekulize_and_reports_missing_depiction() {
    let fixture = SyntheticAdapter::build();
    let adapter = ChemistryAdapter::load(fixture.library_path(), ABI_VERSION)
        .expect("an ABI-2 adapter without the extension still loads");

    assert_eq!(adapter.abi_version(), ABI_VERSION);
    assert!(!adapter.supports_generate_2d());
    assert!(
        adapter
            .kekulize(&[])
            .expect("required ABI-2 kekulize remains callable")
            .is_empty()
    );
    assert!(matches!(
        adapter.generate_2d(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "generate_2d_coordinates"
        })
    ));
}

struct SyntheticAdapter {
    directory: PathBuf,
    library: PathBuf,
}

impl SyntheticAdapter {
    fn build() -> Self {
        let directory =
            std::env::temp_dir().join(format!("ferrum-chemistry-sys-abi2-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("create synthetic adapter directory");
        let source = directory.join("adapter.c");
        let library = directory.join(library_name());
        fs::write(&source, GRAPHMOL_ONLY_ADAPTER).expect("write synthetic adapter source");

        let mut compiler = Command::new("cc");
        add_shared_library_flags(&mut compiler);
        let output = compiler
            .arg(&source)
            .args(["-o"])
            .arg(&library)
            .output()
            .expect("run C compiler for synthetic adapter");
        assert!(
            output.status.success(),
            "compile synthetic adapter: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { directory, library }
    }

    fn library_path(&self) -> &Path {
        &self.library
    }
}

#[cfg(target_os = "macos")]
fn add_shared_library_flags(compiler: &mut Command) {
    compiler.arg("-dynamiclib");
}

#[cfg(target_os = "linux")]
fn add_shared_library_flags(compiler: &mut Command) {
    compiler.args(["-shared", "-fPIC"]);
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn add_shared_library_flags(_: &mut Command) {
    panic!("the ABI-2 optional-symbol fixture needs a supported C shared-library compiler");
}

impl Drop for SyntheticAdapter {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn library_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "libgraphmol_only_abi2.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "libgraphmol_only_abi2.so"
    }
    #[cfg(target_os = "windows")]
    {
        "graphmol_only_abi2.dll"
    }
}

const GRAPHMOL_ONLY_ADAPTER: &str = r#"
#include <stdint.h>

typedef struct ferrum_chem_owned_buffer {
    uint8_t *data;
    uint64_t len;
} ferrum_chem_owned_buffer;

uint32_t ferrum_chem_abi_version(void) { return 2U; }

uint32_t ferrum_chem_kekulize_v1(
    const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) {
    (void)request;
    (void)request_len;
    response->data = 0;
    response->len = 0;
    return 0U;
}

void ferrum_chem_owned_buffer_free_v1(ferrum_chem_owned_buffer *response) {
    response->data = 0;
    response->len = 0;
}
"#;
