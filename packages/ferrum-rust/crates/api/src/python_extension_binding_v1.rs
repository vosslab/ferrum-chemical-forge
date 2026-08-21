//! Private PyO3 transport for the Ferrum wheel.
//!
//! The initialized extension module is the sole authority for the packaged
//! chemistry adapter. Its filename is captured once during `PyInit`; later
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

#[cfg(feature = "response-size-e2e-harness")]
const RESPONSE_SIZE_E2E_LIMIT_ENV: &str = "FERRUM_SMARTS_RESPONSE_SIZE_E2E_LIMIT";

#[cfg(feature = "response-size-e2e-harness")]
fn response_size_e2e_limit() -> Option<usize> {
    std::env::var(RESPONSE_SIZE_E2E_LIMIT_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| (1..=crate::DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1).contains(limit))
}

#[path = "python_binding/lib.rs"]
mod python_binding;

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstalledWheelRuntimeV1 {
    library_path: PathBuf,
}

impl InstalledWheelRuntimeV1 {
    fn from_extension_module(module: &Bound<'_, PyModule>) -> PyResult<Self> {
        let module_file = module
            .getattr("__file__")?
            .extract::<PathBuf>()
            .map_err(|_| {
                PyRuntimeError::new_err("Ferrum extension has no usable installed location")
            })?;
        Self::from_extension_path(module_file)
    }

    fn from_extension_path(module_file: PathBuf) -> PyResult<Self> {
        if !is_extension_module_path(&module_file) {
            return Err(PyRuntimeError::new_err(
                "Ferrum extension has no usable installed location",
            ));
        }
        let directory = module_file.parent().ok_or_else(|| {
            PyRuntimeError::new_err("Ferrum extension has no usable installed location")
        })?;
        // Maturin may install the extension at wheel root or inside the public
        // package. In either form the sealed adapter closure is wheel-local.
        let closure_directory = if directory
            .file_name()
            .is_some_and(|name| name == "ferrum_chem")
        {
            directory.parent().ok_or_else(|| {
                PyRuntimeError::new_err("Ferrum extension has no usable installed location")
            })?
        } else {
            directory
        };
        Ok(Self {
            library_path: closure_directory
                .join(".dylibs")
                .join("libferrum_chem.dylib"),
        })
    }

    fn protocol_runtime(&self) -> TrustedLibraryChemistryRuntimeV1 {
        TrustedLibraryChemistryRuntimeV1::from_trusted_library(self.library_path.clone())
    }

    fn with_installed_wheel_chemistry_v1<T>(
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

impl ChemistryRuntimeV1 for InstalledWheelRuntimeV1 {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        self.with_installed_wheel_chemistry_v1(operation)
    }
}

static INSTALLED_WHEEL_RUNTIME_V1: OnceLock<InstalledWheelRuntimeV1> = OnceLock::new();

fn initialize_installed_wheel_runtime_v1(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let runtime = InstalledWheelRuntimeV1::from_extension_module(module)?;
    match INSTALLED_WHEEL_RUNTIME_V1.get() {
        Some(installed) if installed == &runtime => Ok(()),
        Some(_) => Err(PyRuntimeError::new_err(
            "Ferrum extension runtime was already initialized from another location",
        )),
        None => INSTALLED_WHEEL_RUNTIME_V1
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
pub(super) fn installed_wheel_native_engine_v1() -> Result<NativeChemEngine, ChemistryError> {
    let Some(installed) = INSTALLED_WHEEL_RUNTIME_V1.get() else {
        return Err(ChemistryError::OperationUnavailable {
            operation: "installed_wheel_runtime",
        });
    };
    installed.load_native_engine()
}

/// Return the private adapter locator needed by legacy worker-thread closures.
///
/// The value is copied only inside this extension module hierarchy and always
/// derives from the one `PyInit` runtime capture.
pub(crate) fn installed_wheel_library_path_v1() -> Option<PathBuf> {
    INSTALLED_WHEEL_RUNTIME_V1
        .get()
        .map(InstalledWheelRuntimeV1::library_path)
}

/// Installed-wheel capability resolver used only by generic local interchange
/// preparation.  It has no format input, so decoder policy remains in the
/// descriptor-owned generic core.
pub(crate) struct InstalledWheelInterchangeRuntimeResolverV1;

impl crate::document_interchange_import_v1::LocalInterchangeRuntimeResolverV1
    for InstalledWheelInterchangeRuntimeResolverV1
{
    fn chemistry_runtime(
        &self,
    ) -> Result<TrustedLibraryChemistryRuntimeV1, crate::InterchangeImportRefusalV1> {
        installed_wheel_library_path_v1()
            .map(TrustedLibraryChemistryRuntimeV1::from_trusted_library)
            .ok_or_else(|| {
                crate::InterchangeImportRefusalV1::for_reason(
                    crate::InterchangeImportRefusalReasonV1::ChemistryRuntimeUnavailable,
                )
            })
    }
}

fn execute_operation_from_installed_wheel_v1(
    request_json: &str,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    let Some(installed) = INSTALLED_WHEEL_RUNTIME_V1.get() else {
        return execute_operation_v1(request_json);
    };
    let runtime = installed.protocol_runtime();
    #[cfg(feature = "response-size-e2e-harness")]
    if let Some(response_limit) = response_size_e2e_limit() {
        return crate::protocol::execute_operation_with_runtime_and_smarts_response_limit_for_test(
            request_json,
            &runtime,
            response_limit,
        );
    }
    execute_operation_with_runtime_v1(request_json, &runtime)
}

/// Initialize the only Python ABI module exported by the `cdylib`.
#[pymodule]
fn ferrum_chem(module: &Bound<'_, PyModule>) -> PyResult<()> {
    initialize_installed_wheel_runtime_v1(module)?;
    python_binding::initialize(module)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::InstalledWheelRuntimeV1;

    #[test]
    fn installed_runtime_derives_the_sealed_adapter_once_from_package_extension_path() {
        let wheel_root = std::env::temp_dir().join(format!(
            "ferrum-api-wheel-runtime-test-{}",
            std::process::id()
        ));
        let package_directory = wheel_root.join("ferrum_chem");
        fs::create_dir_all(&package_directory).expect("temporary package directory");
        let extension_path = package_directory.join("ferrum_chem.cpython-312-darwin.so");
        fs::write(&extension_path, []).expect("temporary extension file");
        let runtime = InstalledWheelRuntimeV1::from_extension_path(extension_path)
            .expect("package extension path is an admitted installation origin");
        assert_eq!(
            runtime.library_path,
            wheel_root.join(".dylibs/libferrum_chem.dylib")
        );
        fs::remove_dir_all(wheel_root).expect("remove temporary wheel directory");
    }

    #[test]
    fn installed_runtime_refuses_non_extension_origins() {
        assert!(
            InstalledWheelRuntimeV1::from_extension_path(PathBuf::from("/tmp/ferrum_chem.py"))
                .is_err()
        );
    }
}
