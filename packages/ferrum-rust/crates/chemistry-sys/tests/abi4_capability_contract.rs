//! ABI-4 adapters must expose their capability contract and required symbols.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use ferrum_chemistry_sys::{
    AdapterError, ChemistryAdapter, FERRUM_CHEM_ADAPTER_ABI_VERSION,
    FERRUM_CHEM_ALL_KNOWN_CAPABILITIES, FERRUM_CHEM_CAPABILITY_INCHI,
    FERRUM_CHEM_CAPABILITY_KEKULIZE, FERRUM_CHEM_CAPABILITY_MOLFILE,
    FERRUM_CHEM_CAPABILITY_MOLFILE_READ, FERRUM_CHEM_CAPABILITY_SDF_READ,
    FERRUM_CHEM_CAPABILITY_SDF_WRITE, FERRUM_CHEM_CAPABILITY_SMARTS,
    FERRUM_CHEM_MAX_RESPONSE_BYTES,
};

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_adapter_loads_required_symbols_and_honors_capability_bits() {
    let fixture = SyntheticAdapter::build();
    let adapter = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    )
    .expect("an ABI-4 adapter with its required symbols loads");

    assert_eq!(
        u64::from(adapter.abi_version()),
        FERRUM_CHEM_ADAPTER_ABI_VERSION
    );
    assert_eq!(adapter.capabilities(), FERRUM_CHEM_CAPABILITY_KEKULIZE);
    assert!(!adapter.supports_generate_2d());
    assert!(
        adapter
            .kekulize(&[])
            .expect("required ABI-4 kekulize remains callable")
            .is_empty()
    );
    assert!(matches!(
        adapter.generate_2d(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "generate_2d_coordinates"
        })
    ));
    assert!(matches!(
        adapter.molecule_to_smarts(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "molecule_to_smarts"
        })
    ));
    assert!(matches!(
        adapter.molecule_to_molblock(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "molecule_to_molblock"
        })
    ));
    assert!(matches!(
        adapter.records_to_sdf(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "records_to_sdf"
        })
    ));
    assert!(matches!(
        adapter.sdf_to_records(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "sdf_to_records"
        })
    ));
    assert!(matches!(
        adapter.molblock_to_molecule(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "molblock_to_molecule"
        })
    ));
    assert!(matches!(
        adapter.inchi_to_molecule(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "inchi_to_molecule"
        })
    ));
    assert!(matches!(
        adapter.molecule_to_inchi(&[]),
        Err(AdapterError::OperationUnavailable {
            operation: "molecule_to_inchi"
        })
    ));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_adapter_releases_an_oversized_foreign_result_without_reading_it() {
    let fixture = SyntheticAdapter::build();
    let adapter = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    )
    .expect("the synthetic ABI-4 adapter loads");

    let error = adapter
        .kekulize(&[1])
        .expect_err("the safe boundary rejects foreign output before creating a slice");
    assert!(matches!(
        error,
        AdapterError::ResponseTooLarge {
            length,
            maximum,
        } if length == FERRUM_CHEM_MAX_RESPONSE_BYTES + 1
            && maximum == FERRUM_CHEM_MAX_RESPONSE_BYTES
    ));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_rejects_unknown_advertised_capability_bits() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | !FERRUM_CHEM_ALL_KNOWN_CAPABILITIES,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("unknown capability bits must fail at adapter load"),
    };
    assert!(matches!(error, AdapterError::UnknownCapabilities { .. }));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_requires_each_advertised_operation_symbol() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_SMARTS,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );

    assert!(matches!(result, Err(AdapterError::Load(_))));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_requires_the_advertised_molfile_symbol() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_MOLFILE,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );

    assert!(matches!(result, Err(AdapterError::Load(_))));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_requires_the_advertised_sdf_writer_symbol() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_SDF_WRITE,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );

    assert!(matches!(result, Err(AdapterError::Load(_))));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_requires_the_advertised_sdf_reader_symbol() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_SDF_READ,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );

    assert!(matches!(result, Err(AdapterError::Load(_))));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_requires_the_advertised_molfile_reader_symbol() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_MOLFILE_READ,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );

    assert!(matches!(result, Err(AdapterError::Load(_))));
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn abi4_loader_requires_the_advertised_inchi_symbol_set() {
    let fixture = SyntheticAdapter::build_with_capabilities(
        FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_INCHI,
    );
    let result = ChemistryAdapter::load(
        fixture.library_path(),
        u32::try_from(FERRUM_CHEM_ADAPTER_ABI_VERSION).expect("header ABI fits u32"),
    );

    assert!(matches!(result, Err(AdapterError::Load(_))));
}

struct SyntheticAdapter {
    directory: PathBuf,
    library: PathBuf,
}

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

impl SyntheticAdapter {
    fn build() -> Self {
        Self::build_with_capabilities(FERRUM_CHEM_CAPABILITY_KEKULIZE)
    }

    fn build_with_capabilities(capabilities: u64) -> Self {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "ferrum-chemistry-sys-abi4-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("create synthetic adapter directory");
        let source = directory.join("adapter.c");
        let library = directory.join(library_name());
        fs::write(
            &source,
            GRAPHMOL_ONLY_ADAPTER
                .replace(
                    "FERRUM_ABI_VERSION",
                    &FERRUM_CHEM_ADAPTER_ABI_VERSION.to_string(),
                )
                .replace("FERRUM_CAPABILITY_MASK", &capabilities.to_string())
                .replace(
                    "FERRUM_OVERSIZED_RESPONSE_LENGTH",
                    &(FERRUM_CHEM_MAX_RESPONSE_BYTES + 1).to_string(),
                ),
        )
        .expect("write synthetic adapter source");

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
    panic!("the ABI-4 fixture needs a supported C shared-library compiler");
}

impl Drop for SyntheticAdapter {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn library_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "libgraphmol_abi4.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "libgraphmol_abi4.so"
    }
    #[cfg(target_os = "windows")]
    {
        "graphmol_abi4.dll"
    }
}

const GRAPHMOL_ONLY_ADAPTER: &str = r#"
#include <stdint.h>

typedef struct ferrum_chem_owned_buffer {
    uint8_t *data;
    uint64_t len;
} ferrum_chem_owned_buffer;

uint32_t ferrum_chem_abi_version(void) { return FERRUM_ABI_VERSION; }
uint64_t ferrum_chem_capabilities_v1(void) { return FERRUM_CAPABILITY_MASK; }

uint32_t ferrum_chem_kekulize_v1(
    const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) {
    static uint8_t payload = 7;
    if (request_len == 1 && request[0] == 1) {
        response->data = &payload;
        response->len = FERRUM_OVERSIZED_RESPONSE_LENGTH;
        return 0U;
    }
    response->data = 0;
    response->len = 0;
    return 0U;
}

uint32_t ferrum_chem_generate_2d_v1(
    const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) {
    (void)request;
    (void)request_len;
    response->data = 0;
    response->len = 0;
    return 0U;
}

uint32_t ferrum_chem_smiles_to_molecule_v1(
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
