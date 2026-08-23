//! External-consumer proof for Ferrum-Chem's typed native adapter surface.
//!
//! The public-facade inventory comes from Rustdoc JSON's compiler-emitted item
//! tree, never from HTML or Rust source text. Passing `--document-hidden-items`
//! is essential: `#[doc(hidden)]` does not make an item inaccessible to an
//! external Rust crate.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;

static TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

const APPROVED_TYPED_FACADE: &str = r#"
constant ADAPTER_ABI_VERSION
constant INCHI_INSPECTION_SCHEMA_V1
constant INCHI_MAX_INPUT_BYTES
constant INTERCHANGE_MAX_TEXT_BYTES_V1
constant MOLBLOCK_INSPECTION_SCHEMA_V1
constant MOLBLOCK_MAX_INPUT_BYTES
constant NATIVE_SMILES_MAX_INPUT_BYTES
constant NATIVE_SMILES_MAX_OUTPUT_BYTES
constant OXIDATION_STATE_CONVENTION_V1
constant SDF_INSPECTION_SCHEMA_V1
constant SDF_MAX_INPUT_BYTES
constant SMILES_INSPECTION_SCHEMA_V1
enum AtomChirality
enum BondDirection
enum BondOrder
enum BondStereo
enum CanonicalSmilesError
enum ChemistryError
enum CompositionAggregationError
enum CompositionBuildError
enum ExplicitAdapterError
enum InchiExportError
enum InchiInspectionError
enum InchiMode
enum InterchangeCodecErrorV1
enum InterchangeFormatV1
enum KekulizeOptionsError
enum MolGraphError
enum MolblockExportError
enum MolblockInspectionError
enum MolblockVersion
enum OxidationStateErrorV1
enum OxidationStateObservationV1
enum OxidationStateResourceV1
struct OxidationStateRootAdmissionV1
enum OxidationStateUnavailableReasonV1
enum SdfError
enum SdfExportError
enum SdfInspectionError
enum SmartsExportError
enum SmartsMatchOptionsError
enum SmartsMatchUnavailableReason
enum SmilesInspectionError
function canonical_smiles_from_smiles
function compose_sdf_record
function decode_non_cdml_interchange_v1
function encode_non_cdml_interchange_v1
function inchi_from_smiles
function inspect_inchi
function inspect_molblock
function inspect_sdf
function inspect_smiles
function load_explicit_adapter
function molblock_from_smiles
function molecule_inspection_facts
function admit_oxidation_state_root_v1
function observe_admitted_oxidation_state_v1
function observe_oxidation_state_v1
function sdf_from_smiles
function smarts_from_smiles
function validate_inchi_input
function validate_molblock_input
function validate_molblock_title
function validate_sdf_input
function validate_smiles_input
struct AtomicNumber
struct CompositionElementKey
struct Coordinates
struct ElementCount
struct ElementMassPercentage
struct ImportedSdfRecord
struct InchiInspectionV1
struct InterchangePropertyV1
struct InterchangeRecordV1
struct KekulizeOptions
struct MolAtom
struct MolBond
struct MolGraph
struct MolblockInspectionV1
struct MoleculeComposition
struct MoleculeCompositionEntry
struct MoleculeInspectionFactsV1
struct NativeChemEngine
struct Point2
struct SdfInspectionV1
struct SdfProperty
struct SdfPropertyInspectionV1
struct SdfRecord
struct SdfRecordInspectionV1
struct SmilesAtomInspectionV1
struct SmilesBondInspectionV1
struct SmilesInspectionV1
struct SmilesMolecule
struct SmilesPointInspectionV1
struct SmartsMatchOptions
struct SmartsMatchResult
struct UnavailableChemEngine
trait ChemEngine
"#;

const FORBIDDEN_RAW_IMPORTS: &[(&str, &str)] = &[
    (
        "the private raw adapter module",
        "use ferrum_chemistry::native_engine::adapter_boundary::ChemistryAdapter;\nfn main() {}\n",
    ),
    (
        "the private foreign output buffer",
        "use ferrum_chemistry::native_engine::adapter_boundary_buffer::OutputBuffer;\nfn main() {}\n",
    ),
    (
        "a private ABI/wire constant",
        "use ferrum_chemistry::FERRUM_CHEM_GRAPH_WIRE_VERSION;\nfn main() {}\n",
    ),
];

