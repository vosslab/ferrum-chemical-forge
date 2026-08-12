#[path = "../src/adapter_abi.rs"]
mod adapter_abi;

#[test]
fn matching_adapter_abi_is_accepted() {
    assert!(adapter_abi::ensure_supported_adapter_abi_version(2, 2).is_ok());
}

#[test]
fn unsupported_adapter_abi_has_an_actionable_error() {
    let error = adapter_abi::ensure_supported_adapter_abi_version(3, 2)
        .expect_err("future adapter ABI must be rejected");
    assert_eq!(
        error.to_string(),
        "Ferrum-Chem adapter ABI 3 is unsupported; this Ferrum API requires ABI 2"
    );
}
