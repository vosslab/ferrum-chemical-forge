use std::fs;

use pyo3::{Python, exceptions::PyTypeError, types::PyModule};

use super::*;

const SINGLE_ATOM_SDF_V1: &str = concat!(
    "Ferrum SDF\n",
    "  Ferrum\n",
    "\n",
    "  1  0  0  0  0  0  0  0  0  0  0 V2000\n",
    "    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n",
    "M  END\n",
    "$$$$\n",
);
const TWO_ATOM_CML_V1: &str = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0"/><atom id="a2" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a1 a2" order="1"/></bondArray></molecule></cml>"#;

fn temporary_sdf_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ferrum-pyo3-sdf-{}-{}.sdf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("wall clock")
            .as_nanos(),
    ))
}

fn temporary_cml_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ferrum-pyo3-cml-{}-{}.cml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("wall clock")
            .as_nanos(),
    ))
}

fn issued_descriptor<'py>(
    descriptors: &pyo3::Bound<'py, pyo3::PyAny>,
    suffix: &str,
) -> pyo3::Bound<'py, pyo3::PyAny> {
    descriptors
        .try_iter()
        .expect("descriptors iterate")
        .find_map(|descriptor| {
            let descriptor = descriptor.expect("descriptor");
            descriptor
                .getattr("suffixes")
                .expect("suffixes")
                .contains(suffix)
                .expect("suffix list")
                .then_some(descriptor)
        })
        .expect("registered descriptor")
}

#[test]
fn interchange_preparation_requires_a_descriptor_issued_opaque_route_handle() {
    Python::initialize();
    Python::attach(|py| {
        let module = PyModule::new(py, "ferrum_chem").expect("extension module");
        super::super::binding::initialize(&module).expect("extension module registers");
        assert!(
            !module
                .hasattr("prepare_sdf_file_v1")
                .expect("module surface is inspectable"),
            "descriptor-free SDF file import must not remain public"
        );
        let document_session = module.getattr("DocumentSession").expect("session type");
        let descriptors = document_session
            .call_method0("local_interchange_open_descriptors_v1")
            .expect("descriptors issue");
        let cml_descriptor = issued_descriptor(&descriptors, ".cml");
        let sdf_descriptor = issued_descriptor(&descriptors, ".sdf");
        let cml_handle = cml_descriptor
            .getattr("route_handle")
            .expect("opaque handle");

        assert!(
            cml_handle
                .get_type()
                .call0()
                .is_err_and(|error| error.is_instance_of::<PyTypeError>(py))
        );
        let copy_module = py.import("copy").expect("copy module");
        assert!(
            copy_module
                .call_method1("copy", (&cml_handle,))
                .is_err_and(|error| error.is_instance_of::<PyTypeError>(py))
        );
        assert!(
            copy_module
                .call_method1("deepcopy", (&cml_handle,))
                .is_err_and(|error| error.is_instance_of::<PyTypeError>(py))
        );
        assert!(
            document_session
                .call_method1(
                    "prepare_local_interchange_file_v1",
                    (
                        "/definitely-not-an-interchange-file.cml",
                        "cml_simple_molecule_import_v1"
                    ),
                )
                .is_err_and(|error| error.is_instance_of::<PyTypeError>(py))
        );
        assert!(
            document_session
                .call_method1(
                    "prepare_local_interchange_file_v1",
                    ("/definitely-not-an-interchange-file.cml", cml_handle),
                )
                .is_err_and(|error| !error.is_instance_of::<PyTypeError>(py))
        );

        let path = temporary_sdf_path();
        fs::write(&path, SINGLE_ATOM_SDF_V1).expect("write valid SDF");
        let sdf_handle = sdf_descriptor
            .getattr("route_handle")
            .expect("SDF route handle");
        assert!(
            document_session
                .call_method1(
                    "read_local_interchange_utf8_v1",
                    (path.to_string_lossy().as_ref(), "sdf"),
                )
                .is_err_and(|error| error.is_instance_of::<PyTypeError>(py)),
            "the text reader requires a registry-issued opaque handle"
        );
        assert_eq!(
            document_session
                .call_method1(
                    "read_local_interchange_utf8_v1",
                    (path.to_string_lossy().as_ref(), &sdf_handle),
                )
                .expect("registered SDF source reads as text")
                .extract::<String>()
                .expect("registered SDF source is text"),
            SINGLE_ATOM_SDF_V1,
        );
        assert!(
            document_session
                .call_method1(
                    "read_local_interchange_utf8_v1",
                    ("/definitely-not-an-interchange-file.sdf", &sdf_handle),
                )
                .is_err_and(|error| !error.is_instance_of::<PyTypeError>(py)),
            "the issued reader maps missing local sources through its typed refusal"
        );
        let prepared = document_session
            .call_method1(
                "prepare_local_interchange_file_v1",
                (path.to_string_lossy().as_ref(), sdf_handle),
            )
            .expect("registered SDF descriptor prepares a new document");
        let summary = prepared
            .getattr("interchange_summary")
            .expect("safe generic receipt");
        assert!(!summary.is_none(), "interchange receipts carry a summary");
        assert_eq!(
            summary
                .getattr("source_kind")
                .expect("source kind")
                .extract::<String>()
                .expect("source kind is text"),
            "regular_file"
        );
        assert_eq!(
            summary
                .getattr("imported_record_count")
                .expect("record count")
                .extract::<u64>()
                .expect("record count is an integer"),
            1
        );
        assert_eq!(
            summary
                .getattr("atom_count")
                .expect("atom count")
                .extract::<u64>()
                .expect("atom count is an integer"),
            1
        );
        assert_eq!(
            summary
                .getattr("bond_count")
                .expect("bond count")
                .extract::<u64>()
                .expect("bond count is an integer"),
            0
        );
        assert!(
            summary
                .getattr("format_id")
                .expect("format id")
                .is_instance_of::<pyo3::types::PyString>()
        );
        assert!(
            summary
                .getattr("profile_id")
                .expect("profile id")
                .is_instance_of::<pyo3::types::PyString>()
        );

        let admission = prepared
            .call_method0("take_admission_v1")
            .expect("redeem once");
        let session = admission.get_item(0).expect("admitted session");
        let revision = session
            .call_method0("snapshot")
            .expect("session snapshot")
            .getattr("revision")
            .expect("revision")
            .extract::<u64>()
            .expect("revision is an integer");
        assert!(
            prepared.call_method0("take_admission_v1").is_err(),
            "replay refuses"
        );
        assert_eq!(
            session
                .call_method0("snapshot")
                .expect("session snapshot after replay")
                .getattr("revision")
                .expect("revision after replay")
                .extract::<u64>()
                .expect("revision after replay is an integer"),
            revision,
            "replay cannot mutate the redeemed session"
        );
        fs::remove_file(path).expect("remove SDF");
    });
}