#[test]
fn workspace_has_no_raw_chemistry_sys_package_or_dependency() {
    let workspace = workspace_root();
    let metadata = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version=1", "--no-deps"])
        .current_dir(&workspace)
        .output()
        .expect("read the current Ferrum-Chem workspace metadata");
    assert_success(&metadata, "read workspace metadata");

    let metadata_text = String::from_utf8(metadata.stdout).expect("Cargo metadata is UTF-8");
    assert!(
        !metadata_text.contains("ferrum-chemistry-sys"),
        "the retired raw chemistry sys package must not remain in Cargo metadata"
    );
    assert!(
        !workspace.join("crates/chemistry-sys/Cargo.toml").exists(),
        "the retired raw chemistry sys package manifest must not remain"
    );

    let typed_consumer = check_consumer(
        "typed-chemistry",
        "ferrum-chemistry = { path = \"CHEMISTRY_PATH\" }",
        "use ferrum_chemistry::{ChemEngine, NativeChemEngine, OxidationStateObservationV1, SmartsMatchOptions, SmartsMatchResult, observe_oxidation_state_v1};\n\
         fn accepts_typed_engine(_: &dyn ChemEngine) {}\n\
         fn accepts_match(_: SmartsMatchResult) {}\n\
         fn accepts_oxidation(_: OxidationStateObservationV1) {}\n\
         fn main() { let _ = NativeChemEngine::load; let _ = SmartsMatchOptions::new; let _ = observe_oxidation_state_v1; let _ = accepts_typed_engine; let _ = accepts_match; let _ = accepts_oxidation; }\n",
    );
    assert_success(
        &typed_consumer,
        "an external consumer reaches only typed ferrum-chemistry behavior",
    );

    for (description, source) in FORBIDDEN_RAW_IMPORTS {
        let forbidden_consumer = check_consumer(
            &format!("forbidden-{}", slug(description)),
            "ferrum-chemistry = { path = \"CHEMISTRY_PATH\" }",
            source,
        );
        assert!(
            !forbidden_consumer.status.success(),
            "external consumer unexpectedly imported {description}:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&forbidden_consumer.stdout),
            String::from_utf8_lossy(&forbidden_consumer.stderr),
        );
    }

    let retired_consumer = check_consumer(
        "retired-raw-package",
        "ferrum-chemistry-sys = { path = \"RETIRED_PATH\" }",
        "fn main() {}\n",
    );
    assert!(
        !retired_consumer.status.success(),
        "the retired raw chemistry sys package must reject a direct consumer dependency"
    );
}

#[test]
fn compiler_derived_inventory_is_exactly_the_approved_typed_facade() {
    assert_eq!(
        rustdoc_inventory(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            "ferrum_chemistry",
            "ferrum-chemistry"
        ),
        approved_facade(),
        "every externally reachable ferrum-chemistry item must be deliberately approved"
    );
}

#[test]
fn compiler_derived_inventory_rejects_hidden_macro_include_and_reexport_raw_items() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/public_surface_oracle_raw_regressions");
    let actual = rustdoc_inventory(
        &fixture,
        "public_surface_oracle_raw_regressions",
        "public-surface-oracle-raw-regressions",
    );
    for item in [
        ("struct", "HiddenRawAdapter"),
        ("macro", "hidden_raw_adapter_macro"),
        ("struct", "IncludedRawWire"),
        ("struct", "ReexportedRawBuffer"),
    ] {
        assert!(
            actual.contains(&(item.0.to_owned(), item.1.to_owned())),
            "the compiler-derived inventory must expose the {} {} regression item",
            item.0,
            item.1
        );
        assert!(
            !approved_facade().contains(&(item.0.to_owned(), item.1.to_owned())),
            "the regression item must be rejected by the approved facade"
        );
    }
}

fn approved_facade() -> BTreeSet<(String, String)> {
    APPROVED_TYPED_FACADE
        .trim()
        .lines()
        .map(|line| {
            let (kind, name) = line
                .split_once(' ')
                .expect("approved facade entries are `kind name`");
            (kind.to_owned(), name.to_owned())
        })
        .collect()
}

