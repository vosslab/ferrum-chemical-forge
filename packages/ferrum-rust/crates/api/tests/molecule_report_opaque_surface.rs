//! External-consumer proof that molecule reports are DTOs, not Rust capabilities.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const POSITIVE: &str = r##"
use ferrum_api::{
    OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1,
    execute_operation_v1,
};
use ferrum_document::DocumentSession;

fn main() {
    let document = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id='m'><atom id='a' name='C'><point x='1' y='2'/></atom></molecule></cdml>";
    let session = DocumentSession::load(document).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
	let molecule_id = observation.projection().molecules()[0]
		.id()
		.expect("fixture molecule has a durable identifier")
		.as_str()
		.to_owned();
    let digest = observation.snapshot().digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let request = format!(r#"{{
        "schema":"ferrum-operation-request-v1",
        "request_id":"opaque-report-route",
        "operation":{{"kind":"document.molecule.report.v1",
            "document":{document:?},
            "expected_revision":0,
            "expected_digest_hex":"{digest}",
            "molecule_ids":[{molecule_id:?}]}}
    }}"#);
    let response = execute_operation_v1(&request).expect("public report JSON decodes");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("unbundled public route must return its typed chemistry refusal");
    };
    assert_eq!(response.request_id.as_deref(), Some("opaque-report-route"));
    assert_eq!(response.error.category, OperationProtocolErrorCategoryV1::ChemistryUnavailable);
    let serialized = serde_json::to_string(&response).expect("refusal serializes");
	assert!(serialized.contains("document.molecule.report.v1"));
    assert!(!serialized.contains(".dylibs"));
    assert!(!serialized.contains("libferrum_chem"));
    assert!(!serialized.contains("/Users/"));
}
"##;

const FORBIDDEN: &[(&str, &str)] = &[
    (
        "private extension binding",
        "use ferrum_api::python_extension_binding_v1; fn main() {}",
    ),
    (
        "private report module",
        "use ferrum_api::molecule_report_core_v1; fn main() {}",
    ),
    (
        "prepared report",
        "use ferrum_api::PreparedDocumentMoleculeReportV1; fn main() {}",
    ),
    (
        "trusted runtime",
        "use ferrum_api::TrustedLibraryChemistryRuntimeV1; fn main() {}",
    ),
    (
        "runtime-aware executor",
        "use ferrum_api::execute_operation_with_runtime_v1; fn main() {}",
    ),
    (
        "runtime path constructor",
        "use ferrum_api::protocol::runtime::TrustedLibraryChemistryRuntimeV1; fn main() {}",
    ),
    (
        "report executor",
        "use ferrum_api::execute_prepared_document_molecule_report_v1; fn main() {}",
    ),
    (
        "chemistry engine reexport",
        "use ferrum_api::ChemEngine; fn main() {}",
    ),
    ("graph reexport", "use ferrum_api::MolGraph; fn main() {}"),
    (
        "private composition receipt",
        "use ferrum_api::MoleculeComposition; fn main() {}",
    ),
    (
        "private composition entry",
        "use ferrum_api::ElementMassPercentage; fn main() {}",
    ),
    (
        "obsolete SMARTS origin",
        "use ferrum_api::DocumentSmartsQueryOriginV1; fn main() {}",
    ),
    (
        "obsolete SMARTS origin alias",
        "use ferrum_api::DocumentSmartsQueryOriginV1 as HiddenOrigin; fn main() {}",
    ),
    (
        "obsolete SMARTS origin glob",
        "use ferrum_api::*; fn main() { let _: DocumentSmartsQueryOriginV1; }",
    ),
    (
        "obsolete SMARTS origin macro reexport",
        "macro_rules! hidden { () => { pub use ferrum_api::DocumentSmartsQueryOriginV1; }; } hidden!(); fn main() {}",
    ),
];
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Consumer {
    root: PathBuf,
}

impl Consumer {
    fn new(features: &'static str) -> Self {
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ferrum-molecule-report-surface-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("src")).expect("test consumer directory");
        let manifest = format!(
            "[package]\nname=\"external_consumer\"\nversion=\"0.0.0\"\nedition=\"2024\"\npublish=false\n\n[workspace]\n\n[dependencies]\nferrum-api={{path=\"{}\", features=[{features}]}}\nferrum-document={{path=\"{}\"}}\nserde_json=\"*\"\n",
            Path::new(env!("CARGO_MANIFEST_DIR")).display(),
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("api has sibling crates")
                .join("document")
                .display(),
        );
        std::fs::write(root.join("Cargo.toml"), manifest).expect("test manifest");
        let workspace_lock = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("api crate has a workspace root")
            .join("Cargo.lock");
        std::fs::copy(workspace_lock, root.join("Cargo.lock")).expect("test lockfile");
        Self { root }
    }

    fn write(&self, source: &str) {
        std::fs::write(self.root.join("src/main.rs"), source).expect("test source");
    }

    fn run(&self) -> std::process::ExitStatus {
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["run", "--quiet", "--offline", "--manifest-path"])
            .arg(self.root.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", self.root.join("target"))
            .status()
            .expect("Cargo starts")
    }
}

impl Drop for Consumer {
    fn drop(&mut self) {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn external_consumer_can_only_use_the_json_protocol_surface() {
    for features in ["", "\"python-binding\""] {
        let consumer = Consumer::new(features);
        consumer.write(POSITIVE);
        assert!(
            consumer.run().success(),
            "{features:?} positive route failed"
        );
        for (description, source) in FORBIDDEN {
            consumer.write(source);
            assert!(
                !consumer.run().success(),
                "{description} unexpectedly became public with {features:?}"
            );
        }
    }
}