#[test]
fn cml_interchange_admission_observation_matches_committed_snapshot() {
    Python::initialize();
    Python::attach(|py| {
        let module = PyModule::new(py, "ferrum_chem").expect("extension module");
        super::super::binding::initialize(&module).expect("extension module registers");
        let document_session = module.getattr("DocumentSession").expect("session type");
        let descriptors = document_session
            .call_method0("local_interchange_open_descriptors_v1")
            .expect("descriptors issue");
        let cml_handle = issued_descriptor(&descriptors, ".cml")
            .getattr("route_handle")
            .expect("CML route handle");
        let path = temporary_cml_path();
        fs::write(&path, TWO_ATOM_CML_V1).expect("write valid CML");

        let prepared = document_session
            .call_method1(
                "prepare_local_interchange_file_v1",
                (path.to_string_lossy().as_ref(), cml_handle),
            )
            .expect("registered CML descriptor prepares a new document");
        let admission = prepared
            .call_method0("take_admission_v1")
            .expect("redeem CML admission");
        assert_eq!(
            admission
                .get_item(3)
                .expect("admitted source kind")
                .extract::<String>()
                .expect("admitted source kind is text"),
            "cml",
            "the CML descriptor authenticates the local origin provenance"
        );
        let snapshot = admission
            .get_item(0)
            .expect("admitted session")
            .call_method0("snapshot")
            .expect("committed session snapshot");
        let observed_snapshot = admission
            .get_item(1)
            .expect("render observation")
            .getattr("document")
            .expect("observation document")
            .getattr("snapshot")
            .expect("observation snapshot");
        assert_eq!(
            snapshot
                .getattr("revision")
                .expect("committed revision")
                .extract::<u64>()
                .expect("committed revision is an integer"),
            1,
            "the imported record is the first history transition"
        );
        for fact in ["revision", "digest"] {
            assert_eq!(
                observed_snapshot
                    .getattr(fact)
                    .expect("observation fact")
                    .to_string(),
                snapshot.getattr(fact).expect("session fact").to_string(),
                "the admission observation and session snapshot share {fact}"
            );
        }
        fs::remove_file(path).expect("remove CML");
    });
}