/// Builds the pinned nightly Rustdoc JSON item tree and reads its public
/// crate-root inventory. This intentionally never opens an HTML page.
fn rustdoc_inventory(
    crate_root: &Path,
    crate_name: &str,
    package_name: &str,
) -> BTreeSet<(String, String)> {
    let root = temporary_work_root(package_name);
    let output = Command::new("rustup")
        .args([
            "run",
            "nightly",
            "cargo",
            "rustdoc",
            "--quiet",
            "--manifest-path",
        ])
        .arg(crate_root.join("Cargo.toml"))
        .args(["--lib", "--target-dir"])
        .arg(&root)
        .args([
            "--",
            "-Z",
            "unstable-options",
            "--output-format",
            "json",
            "--document-hidden-items",
        ])
        .output()
        .expect("run nightly Rustdoc JSON for public API inventory");
    assert_success(&output, "build nightly Rustdoc JSON inventory");
    let index_path = root.join("doc").join(format!("{crate_name}.json"));
    let index = fs::read_to_string(&index_path).unwrap_or_else(|error| {
        panic!(
            "read Rustdoc JSON inventory {}: {error}",
            index_path.display()
        )
    });
    parse_rustdoc_json_root_inventory(&index)
}

fn parse_rustdoc_json_root_inventory(index: &str) -> BTreeSet<(String, String)> {
    let document: Value = serde_json::from_str(index).expect("Rustdoc JSON inventory is JSON");
    let items = root_module_items(&document);
    let index = document["index"]
        .as_object()
        .expect("Rustdoc JSON has an item index");
    items
        .iter()
        .filter_map(|item_id| {
            let item_id = item_id
                .as_u64()
                .expect("root module item IDs are numeric")
                .to_string();
            let item = index
                .get(&item_id)
                .expect("root module item resolves in Rustdoc JSON");
            if item["visibility"] != "public" {
                return None;
            }
            if let Some(import) = item["inner"].get("use") {
                let name = import["name"].as_str().expect("public use has a name");
                let target = import["id"]
                    .as_u64()
                    .expect("public use has a target item ID")
                    .to_string();
                return Some((
                    rustdoc_item_kind(index.get(&target).expect("public use target resolves")),
                    name.to_owned(),
                ));
            }
            Some((
                rustdoc_item_kind(item),
                item["name"]
                    .as_str()
                    .expect("public root item has a name")
                    .to_owned(),
            ))
        })
        .collect()
}

fn root_module_items(document: &Value) -> &Vec<Value> {
    let root = document["root"]
        .as_u64()
        .expect("Rustdoc JSON has a root item ID")
        .to_string();
    document["index"][root]["inner"]["module"]["items"]
        .as_array()
        .expect("Rustdoc root is a module")
}

fn rustdoc_item_kind(item: &Value) -> String {
    item["inner"]
        .as_object()
        .expect("Rustdoc item has an inner kind")
        .keys()
        .next()
        .expect("Rustdoc item has one inner kind")
        .clone()
}

fn check_consumer(name: &str, dependency: &str, source: &str) -> Output {
    let root = temporary_work_root(name);
    fs::create_dir_all(root.join("src")).expect("create isolated consumer source directory");
    let chemistry_path = workspace_root().join("crates/chemistry");
    let retired_path = workspace_root().join("crates/chemistry-sys");
    let dependency = dependency
        .replace("CHEMISTRY_PATH", &chemistry_path.display().to_string())
        .replace("RETIRED_PATH", &retired_path.display().to_string());
    fs::write(root.join("Cargo.toml"), format!("[package]\nname = \"ferrum-chemistry-surface-{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n{dependency}\n")).expect("write isolated consumer manifest");
    fs::write(root.join("src/main.rs"), source).expect("write isolated consumer source");
    Command::new(env!("CARGO"))
        .args(["check", "--quiet"])
        .current_dir(&root)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .expect("compile isolated external consumer")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("chemistry manifest is nested under the workspace")
        .to_path_buf()
}

fn temporary_work_root(name: &str) -> PathBuf {
    let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ferrum-chemistry-surface-{}-{name}-{sequence}",
        std::process::id()
    ))
}

fn slug(text: &str) -> String {
    text.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                byte as char
            } else {
                '-'
            }
        })
        .collect()
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
