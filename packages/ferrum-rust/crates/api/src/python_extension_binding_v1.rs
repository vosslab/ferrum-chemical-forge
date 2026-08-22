//! Private PyO3 transport for Ferrum's staged local extension.
//!
//! The initialized extension module is the sole authority for the packaged
//! chemistry adapter. Its staged module filename is captured once during `PyInit`; later
//! requests receive no module, path, runtime, or discovery input.

use std::{path::PathBuf, sync::OnceLock};

use ferrum_chemistry::{ChemEngine, ChemistryError, NativeChemEngine};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::{
    OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1, execute_operation_v1,
    protocol::{
        execute_operation_with_runtime_v1,
        runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1, TrustedLibraryChemistryRuntimeV1},
    },
};

#[path = "python_binding/lib.rs"]
mod python_binding;

#[derive(Clone, Debug, Eq, PartialEq)]
struct StagedExtensionRuntimeV1 {
    library_path: PathBuf,
}

impl StagedExtensionRuntimeV1 {
    fn from_extension_module(module: &Bound<'_, PyModule>) -> PyResult<Self> {
        let module_file = module
            .getattr("__file__")?
            .extract::<PathBuf>()
            .map_err(|_| {
                PyRuntimeError::new_err("Ferrum extension has no usable staged location")
            })?;
        Self::from_extension_path(module_file)
    }

    fn from_extension_path(module_file: PathBuf) -> PyResult<Self> {
        if !is_extension_module_path(&module_file) {
            return Err(PyRuntimeError::new_err(
                "Ferrum extension has no usable staged location",
            ));
        }
        let directory = module_file.parent().ok_or_else(|| {
            PyRuntimeError::new_err("Ferrum extension has no usable staged location")
        })?;
        // The local runtime stages the extension at the runtime root. The
        // sealed adapter closure is always relative to that admitted module.
        Ok(Self {
            library_path: directory
                .join(".dylibs")
                .join("libferrum_chem.dylib"),
        })
    }

    fn protocol_runtime(&self) -> TrustedLibraryChemistryRuntimeV1 {
        TrustedLibraryChemistryRuntimeV1::from_trusted_library(self.library_path.clone())
    }

    fn with_staged_extension_chemistry_v1<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        let engine = NativeChemEngine::load(&self.library_path)
            .map_err(ChemistryRuntimeErrorV1::Chemistry)?;
        operation(&engine)
    }

    fn load_native_engine(&self) -> Result<NativeChemEngine, ChemistryError> {
        NativeChemEngine::load(&self.library_path)
    }

    fn library_path(&self) -> PathBuf {
        self.library_path.clone()
    }
}

impl ChemistryRuntimeV1 for StagedExtensionRuntimeV1 {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        self.with_staged_extension_chemistry_v1(operation)
    }
}

static STAGED_EXTENSION_RUNTIME_V1: OnceLock<StagedExtensionRuntimeV1> = OnceLock::new();

fn initialize_staged_extension_runtime_v1(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let runtime = StagedExtensionRuntimeV1::from_extension_module(module)?;
    match STAGED_EXTENSION_RUNTIME_V1.get() {
        Some(installed) if installed == &runtime => Ok(()),
        Some(_) => Err(PyRuntimeError::new_err(
            "Ferrum extension runtime was already initialized from another location",
        )),
        None => STAGED_EXTENSION_RUNTIME_V1
            .set(runtime)
            .map_err(|_| PyRuntimeError::new_err("Ferrum extension runtime initialization failed")),
    }
}

fn is_extension_module_path(path: &std::path::Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ferrum_chem") && name.ends_with(".so"))
}

/// Return a direct engine only through the immutable runtime captured by PyInit.
///
/// This remains module-private: no Python caller receives the runtime, engine,
/// adapter path, or a loader input.
pub(super) fn staged_extension_native_engine_v1() -> Result<NativeChemEngine, ChemistryError> {
    let Some(installed) = STAGED_EXTENSION_RUNTIME_V1.get() else {
        return Err(ChemistryError::OperationUnavailable {
            operation: "staged_extension_runtime",
        });
    };
    installed.load_native_engine()
}

/// Return the private adapter locator needed by extension worker-thread closures.
///
/// The value is copied only inside this extension module hierarchy and always
/// derives from the one `PyInit` runtime capture.
pub(crate) fn staged_extension_library_path_v1() -> Option<PathBuf> {
    STAGED_EXTENSION_RUNTIME_V1
        .get()
        .map(StagedExtensionRuntimeV1::library_path)
}

/// Staged-extension capability resolver used only by generic local interchange
/// preparation.  It has no format input, so decoder policy remains in the
/// descriptor-owned generic core.
pub(crate) struct StagedExtensionInterchangeRuntimeResolverV1;

impl crate::document_interchange_import_v1::LocalInterchangeRuntimeResolverV1
    for StagedExtensionInterchangeRuntimeResolverV1
{
    fn chemistry_runtime(
        &self,
    ) -> Result<TrustedLibraryChemistryRuntimeV1, crate::InterchangeImportRefusalV1> {
        staged_extension_library_path_v1()
            .map(TrustedLibraryChemistryRuntimeV1::from_trusted_library)
            .ok_or_else(|| {
                crate::InterchangeImportRefusalV1::for_reason(
                    crate::InterchangeImportRefusalReasonV1::ChemistryRuntimeUnavailable,
                )
            })
    }
}

fn execute_operation_from_staged_extension_v1(
    request_json: &str,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    let Some(installed) = STAGED_EXTENSION_RUNTIME_V1.get() else {
        return execute_operation_v1(request_json);
    };
    let runtime = installed.protocol_runtime();
    execute_operation_with_runtime_v1(request_json, &runtime)
}

/// Initialize the sealed Python transport from the extension crate's ABI entrypoint.
///
/// The API crate owns protocol behavior and the package-local chemistry adapter
/// invariant. The `ferrum-api-python` crate owns the ABI-specific `PyInit`
/// symbol and calls this function exactly once for each Python module import.
pub fn initialize_python_extension_v1(module: &Bound<'_, PyModule>) -> PyResult<()> {
    initialize_staged_extension_runtime_v1(module)?;
    python_binding::initialize(module)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::StagedExtensionRuntimeV1;

    #[test]
    fn staged_extension_runtime_derives_the_sealed_adapter_from_flat_module_path() {
        let runtime_root = std::env::temp_dir().join(format!(
            "ferrum-api-staged-extension-runtime-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&runtime_root).expect("temporary staged runtime directory");
        let extension_path = runtime_root.join("ferrum_chem.cpython-312-darwin.so");
        fs::write(&extension_path, []).expect("temporary extension file");
        let runtime = StagedExtensionRuntimeV1::from_extension_path(extension_path)
            .expect("flat staged extension path is admitted");
        assert_eq!(
            runtime.library_path,
            runtime_root.join(".dylibs/libferrum_chem.dylib")
        );
        fs::remove_dir_all(runtime_root).expect("remove temporary staged runtime directory");
    }

    #[test]
    fn staged_extension_runtime_refuses_non_extension_origins() {
        assert!(
            StagedExtensionRuntimeV1::from_extension_path(PathBuf::from("/tmp/ferrum_chem.py"))
                .is_err()
        );
    }
}
